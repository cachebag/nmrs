use crate::constants::{frequency, signal_strength, wifi_mode};

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

pub(crate) fn bars_from_strength(s: u8) -> &'static str {
    match s {
        0..=signal_strength::BAR_1_MAX => "▂___",
        signal_strength::BAR_2_MIN..=signal_strength::BAR_2_MAX => "▂▄__",
        signal_strength::BAR_3_MIN..=signal_strength::BAR_3_MAX => "▂▄▆_",
        _ => "▂▄▆█",
    }
}

pub(crate) fn mode_to_string(m: u32) -> &'static str {
    match m {
        wifi_mode::ADHOC => "Adhoc",
        wifi_mode::INFRA => "Infra",
        wifi_mode::AP => "AP",
        _ => "Unknown",
    }
}
