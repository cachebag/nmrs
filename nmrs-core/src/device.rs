use std::time::Duration;
use zbus::{Connection, Result};

use crate::models::{Device, DeviceState, DeviceType};
use crate::proxies::{NMDeviceProxy, NMProxy};

pub(crate) async fn list_devices(conn: &Connection) -> Result<Vec<Device>> {
    let proxy = NMProxy::new(conn).await?;
    let paths = proxy.get_devices().await?;

    let mut devices = Vec::new();
    for p in paths {
        let d_proxy = NMDeviceProxy::builder(conn)
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

pub(crate) async fn wait_for_wifi_ready(conn: &Connection) -> Result<()> {
    for _ in 0..20 {
        let devices = list_devices(conn).await?;
        for dev in devices {
            if dev.device_type == DeviceType::Wifi
                && (dev.state == DeviceState::Disconnected || dev.state == DeviceState::Activated)
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

pub(crate) async fn set_wifi_enabled(conn: &Connection, value: bool) -> Result<()> {
    let nm = NMProxy::new(conn).await?;
    nm.set_wireless_enabled(value).await
}

pub(crate) async fn wifi_enabled(conn: &Connection) -> Result<bool> {
    let nm = NMProxy::new(conn).await?;
    nm.wireless_enabled().await
}
