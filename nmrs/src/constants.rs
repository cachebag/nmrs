//! Constants for NetworkManager D-Bus interface values.
//!
//! These constants correspond to the numeric codes used by NetworkManager's
//! D-Bus API for device types, states, security flags, and other values.

/// NetworkManager device type constants.
pub mod device_type {
    // pub const ETHERNET: u32 = 1;
    pub const WIFI: u32 = 2;
    // pub const WIFI_P2P: u32 = 30;
    // pub const LOOPBACK: u32 = 32;
}

/// NetworkManager device state constants
pub mod device_state {
    pub const UNAVAILABLE: u32 = 20;
    pub const DISCONNECTED: u32 = 30;
    pub const CONFIG: u32 = 50;
    pub const ACTIVATED: u32 = 100;
    pub const FAILED: u32 = 120;
}

/// WiFi security flag constants
pub mod security_flags {
    pub const WEP: u32 = 0x1;
    pub const PSK: u32 = 0x0100;
    pub const EAP: u32 = 0x0200;
}

/// WiFi mode constants
pub mod wifi_mode {
    pub const ADHOC: u32 = 1;
    pub const INFRA: u32 = 2;
    pub const AP: u32 = 3;
}

/// Timeout and delay constants (in milliseconds)
pub mod timeouts {
    use std::time::Duration;

    pub const DISCONNECT_POLL_INTERVAL_MS: u64 = 300;
    pub const DISCONNECT_FINAL_DELAY_MS: u64 = 500;
    pub const CONNECTION_POLL_INTERVAL_MS: u64 = 500;
    pub const SCAN_WAIT_SECONDS: u64 = 3;

    pub fn disconnect_poll_interval() -> Duration {
        Duration::from_millis(DISCONNECT_POLL_INTERVAL_MS)
    }

    pub fn disconnect_final_delay() -> Duration {
        Duration::from_millis(DISCONNECT_FINAL_DELAY_MS)
    }

    pub fn connection_poll_interval() -> Duration {
        Duration::from_millis(CONNECTION_POLL_INTERVAL_MS)
    }

    pub fn scan_wait() -> Duration {
        Duration::from_secs(SCAN_WAIT_SECONDS)
    }
}

/// Retry count constants
pub mod retries {
    pub const DISCONNECT_MAX_RETRIES: u32 = 10;
    pub const CONNECTION_MAX_RETRIES: u32 = 40;
    pub const CONNECTION_CONFIG_STUCK_THRESHOLD: u32 = 15;
    pub const CONNECTION_STUCK_CHECK_START: u32 = 10;
    pub const WIFI_READY_MAX_RETRIES: u32 = 20;
}

/// Signal strength thresholds for bar display
pub mod signal_strength {
    pub const BAR_1_MAX: u8 = 24;
    pub const BAR_2_MIN: u8 = BAR_1_MAX + 1;
    pub const BAR_2_MAX: u8 = 49;
    pub const BAR_3_MIN: u8 = BAR_2_MAX + 1;
    pub const BAR_3_MAX: u8 = 74;
}

/// WiFi frequency constants (MHz)
pub mod frequency {
    pub const BAND_2_4_START: u32 = 2412;
    pub const BAND_2_4_END: u32 = 2472;
    pub const BAND_2_4_CH14: u32 = 2484;
    pub const BAND_5_START: u32 = 5000;
    pub const BAND_5_END: u32 = 5900;
    pub const BAND_6_START: u32 = 5955;
    pub const BAND_6_END: u32 = 7115;
    pub const CHANNEL_SPACING: u32 = 5;
}

/// Rate conversion constants
pub mod rate {
    pub const KBIT_TO_MBPS: u32 = 1000;
}
