//! Bluetooth connection management module.
//!
//! This module provides functions to create and manage Bluetooth network connections
//! using NetworkManager's D-Bus API. It includes builders for Bluetooth PAN (Personal Area
//! Network) connections and DUN (Dial-Up Networking) connections.
//!
//! # Usage
//!
//! Most users should use the high-level [`NetworkManager`](crate::NetworkManager) API
//! instead of calling these builders directly. These are exposed for advanced use cases
//! where you need fine-grained control over connection settings.
//!
//! # Example
//!
//! ```rust
//! use nmrs::builders::build_bluetooth_connection;
//! use nmrs::models::BluetoothSettings;
//!
//! let bt_settings = BluetoothSettings {
//!    bdaddr: "00:1A:7D:DA:71:13".into(),
//!    bt_device_type: "pan".into(),
//! };
//! ```

use std::collections::HashMap;
use zvariant::Value;

use crate::{models::BluetoothSettings, ConnectionOptions};

/// Builds the `connection` section with type, id, uuid, and autoconnect settings.
pub fn base_connection_section(
    name: &str,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("type", Value::from("bluetooth"));
    s.insert("id", Value::from(name.to_string()));
    s.insert("uuid", Value::from(uuid::Uuid::new_v4().to_string()));
    s.insert("autoconnect", Value::from(opts.autoconnect));

    if let Some(p) = opts.autoconnect_priority {
        s.insert("autoconnect-priority", Value::from(p));
    }

    if let Some(r) = opts.autoconnect_retries {
        s.insert("autoconnect-retries", Value::from(r));
    }

    s
}

/// Builds a Bluetooth connection settings dictionary.
fn bluetooth_section(settings: &BluetoothSettings) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("bdaddr", Value::from(settings.bdaddr.clone()));
    s.insert("type", Value::from(settings.bt_device_type.clone()));
    s
}

pub fn build_bluetooth_connection(
    name: &str,
    settings: &BluetoothSettings,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, HashMap<&'static str, Value<'static>>> {
    let mut conn: HashMap<&'static str, HashMap<&'static str, Value<'static>>> = HashMap::new();

    // Base connections
    conn.insert("connection", base_connection_section(name, opts));
    conn.insert("bluetooth", bluetooth_section(settings));

    let mut ipv4 = HashMap::new();
    ipv4.insert("method", Value::from("auto"));
    conn.insert("ipv4", ipv4);

    let mut ipv6 = HashMap::new();
    ipv6.insert("method", Value::from("auto"));
    conn.insert("ipv6", ipv6);

    conn
}
