pub(crate) fn channel_from_freq(mhz: u32) -> Option<u16> {
    match mhz {
        2412..=2472 => Some(((mhz - 2412) / 5 + 1) as u16), // ch 1..13
        2484 => Some(14),
        5000..=5900 => Some(((mhz - 5000) / 5) as u16), // common 5 GHz mapping
        5955..=7115 => Some(((mhz - 5955) / 5 + 1) as u16), // 6 GHz ch 1..233
        _ => None,
    }
}

pub(crate) fn bars_from_strength(s: u8) -> &'static str {
    match s {
        0..=24 => "▂___",
        25..=49 => "▂▄__",
        50..=74 => "▂▄▆_",
        _ => "▂▄▆█",
    }
}

pub(crate) fn mode_to_string(m: u32) -> &'static str {
    match m {
        1 => "Adhoc",
        2 => "Infra",
        3 => "AP",
        _ => "Unknown",
    }
}
