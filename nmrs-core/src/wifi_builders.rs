use models::ConnectionOptions;
use std::collections::HashMap;
use zvariant::Value;

use crate::models::{self, EapMethod};

fn bytes(val: &str) -> Vec<u8> {
    val.as_bytes().to_vec()
}

fn string_array(xs: &[&str]) -> Value<'static> {
    let vals: Vec<String> = xs.iter().map(|s| s.to_string()).collect();
    Value::from(vals)
}

fn base_wifi_section(ssid: &str) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("ssid", Value::from(bytes(ssid)));
    s.insert("mode", Value::from("infrastructure"));
    s
}

fn base_connection_section(
    ssid: &str,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("type", Value::from("802-11-wireless"));
    s.insert("id", Value::from(ssid.to_string()));
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

fn build_psk_security(psk: &str) -> HashMap<&'static str, Value<'static>> {
    let mut sec = HashMap::new();

    sec.insert("key-mgmt", Value::from("wpa-psk"));
    sec.insert("psk", Value::from(psk.to_string()));
    sec.insert("psk-flags", Value::from(0u32)); // 0 = agent-owned, provided during activation
    sec.insert("auth-alg", Value::from("open"));

    sec.insert("proto", string_array(&["rsn"]));
    sec.insert("pairwise", string_array(&["ccmp"]));
    sec.insert("group", string_array(&["ccmp"]));

    sec
}

fn build_eap_security(
    opts: &models::EapOptions,
) -> (
    HashMap<&'static str, Value<'static>>,
    HashMap<&'static str, Value<'static>>,
) {
    let mut sec = HashMap::new();
    sec.insert("key-mgmt", Value::from("wpa-eap"));
    sec.insert("auth-alg", Value::from("open"));
    // same hardening tips as psk
    // proto, pairwise, group, etc.

    // 802-1x
    let mut e1x = HashMap::new();

    let eap_str = match opts.method {
        EapMethod::Peap => "peap",
        EapMethod::Ttls => "ttls",
    };
    e1x.insert("eap", string_array(&[eap_str]));
    e1x.insert("identity", Value::from(opts.identity.clone()));
    e1x.insert("password", Value::from(opts.password.clone()));

    if let Some(ai) = &opts.anonymous_identity {
        e1x.insert("anonymous-identity", Value::from(ai.clone()));
    }

    // Phase 2
    let p2 = match opts.phase2 {
        models::Phase2::Mschapv2 => "mschapv2",
        models::Phase2::Pap => "pap",
    };
    e1x.insert("phase2-auth", Value::from(p2));

    // CA handling
    // Note that sometimes Uni's don't require certs from students to connect
    if opts.system_ca_certs {
        e1x.insert("system-ca-certs", Value::from(true));
    }
    if let Some(cert) = &opts.ca_cert_path {
        // must be a file:// URL
        e1x.insert("ca-cert", Value::from(cert.clone()));
    }
    if let Some(dom) = &opts.domain_suffix_match {
        e1x.insert("domain-suffix-match", Value::from(dom.clone()));
    }

    (sec, e1x)
}

pub fn build_wifi_connection(
    ssid: &str,
    security: &models::WifiSecurity,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, HashMap<&'static str, Value<'static>>> {
    let mut conn: HashMap<&'static str, HashMap<&'static str, Value<'static>>> = HashMap::new();

    // base connections
    conn.insert("connection", base_connection_section(ssid, opts));
    conn.insert("802-11-wireless", base_wifi_section(ssid));

    // Add IPv4 and IPv6 configuration to prevent state 60 stall
    // TODO: Expand upon auto/manual configuration options
    let mut ipv4 = HashMap::new();
    ipv4.insert("method", Value::from("auto"));
    conn.insert("ipv4", ipv4);

    let mut ipv6 = HashMap::new();
    ipv6.insert("method", Value::from("auto"));
    conn.insert("ipv6", ipv6);

    match security {
        models::WifiSecurity::Open => {}

        models::WifiSecurity::WpaPsk { psk } => {
            // point wireless at security section
            if let Some(w) = conn.get_mut("802-11-wireless") {
                w.insert("security", Value::from("802-11-wireless-security"));
            }

            let sec = build_psk_security(psk);
            conn.insert("802-11-wireless-security", sec);
        }

        models::WifiSecurity::WpaEap { opts } => {
            if let Some(w) = conn.get_mut("802-11-wireless") {
                w.insert("security", Value::from("802-11-wireless-security"));
            }

            let (mut sec, e1x) = build_eap_security(opts);
            sec.insert("auth-alg", Value::from("open"));
            conn.insert("802-11-wireless-security", sec);
            conn.insert("802-1x", e1x);
        }
    }

    conn
}
