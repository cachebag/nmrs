//! Core Bluetooth connection management logic.
//!
//! This module contains the internal implementation details for managing
//! Bluetooth devices and connections.
//!
//! Similar to other device types, it handles scanning, connecting, and monitoring
//! Bluetooth devices using NetworkManager's D-Bus API.

use log::debug;
use zbus::Connection;
use zvariant::OwnedObjectPath;

use crate::builders::bluetooth;
use crate::core::connection_settings::get_saved_connection_path;
use crate::dbus::BluezDeviceExtProxy;
use crate::monitoring::bluetooth::Bluetooth;
use crate::monitoring::transport::ActiveTransport;
use crate::ConnectionError;
use crate::{dbus::NMProxy, models::BluetoothIdentity, Result};

/// Populated Bluetooth device information via BlueZ.
///
/// Given a Bluetooth device address (BDADDR), this function queries BlueZ
/// over D-Bus to retrieve the device's name and alias. It constructs the
/// appropriate D-Bus object path based on the BDADDR format.
///
/// NetworkManager does not expose Bluetooth device names/aliases directly,
/// hence this additional step is necessary to obtain user-friendly
/// identifiers for Bluetooth devices. (See `BluezDeviceExtProxy` for details.)
pub(crate) async fn populate_bluez_info(
    conn: &Connection,
    bdaddr: &str,
) -> Result<(Option<String>, Option<String>)> {
    // [variable prefix]/{hci0,hci1,...}/dev_XX_XX_XX_XX_XX_XX
    // This replaces ':' with '_' in the BDADDR to form the correct D-Bus object path.
    // TODO: Instead of hardcoding hci0, we should determine the actual adapter name.
    let bluez_path = format!("/org/bluez/hci0/dev_{}", bdaddr.replace(':', "_"));

    match BluezDeviceExtProxy::builder(conn)
        .path(bluez_path)?
        .build()
        .await
    {
        Ok(proxy) => {
            let name = proxy.name().await.ok();
            let alias = proxy.alias().await.ok();
            Ok((name, alias))
        }
        Err(_) => Ok((None, None)),
    }
}

