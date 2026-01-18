//! WiFi connection builder with type-safe API.
//!
//! Provides a fluent builder interface for constructing WiFi connection settings
//! with support for different security modes (Open, WPA-PSK, WPA-EAP).

use std::collections::HashMap;
use zvariant::Value;

use super::connection_builder::ConnectionBuilder;
use crate::api::models::{self, ConnectionOptions, EapMethod};

/// WiFi band selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiBand {
    /// 2.4 GHz band
    Bg,
    /// 5 GHz band
    A,
}

/// Builder for WiFi (802.11) connections.
///
/// This builder provides a type-safe, ergonomic API for creating WiFi connection
/// settings. It wraps `ConnectionBuilder` and adds WiFi-specific configuration.
///
/// # Examples
///
/// ## Open Network
///
/// ```rust
/// use nmrs::builders::WifiConnectionBuilder;
///
/// let settings = WifiConnectionBuilder::new("CoffeeShop-WiFi")
///     .open()
///     .autoconnect(true)
///     .build();
/// ```
///
/// ## WPA-PSK (Personal)
///
/// ```rust
/// use nmrs::builders::WifiConnectionBuilder;
///
/// let settings = WifiConnectionBuilder::new("HomeNetwork")
///     .wpa_psk("my_secure_password")
///     .autoconnect(true)
///     .autoconnect_priority(10)
///     .build();
/// ```
///
/// ## WPA-EAP (Enterprise)
///
/// ```rust
/// use nmrs::builders::WifiConnectionBuilder;
/// use nmrs::{EapOptions, EapMethod, Phase2};
///
/// let eap_opts = EapOptions::new("user@company.com", "password")
///     .with_domain_suffix_match("company.com")
///     .with_system_ca_certs(true)
///     .with_method(EapMethod::Peap)
///     .with_phase2(Phase2::Mschapv2);
///
/// let settings = WifiConnectionBuilder::new("CorpNetwork")
///     .wpa_eap(eap_opts)
///     .autoconnect(false)
///     .build();
/// ```
pub struct WifiConnectionBuilder {
    inner: ConnectionBuilder,
    ssid: String,
    security_configured: bool,
    hidden: Option<bool>,
    band: Option<WifiBand>,
    bssid: Option<String>,
}

impl WifiConnectionBuilder {
    /// Creates a new WiFi connection builder for the specified SSID.
    ///
    /// By default, the connection is configured as an open network. Use
    /// `.wpa_psk()` or `.wpa_eap()` to add security.
    pub fn new(ssid: impl Into<String>) -> Self {
        let ssid = ssid.into();
        let inner = ConnectionBuilder::new("802-11-wireless", &ssid);

        Self {
            inner,
            ssid,
            security_configured: false,
            hidden: None,
            band: None,
            bssid: None,
        }
    }

    /// Configures this as an open (unsecured) network.
    ///
    /// This is the default, but can be called explicitly for clarity.
    pub fn open(self) -> Self {
        // Open networks don't need a security section
        Self {
            security_configured: true,
            ..self
        }
    }

    /// Configures WPA-PSK (Personal) security with the given passphrase.
    ///
    /// Uses WPA2 (RSN) with CCMP encryption.
    pub fn wpa_psk(mut self, psk: impl Into<String>) -> Self {
        let mut security = HashMap::new();
        security.insert("key-mgmt", Value::from("wpa-psk"));
        security.insert("psk", Value::from(psk.into()));
        security.insert("psk-flags", Value::from(0u32));
        security.insert("auth-alg", Value::from("open"));

        // Enforce WPA2 with AES
        security.insert("proto", Self::string_array(&["rsn"]));
        security.insert("pairwise", Self::string_array(&["ccmp"]));
        security.insert("group", Self::string_array(&["ccmp"]));

        self.inner = self
            .inner
            .with_section("802-11-wireless-security", security);
        self.security_configured = true;
        self
    }

