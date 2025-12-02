use std::collections::HashMap;
use zbus::{Connection, Result};

use crate::constants::{device_type, security_flags};
use crate::models::Network;
use crate::proxies::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::utils::{decode_ssid_or_hidden, strength_or_zero};

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

pub(crate) async fn list_networks(conn: &Connection) -> Result<Vec<Network>> {
    let nm = NMProxy::new(conn).await?;
    let devices = nm.get_devices().await?;

    let mut networks: HashMap<(String, u32), Network> = HashMap::new();

    for dp in devices {
        let d_proxy = NMDeviceProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;
        if d_proxy.device_type().await? != 2 {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;

        for ap_path in wifi.get_all_access_points().await? {
            let ap = NMAccessPointProxy::builder(conn)
                .path(ap_path.clone())?
                .build()
                .await?;
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

            let new_net = Network {
                device: dp.to_string(),
                ssid: ssid.clone(),
                bssid: Some(bssid),
                strength: Some(strength),
                frequency: Some(frequency),
                secured,
                is_psk,
                is_eap,
            };

            // Use (SSID, frequency) as key to separate 2.4GHz and 5GHz
            networks
                .entry((ssid.clone(), frequency))
                .and_modify(|n| {
                    if strength > strength_or_zero(n.strength) {
                        *n = new_net.clone();
                    }
                    if new_net.secured {
                        n.secured = true;
                    }
                })
                .or_insert(new_net);
        }
    }

    let result: Vec<Network> = networks.into_values().collect();
    Ok(result)
}
