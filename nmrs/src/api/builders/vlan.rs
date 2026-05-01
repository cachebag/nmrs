//! VLAN (802.1Q) connection builder.
//!
//! This module provides functions to create VLAN connection settings
//! for NetworkManager.

use std::collections::HashMap;
use zvariant::Value;

use crate::ConnectionOptions;
use crate::api::models::{ConnectionError, VlanConfig};

/// Builds a VLAN connection settings dictionary for NetworkManager.
///
/// Creates all necessary settings sections for a VLAN connection including
/// the connection metadata, VLAN-specific settings, and IP configuration.
///
/// # Arguments
///
/// * `config` - VLAN configuration
/// * `opts` - Connection options (autoconnect, priority, etc.)
///
/// # Errors
///
/// Returns `ConnectionError::InvalidVlanId` if the VLAN ID is out of range.
/// Returns `ConnectionError::InvalidInput` if the parent interface is empty.
///
/// # Examples
///
/// ```rust
/// use nmrs::builders::build_vlan_connection;
/// use nmrs::{VlanConfig, ConnectionOptions};
///
/// let config = VlanConfig::new("eth0", 100)
///     .with_connection_name("Office VLAN");
/// let opts = ConnectionOptions::new(true);
///
/// let settings = build_vlan_connection(&config, &opts).unwrap();
/// ```
pub fn build_vlan_connection(
    config: &VlanConfig,
    opts: &ConnectionOptions,
) -> Result<HashMap<&'static str, HashMap<&'static str, Value<'static>>>, ConnectionError> {
    config.validate()?;

    let mut conn: HashMap<&'static str, HashMap<&'static str, Value<'static>>> = HashMap::new();

    // Connection section
    conn.insert("connection", connection_section(config, opts));

    // VLAN section
    conn.insert("vlan", vlan_section(config));

    // IPv4 section (auto by default)
    let mut ipv4 = HashMap::new();
    ipv4.insert("method", Value::from("auto"));
    conn.insert("ipv4", ipv4);

    // IPv6 section (auto by default)
    let mut ipv6 = HashMap::new();
    ipv6.insert("method", Value::from("auto"));
    conn.insert("ipv6", ipv6);

    Ok(conn)
}

fn connection_section(
    config: &VlanConfig,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();

    s.insert("type", Value::from("vlan"));
    s.insert("id", Value::from(config.effective_connection_name()));
    s.insert("uuid", Value::from(uuid::Uuid::new_v4().to_string()));
    s.insert("autoconnect", Value::from(opts.autoconnect));
    s.insert(
        "interface-name",
        Value::from(config.effective_interface_name()),
    );

    if let Some(p) = opts.autoconnect_priority {
        s.insert("autoconnect-priority", Value::from(p));
    }

    if let Some(r) = opts.autoconnect_retries {
        s.insert("autoconnect-retries", Value::from(r));
    }

    s
}