    /// Configures WPA-EAP (Enterprise) security with 802.1X authentication.
    ///
    /// Supports PEAP and TTLS methods with various inner authentication protocols.
    pub fn wpa_eap(mut self, opts: models::EapOptions) -> Self {
        let mut security = HashMap::new();
        security.insert("key-mgmt", Value::from("wpa-eap"));
        security.insert("auth-alg", Value::from("open"));

        self.inner = self
            .inner
            .with_section("802-11-wireless-security", security);

        // Build 802.1x section
        let mut e1x = HashMap::new();

        let eap_str = match opts.method {
            EapMethod::Peap => "peap",
            EapMethod::Ttls => "ttls",
        };
        e1x.insert("eap", Self::string_array(&[eap_str]));
        e1x.insert("identity", Value::from(opts.identity));
        e1x.insert("password", Value::from(opts.password));

        if let Some(ai) = opts.anonymous_identity {
            e1x.insert("anonymous-identity", Value::from(ai));
        }

        let p2 = match opts.phase2 {
            models::Phase2::Mschapv2 => "mschapv2",
            models::Phase2::Pap => "pap",
        };
        e1x.insert("phase2-auth", Value::from(p2));

        if opts.system_ca_certs {
            e1x.insert("system-ca-certs", Value::from(true));
        }
        if let Some(cert) = opts.ca_cert_path {
            e1x.insert("ca-cert", Value::from(cert));
        }
        if let Some(dom) = opts.domain_suffix_match {
            e1x.insert("domain-suffix-match", Value::from(dom));
        }

        self.inner = self.inner.with_section("802-1x", e1x);
        self.security_configured = true;
        self
    }

    /// Marks this network as hidden (doesn't broadcast SSID).
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Restricts connection to a specific WiFi band.
    pub fn band(mut self, band: WifiBand) -> Self {
        self.band = Some(band);
        self
    }

    /// Restricts connection to a specific access point by BSSID (MAC address).
    ///
    /// Format: "00:11:22:33:44:55"
    pub fn bssid(mut self, bssid: impl Into<String>) -> Self {
        self.bssid = Some(bssid.into());
        self
    }

    // Delegation methods to inner ConnectionBuilder

    /// Applies connection options (autoconnect settings).
    pub fn options(mut self, opts: &ConnectionOptions) -> Self {
        self.inner = self.inner.options(opts);
        self
    }

    /// Enables or disables automatic connection.
    pub fn autoconnect(mut self, enabled: bool) -> Self {
        self.inner = self.inner.autoconnect(enabled);
        self
    }

    /// Sets autoconnect priority (higher values preferred).
    pub fn autoconnect_priority(mut self, priority: i32) -> Self {
        self.inner = self.inner.autoconnect_priority(priority);
        self
    }

    /// Sets autoconnect retry limit.
    pub fn autoconnect_retries(mut self, retries: i32) -> Self {
        self.inner = self.inner.autoconnect_retries(retries);
        self
    }

    /// Configures IPv4 to use DHCP.
    pub fn ipv4_auto(mut self) -> Self {
        self.inner = self.inner.ipv4_auto();
        self
    }

    /// Configures IPv6 to use SLAAC/DHCPv6.
    pub fn ipv6_auto(mut self) -> Self {
        self.inner = self.inner.ipv6_auto();
        self
    }

    /// Disables IPv6.
    pub fn ipv6_ignore(mut self) -> Self {
        self.inner = self.inner.ipv6_ignore();
        self
    }

    /// Builds the final connection settings dictionary.
    ///
    /// This method adds the WiFi-specific "802-11-wireless" section and links
    /// it to the security section if configured.
    pub fn build(mut self) -> HashMap<&'static str, HashMap<&'static str, Value<'static>>> {
        // Build the 802-11-wireless section
        let mut wireless = HashMap::new();
        wireless.insert("ssid", Value::from(self.ssid.as_bytes().to_vec()));
        wireless.insert("mode", Value::from("infrastructure"));

        // Add optional WiFi settings
        if let Some(hidden) = self.hidden {
            wireless.insert("hidden", Value::from(hidden));
        }

        if let Some(band) = self.band {
            let band_str = match band {
                WifiBand::Bg => "bg",
                WifiBand::A => "a",
            };
            wireless.insert("band", Value::from(band_str));
        }

        if let Some(bssid) = self.bssid {
            wireless.insert("bssid", Value::from(bssid));
        }

        // Link to security section if security is configured (not open)
        if self.security_configured && !self.ssid.is_empty() {
            // Check if we actually have a security section (not just open)
            // Open networks don't have the security section
            wireless.insert("security", Value::from("802-11-wireless-security"));
        }

        self.inner = self.inner.with_section("802-11-wireless", wireless);

