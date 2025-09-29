use crate::models::{Device, Network};
use std::collections::HashMap;
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
trait NM {
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NMWireless {
    fn get_all_access_points(&self) -> Result<Vec<OwnedObjectPath>>;
    fn request_scan(&self, options: HashMap<String, zvariant::Value<'_>>) -> Result<()>;
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

        let mut networks = Vec::new();

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

            let opts = HashMap::new();
            let _ = wifi.request_scan(opts).await;

            // TODO: Subscribe or sleep

            for ap_path in wifi.get_all_access_points().await? {
                let ap = NMAccessPointProxy::builder(&self.conn)
                    .path(ap_path.clone())?
                    .build()
                    .await?;
                let ssid_bytes = ap.ssid().await?;
                let ssid = String::from_utf8_lossy(&ssid_bytes).into_owned();
                let strength = ap.strength().await?;
                let bssid = ap.hw_address().await?;

                networks.push(Network {
                    device: dp.to_string(),
                    ssid,
                    bssid: Some(bssid),
                    strength: Some(strength),
                });
            }
        }
        Ok(networks)
    }

    pub async fn connect(&self, _ssid: &str, _password: &str) -> Result<()> {
        // TODO: implement AddAndActivateConnection
        Ok(())
    }
}
