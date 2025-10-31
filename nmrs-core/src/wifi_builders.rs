use std::collections::HashMap;
use zvariant::Value;

use crate::models;

fn bytes(val: &str) -> Vec<u8> {
    val.as_bytes().to_vec()
}

fn base_wifi_section(ssid: &str) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("ssid", Value::from(bytes(ssid)));
    s.insert("mode", Value::from("infrastructure"));
    s
}

fn base_connection_section(ssid: &str) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("type", Value::from("802-11-wireless"));
    s.insert("id", Value::from(ssid.to_string()));
    s.insert("uuid", Value::from(uuid::Uuid::new_v4().to_string()));
    s.insert("autoconnect", Value::from(true));
    s
}

fn build_psk_security(psk: &str) -> HashMap<&'static str, Value<'static>> {
    let mut sec = HashMap::new();
    sec.insert("key-mgmt", Value::from("wpa-psk"));
    sec.insert("psk", Value::from(psk.to_string()));
    // hardening maybe
    // sec.insert("proto", Value::from(vec!["rsn"]));
    // pairwise
    // etc...
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
    sec.insert("auth-alg", Value::from("OPEN"));
    // same hardening tips as psk
    // proto, pairwise, group, etc.

    // 802-1x
    let mut e1x = HashMap::new();
    let eap_vec = match opts.method {
        models::EapMethod::Peap => vec!["peap"],
        models::EapMethod::Ttls => vec!["ttls"],
    };
    e1x.insert("eap", Value::from(eap_vec));
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
) -> HashMap<&'static str, HashMap<&'static str, Value<'static>>> {
    let mut conn: HashMap<&'static str, HashMap<&'static str, Value<'static>>> = HashMap::new();
    conn.insert("connection", base_connection_section(ssid));
    conn.insert("802-11-wireless", base_wifi_section(ssid));

    match security {
        models::WifiSecurity::Open => {}

        models::WifiSecurity::WpaPsk { psk } => {
            conn.insert("802-11-wireless-security", build_psk_security(psk.as_str()));
        }

        models::WifiSecurity::WpaEap { opts } => {
            let (sec, e1x) = build_eap_security(&opts);
            conn.insert("802-11-wireless-security", sec);
            conn.insert("802-1x", e1x);
        }
    }
    conn
}