        self.inner.build()
    }

    // Helper functions

    fn string_array(xs: &[&str]) -> Value<'static> {
        let vals: Vec<String> = xs.iter().map(|s| s.to_string()).collect();
        Value::from(vals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EapOptions, Phase2};

    #[test]
    fn builds_open_wifi() {
        let settings = WifiConnectionBuilder::new("OpenNetwork")
            .open()
            .autoconnect(true)
            .ipv4_auto()
            .ipv6_auto()
            .build();

        assert!(settings.contains_key("connection"));
        assert!(settings.contains_key("802-11-wireless"));
        assert!(settings.contains_key("ipv4"));
        assert!(settings.contains_key("ipv6"));
        assert!(!settings.contains_key("802-11-wireless-security"));

        let wireless = settings.get("802-11-wireless").unwrap();
        assert_eq!(
            wireless.get("ssid"),
            Some(&Value::from(b"OpenNetwork".to_vec()))
        );
    }

    #[test]
    fn builds_wpa_psk_wifi() {
        let settings = WifiConnectionBuilder::new("SecureNet")
            .wpa_psk("password123")
            .ipv4_auto()
            .ipv6_auto()
            .build();

        assert!(settings.contains_key("802-11-wireless-security"));

        let security = settings.get("802-11-wireless-security").unwrap();
        assert_eq!(security.get("key-mgmt"), Some(&Value::from("wpa-psk")));
        assert_eq!(
            security.get("psk"),
            Some(&Value::from("password123".to_string()))
        );

        let wireless = settings.get("802-11-wireless").unwrap();
        assert_eq!(
            wireless.get("security"),
            Some(&Value::from("802-11-wireless-security"))
        );
    }

    #[test]
    fn builds_wpa_eap_wifi() {
        let eap_opts = EapOptions {
            identity: "user@example.com".into(),
            password: "secret".into(),
            anonymous_identity: Some("anon@example.com".into()),
            domain_suffix_match: Some("example.com".into()),
            ca_cert_path: None,
            system_ca_certs: true,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        };

        let settings = WifiConnectionBuilder::new("Enterprise")
            .wpa_eap(eap_opts)
            .autoconnect(false)
            .ipv4_auto()
            .ipv6_auto()
            .build();

        assert!(settings.contains_key("802-11-wireless-security"));
        assert!(settings.contains_key("802-1x"));

        let security = settings.get("802-11-wireless-security").unwrap();
        assert_eq!(security.get("key-mgmt"), Some(&Value::from("wpa-eap")));

        let e1x = settings.get("802-1x").unwrap();
        assert_eq!(
            e1x.get("identity"),
            Some(&Value::from("user@example.com".to_string()))
        );
        assert_eq!(e1x.get("phase2-auth"), Some(&Value::from("mschapv2")));
    }

    #[test]
    fn configures_hidden_network() {
        let settings = WifiConnectionBuilder::new("HiddenSSID")
            .open()
            .hidden(true)
            .ipv4_auto()
            .build();

        let wireless = settings.get("802-11-wireless").unwrap();
        assert_eq!(wireless.get("hidden"), Some(&Value::from(true)));
    }

    #[test]
    fn configures_specific_band() {
        let settings = WifiConnectionBuilder::new("5GHz-Only")
            .open()
            .band(WifiBand::A)
            .ipv4_auto()
            .build();

        let wireless = settings.get("802-11-wireless").unwrap();
        assert_eq!(wireless.get("band"), Some(&Value::from("a")));
    }

    #[test]
    fn configures_bssid() {
        let settings = WifiConnectionBuilder::new("SpecificAP")
            .open()
            .bssid("00:11:22:33:44:55")
            .ipv4_auto()
            .build();

        let wireless = settings.get("802-11-wireless").unwrap();
        assert_eq!(
            wireless.get("bssid"),
            Some(&Value::from("00:11:22:33:44:55"))
        );
    }

    #[test]
    fn applies_connection_options() {
        let opts = ConnectionOptions {
            autoconnect: false,
            autoconnect_priority: Some(5),
            autoconnect_retries: Some(3),
        };

        let settings = WifiConnectionBuilder::new("TestNet")
            .open()
            .options(&opts)
            .ipv4_auto()
            .build();

        let conn = settings.get("connection").unwrap();
        assert_eq!(conn.get("autoconnect"), Some(&Value::from(false)));
        assert_eq!(conn.get("autoconnect-priority"), Some(&Value::from(5i32)));
    }
}
