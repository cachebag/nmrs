//! Network information and current connection status.
//!
//! Provides functions to retrieve detailed information about networks
//! and query the current connection state.

use zbus::Connection;

use crate::Result;
use crate::constants::{device_type, rate, security_flags};
use crate::models::{ConnectionError, Network, NetworkInfo};
use crate::proxies::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::try_log;
use crate::utils::{
    bars_from_strength, channel_from_freq, decode_ssid_or_empty, mode_to_string, strength_or_zero,
};

/// Returns detailed information about a network.
///
/// Queries the access point for comprehensive details including:
/// - BSSID (MAC address)
/// - Signal strength and visual bars
/// - Frequency and channel
/// - Wi-Fi mode (infrastructure, adhoc, etc.)
/// - Connection speed (actual if connected, max otherwise)
/// - Security capabilities (WEP, WPA, WPA2, PSK, 802.1X)
/// - Current connection status
pub(crate) async fn show_details(conn: &Connection, net: &Network) -> Result<NetworkInfo> {
    let nm = NMProxy::new(conn).await?;
    let active_ssid = current_ssid(conn).await;
    let is_connected = active_ssid.as_deref() == Some(&net.ssid);

    for dp in nm.get_devices().await? {
        let dev = NMDeviceProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;
        if dev.device_type().await? != device_type::WIFI {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;

        let actual_bitrate = if is_connected {
            wifi.bitrate().await.ok()
        } else {
            None
        };

        let target_ap_path = if is_connected {
            let active_ap = wifi.active_access_point().await?;
            if active_ap.as_str() != "/" {
                Some(active_ap)
            } else {
                None
            }
        } else {
            None
        };

        let ap_paths = if let Some(active_path) = target_ap_path {
            vec![active_path]
        } else {
            wifi.get_all_access_points().await?
        };

        for ap_path in ap_paths {
            let ap = NMAccessPointProxy::builder(conn)
                .path(ap_path.clone())?
                .build()
                .await?;

            let ssid_bytes = ap.ssid().await?;
            if decode_ssid_or_empty(&ssid_bytes) == net.ssid {
                let strength = strength_or_zero(net.strength);
                let bssid = ap.hw_address().await?;
                let flags = ap.flags().await?;
                let wpa_flags = ap.wpa_flags().await?;
                let rsn_flags = ap.rsn_flags().await?;
                let freq = ap.frequency().await.ok();
                let max_br = ap.max_bitrate().await.ok();
                let mode_raw = ap.mode().await.ok();

                let wep = (flags & security_flags::WEP) != 0 && wpa_flags == 0 && rsn_flags == 0;
                let wpa1 = wpa_flags != 0;
                let wpa2_or_3 = rsn_flags != 0;
                let psk = ((wpa_flags | rsn_flags) & security_flags::PSK) != 0;
                let eap = ((wpa_flags | rsn_flags) & security_flags::EAP) != 0;

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

                let status = if is_connected {
                    "Connected".to_string()
                } else {
                    "Disconnected".to_string()
                };

                let channel = freq.and_then(channel_from_freq);
                let rate_mbps = actual_bitrate
                    .or(max_br)
                    .map(|kbit| kbit / rate::KBIT_TO_MBPS);
                let bars = bars_from_strength(strength).to_string();
                let mode = mode_raw
                    .map(mode_to_string)
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
    Err(ConnectionError::NotFound)
}

/// Returns the SSID of the currently connected Wi-Fi network.
///
/// Checks all Wi-Fi devices for an active access point and returns
/// its SSID. Returns `None` if not connected to any Wi-Fi network.
///
/// Uses the `try_log!` macro to gracefully handle errors without
/// propagating them, since this is often used in non-critical contexts.
pub(crate) async fn current_ssid(conn: &Connection) -> Option<String> {
    let nm = try_log!(NMProxy::new(conn).await, "Failed to create NM proxy");
    let devices = try_log!(nm.get_devices().await, "Failed to get devices");

    for dp in devices {
        let dev_builder = try_log!(
            NMDeviceProxy::builder(conn).path(dp.clone()),
            "Failed to create device proxy builder"
        );
        let dev = try_log!(dev_builder.build().await, "Failed to build device proxy");

        let dev_type = try_log!(dev.device_type().await, "Failed to get device type");
        if dev_type != device_type::WIFI {
            continue;
        }

        let wifi_builder = try_log!(
            NMWirelessProxy::builder(conn).path(dp.clone()),
            "Failed to create wireless proxy builder"
        );
        let wifi = try_log!(wifi_builder.build().await, "Failed to build wireless proxy");

        if let Ok(active_ap) = wifi.active_access_point().await
            && active_ap.as_str() != "/"
        {
            let ap_builder = try_log!(
                NMAccessPointProxy::builder(conn).path(active_ap),
                "Failed to create access point proxy builder"
            );
            let ap = try_log!(
                ap_builder.build().await,
                "Failed to build access point proxy"
            );
            let ssid_bytes = try_log!(ap.ssid().await, "Failed to get SSID bytes");
            let ssid = decode_ssid_or_empty(&ssid_bytes);
            return Some(ssid);
        }
    }
    None
}

/// Returns the SSID and frequency of the current Wi-Fi connection.
///
/// Similar to `current_ssid` but also returns the operating frequency
/// in MHz, useful for determining if connected to 2.4GHz or 5GHz band.
pub(crate) async fn current_connection_info(conn: &Connection) -> Option<(String, Option<u32>)> {
    let nm = try_log!(NMProxy::new(conn).await, "Failed to create NM proxy");
    let devices = try_log!(nm.get_devices().await, "Failed to get devices");

    for dp in devices {
        let dev_builder = try_log!(
            NMDeviceProxy::builder(conn).path(dp.clone()),
            "Failed to create device proxy builder"
        );
        let dev = try_log!(dev_builder.build().await, "Failed to build device proxy");

        let dev_type = try_log!(dev.device_type().await, "Failed to get device type");
        if dev_type != device_type::WIFI {
            continue;
        }

        let wifi_builder = try_log!(
            NMWirelessProxy::builder(conn).path(dp.clone()),
            "Failed to create wireless proxy builder"
        );
        let wifi = try_log!(wifi_builder.build().await, "Failed to build wireless proxy");

        if let Ok(active_ap) = wifi.active_access_point().await
            && active_ap.as_str() != "/"
        {
            let ap_builder = try_log!(
                NMAccessPointProxy::builder(conn).path(active_ap),
                "Failed to create access point proxy builder"
            );
            let ap = try_log!(
                ap_builder.build().await,
                "Failed to build access point proxy"
            );
            let ssid_bytes = try_log!(ap.ssid().await, "Failed to get SSID bytes");
            let ssid = decode_ssid_or_empty(&ssid_bytes);
            let frequency = ap.frequency().await.ok();
            return Some((ssid, frequency));
        }
    }
    None
}
