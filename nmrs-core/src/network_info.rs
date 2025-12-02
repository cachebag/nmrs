use zbus::{Connection, Result};

use crate::constants::{device_type, rate, security_flags};
use crate::models::{Network, NetworkInfo};
use crate::proxies::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::utils::{bars_from_strength, channel_from_freq, mode_to_string};

pub(crate) async fn show_details(conn: &Connection, net: &Network) -> Result<NetworkInfo> {
    let nm = NMProxy::new(conn).await?;
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

        for ap_path in wifi.get_all_access_points().await? {
            let ap = NMAccessPointProxy::builder(conn)
                .path(ap_path.clone())?
                .build()
                .await?;

            let ssid_bytes = ap.ssid().await?;
            if std::str::from_utf8(&ssid_bytes).unwrap_or("") == net.ssid {
                let strength = net.strength.unwrap_or(0);
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

                let active_ssid = current_ssid(conn).await;
                let status = if active_ssid.as_deref() == Some(&net.ssid) {
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
    Err(zbus::Error::Failure("Network not found".into()))
}

pub(crate) async fn current_ssid(conn: &Connection) -> Option<String> {
    // find active Wi-Fi device
    let nm = NMProxy::new(conn).await.ok()?;
    let devices = nm.get_devices().await.ok()?;

    for dp in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dp.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        if dev.device_type().await.ok()? != 2 {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dp.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        if let Ok(active_ap) = wifi.active_access_point().await
            && active_ap.as_str() != "/"
        {
            let builder = NMAccessPointProxy::builder(conn).path(active_ap).ok()?;
            let ap = builder.build().await.ok()?;
            let ssid_bytes = ap.ssid().await.ok()?;
            let ssid = std::str::from_utf8(&ssid_bytes).ok()?;
            return Some(ssid.to_string());
        }
    }
    None
}

pub(crate) async fn current_connection_info(conn: &Connection) -> Option<(String, Option<u32>)> {
    let nm = NMProxy::new(conn).await.ok()?;
    let devices = nm.get_devices().await.ok()?;

    for dp in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dp.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        if dev.device_type().await.ok()? != 2 {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dp.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        if let Ok(active_ap) = wifi.active_access_point().await
            && active_ap.as_str() != "/"
        {
            let builder = NMAccessPointProxy::builder(conn).path(active_ap).ok()?;
            let ap = builder.build().await.ok()?;
            let ssid_bytes = ap.ssid().await.ok()?;
            let ssid = std::str::from_utf8(&ssid_bytes).ok()?;
            let frequency = ap.frequency().await.ok();
            return Some((ssid.to_string(), frequency));
        }
    }
    None
}
