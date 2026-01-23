//! Wi-Fi network scanning and enumeration.
//!
//! Provides functions to trigger Wi-Fi scans and list visible networks
//! with their properties (SSID, signal strength, security type).

use std::collections::HashMap;
use zbus::Connection;

use crate::api::models::Network;
use crate::dbus::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::monitoring::info::current_ssid;
use crate::types::constants::{device_type, security_flags};
use crate::util::utils::{
    decode_ssid_or_empty, decode_ssid_or_hidden, for_each_access_point,
    get_ip_addresses_from_active_connection,
};
use crate::Result;

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
                ssid: ssid.to_string(),
                bssid: Some(bssid),
                strength: Some(strength),
                frequency: Some(frequency),
                secured,
                is_psk,
                is_eap,
                ip4_address: None,
                ip6_address: None,
            };

            Ok(Some((ssid, frequency, network)))
        })
    })
    .await?;

    // Deduplicate: use (SSID, frequency) as key, keep strongest signal
    for (ssid, frequency, new_net) in all_networks {
        networks
            .entry((ssid.to_string(), frequency))
            .and_modify(|n| n.merge_ap(&new_net))
            .or_insert(new_net);
    }

    Ok(networks.into_values().collect())
}

/// Returns the full Network object for the currently connected WiFi network.
///
/// Returns `None` if not connected to any WiFi network.
pub(crate) async fn current_network(conn: &Connection) -> Result<Option<Network>> {
    // Get current SSID
    let current_ssid = match current_ssid(conn).await {
        Some(ssid) => ssid,
        None => return Ok(None),
    };

    // Find the WiFi device and active access point
    let nm = NMProxy::new(conn).await?;
    let devices = nm.get_devices().await?;

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

        let ap_path = wifi.active_access_point().await?;
        if ap_path.as_str() == "/" {
            continue;
        }

        let ap = NMAccessPointProxy::builder(conn)
            .path(ap_path)?
            .build()
            .await?;

        let ssid_bytes = ap.ssid().await?;
        let ssid = decode_ssid_or_empty(&ssid_bytes);

        if ssid != current_ssid {
            continue;
        }

        // Found the active AP, build Network object
        let strength = ap.strength().await?;
        let bssid = ap.hw_address().await?;
        let flags = ap.flags().await?;
        let wpa = ap.wpa_flags().await?;
        let rsn = ap.rsn_flags().await?;
        let frequency = ap.frequency().await?;

        let secured = (flags & security_flags::WEP) != 0 || wpa != 0 || rsn != 0;
        let is_psk = (wpa & security_flags::PSK) != 0 || (rsn & security_flags::PSK) != 0;
        let is_eap = (wpa & security_flags::EAP) != 0 || (rsn & security_flags::EAP) != 0;

        let interface = dev.interface().await.unwrap_or_default();

        // Get IP addresses from active connection
        let (ip4_address, ip6_address) = if let Ok(active_conn_path) = dev.active_connection().await
        {
            if active_conn_path.as_str() != "/" {
                get_ip_addresses_from_active_connection(conn, &active_conn_path).await
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        return Ok(Some(Network {
            device: interface,
            ssid: ssid.to_string(),
            bssid: Some(bssid),
            strength: Some(strength),
            frequency: Some(frequency),
            secured,
            is_psk,
            is_eap,
            ip4_address,
            ip6_address,
        }));
    }

    Ok(None)
}
