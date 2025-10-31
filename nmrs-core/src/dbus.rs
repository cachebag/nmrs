use crate::models::{Device, DeviceState, DeviceType, Network};
use crate::wifi_builders::build_wifi_connection;
use futures_timer::Delay;
use std::collections::HashMap;
use std::time::Duration;
use zbus::Connection;
use zbus::Result;
use zbus::proxy;
use zvariant::{ObjectPath, OwnedObjectPath};

pub struct NetworkManager {
    conn: Connection,
    cache: tokio::sync::RwLock<Vec<Network>>,
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
}

impl NetworkManager {
    pub async fn new() -> zbus::Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self {
            conn,
            cache: tokio::sync::RwLock::new(Vec::new()),
        })
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
        if self.cache.read().await.is_empty() {
            Delay::new(Duration::from_millis(800)).await;
        }
        let devices = nm.get_devices().await?;

        let mut networks: HashMap<String, Network> = HashMap::new();

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

                let secured = (flags & 0x1) != 0 || wpa != 0 || rsn != 0;
                let is_psk = (wpa & 0x0100) != 0 || (rsn & 0x0100) != 0;
                let is_eap = (wpa & 0x0200) != 0 || (rsn & 0x0200) != 0;

                // println!("{} → WPA={wpa:#06x} RSN={rsn:#06x} → EAP={is_eap}", ssid);

                let new_net = Network {
                    device: dp.to_string(),
                    ssid: ssid.clone(),
                    bssid: Some(bssid),
                    strength: Some(strength),
                    secured,
                    is_psk,
                    is_eap,
                };

                networks
                    .entry(ssid)
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

        if !result.is_empty() {
            println!("cache updates with {} networks", result.len());
            *self.cache.write().await = result.clone();
        }

        if result.is_empty() {
            println!("using cached results here");
            Ok(self.cache.read().await.clone())
        } else {
            Ok(result)
        }
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
        let devices = nm.get_devices().await?;

        let mut wifi_device: Option<OwnedObjectPath> = None;
        for dp in devices {
            let dev = NMDeviceProxy::builder(&self.conn)
                .path(dp.clone())?
                .build()
                .await?;
            if dev.device_type().await? == 2 {
                wifi_device = Some(dp.clone());
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
            if active == ssid {
                eprintln!("Already connected to {active}, skipping connect()");
                return Ok(());
            } else {
                eprintln!("Currently connected to {active}, disconnecting before reconnecting...");
                if let Ok(conns) = nm.active_connections().await {
                    for conn_path in conns {
                        let _ = nm.deactivate_connection(conn_path).await;
                    }
                }

                for _ in 0..10 {
                    let d = NMDeviceProxy::builder(&self.conn)
                        .path(wifi_device.clone())?
                        .build()
                        .await?;
                    if DeviceState::from(d.state().await?) == DeviceState::Disconnected {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
            }
        }

        let _ = wifi.request_scan(HashMap::new()).await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let mut ap_path: Option<OwnedObjectPath> = None;
        for ap in wifi.get_all_access_points().await? {
            let apx = NMAccessPointProxy::builder(&self.conn)
                .path(ap.clone())?
                .build()
                .await?;
            let ssid_bytes = apx.ssid().await?;
            let ap_ssid = std::str::from_utf8(&ssid_bytes).unwrap_or("");
            if ap_ssid == ssid {
                ap_path = Some(ap.clone());
                break;
            }
        }

        let settings = build_wifi_connection(ssid, &creds);

        if matches!(creds, crate::models::WifiSecurity::Open) {
            println!("Connecting to open network '{ssid}'");
            nm.add_and_activate_connection(
                settings,
                wifi_device.clone(),
                ObjectPath::from_str_unchecked("/").into(),
            )
            .await?;
        } else {
            let specific_object =
                ap_path.unwrap_or_else(|| ObjectPath::from_str_unchecked("/").into());
            nm.add_and_activate_connection(settings, wifi_device.clone(), specific_object)
                .await?;
        }

        println!("Connection request for '{ssid}' submitted successfully");
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
}
