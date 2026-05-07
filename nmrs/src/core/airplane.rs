//! Airplane-mode aggregation logic.
//!
//! Combines radio state from NetworkManager (Wi-Fi, WWAN), BlueZ (Bluetooth
//! adapter power), and kernel rfkill into a single [`AirplaneModeState`].
//!
//! Each radio's state carries a `present` flag so consumers can ignore radios
//! the host does not actually have (no Wi-Fi card, no modem, BlueZ not
//! running) instead of blocking airplane-mode aggregation forever.

use std::time::Duration;

use futures::{FutureExt, StreamExt, future};
use futures_timer::Delay;
use log::warn;
use std::pin::pin;
use zbus::Connection;

use crate::api::models::{AirplaneModeState, RadioState};
use crate::core::rfkill::read_rfkill;
use crate::dbus::{BluezAdapterProxy, NMDeviceProxy, NMProxy};
use crate::types::constants::device_type;
use crate::{ConnectionError, Result};

/// Maximum time to wait for a BlueZ adapter's `Powered` property to reflect
/// a write before we give up and return Ok anyway. BlueZ usually settles in
/// well under a second; we cap at two to avoid hanging UI consumers.
const BLUEZ_POWER_SETTLE_TIMEOUT: Duration = Duration::from_secs(2);

/// Reads Wi-Fi radio state from NetworkManager, cross-referenced with rfkill.
pub(crate) async fn wifi_state(conn: &Connection) -> Result<RadioState> {
    let nm = NMProxy::new(conn).await?;
    let enabled = nm.wireless_enabled().await?;
    let nm_hw = nm.wireless_hardware_enabled().await?;

    let rfkill = read_rfkill();
    let hardware_enabled = reconcile_hardware(nm_hw, rfkill.wlan_hard_block, "wifi");
    let present = has_device_of_type(conn, device_type::WIFI).await;

    Ok(RadioState::with_presence(
        enabled,
        hardware_enabled,
        present,
    ))
}

/// Reads WWAN radio state from NetworkManager, cross-referenced with rfkill.
pub(crate) async fn wwan_state(conn: &Connection) -> Result<RadioState> {
    let nm = NMProxy::new(conn).await?;
    let enabled = nm.wwan_enabled().await?;
    let nm_hw = nm.wwan_hardware_enabled().await?;

    let rfkill = read_rfkill();
    let hardware_enabled = reconcile_hardware(nm_hw, rfkill.wwan_hard_block, "wwan");
    let present = has_device_of_type(conn, device_type::MODEM).await;

    Ok(RadioState::with_presence(
        enabled,
        hardware_enabled,
        present,
    ))
}

/// Reads Bluetooth radio state from BlueZ adapters, cross-referenced with rfkill.
///
/// If BlueZ is not running or no adapters exist, returns a `RadioState`
/// with `present = false` so callers can ignore Bluetooth entirely on
/// hosts that don't have it.
pub(crate) async fn bluetooth_radio_state(conn: &Connection) -> Result<RadioState> {
    let adapter_paths = match enumerate_bluetooth_adapters(conn).await {
        Ok(paths) if !paths.is_empty() => paths,
        Ok(_) | Err(_) => {
            return Ok(RadioState::with_presence(false, false, false));
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

    Ok(RadioState::with_presence(
        any_powered,
        hardware_enabled,
        true,
    ))
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
/// After writing `Powered` we wait up to [`BLUEZ_POWER_SETTLE_TIMEOUT`] for
/// the adapter's reported state to actually flip. Otherwise a consumer that
/// re-reads [`bluetooth_radio_state`] right after this call can observe the
/// pre-toggle value briefly and conclude the toggle didn't take effect.
///
/// If BlueZ is not running, returns [`ConnectionError::BluezUnavailable`].
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
            wait_for_powered(&proxy, enabled, BLUEZ_POWER_SETTLE_TIMEOUT).await;
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
/// Does not fail fast — attempts all three and returns the first error,
/// except that a missing Bluetooth stack is not treated as a failure
/// (`BluezUnavailable` is silently downgraded to a no-op).
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
    match bt_res {
        Ok(()) => {}
        Err(ConnectionError::BluezUnavailable(_)) => {
            // No Bluetooth on this host — that's fine, don't fail the whole
            // call (and don't leave callers thinking the wifi/wwan flips
            // didn't happen).
        }
        Err(e) => return Err(e),
    }
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

/// Returns `true` if NetworkManager has at least one device of the given type.
///
/// Failures (D-Bus error, no NM running) are treated as "no devices of this
/// type" — we'd rather mark a radio as absent than spuriously block the
/// airplane-mode aggregator.
async fn has_device_of_type(conn: &Connection, type_code: u32) -> bool {
    let Ok(nm) = NMProxy::new(conn).await else {
        return false;
    };
    let Ok(paths) = nm.get_devices().await else {
        return false;
    };
    for p in paths {
        let Ok(builder) = NMDeviceProxy::builder(conn).path(p) else {
            continue;
        };
        let Ok(dev) = builder.build().await else {
            continue;
        };
        if let Ok(t) = dev.device_type().await
            && t == type_code
        {
            return true;
        }
    }
    false
}

/// Waits for a BlueZ adapter's `Powered` property to settle on `target`.
///
/// Subscribes to `PropertiesChanged` on `Powered` first, then re-reads the
/// current value (so we don't miss a fast transition that happened between
/// the `set_powered` write and the subscription). Returns when the property
/// matches `target` or when `timeout` elapses (whichever comes first).
async fn wait_for_powered(proxy: &BluezAdapterProxy<'_>, target: bool, timeout: Duration) {
    let mut stream = proxy.receive_powered_changed().await;

    if proxy.powered().await.unwrap_or(target) == target {
        return;
    }

    let watcher = async {
        while let Some(change) = stream.next().await {
            if let Ok(value) = change.get().await
                && value == target
            {
                return;
            }
        }
    };

    let watcher = pin!(watcher.fuse());
    let timer = pin!(Delay::new(timeout).fuse());
    let _ = future::select(watcher, timer).await;
}