fn vlan_section(config: &VlanConfig) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();

    s.insert("parent", Value::from(config.parent.clone()));
    s.insert("id", Value::from(u32::from(config.id)));

    if let Some(flags) = config.flags {
        s.insert("flags", Value::from(flags));
    }

    if let Some(ref map) = config.ingress_priority_map {
        let entries: Vec<Value<'static>> = map.iter().map(|e| Value::from(e.clone())).collect();
        s.insert("ingress-priority-map", Value::Array(entries.into()));
    }

    if let Some(ref map) = config.egress_priority_map {
        let entries: Vec<Value<'static>> = map.iter().map(|e| Value::from(e.clone())).collect();
        s.insert("egress-priority-map", Value::Array(entries.into()));
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_opts() -> ConnectionOptions {
        ConnectionOptions {
            autoconnect: true,
            autoconnect_priority: Some(10),
            autoconnect_retries: Some(3),
        }
    }

    #[test]
    fn builds_basic_vlan_connection() {
        let config = VlanConfig::new("eth0", 100);
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();

        assert!(conn.contains_key("connection"));
        assert!(conn.contains_key("vlan"));
        assert!(conn.contains_key("ipv4"));
        assert!(conn.contains_key("ipv6"));
    }

    #[test]
    fn connection_section_has_correct_type() {
        let config = VlanConfig::new("eth0", 100);
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let connection = conn.get("connection").unwrap();

        if let Some(Value::Str(t)) = connection.get("type") {
            assert_eq!(t.as_str(), "vlan");
        } else {
            panic!("type field missing or wrong type");
        }
    }

    #[test]
    fn vlan_section_has_parent_and_id() {
        let config = VlanConfig::new("enp3s0", 200);
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let vlan = conn.get("vlan").unwrap();

        if let Some(Value::Str(parent)) = vlan.get("parent") {
            assert_eq!(parent.as_str(), "enp3s0");
        } else {
            panic!("parent field missing or wrong type");
        }

        if let Some(Value::U32(id)) = vlan.get("id") {
            assert_eq!(*id, 200);
        } else {
            panic!("id field missing or wrong type");
        }
    }

    #[test]
    fn uses_default_interface_name() {
        let config = VlanConfig::new("eth0", 100);
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let connection = conn.get("connection").unwrap();

        if let Some(Value::Str(name)) = connection.get("interface-name") {
            assert_eq!(name.as_str(), "eth0.100");
        } else {
            panic!("interface-name field missing or wrong type");
        }
    }

    #[test]
    fn uses_custom_interface_name() {
        let config = VlanConfig::new("eth0", 100).with_interface_name("office-vlan");
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let connection = conn.get("connection").unwrap();

        if let Some(Value::Str(name)) = connection.get("interface-name") {
            assert_eq!(name.as_str(), "office-vlan");
        } else {
            panic!("interface-name field missing or wrong type");
        }
    }

    #[test]
    fn uses_default_connection_name() {
        let config = VlanConfig::new("eth0", 100);
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let connection = conn.get("connection").unwrap();

        if let Some(Value::Str(name)) = connection.get("id") {
            assert_eq!(name.as_str(), "VLAN 100 on eth0");
        } else {
            panic!("id field missing or wrong type");
        }
    }

    #[test]
    fn uses_custom_connection_name() {
        let config = VlanConfig::new("eth0", 100).with_connection_name("Office Network");
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let connection = conn.get("connection").unwrap();

        if let Some(Value::Str(name)) = connection.get("id") {
            assert_eq!(name.as_str(), "Office Network");
        } else {
            panic!("id field missing or wrong type");
        }
    }

    #[test]
    fn includes_vlan_flags() {
        let config = VlanConfig::new("eth0", 100).with_flags(0x5);
        let opts = test_opts();

        let conn = build_vlan_connection(&config, &opts).unwrap();
        let vlan = conn.get("vlan").unwrap();

        if let Some(Value::U32(flags)) = vlan.get("flags") {
            assert_eq!(*flags, 0x5);
        } else {
            panic!("flags field missing or wrong type");
        }
    }

    #[test]
    fn rejects_invalid_vlan_id_zero() {
        let config = VlanConfig::new("eth0", 0);
        let opts = test_opts();

        let result = build_vlan_connection(&config, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_vlan_id_too_high() {
        let config = VlanConfig::new("eth0", 4095);
        let opts = test_opts();

        let result = build_vlan_connection(&config, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_parent() {
        let config = VlanConfig::new("", 100);
        let opts = test_opts();

        let result = build_vlan_connection(&config, &opts);
        assert!(result.is_err());
    }

    #[test]
    fn uuid_is_unique() {
        let config = VlanConfig::new("eth0", 100);
        let opts = test_opts();

        let conn1 = build_vlan_connection(&config, &opts).unwrap();
        let conn2 = build_vlan_connection(&config, &opts).unwrap();

        let uuid1 = conn1
            .get("connection")
            .and_then(|c| c.get("uuid"))
            .map(|v| match v {
                Value::Str(s) => s.as_str(),
                _ => "",
            })
            .unwrap_or("");

        let uuid2 = conn2
            .get("connection")
            .and_then(|c| c.get("uuid"))
            .map(|v| match v {
                Value::Str(s) => s.as_str(),
                _ => "",
            })
            .unwrap_or("");

        assert_ne!(uuid1, uuid2, "UUIDs should be unique");
    }
}
