use crate::models::{
    ConnectionOptions, Device, DeviceState, DeviceType, Network, NetworkInfo, WifiSecurity,
};
use crate::wifi_builders::build_wifi_connection;
use futures_timer::Delay;
use std::collections::HashMap;
use std::time::Duration;
use zbus::Connection;
use zbus::Result;
use zbus::proxy;
use zvariant::OwnedObjectPath;

pub struct NetworkManager {
    conn: Connection,
}

// Proxies for D-Bus interfaces
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NMDevice {
    #[zbus(property)]
    fn interface(&self) -> Result<String>;

    #[zbus(property)]
    fn device_type(&self) -> Result<u32>;

    #[zbus(property)]
    fn state(&self) -> Result<u32>;

    #[zbus(property)]
    fn managed(&self) -> Result<bool>;

    #[zbus(property)]
    fn driver(&self) -> Result<String>;

    #[zbus(property)]
    fn state_reason(&self) -> Result<(u32, u32)>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NM {
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    fn add_and_activate_connection(
        &self,
        connection: HashMap<&str, HashMap<&str, zvariant::Value<'_>>>,
        device: OwnedObjectPath,
        specific_object: OwnedObjectPath,
    ) -> zbus::Result<(OwnedObjectPath, OwnedObjectPath)>;

    fn activate_connection(
        &self,
        connection: OwnedObjectPath,
        device: OwnedObjectPath,
        specific_object: OwnedObjectPath,
    ) -> zbus::Result<OwnedObjectPath>;

    fn deactivate_connection(&self, active_connection: OwnedObjectPath) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMWireless {
    fn get_all_access_points(&self) -> Result<Vec<OwnedObjectPath>>;
    fn request_scan(&self, options: HashMap<String, zvariant::Value<'_>>) -> Result<()>;

    #[zbus(signal)]
    fn access_point_added(&self, path: OwnedObjectPath);

    #[zbus(signal)]
    fn access_point_removed(&self, path: OwnedObjectPath);

    #[zbus(property)]
    fn active_access_point(&self) -> Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NMAccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> Result<Vec<u8>>;

    #[zbus(property)]
    fn strength(&self) -> Result<u8>;

    #[zbus(property)]
    fn hw_address(&self) -> Result<String>;

    #[zbus(property)]
    fn flags(&self) -> Result<u32>;

    #[zbus(property)]
    fn wpa_flags(&self) -> Result<u32>;

    #[zbus(property)]
    fn rsn_flags(&self) -> Result<u32>;

    #[zbus(property)]
    fn frequency(&self) -> Result<u32>;

    #[zbus(property)]
    fn max_bitrate(&self) -> Result<u32>;

    #[zbus(property)]
    fn mode(&self) -> Result<u32>;
}

impl NetworkManager {
    pub async fn new() -> zbus::Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        let proxy = NMProxy::new(&self.conn).await?;
        let paths = proxy.get_devices().await?;

        let mut devices = Vec::new();
        for p in paths {
            let d_proxy = NMDeviceProxy::builder(&self.conn)
                .path(p.clone())?
                .build()
                .await?;

            let interface = d_proxy.interface().await?;
            let raw_type = d_proxy.device_type().await?;
            let device_type = raw_type.into();
            let raw_state = d_proxy.state().await?;
            let state = raw_state.into();
            let managed = d_proxy.managed().await.ok();
            let driver = d_proxy.driver().await.ok();

            devices.push(Device {
                path: p.to_string(),
                interface,
                device_type,
                state,
                managed,
                driver,
            });
        }
        Ok(devices)
    }

    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        let nm = NMProxy::new(&self.conn).await?;
        let devices = nm.get_devices().await?;

        let mut networks: HashMap<(String, u32), Network> = HashMap::new();

        for dp in devices {
            let d_proxy = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;
            if d_proxy.device_type().await? != 2 {
                continue;
            }

            let wifi = NMWirelessProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;

            for ap_path in wifi.get_all_access_points().await? {
                let ap = NMAccessPointProxy::builder(&self.conn)
                    .path(ap_path.clone())?
                    .build()
                    .await?;
                let ssid_bytes = ap.ssid().await?;
                let ssid = if ssid_bytes.is_empty() {
                    "<Hidden Network>".to_string()
                } else {
                    let s = std::str::from_utf8(&ssid_bytes).unwrap_or("<Hidden Network");
                    s.to_string()
                };
                let strength = ap.strength().await?;
                let bssid = ap.hw_address().await?;
                let flags = ap.flags().await?;
                let wpa = ap.wpa_flags().await?;
                let rsn = ap.rsn_flags().await?;
                let frequency = ap.frequency().await?;

                let secured = (flags & 0x1) != 0 || wpa != 0 || rsn != 0;
                let is_psk = (wpa & 0x0100) != 0 || (rsn & 0x0100) != 0;
                let is_eap = (wpa & 0x0200) != 0 || (rsn & 0x0200) != 0;

                let new_net = Network {
                    device: dp.to_string(),
                    ssid: ssid.clone(),
                    bssid: Some(bssid),
                    strength: Some(strength),
                    frequency: Some(frequency),
                    secured,
                    is_psk,
                    is_eap,
                };

                // Use (SSID, frequency) as key to separate 2.4GHz and 5GHz
                networks
                    .entry((ssid.clone(), frequency))
                    .and_modify(|n| {
                        if strength > n.strength.unwrap_or(0) {
                            *n = new_net.clone();
                        }
                        if new_net.secured {
                            n.secured = true;
                        }
                    })
                    .or_insert(new_net);
            }
        }

        let result: Vec<Network> = networks.into_values().collect();
        Ok(result)
    }

    pub async fn connect(&self, ssid: &str, creds: crate::models::WifiSecurity) -> Result<()> {
        println!(
            "Connecting to '{}' | secured={} is_psk={} is_eap={}",
            ssid,
            creds.secured(),
            creds.is_psk(),
            creds.is_eap()
        );

        let nm = NMProxy::new(&self.conn).await?;

        let saved_conn_path = self.get_saved_connection_path(ssid).await?;

        let use_saved_connection = if let Some(conn_path) = &saved_conn_path {
            // If PSK is empty, we're trying to use saved credentials
            if creds.is_psk() {
                if let WifiSecurity::WpaPsk { psk } = &creds {
                    if psk.trim().is_empty() {
                        eprintln!("Using saved connection at: {}", conn_path.as_str());
                        true
                    } else {
                        eprintln!(
                            "Have saved connection but new password provided, deleting old and creating new"
                        );
                        let _ = self.delete_connection(conn_path.clone()).await;
                        false
                    }
                } else {
                    false
                }
            } else {
                // For open or EAP, use saved if available
                eprintln!("Using saved connection at: {}", conn_path.as_str());
                true
            }
        } else {
            // No saved connection
            if creds.is_psk()
                && let WifiSecurity::WpaPsk { psk } = &creds
                && psk.trim().is_empty()
            {
                return Err(zbus::Error::Failure(
                    "No saved connection and PSK is empty".into(),
                ));
            }

            false
        };

        let devices = nm.get_devices().await?;
        let mut wifi_device: Option<OwnedObjectPath> = None;

        for dp in devices {
            let dev = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;
            if dev.device_type().await? == 2 {
                wifi_device = Some(dp.clone());
                eprintln!("   Found WiFi device: {dp}");
                break;
            }
        }

        let wifi_device =
            wifi_device.ok_or(zbus::Error::Failure("no Wi-Fi device found".into()))?;

        let wifi = NMWirelessProxy::builder(&self.conn)
            .path(wifi_device.clone())?
            .build()
            .await?;

        if let Some(active) = self.current_ssid().await {
            eprintln!("Currently connected to: {active}");
            if active == ssid {
                eprintln!("Already connected to {active}, skipping connect()");
                return Ok(());
            }
        } else {
            eprintln!("Not currently connected to any network");
        }

        match wifi.request_scan(HashMap::new()).await {
            Ok(_) => eprintln!("Scan requested successfully"),
            Err(e) => eprintln!("Scan request FAILED: {e}"),
        }
        Delay::new(Duration::from_secs(3)).await;
        eprintln!("Scan wait complete");

        let mut ap_path: Option<OwnedObjectPath> = None;
        for ap in wifi.get_all_access_points().await? {
            let apx = NMAccessPointProxy::builder(&self.conn)
                .path(ap.clone())?
                .build()
                .await?;
            let ssid_bytes = apx.ssid().await?;
            let ap_ssid = std::str::from_utf8(&ssid_bytes).unwrap_or("");
            eprintln!("Found AP: '{ap_ssid}'");
            if ap_ssid == ssid {
                ap_path = Some(ap.clone());
                eprintln!("Matched target SSID");
                break;
            }
        }

        if ap_path.is_none() {
            return Err(zbus::Error::Failure(format!("Network '{ssid}' not found")));
        }

        let specific_object = ap_path.unwrap();

        if use_saved_connection {
            if let Some(active) = self.current_ssid().await {
                eprintln!("Disconnecting from {active}.");
                if let Ok(conns) = nm.active_connections().await {
                    for conn_path in conns {
                        eprintln!("Deactivating connection: {conn_path}");
                        let _ = nm.deactivate_connection(conn_path).await;
                    }
                }

                for i in 0..10 {
                    let d = NMDeviceProxy::builder(&self.conn)
                        .path(wifi_device.clone())?
                        .build()
                        .await?;
                    let state = DeviceState::from(d.state().await?);
                    eprintln!("Loop {i}: Device state = {state:?}");

                    if state == DeviceState::Disconnected || state == DeviceState::Unavailable {
                        eprintln!("Device disconnected");
                        break;
                    }

                    Delay::new(Duration::from_millis(300)).await;
                }

                Delay::new(Duration::from_millis(500)).await;
                eprintln!("Disconnect complete");
            }

            let conn_path = saved_conn_path.unwrap();
            eprintln!("Activating saved connection: {}", conn_path.as_str());

            match nm
                .activate_connection(
                    conn_path.clone(),
                    wifi_device.clone(),
                    specific_object.clone(),
                )
                .await
            {
                Ok(active_conn) => {
                    eprintln!(
                        "activate_connection() succeeded, active connection: {}",
                        active_conn.as_str()
                    );

                    Delay::new(Duration::from_millis(500)).await;

                    let dev_check = NMDeviceProxy::builder(&self.conn)
                        .path(wifi_device.clone())?
                        .build()
                        .await?;

                    let check_state = dev_check.state().await?;

                    if check_state == 30 {
                        eprintln!("Connection activated but device stuck in Disconnected state");
                        eprintln!("Saved connection has invalid settings, deleting and retrying");

                        let _ = nm.deactivate_connection(active_conn).await;

                        let _ = self.delete_connection(conn_path).await;

                        let opts = ConnectionOptions {
                            autoconnect: true,
                            autoconnect_priority: None,
                            autoconnect_retries: None,
                        };

                        let settings = build_wifi_connection(ssid, &creds, &opts);

                        eprintln!("Creating fresh connection with corrected settings");
                        match nm
                            .add_and_activate_connection(
                                settings,
                                wifi_device.clone(),
                                specific_object,
                            )
                            .await
                        {
                            Ok(_) => eprintln!("Fresh connection created successfully"),
                            Err(e) => {
                                eprintln!("Fresh connection also failed: {e}");
                                return Err(e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("activate_connection() failed: {e}");
                    eprintln!(
                        "Saved connection may be corrupted, deleting and retrying with fresh connection"
                    );

                    let _ = self.delete_connection(conn_path).await;

                    let opts = ConnectionOptions {
                        autoconnect: true,
                        autoconnect_priority: None,
                        autoconnect_retries: None,
                    };

                    let settings = build_wifi_connection(ssid, &creds, &opts);

                    eprintln!("Creating fresh connection after saved connection failed");
                    return match nm
                        .add_and_activate_connection(settings, wifi_device.clone(), specific_object)
                        .await
                    {
                        Ok(_) => {
                            eprintln!("Successfully created fresh connection");
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("Fresh connection also failed: {e}");
                            Err(e)
                        }
                    };
                }
            }
        } else {
            let opts = ConnectionOptions {
                autoconnect: true,
                autoconnect_priority: None,
                autoconnect_retries: None,
            };

            let settings = build_wifi_connection(ssid, &creds, &opts);

            println!("Creating new connection, settings: \n{settings:#?}");

            if let Some(active) = self.current_ssid().await {
                eprintln!("Disconnecting from {active}.");
                if let Ok(conns) = nm.active_connections().await {
                    for conn_path in conns {
                        eprintln!("Deactivating connection: {conn_path}");
                        let _ = nm.deactivate_connection(conn_path).await;
                    }
                }

                for i in 0..10 {
                    let d = NMDeviceProxy::builder(&self.conn)
                        .path(wifi_device.clone())?
                        .build()
                        .await?;
                    let state = DeviceState::from(d.state().await?);
                    eprintln!("Loop {i}: Device state = {state:?}");

                    if state == DeviceState::Disconnected || state == DeviceState::Unavailable {
                        eprintln!("Device disconnected");
                        break;
                    }

                    Delay::new(Duration::from_millis(300)).await;
                }

                Delay::new(Duration::from_millis(500)).await;
                eprintln!("Disconnect complete");
            }

            match nm
                .add_and_activate_connection(settings, wifi_device.clone(), specific_object)
                .await
            {
                Ok(_) => eprintln!("add_and_activate_connection() succeeded"),
                Err(e) => {
                    eprintln!("add_and_activate_connection() failed: {e}");
                    return Err(e);
                }
            }
        }

        Delay::new(Duration::from_millis(300)).await;

        let dev_proxy = NMDeviceProxy::builder(&self.conn)
            .path(wifi_device.clone())?
            .build()
            .await?;

        let initial_state = dev_proxy.state().await?;
        eprintln!("Dev state after connect(): {initial_state:?}");

        eprintln!("Waiting for connection to complete...");
        let mut connected = false;
        let mut config_state_count = 0;
        for i in 0..40 {
            Delay::new(Duration::from_millis(500)).await;
            match dev_proxy.state().await {
                Ok(state) => {
                    let device_state = DeviceState::from(state);
                    eprintln!("Connection progress {i}: state = {device_state:?} ({state})");

                    if state == 100 {
                        eprintln!("✓ Connection activated successfully!");
                        connected = true;
                        break;
                    } else if state == 120 {
                        eprintln!("✗ Connection failed!");

                        if let Ok(reason) = dev_proxy.state_reason().await {
                            eprintln!("Failure reason code: {reason:?}");
                            let reason_str = match reason.1 {
                                0 => "Unknown",
                                1 => "None",
                                2 => "User disconnected",
                                3 => "Device disconnected",
                                4 => "Carrier changed",
                                7 => "Supplicant disconnected",
                                8 => "Supplicant config failed",
                                9 => "Supplicant failed",
                                10 => "Supplicant timeout",
                                11 => "PPP start failed",
                                15 => "DHCP start failed",
                                16 => "DHCP error",
                                17 => "DHCP failed",
                                24 => "Modem connection failed",
                                25 => "Modem init failed",
                                42 => "Infiniband mode",
                                43 => "Dependency failed",
                                44 => "BR2684 failed",
                                45 => "Mode set failed",
                                46 => "GSM APN select failed",
                                47 => "GSM not searching",
                                48 => "GSM registration denied",
                                49 => "GSM registration timeout",
                                50 => "GSM registration failed",
                                51 => "GSM PIN check failed",
                                52 => "Firmware missing",
                                53 => "Device removed",
                                54 => "Sleeping",
                                55 => "Connection removed",
                                56 => "User requested",
                                57 => "Carrier",
                                58 => "Connection assumed",
                                59 => "Supplicant available",
                                60 => "Modem not found",
                                61 => "Bluetooth failed",
                                62 => "GSM SIM not inserted",
                                63 => "GSM SIM PIN required",
                                64 => "GSM SIM PUK required",
                                65 => "GSM SIM wrong",
                                66 => "InfiniBand mode",
                                67 => "Dependency failed",
                                68 => "BR2684 failed",
                                69 => "Modem manager unavailable",
                                70 => "SSID not found",
                                71 => "Secondary connection failed",
                                72 => "DCB or FCoE setup failed",
                                73 => "Teamd control failed",
                                74 => "Modem failed or no longer available",
                                75 => "Modem now ready and available",
                                76 => "SIM PIN was incorrect",
                                77 => "New connection activation enqueued",
                                78 => "Parent device unreachable",
                                79 => "Parent device changed",
                                _ => "Unknown reason",
                            };
                            eprintln!("Failure details: {reason_str}");
                        }

                        return Err(zbus::Error::Failure(
                            "Connection failed - authentication or network issue".into(),
                        ));
                    } else if state == 50 {
                        config_state_count += 1;
                        if config_state_count > 15 {
                            eprintln!(
                                "✗ Connection stuck in Config state - likely authentication failure"
                            );
                            return Err(zbus::Error::Failure(
                                "Connection failed - authentication failed".into(),
                            ));
                        }
                    } else {
                        config_state_count = 0;
                    }

                    if i > 10 && state == 30 {
                        eprintln!("✗ Connection stuck in disconnected state");
                        return Err(zbus::Error::Failure(
                            "Connection failed - stuck in disconnected state".into(),
                        ));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to check device state: {e}");
                    break;
                }
            }
        }

        if !connected {
            let final_state = dev_proxy.state().await.unwrap_or(0);
            eprintln!("✗ Connection did not complete. Final state: {final_state}");
            if final_state == 50 {
                return Err(zbus::Error::Failure(
                    "Connection failed - authentication failed".into(),
                ));
            }
            return Err(zbus::Error::Failure(
                "Connection failed - timeout waiting for activation".into(),
            ));
        }

        eprintln!("---Connection request for '{ssid}' submitted successfully---");

        Ok(())
    }

    pub async fn wifi_enabled(&self) -> Result<bool> {
        let nm = NMProxy::new(&self.conn).await?;
        nm.wireless_enabled().await
    }

    pub async fn set_wifi_enabled(&self, value: bool) -> zbus::Result<()> {
        let nm = NMProxy::new(&self.conn).await?;
        nm.set_wireless_enabled(value).await
    }

    pub async fn wait_for_wifi_ready(&self) -> Result<()> {
        for _ in 0..20 {
            let devices = self.list_devices().await?;
            for dev in devices {
                if dev.device_type == DeviceType::Wifi
                    && (dev.state == DeviceState::Disconnected
                        || dev.state == DeviceState::Activated)
                {
                    return Ok(());
                }
            }
            futures_timer::Delay::new(Duration::from_secs(1)).await;
        }

        Err(zbus::Error::Failure(
            "Wi-Fi device never became ready".into(),
        ))
    }

    pub async fn scan_networks(&self) -> zbus::Result<()> {
        let nm = NMProxy::new(&self.conn).await?;
        let devices = nm.get_devices().await?;

        for dp in devices {
            let d_proxy = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;

            if d_proxy.device_type().await? != 2 {
                continue;
            }

            let wifi = NMWirelessProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;

            let opts = std::collections::HashMap::new();
            wifi.request_scan(opts).await?;
        }

        Ok(())
    }

    pub async fn current_ssid(&self) -> Option<String> {
        // find active Wi-Fi device
        let nm = NMProxy::new(&self.conn).await.ok()?;
        let devices = nm.get_devices().await.ok()?;

        for dp in devices {
            let dev = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())
                .ok()?
                .build()
                .await
                .ok()?;
            if dev.device_type().await.ok()? != 2 {
                continue;
            }

            let wifi = NMWirelessProxy::builder(&self.conn)
                .path(dp.clone())
                .ok()?
                .build()
                .await
                .ok()?;
            if let Ok(active_ap) = wifi.active_access_point().await
                && active_ap.as_str() != "/"
            {
                let builder = NMAccessPointProxy::builder(&self.conn)
                    .path(active_ap)
                    .ok()?;
                let ap = builder.build().await.ok()?;
                let ssid_bytes = ap.ssid().await.ok()?;
                let ssid = std::str::from_utf8(&ssid_bytes).ok()?;
                return Some(ssid.to_string());
            }
        }
        None
    }

    pub async fn current_connection_info(&self) -> Option<(String, Option<u32>)> {
        let nm = NMProxy::new(&self.conn).await.ok()?;
        let devices = nm.get_devices().await.ok()?;

        for dp in devices {
            let dev = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())
                .ok()?
                .build()
                .await
                .ok()?;
            if dev.device_type().await.ok()? != 2 {
                continue;
            }

            let wifi = NMWirelessProxy::builder(&self.conn)
                .path(dp.clone())
                .ok()?
                .build()
                .await
                .ok()?;
            if let Ok(active_ap) = wifi.active_access_point().await
                && active_ap.as_str() != "/"
            {
                let builder = NMAccessPointProxy::builder(&self.conn)
                    .path(active_ap)
                    .ok()?;
                let ap = builder.build().await.ok()?;
                let ssid_bytes = ap.ssid().await.ok()?;
                let ssid = std::str::from_utf8(&ssid_bytes).ok()?;
                let frequency = ap.frequency().await.ok();
                return Some((ssid.to_string(), frequency));
            }
        }
        None
    }

    pub async fn show_details(&self, net: &Network) -> zbus::Result<NetworkInfo> {
        let nm = NMProxy::new(&self.conn).await?;
        for dp in nm.get_devices().await? {
            let dev = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;
            if dev.device_type().await? != 2 {
                continue;
            }

            let wifi = NMWirelessProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;

            for ap_path in wifi.get_all_access_points().await? {
                let ap = NMAccessPointProxy::builder(&self.conn)
                    .path(ap_path.clone())?
                    .build()
                    .await?;

                let ssid_bytes = ap.ssid().await?;
                if std::str::from_utf8(&ssid_bytes).unwrap_or("") == net.ssid {
                    let strength = net.strength.unwrap_or(0);
                    let bssid = ap.hw_address().await?;
                    let flags = ap.flags().await?;
                    let wpa_flags = ap.wpa_flags().await?;
                    let rsn_flags = ap.rsn_flags().await?;
                    let freq = ap.frequency().await.ok();
                    let max_br = ap.max_bitrate().await.ok();
                    let mode_raw = ap.mode().await.ok();

                    let wep = (flags & 0x1) != 0 && wpa_flags == 0 && rsn_flags == 0;
                    let wpa1 = wpa_flags != 0;
                    let wpa2_or_3 = rsn_flags != 0;
                    let psk = ((wpa_flags | rsn_flags) & 0x0100) != 0;
                    let eap = ((wpa_flags | rsn_flags) & 0x0200) != 0;

                    let mut parts = Vec::new();
                    if wep {
                        parts.push("WEP");
                    }
                    if wpa1 {
                        parts.push("WPA");
                    }
                    if wpa2_or_3 {
                        parts.push("WPA2/WPA3");
                    }
                    if psk {
                        parts.push("PSK");
                    }
                    if eap {
                        parts.push("802.1X");
                    }

                    let security = if parts.is_empty() {
                        "Open".to_string()
                    } else {
                        parts.join(" + ")
                    };

                    let active_ssid = self.current_ssid().await;
                    let status = if active_ssid.as_deref() == Some(&net.ssid) {
                        "Connected".to_string()
                    } else {
                        "Disconnected".to_string()
                    };

                    let channel = freq.and_then(NetworkManager::channel_from_freq);
                    let rate_mbps = max_br.map(|kbit| kbit / 1000);
                    let bars = NetworkManager::bars_from_strength(strength).to_string();
                    let mode = mode_raw
                        .map(NetworkManager::mode_to_string)
                        .unwrap_or("Unknown")
                        .to_string();

                    return Ok(NetworkInfo {
                        ssid: net.ssid.clone(),
                        bssid,
                        strength,
                        freq,
                        channel,
                        mode,
                        rate_mbps,
                        bars,
                        security,
                        status,
                    });
                }
            }
        }
        Err(zbus::Error::Failure("Network not found".into()))
    }

    fn channel_from_freq(mhz: u32) -> Option<u16> {
        match mhz {
            2412..=2472 => Some(((mhz - 2412) / 5 + 1) as u16), // ch 1..13
            2484 => Some(14),
            5000..=5900 => Some(((mhz - 5000) / 5) as u16), // common 5 GHz mapping
            5955..=7115 => Some(((mhz - 5955) / 5 + 1) as u16), // 6 GHz ch 1..233
            _ => None,
        }
    }

    fn bars_from_strength(s: u8) -> &'static str {
        match s {
            0..=24 => "▂___",
            25..=49 => "▂▄__",
            50..=74 => "▂▄▆_",
            _ => "▂▄▆█",
        }
    }

    fn mode_to_string(m: u32) -> &'static str {
        match m {
            1 => "Adhoc",
            2 => "Infra",
            3 => "AP",
            _ => "Unknown",
        }
    }

    pub async fn has_saved_connection(&self, ssid: &str) -> zbus::Result<bool> {
        self.get_saved_connection_path(ssid)
            .await
            .map(|p| p.is_some())
    }

    pub async fn get_saved_connection_path(
        &self,
        ssid: &str,
    ) -> zbus::Result<Option<OwnedObjectPath>> {
        use std::collections::HashMap;
        use zvariant::{OwnedObjectPath, Value};

        let settings = zbus::proxy::Proxy::new(
            &self.conn,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager/Settings",
            "org.freedesktop.NetworkManager.Settings",
        )
        .await?;

        let reply = settings.call_method("ListConnections", &()).await?;
        let conns: Vec<OwnedObjectPath> = reply.body().deserialize()?;

        for cpath in conns {
            let cproxy = zbus::proxy::Proxy::new(
                &self.conn,
                "org.freedesktop.NetworkManager",
                cpath.as_str(),
                "org.freedesktop.NetworkManager.Settings.Connection",
            )
            .await?;

            let msg = cproxy.call_method("GetSettings", &()).await?;
            let body = msg.body();
            let all: HashMap<String, HashMap<String, Value>> = body.deserialize()?;

            if let Some(conn_section) = all.get("connection")
                && let Some(Value::Str(id)) = conn_section.get("id")
                && id == ssid
            {
                return Ok(Some(cpath));
            }
        }

        Ok(None)
    }

    #[deprecated(note = "Use get_saved_connection_path instead")]
    async fn _old_has_saved_connection(&self, ssid: &str) -> zbus::Result<bool> {
        use std::collections::HashMap;
        use zvariant::{OwnedObjectPath, Value};

        let settings = zbus::proxy::Proxy::new(
            &self.conn,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager/Settings",
            "org.freedesktop.NetworkManager.Settings",
        )
        .await?;

        let reply = settings.call_method("ListConnections", &()).await?;
        let conns: Vec<OwnedObjectPath> = reply.body().deserialize()?;

        for cpath in conns {
            let cproxy = zbus::proxy::Proxy::new(
                &self.conn,
                "org.freedesktop.NetworkManager",
                cpath.as_str(),
                "org.freedesktop.NetworkManager.Settings.Connection",
            )
            .await?;

            let msg = cproxy.call_method("GetSettings", &()).await?;
            let body = msg.body();
            let all: HashMap<String, HashMap<String, Value>> = body.deserialize()?;

            if let Some(conn_section) = all.get("connection")
                && let Some(Value::Str(id)) = conn_section.get("id")
                && id == ssid
                && let Some(Value::Bool(ac)) = conn_section.get("autoconnect")
            {
                eprintln!("autoconnect: {ac}");
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn forget(&self, ssid: &str) -> zbus::Result<()> {
        use std::collections::HashMap;
        use zvariant::{OwnedObjectPath, Value};

        eprintln!("Starting forget operation for: {ssid}");

        let nm = NMProxy::new(&self.conn).await?;

        let devices = nm.get_devices().await?;
        for dev_path in &devices {
            let dev = NMDeviceProxy::builder(&self.conn)
                .path(dev_path.clone())?
                .build()
                .await?;
            if dev.device_type().await? != 2 {
                continue;
            }

            let wifi = NMWirelessProxy::builder(&self.conn)
                .path(dev_path.clone())?
                .build()
                .await?;
            if let Ok(ap_path) = wifi.active_access_point().await
                && ap_path.as_str() != "/"
            {
                let ap = NMAccessPointProxy::builder(&self.conn)
                    .path(ap_path.clone())?
                    .build()
                    .await?;
                if let Ok(bytes) = ap.ssid().await
                    && std::str::from_utf8(&bytes).ok() == Some(ssid)
                {
                    eprintln!("Disconnecting from active network: {ssid}");
                    let dev_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(&self.conn)
                        .destination("org.freedesktop.NetworkManager")?
                        .path(dev_path.clone())?
                        .interface("org.freedesktop.NetworkManager.Device")?
                        .build()
                        .await?;

                    match dev_proxy.call_method("Disconnect", &()).await {
                        Ok(_) => eprintln!("Disconnect call succeeded"),
                        Err(e) => eprintln!("Disconnect call failed: {e}"),
                    }

                    eprintln!("About to enter wait loop...");
                    for i in 0..20 {
                        Delay::new(Duration::from_millis(200)).await;
                        match dev.state().await {
                            Ok(current_state) => {
                                eprintln!("Wait loop {i}: device state = {current_state}");
                                if current_state == 30 || current_state == 20 {
                                    eprintln!("Device reached disconnected state");
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get device state in wait loop {i}: {e}");
                                break;
                            }
                        }
                    }
                    eprintln!("Wait loop completed");
                }
            }
        }

        eprintln!("Starting connection deletion phase...");

        let settings: zbus::Proxy<'_> = zbus::proxy::Builder::new(&self.conn)
            .destination("org.freedesktop.NetworkManager")?
            .path("/org/freedesktop/NetworkManager/Settings")?
            .interface("org.freedesktop.NetworkManager.Settings")?
            .build()
            .await?;

        let list_reply = settings.call_method("ListConnections", &()).await?;
        let conns: Vec<OwnedObjectPath> = list_reply.body().deserialize()?;

        let mut deleted_count = 0;

        for cpath in conns {
            let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(&self.conn)
                .destination("org.freedesktop.NetworkManager")?
                .path(cpath.clone())?
                .interface("org.freedesktop.NetworkManager.Settings.Connection")?
                .build()
                .await?;

            if let Ok(msg) = cproxy.call_method("GetSettings", &()).await {
                let body = msg.body();
                let settings_map: HashMap<String, HashMap<String, Value>> = body.deserialize()?;

                let mut should_delete = false;

                if let Some(conn_sec) = settings_map.get("connection")
                    && let Some(Value::Str(id)) = conn_sec.get("id")
                    && id.as_str() == ssid
                {
                    should_delete = true;
                    eprintln!("Found connection by ID: {id}");
                }

                if let Some(wifi_sec) = settings_map.get("802-11-wireless")
                    && let Some(Value::Array(arr)) = wifi_sec.get("ssid")
                {
                    let mut raw = Vec::new();
                    for v in arr.iter() {
                        if let Ok(b) = u8::try_from(v.clone()) {
                            raw.push(b);
                        }
                    }
                    if std::str::from_utf8(&raw).ok() == Some(ssid) {
                        should_delete = true;
                        eprintln!("Found connection by SSID match");
                    }
                }

                if let Some(wsec) = settings_map.get("802-11-wireless-security") {
                    let missing_psk = !wsec.contains_key("psk");
                    let empty_psk = matches!(wsec.get("psk"), Some(Value::Str(s)) if s.is_empty());

                    if (missing_psk || empty_psk) && should_delete {
                        eprintln!("Connection has missing/empty PSK, will delete");
                    }
                }

                if should_delete {
                    match cproxy.call_method("Delete", &()).await {
                        Ok(_) => {
                            deleted_count += 1;
                            eprintln!("Deleted connection: {}", cpath.as_str());
                        }
                        Err(e) => {
                            eprintln!("Failed to delete connection {}: {}", cpath.as_str(), e);
                        }
                    }
                }
            }
        }

        if deleted_count > 0 {
            eprintln!("Successfully deleted {deleted_count} connection(s) for '{ssid}'");
            Ok(())
        } else {
            eprintln!("No saved connections found for '{ssid}'");
            Err(zbus::Error::Failure(format!(
                "No saved connection for {ssid}"
            )))
        }
    }

    async fn delete_connection(&self, conn_path: OwnedObjectPath) -> zbus::Result<()> {
        let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(&self.conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(conn_path.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await?;

        cproxy.call_method("Delete", &()).await?;
        eprintln!("Deleted connection: {}", conn_path.as_str());
        Ok(())
    }
}
