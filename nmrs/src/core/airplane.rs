//! Airplane-mode aggregation logic.
//!
//! Combines radio state from NetworkManager (Wi-Fi, WWAN), BlueZ (Bluetooth
//! adapter power), and kernel rfkill into a single [`AirplaneModeState`].

use log::warn;
use zbus::Connection;

use crate::api::models::{AirplaneModeState, RadioState};
use crate::core::rfkill::read_rfkill;
use crate::dbus::{BluezAdapterProxy, NMProxy};
use crate::{ConnectionError, Result};

/// Reads Wi-Fi radio state from NetworkManager, cross-referenced with rfkill.
pub(crate) async fn wifi_state(conn: &Connection) -> Result<RadioState> {
    let nm = NMProxy::new(conn).await?;
    let enabled = nm.wireless_enabled().await?;
    let nm_hw = nm.wireless_hardware_enabled().await?;

    let rfkill = read_rfkill();
    let hardware_enabled = reconcile_hardware(nm_hw, rfkill.wlan_hard_block, "wifi");

    Ok(RadioState::new(enabled, hardware_enabled))
}

/// Reads WWAN radio state from NetworkManager, cross-referenced with rfkill.
pub(crate) async fn wwan_state(conn: &Connection) -> Result<RadioState> {
    let nm = NMProxy::new(conn).await?;
    let enabled = nm.wwan_enabled().await?;
    let nm_hw = nm.wwan_hardware_enabled().await?;

    let rfkill = read_rfkill();
    let hardware_enabled = reconcile_hardware(nm_hw, rfkill.wwan_hard_block, "wwan");

    Ok(RadioState::new(enabled, hardware_enabled))
}

/// Reads Bluetooth radio state from BlueZ adapters, cross-referenced with rfkill.
///
/// If BlueZ is not running or no adapters exist, returns
/// `RadioState { enabled: true, hardware_enabled: false }` — "hardware killed"
/// is the honest answer when there is no Bluetooth stack.
pub(crate) async fn bluetooth_radio_state(conn: &Connection) -> Result<RadioState> {
    let adapter_paths = match enumerate_bluetooth_adapters(conn).await {
        Ok(paths) if !paths.is_empty() => paths,
        Ok(_) | Err(_) => {
            return Ok(RadioState::new(true, false));
        }
    };

    let mut any_powered = false;
    for path in &adapter_paths {
        match BluezAdapterProxy::builder(conn)
            .path(path.as_str())?
            .build()
            .await
        {
            Ok(proxy) => {
                if proxy.powered().await.unwrap_or(false) {
                    any_powered = true;
                    break;
                }
            }
            Err(e) => {
                warn!("failed to query BlueZ adapter {}: {}", path, e);
            }
        }
    }

    let rfkill = read_rfkill();
    let hardware_enabled = !rfkill.bluetooth_hard_block;

    Ok(RadioState::new(any_powered, hardware_enabled))
}

/// Returns the combined airplane mode state for all radios.
pub(crate) async fn airplane_mode_state(conn: &Connection) -> Result<AirplaneModeState> {
    let (wifi, wwan, bt) = futures::future::join3(
        wifi_state(conn),
        wwan_state(conn),
        bluetooth_radio_state(conn),
    )
    .await;

    Ok(AirplaneModeState::new(wifi?, wwan?, bt?))
}

/// Enables or disables wireless radio (software toggle).
pub(crate) async fn set_wireless_enabled(conn: &Connection, enabled: bool) -> Result<()> {
    let nm = NMProxy::new(conn).await?;
    Ok(nm.set_wireless_enabled(enabled).await?)
}

/// Enables or disables WWAN radio (software toggle).
pub(crate) async fn set_wwan_enabled(conn: &Connection, enabled: bool) -> Result<()> {
    let nm = NMProxy::new(conn).await?;
    Ok(nm.set_wwan_enabled(enabled).await?)
}

/// Enables or disables Bluetooth radio by toggling all BlueZ adapters.
///
/// If BlueZ is not running, returns `BluezUnavailable`.
pub(crate) async fn set_bluetooth_radio_enabled(conn: &Connection, enabled: bool) -> Result<()> {
    let adapter_paths = enumerate_bluetooth_adapters(conn).await.map_err(|e| {
        ConnectionError::BluezUnavailable(format!("failed to enumerate adapters: {e}"))
    })?;

    if adapter_paths.is_empty() {
        return Err(ConnectionError::BluezUnavailable(
            "no Bluetooth adapters found".to_string(),
        ));
    }

    let mut first_err: Option<ConnectionError> = None;
    for path in &adapter_paths {
        let result: Result<()> = async {
            let proxy = BluezAdapterProxy::builder(conn)
                .path(path.as_str())?
                .build()
                .await?;
            proxy.set_powered(enabled).await?;
            Ok(())
        }
        .await;

        if let Err(e) = result {
            warn!("failed to set Powered on {}: {}", path, e);
            if first_err.is_none() {
                first_err = Some(e);
            }
        }
    }

    match first_err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Flips all three radios in parallel.
///
/// `enabled = true` means airplane mode **on** (radios **off**).
/// Does not fail fast — attempts all three and returns the first error.
pub(crate) async fn set_airplane_mode(conn: &Connection, enabled: bool) -> Result<()> {
    let radio_on = !enabled;

    let (wifi_res, wwan_res, bt_res) = futures::future::join3(
        set_wireless_enabled(conn, radio_on),
        set_wwan_enabled(conn, radio_on),
        set_bluetooth_radio_enabled(conn, radio_on),
    )
    .await;

    // Return the first error, but don't short-circuit — all three have been attempted.
    wifi_res?;
    wwan_res?;
    bt_res?;
    Ok(())
}

/// Enumerates BlueZ Bluetooth adapters via the ObjectManager interface.
///
/// Returns adapter object paths (e.g. `/org/bluez/hci0`).
async fn enumerate_bluetooth_adapters(conn: &Connection) -> Result<Vec<String>> {
    let manager = zbus::fdo::ObjectManagerProxy::builder(conn)
        .destination("org.bluez")?
        .path("/")?
        .build()
        .await
        .map_err(|e| {
            ConnectionError::BluezUnavailable(format!("failed to connect to BlueZ: {e}"))
        })?;

    let objects = manager.get_managed_objects().await.map_err(|e| {
        ConnectionError::BluezUnavailable(format!("failed to enumerate BlueZ objects: {e}"))
    })?;

    let adapters: Vec<String> = objects
        .into_iter()
        .filter(|(_, ifaces)| ifaces.contains_key("org.bluez.Adapter1"))
        .map(|(path, _)| path.to_string())
        .collect();

    Ok(adapters)
}

/// Reconciles NM's hardware-enabled flag with rfkill. If they disagree, trust rfkill.
fn reconcile_hardware(nm_hardware_enabled: bool, rfkill_hard_block: bool, radio: &str) -> bool {
    if nm_hardware_enabled && rfkill_hard_block {
        warn!(
            "{radio}: NM reports hardware enabled but rfkill reports hard block — trusting rfkill"
        );
        return false;
    }
    nm_hardware_enabled && !rfkill_hard_block
}
