use zbus::{Connection, Result};

use crate::connection::{connect, forget};
use crate::connection_settings::{get_saved_connection_path, has_saved_connection};
use crate::device::{list_devices, set_wifi_enabled, wait_for_wifi_ready, wifi_enabled};
use crate::models::{Device, Network, NetworkInfo, WifiSecurity};
use crate::network_info::{current_connection_info, current_ssid, show_details};
use crate::scan::{list_networks, scan_networks};

pub struct NetworkManager {
    conn: Connection,
}

impl NetworkManager {
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        list_devices(&self.conn).await
    }

    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        list_networks(&self.conn).await
    }

    pub async fn connect(&self, ssid: &str, creds: WifiSecurity) -> Result<()> {
        connect(&self.conn, ssid, creds).await
    }

    pub async fn wifi_enabled(&self) -> Result<bool> {
        wifi_enabled(&self.conn).await
    }

    pub async fn set_wifi_enabled(&self, value: bool) -> Result<()> {
        set_wifi_enabled(&self.conn, value).await
    }

    pub async fn wait_for_wifi_ready(&self) -> Result<()> {
        wait_for_wifi_ready(&self.conn).await
    }

    pub async fn scan_networks(&self) -> Result<()> {
        scan_networks(&self.conn).await
    }

    pub async fn current_ssid(&self) -> Option<String> {
        current_ssid(&self.conn).await
    }

    pub async fn current_connection_info(&self) -> Option<(String, Option<u32>)> {
        current_connection_info(&self.conn).await
    }

    pub async fn show_details(&self, net: &Network) -> Result<NetworkInfo> {
        show_details(&self.conn, net).await
    }

    pub async fn has_saved_connection(&self, ssid: &str) -> Result<bool> {
        has_saved_connection(&self.conn, ssid).await
    }

    pub async fn get_saved_connection_path(
        &self,
        ssid: &str,
    ) -> Result<Option<zvariant::OwnedObjectPath>> {
        get_saved_connection_path(&self.conn, ssid).await
    }

    pub async fn forget(&self, ssid: &str) -> Result<()> {
        forget(&self.conn, ssid).await
    }
}
