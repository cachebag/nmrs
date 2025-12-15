//! Real-time network monitoring using D-Bus signals.
//!
//! Provides functionality to monitor access point changes (additions/removals)
//! in real-time without needing to poll. This enables live UI updates.

use futures::stream::{Stream, StreamExt};
use log::{debug, warn};
use std::pin::Pin;
use zbus::Connection;

use crate::api::models::ConnectionError;
use crate::dbus::{NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::types::constants::device_type;
use crate::Result;

/// Monitors access point changes on all Wi-Fi devices.
///
/// Subscribes to `AccessPointAdded` and `AccessPointRemoved` signals on all
/// wireless devices. When any signal is received, invokes the callback to
/// notify the caller that the network list has changed.
///
/// This function runs indefinitely until an error occurs or the connection
/// is lost. Run it in a background task.
///
/// # Example
///
/// ```ignore
/// let nm = NetworkManager::new().await?;
/// nm.monitor_network_changes(|| {
///     println!("Network list changed, refresh UI!");
/// }).await?;
/// ```
pub async fn monitor_network_changes<F>(conn: &Connection, callback: F) -> Result<()>
where
    F: Fn() + 'static,
{
    let nm = NMProxy::new(conn).await?;
    let devices = nm.get_devices().await?;

    // Use dynamic dispatch to handle different signal stream types
    let mut streams: Vec<Pin<Box<dyn Stream<Item = _>>>> = Vec::new();

    // Subscribe to signals on all Wi-Fi devices
    for dev_path in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dev_path.clone())?
            .build()
            .await?;

        if dev.device_type().await? != device_type::WIFI {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dev_path.clone())?
            .build()
            .await?;

        let added_stream = wifi.receive_access_point_added().await?;
        let removed_stream = wifi.receive_access_point_removed().await?;

        // Box both streams as trait objects
        streams.push(Box::pin(added_stream.map(|_| ())));
        streams.push(Box::pin(removed_stream.map(|_| ())));

        debug!("Subscribed to AP signals on device: {dev_path}");
    }

    if streams.is_empty() {
        warn!("No Wi-Fi devices found to monitor");
        return Err(ConnectionError::NoWifiDevice);
    }

    debug!(
        "Monitoring {} signal streams for network changes",
        streams.len()
    );

    // Merge all streams and listen for any signal
    let mut merged = futures::stream::select_all(streams);

    while let Some(_signal) = merged.next().await {
        debug!("Network change detected");
        callback();
    }

    warn!("Network monitoring stream ended unexpectedly");
    Err(ConnectionError::Stuck("monitoring stream ended".into()))
}
