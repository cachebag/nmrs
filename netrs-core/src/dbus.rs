use crate::models::{Device, Network};
use zbus::Connection;
use zbus::Result;
use zbus::proxy;
use zvariant::OwnedObjectPath;

pub struct NetworkManager {
    conn: Connection,
}

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
        // TODO: query NetworkManager via D-Bus
        Ok(vec![])
    }

    pub async fn connect(&self, _ssid: &str, _password: &str) -> Result<()> {
        // TODO: implement AddAndActivateConnection
        Ok(())
    }
}
