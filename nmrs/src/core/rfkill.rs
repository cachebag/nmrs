//! Kernel rfkill state reader via sysfs.
//!
//! Reads `/sys/class/rfkill/*/type` and `/sys/class/rfkill/*/hard` to detect
//! hardware radio kill switches. This is a fallback for cases where
//! NetworkManager's `*HardwareEnabled` properties disagree with the kernel.

use std::fs;
use std::path::Path;

/// Snapshot of hardware (hard-block) rfkill state for each radio type.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RfkillSnapshot {
    /// `true` if any WLAN rfkill entry reports a hard block.
    pub wlan_hard_block: bool,
    /// `true` if any WWAN rfkill entry reports a hard block.
    pub wwan_hard_block: bool,
    /// `true` if any Bluetooth rfkill entry reports a hard block.
    pub bluetooth_hard_block: bool,
}

/// Reads the current rfkill hardware-block state from sysfs.
///
/// Returns an all-false snapshot if `/sys/class/rfkill` is unreadable
/// (common in containers and CI environments).
pub(crate) fn read_rfkill() -> RfkillSnapshot {
    let rfkill_dir = Path::new("/sys/class/rfkill");

    let entries = match fs::read_dir(rfkill_dir) {
        Ok(e) => e,
        Err(_) => return RfkillSnapshot::default(),
    };

    let mut snapshot = RfkillSnapshot::default();

    for entry in entries.flatten() {
        let path = entry.path();

        let type_str = match fs::read_to_string(path.join("type")) {
            Ok(s) => s.trim().to_string(),
            Err(_) => continue,
        };

        let hard_blocked = match fs::read_to_string(path.join("hard")) {
            Ok(s) => s.trim() == "1",
            Err(_) => false,
        };

        if hard_blocked {
            match type_str.as_str() {
                "wlan" => snapshot.wlan_hard_block = true,
                "wwan" => snapshot.wwan_hard_block = true,
                "bluetooth" => snapshot.bluetooth_hard_block = true,
                _ => {}
            }
        }
    }

    snapshot
}
