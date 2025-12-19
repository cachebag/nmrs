//! Network information and detailed network status.
//!
//! Provides functions to retrieve detailed information about WiFi networks,
//! including security capabilities, signal strength, and connection details.

use zbus::Connection;

use crate::api::models::{ConnectionError, Network, NetworkInfo};
use crate::monitoring::wifi::current_ssid;
use crate::types::constants::{rate, security_flags};
use crate::util::utils::{
    bars_from_strength, channel_from_freq, decode_ssid_or_empty, for_each_access_point,
    mode_to_string, strength_or_zero,
};
use crate::Result;

/// Returns detailed information about a WiFi network.
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
    let active_ssid = current_ssid(conn).await;
    let is_connected_outer = active_ssid.as_deref() == Some(&net.ssid);
    let target_ssid_outer = net.ssid.clone();
    let target_strength = net.strength;

    let results = for_each_access_point(conn, |ap| {
        let target_ssid = target_ssid_outer.clone();
        let is_connected = is_connected_outer;
        Box::pin(async move {
            let ssid_bytes = ap.ssid().await?;
            if decode_ssid_or_empty(&ssid_bytes) != target_ssid {
                return Ok(None);
            }

            let strength = strength_or_zero(target_strength);
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
            let rate_mbps = max_br.map(|kbit| kbit / rate::KBIT_TO_MBPS);
            let bars = bars_from_strength(strength).to_string();
            let mode = mode_raw
                .map(mode_to_string)
                .unwrap_or("Unknown")
                .to_string();

            Ok(Some(NetworkInfo {
                ssid: target_ssid,
                bssid,
                strength,
                freq,
                channel,
                mode,
                rate_mbps,
                bars,
                security,
                status,
            }))
        })
    })
    .await?;

    results.into_iter().next().ok_or(ConnectionError::NotFound)
}
