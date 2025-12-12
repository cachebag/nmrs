//! Wi-Fi network scanning and enumeration.
//!
//! Provides functions to trigger Wi-Fi scans and list visible networks
//! with their properties (SSID, signal strength, security type).

use std::collections::HashMap;
use zbus::Connection;

use crate::Result;
use crate::constants::{device_type, security_flags};
use crate::models::Network;
use crate::proxies::{NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::utils::{decode_ssid_or_hidden, for_each_access_point};

/// Triggers a Wi-Fi scan on all wireless devices.
///
/// Requests NetworkManager to scan for available networks. The scan
/// runs asynchronously; call `list_networks` after a delay to see results.
pub(crate) async fn scan_networks(conn: &Connection) -> Result<()> {
    let nm = NMProxy::new(conn).await?;
    let devices = nm.get_devices().await?;

    for dp in devices {
        let d_proxy = NMDeviceProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;

        if d_proxy.device_type().await? != device_type::WIFI {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;

        let opts = std::collections::HashMap::new();
        wifi.request_scan(opts).await?;
    }

    Ok(())
}

/// Lists all visible Wi-Fi networks.
///
/// Enumerates access points from all Wi-Fi devices and returns a deduplicated
/// list of networks. Networks are keyed by (SSID, frequency) to distinguish
/// 2.4GHz and 5GHz bands of the same network.
///
/// When multiple access points share the same SSID and frequency (e.g., mesh
/// networks), the one with the strongest signal is returned.
pub(crate) async fn list_networks(conn: &Connection) -> Result<Vec<Network>> {
    let mut networks: HashMap<(String, u32), Network> = HashMap::new();

    let all_networks = for_each_access_point(conn, |ap| {
        Box::pin(async move {
            let ssid_bytes = ap.ssid().await?;
            let ssid = decode_ssid_or_hidden(&ssid_bytes);
            let strength = ap.strength().await?;
            let bssid = ap.hw_address().await?;
            let flags = ap.flags().await?;
            let wpa = ap.wpa_flags().await?;
            let rsn = ap.rsn_flags().await?;
            let frequency = ap.frequency().await?;

            let secured = (flags & security_flags::WEP) != 0 || wpa != 0 || rsn != 0;
            let is_psk = (wpa & security_flags::PSK) != 0 || (rsn & security_flags::PSK) != 0;
            let is_eap = (wpa & security_flags::EAP) != 0 || (rsn & security_flags::EAP) != 0;

            let network = Network {
                device: String::new(),
                ssid: ssid.clone(),
                bssid: Some(bssid),
                strength: Some(strength),
                frequency: Some(frequency),
                secured,
                is_psk,
                is_eap,
            };

            Ok(Some((ssid, frequency, network)))
        })
    })
    .await?;

    // Deduplicate: use (SSID, frequency) as key, keep strongest signal
    for (ssid, frequency, new_net) in all_networks {
        let strength = new_net.strength.unwrap_or(0);
        networks
            .entry((ssid, frequency))
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

    Ok(networks.into_values().collect())
}
