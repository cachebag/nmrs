use crate::models::{Device, DeviceState, DeviceType, Network};
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
                let ssid = std::str::from_utf8(&ssid_bytes)
                    .unwrap_or("<Hidden Network>")
                    .to_string();
                let strength = ap.strength().await?;
                let bssid = ap.hw_address().await?;
                let flags = ap.flags().await?;
                let wpa = ap.wpa_flags().await?;
                let rsn = ap.rsn_flags().await?;

                let secured = (flags & 0x1) != 0 || wpa != 0 || rsn != 0;

                let new_net = Network {
                    device: dp.to_string(),
                    ssid: ssid.clone(),
                    bssid: Some(bssid),
                    strength: Some(strength),
                    secured,
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
        Ok(networks.into_values().collect())
    }

    pub async fn connect(&self, _ssid: &str, _password: &str) -> Result<()> {
        // TODO: implement AddAndActivateConnection
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
            // FIXME: longer? shorter? is this even where the issue is?
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
}