/// Connects to a Bluetooth device using NetworkManager.
///
/// This function establishes a Bluetooth network connection. The flow:
/// 1. Check if already connected to this device
/// 2. Find the Bluetooth hardware adapter
/// 3. Check for an existing saved connection
/// 4. Either activate the saved connection or create a new one
/// 5. Wait for the connection to reach the activated state
///
/// **Important:** The Bluetooth device must already be paired via BlueZ
/// (using `bluetoothctl` or similar) before NetworkManager can connect to it.
///
/// # Arguments
///
/// * `conn` - D-Bus connection
/// * `name` - Connection name/identifier
/// * `settings` - Bluetooth device settings (bdaddr and type)
///
/// # Example
///
/// ```no_run
/// use nmrs::models::{BluetoothIdentity, BluetoothNetworkRole};
///
/// let settings = BluetoothIdentity {
///     bdaddr: "C8:1F:E8:F0:51:57".into(),
///     bt_device_type: BluetoothNetworkRole::PanU,
/// };
/// // connect_bluetooth(&conn, "My Phone", &settings).await?;
/// ```
pub(crate) async fn connect_bluetooth(
    conn: &Connection,
    name: &str,
    settings: &BluetoothIdentity,
) -> Result<()> {
    debug!(
        "Connecting to '{}' (Bluetooth) | bdaddr={} type={:?}",
        name, settings.bdaddr, settings.bt_device_type
    );

    let nm = NMProxy::new(conn).await?;

    // Check if already connected to this device
    if let Some(active) = Bluetooth::current(conn).await {
        debug!("Currently connected to Bluetooth device: {active}");
        if active == settings.bdaddr {
            debug!("Already connected to {active}, skipping connect()");
            return Ok(());
        }
    } else {
        debug!("Not currently connected to any Bluetooth device");
    }

    // Find the Bluetooth hardware adapter
    // Note: Unlike WiFi, Bluetooth connections in NetworkManager don't require
    // specifying a specific device. We use "/" to let NetworkManager auto-select.
    let bt_device = OwnedObjectPath::try_from("/")
        .map_err(|e| ConnectionError::InvalidAddress(format!("Invalid device path: {}", e)))?;
    debug!("Using auto-select device path for Bluetooth connection");

    // Check for saved connection
    let saved = get_saved_connection_path(conn, name).await?;

    // For Bluetooth, the "specific_object" is the remote device's D-Bus path
    // Format: /org/bluez/hci0/dev_XX_XX_XX_XX_XX_XX
    // TODO: Instead of hardcoding the hci0, we should use the actual hardware adapter name.
    let specific_object = OwnedObjectPath::try_from(format!(
        "/org/bluez/hci0/dev_{}",
        settings.bdaddr.replace(':', "_")
    ))
    .map_err(|e| ConnectionError::InvalidAddress(format!("Invalid BlueZ path: {}", e)))?;

    match saved {
        Some(saved_path) => {
            debug!(
                "Activating saved Bluetooth connection: {}",
                saved_path.as_str()
            );
            let active_conn = nm
                .activate_connection(saved_path, bt_device.clone(), specific_object)
                .await?;

            crate::core::state_wait::wait_for_connection_activation(conn, &active_conn).await?;
        }
        None => {
            debug!("No saved connection found, creating new Bluetooth connection");
            let opts = crate::api::models::ConnectionOptions {
                autoconnect: false, // Bluetooth typically doesn't auto-connect
                autoconnect_priority: None,
                autoconnect_retries: None,
            };

            let connection_settings = bluetooth::build_bluetooth_connection(name, settings, &opts);

            println!(
                "Creating Bluetooth connection with settings: {:#?}",
                connection_settings
            );

            let (_, active_conn) = nm
                .add_and_activate_connection(
                    connection_settings,
                    bt_device.clone(),
                    specific_object,
                )
                .await?;

            crate::core::state_wait::wait_for_connection_activation(conn, &active_conn).await?;
        }
    }

    log::info!("Successfully connected to Bluetooth device '{name}'");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BluetoothNetworkRole;

    #[test]
    fn test_bluez_path_format() {
        // Test that bdaddr format is converted correctly for D-Bus path
        let bdaddr = "00:1A:7D:DA:71:13";
        let expected_path = "/org/bluez/hci0/dev_00_1A_7D_DA_71_13";
        let actual_path = format!("/org/bluez/hci0/dev_{}", bdaddr.replace(':', "_"));
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_bluez_path_format_various_addresses() {
        let test_cases = vec![
            ("AA:BB:CC:DD:EE:FF", "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF"),
            ("00:00:00:00:00:00", "/org/bluez/hci0/dev_00_00_00_00_00_00"),
            ("C8:1F:E8:F0:51:57", "/org/bluez/hci0/dev_C8_1F_E8_F0_51_57"),
        ];

        for (bdaddr, expected_path) in test_cases {
            let actual_path = format!("/org/bluez/hci0/dev_{}", bdaddr.replace(':', "_"));
            assert_eq!(actual_path, expected_path, "Failed for bdaddr: {}", bdaddr);
        }
    }

    #[test]
    fn test_bluetooth_identity_structure() {
        let identity = BluetoothIdentity {
            bdaddr: "00:1A:7D:DA:71:13".into(),
            bt_device_type: BluetoothNetworkRole::PanU,
        };

        assert_eq!(identity.bdaddr, "00:1A:7D:DA:71:13");
        assert!(matches!(
            identity.bt_device_type,
            BluetoothNetworkRole::PanU
        ));
    }

    // Note: Most of the core connection functions require a real D-Bus connection
    // and NetworkManager running, so they are better suited for integration tests.
}
