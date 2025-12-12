//! Utility functions for Wi-Fi data conversion and display.
//!
//! Provides helpers for converting between Wi-Fi data representations:
//! frequency to channel, signal strength to visual bars, SSID bytes to strings.

use log::warn;
use std::str;

use crate::constants::{frequency, signal_strength, wifi_mode};

/// Converts a Wi-Fi frequency in MHz to a channel number.
///
/// Supports 2.4GHz (channels 1-14), 5GHz, and 6GHz bands.
/// Returns `None` for frequencies outside known Wi-Fi bands.
pub(crate) fn channel_from_freq(mhz: u32) -> Option<u16> {
    match mhz {
        frequency::BAND_2_4_START..=frequency::BAND_2_4_END => {
            Some(((mhz - frequency::BAND_2_4_START) / frequency::CHANNEL_SPACING + 1) as u16)
        }
        frequency::BAND_2_4_CH14 => Some(14),
        frequency::BAND_5_START..=frequency::BAND_5_END => {
            Some(((mhz - frequency::BAND_5_START) / frequency::CHANNEL_SPACING) as u16)
        }
        frequency::BAND_6_START..=frequency::BAND_6_END => {
            Some(((mhz - frequency::BAND_6_START) / frequency::CHANNEL_SPACING + 1) as u16)
        }
        _ => None,
    }
}

/// Converts signal strength (0-100) to a visual bar representation.
///
/// Returns a 4-character string using Unicode block characters:
/// - 0-24%:   `▂___` (1 bar)
/// - 25-49%:  `▂▄__` (2 bars)
/// - 50-74%:  `▂▄▆_` (3 bars)
/// - 75-100%: `▂▄▆█` (4 bars)
pub(crate) fn bars_from_strength(s: u8) -> &'static str {
    match s {
        0..=signal_strength::BAR_1_MAX => "▂___",
        signal_strength::BAR_2_MIN..=signal_strength::BAR_2_MAX => "▂▄__",
        signal_strength::BAR_3_MIN..=signal_strength::BAR_3_MAX => "▂▄▆_",
        _ => "▂▄▆█",
    }
}

/// Converts a Wi-Fi mode code to a human-readable string.
///
/// Mode codes: 1 = Ad-hoc, 2 = Infrastructure, 3 = Access Point.
pub(crate) fn mode_to_string(m: u32) -> &'static str {
    match m {
        wifi_mode::ADHOC => "Adhoc",
        wifi_mode::INFRA => "Infra",
        wifi_mode::AP => "AP",
        _ => "Unknown",
    }
}

/// Decode SSID bytes, defaulting to "<Hidden Network>" if empty or invalid UTF-8.
/// This is safer than unwrap_or and logs the error.
pub(crate) fn decode_ssid_or_hidden(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    str::from_utf8(bytes)
        .map(|s| s.to_string())
        .unwrap_or_else(|e| {
            warn!("Invalid UTF-8 in SSID during comparison: {e}");
            String::new()
        })
}

/// Decode SSID bytes for comparison purposes, defaulting to empty string if invalid.
pub(crate) fn decode_ssid_or_empty(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    str::from_utf8(bytes)
        .map(|s| s.to_string())
        .unwrap_or_else(|e| {
            warn!("Invalid UTF-8 in SSID during comparison: {e}");
            String::new()
        })
}

/// Safely get signal strength with a default value.
/// This is safer than unwrap_or(0) as it makes the default explicit.
pub(crate) fn strength_or_zero(strength: Option<u8>) -> u8 {
    strength.unwrap_or(0)
}

/// Macro to convert Result to Option with error logging.
/// Usage: `try_log!(result, "context message")?`
#[macro_export]
macro_rules! try_log {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                log::warn!("{}: {:?}", $context, e);
                return None;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_from_freq_2_4ghz() {
        assert_eq!(channel_from_freq(2412), Some(1));
        assert_eq!(channel_from_freq(2437), Some(6));
        assert_eq!(channel_from_freq(2472), Some(13));
        assert_eq!(channel_from_freq(2484), Some(14));
    }

    #[test]
    fn test_channel_from_freq_5ghz() {
        assert_eq!(channel_from_freq(5180), Some(36));
        assert_eq!(channel_from_freq(5220), Some(44));
        assert_eq!(channel_from_freq(5500), Some(100));
    }

    #[test]
    fn test_channel_from_freq_6ghz() {
        assert_eq!(channel_from_freq(5955), Some(1));
        assert_eq!(channel_from_freq(6115), Some(33));
    }

    #[test]
    fn test_channel_from_freq_invalid() {
        assert_eq!(channel_from_freq(1000), None);
        assert_eq!(channel_from_freq(9999), None);
    }

    #[test]
    fn test_bars_from_strength() {
        assert_eq!(bars_from_strength(0), "▂___");
        assert_eq!(bars_from_strength(24), "▂___");
        assert_eq!(bars_from_strength(25), "▂▄__");
        assert_eq!(bars_from_strength(49), "▂▄__");
        assert_eq!(bars_from_strength(50), "▂▄▆_");
        assert_eq!(bars_from_strength(74), "▂▄▆_");
        assert_eq!(bars_from_strength(75), "▂▄▆█");
        assert_eq!(bars_from_strength(100), "▂▄▆█");
    }

    #[test]
    fn test_mode_to_string() {
        assert_eq!(mode_to_string(1), "Adhoc");
        assert_eq!(mode_to_string(2), "Infra");
        assert_eq!(mode_to_string(3), "AP");
        assert_eq!(mode_to_string(99), "Unknown");
    }

    #[test]
    fn test_decode_ssid_or_hidden() {
        assert_eq!(decode_ssid_or_hidden(b"MyNetwork"), "MyNetwork");
        assert_eq!(decode_ssid_or_hidden(b""), "");
        assert_eq!(decode_ssid_or_hidden(b"Test_SSID-123"), "Test_SSID-123");
    }

    #[test]
    fn test_decode_ssid_or_empty() {
        assert_eq!(decode_ssid_or_empty(b"MyNetwork"), "MyNetwork");
        assert_eq!(decode_ssid_or_empty(b""), "");
        // Test with valid UTF-8
        assert_eq!(decode_ssid_or_empty("café".as_bytes()), "café");
    }

    #[test]
    fn test_strength_or_zero() {
        assert_eq!(strength_or_zero(Some(75)), 75);
        assert_eq!(strength_or_zero(Some(0)), 0);
        assert_eq!(strength_or_zero(Some(100)), 100);
        assert_eq!(strength_or_zero(None), 0);
    }
}
