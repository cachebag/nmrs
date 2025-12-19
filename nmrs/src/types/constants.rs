//! Constants for NetworkManager D-Bus interface values.
//!
//! These constants correspond to the numeric codes used by NetworkManager's
//! D-Bus API for device types, states, security flags, and other values.

/// NetworkManager device type constants.
pub mod device_type {
    pub const ETHERNET: u32 = 1;
    pub const WIFI: u32 = 2;
    pub const BLUETOOTH: u32 = 5;
    // pub const WIFI_P2P: u32 = 30;
    // pub const LOOPBACK: u32 = 32;
}

/// NetworkManager device state constants
pub mod device_state {
    pub const UNAVAILABLE: u32 = 20;
    pub const DISCONNECTED: u32 = 30;
    pub const ACTIVATED: u32 = 100;
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

/// Timeout constants for signal-based waiting.
///
/// These timeouts are used with D-Bus signal monitoring instead of polling.
/// They define how long to wait for state transitions before giving up.
pub mod timeouts {
    use std::time::Duration;

    /// Maximum time to wait for Wi-Fi device to become ready (60 seconds).
    ///
    /// Used after enabling Wi-Fi to wait for the hardware to initialize.
    const WIFI_READY_TIMEOUT_SECS: u64 = 60;

    /// Time to wait after requesting a scan before checking results (2 seconds).
    ///
    /// While we could use signals for scan completion, a short delay is
    /// sufficient for now, and simpler for most use cases.
    const SCAN_WAIT_SECS: u64 = 2;

    /// Brief delay after state transitions to allow NetworkManager to stabilize.
    const STABILIZATION_DELAY_MS: u64 = 100;

    /// Returns the Wi-Fi ready timeout duration.
    pub fn wifi_ready_timeout() -> Duration {
        Duration::from_secs(WIFI_READY_TIMEOUT_SECS)
    }

    /// Returns the scan wait duration.
    pub fn scan_wait() -> Duration {
        Duration::from_secs(SCAN_WAIT_SECS)
    }

    /// Returns a brief stabilization delay.
    pub fn stabilization_delay() -> Duration {
        Duration::from_millis(STABILIZATION_DELAY_MS)
    }
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
    pub const BAND_5_START: u32 = 5150;
    pub const BAND_5_END: u32 = 5925;
    pub const BAND_6_START: u32 = 5955;
    pub const BAND_6_END: u32 = 7115;
    pub const CHANNEL_SPACING: u32 = 5;
}

/// Rate conversion constants
pub mod rate {
    pub const KBIT_TO_MBPS: u32 = 1000;
}
