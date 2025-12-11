use futures_timer::Delay;
use zbus::Result;

use crate::constants::{device_state, retries, timeouts};
use crate::proxies::NMDeviceProxy;

pub async fn wait_for_connection_state(dev: &NMDeviceProxy<'_>) -> Result<()> {
    let mut config_stuck = 0;

    for i in 0..retries::CONNECTION_MAX_RETRIES {
        Delay::new(timeouts::connection_poll_interval()).await;

        let raw = dev.state().await?;

        if raw == device_state::ACTIVATED {
            return Ok(());
        }

        if raw == device_state::FAILED {
            if let Ok((_, code)) = dev.state_reason().await {
                let reason = decode_reason(code);
                return Err(zbus::Error::Failure(format!(
                    "connection failed: reason={reason}"
                )));
            }
            return Err(zbus::Error::Failure("connection failed".into()));
        }

        if raw == device_state::CONFIG {
            config_stuck += 1;
            if config_stuck > retries::CONNECTION_CONFIG_STUCK_THRESHOLD {
                return Err(zbus::Error::Failure("connection stuck in config".into()));
            }
        } else {
            config_stuck = 0;
        }

        if i > retries::CONNECTION_STUCK_CHECK_START && raw == device_state::DISCONNECTED {
            return Err(zbus::Error::Failure("connection stuck disconnected".into()));
        }
    }

    let final_state = dev.state().await.unwrap_or(0);
    Err(zbus::Error::Failure(format!(
        "timeout waiting for activation, final_state={final_state}"
    )))
}

fn decode_reason(code: u32) -> &'static str {
    match code {
        0 => "Unknown",
        1 => "None",
        2 => "User disconnected",
        3 => "Device disconnected",
        4 => "Carrier changed",
        7 => "Supplicant disconnected",
        8 => "Supplicant config failed",
        9 => "Supplicant failed",
        10 => "Supplicant timeout",
        11 => "PPP start failed",
        15 => "DHCP start failed",
        16 => "DHCP error",
        17 => "DHCP failed",
        24 => "Modem connection failed",
        25 => "Modem init failed",
        42 => "Infiniband mode",
        43 => "Dependency failed",
        44 => "BR2684 failed",
        45 => "Mode set failed",
        46 => "GSM APN select failed",
        47 => "GSM not searching",
        48 => "GSM registration denied",
        49 => "GSM registration timeout",
        50 => "GSM registration failed",
        51 => "GSM PIN check failed",
        52 => "Firmware missing",
        53 => "Device removed",
        54 => "Sleeping",
        55 => "Connection removed",
        56 => "User requested",
        57 => "Carrier",
        58 => "Connection assumed",
        59 => "Supplicant available",
        60 => "Modem not found",
        61 => "Bluetooth failed",
        62 => "GSM SIM not inserted",
        63 => "GSM SIM PIN required",
        64 => "GSM SIM PUK required",
        65 => "GSM SIM wrong",
        66 => "InfiniBand mode",
        67 => "Dependency failed",
        68 => "BR2684 failed",
        69 => "Modem manager unavailable",
        70 => "SSID not found",
        71 => "Secondary connection failed",
        72 => "DCB or FCoE setup failed",
        73 => "Teamd control failed",
        74 => "Modem failed or no longer available",
        75 => "Modem now ready and available",
        76 => "SIM PIN was incorrect",
        77 => "New connection activation enqueued",
        78 => "Parent device unreachable",
        79 => "Parent device changed",
        _ => "Unknown reason",
    }
}
