use zbus::Connection;
use crate::models::{Network, ConnectionError};

pub struct NetworkManager {
    conn: Connection,
}

impl NetworkManager {
    pub async fn new() -> zbus::Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    pub async fn list_networks(&self) -> Result<Vec<Network>, ConnectionError> {
        // TODO: query NetworkManager via D-Bus
        Ok(vec![])
    }

    pub async fn connect(&self, _ssid: &str, _password: &str) -> Result<(), ConnectionError> {
        // TODO: implement AddAndActivateConnection
        Ok(())
    }
}
