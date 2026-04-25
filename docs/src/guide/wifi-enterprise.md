# WPA-EAP (Enterprise)

WPA-EAP (802.1X) is used by corporate and university networks that require individual user credentials rather than a shared password. nmrs supports PEAP and EAP-TTLS with configurable inner authentication methods.

## Quick Start

```rust
use nmrs::{NetworkManager, WifiSecurity, EapOptions, EapMethod, Phase2};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let eap = EapOptions::new("user@company.com", "my_password")
        .with_method(EapMethod::Peap)
        .with_phase2(Phase2::Mschapv2);

    nm.connect("CorpWiFi", None, WifiSecurity::WpaEap { opts: eap }).await?;

    println!("Connected to enterprise WiFi!");
    Ok(())
}
```

## EAP Methods

nmrs supports two outer EAP methods:

| Method | Description | Common Use |
|--------|-------------|------------|
| `EapMethod::Peap` | Protected EAP — tunnels inner auth in TLS | Corporate networks |
| `EapMethod::Ttls` | Tunneled TLS — flexible inner auth | Universities, ISPs |

## Phase 2 (Inner Authentication)

The inner authentication runs inside the TLS tunnel established by the outer method:

| Method | Description | Typical Pairing |
|--------|-------------|-----------------|
| `Phase2::Mschapv2` | MS-CHAPv2 — challenge-response | PEAP |
| `Phase2::Pap` | PAP — plaintext (protected by TLS tunnel) | TTLS |

## Building EAP Options

### Direct Construction

```rust
use nmrs::{EapOptions, EapMethod, Phase2};

let eap = EapOptions::new("user@company.com", "password")
    .with_method(EapMethod::Peap)
    .with_phase2(Phase2::Mschapv2)
    .with_anonymous_identity("anonymous@company.com")
    .with_domain_suffix_match("company.com")
    .with_system_ca_certs(true);
```

### Builder Pattern

For complex configurations, the builder pattern makes each option explicit:

```rust
use nmrs::{EapOptions, EapMethod, Phase2};

let eap = EapOptions::builder()
    .identity("user@company.com")
    .password("my_password")
    .method(EapMethod::Peap)
    .phase2(Phase2::Mschapv2)
    .anonymous_identity("anonymous@company.com")
    .domain_suffix_match("company.com")
    .system_ca_certs(true)
    .build();
```

## Configuration Reference

| Option | Required | Description |
|--------|----------|-------------|
| `identity` | Yes | Username (usually email) |
| `password` | Yes | User password |
| `method` | Yes | Outer EAP method (PEAP or TTLS) |
| `phase2` | Yes | Inner authentication (MSCHAPv2 or PAP) |
| `anonymous_identity` | No | Outer identity for privacy (sent in the clear) |
| `domain_suffix_match` | No | Verify server certificate domain |
| `ca_cert_path` | No | Path to CA certificate (`file://` URL) |
| `system_ca_certs` | No | Use system CA store (default: `false`) |

## Certificate Validation

For security, you should validate the authentication server's certificate. There are two approaches:

### System CA Certificates

Use the operating system's trusted certificate store:

```rust
let eap = EapOptions::new("user@company.com", "password")
    .with_system_ca_certs(true)
    .with_domain_suffix_match("company.com")
    .with_method(EapMethod::Peap)
    .with_phase2(Phase2::Mschapv2);
```

### Custom CA Certificate

Point to a specific CA certificate file:

```rust
let eap = EapOptions::new("user@company.com", "password")
    .with_ca_cert_path("file:///etc/ssl/certs/company-ca.pem")
    .with_domain_suffix_match("company.com")
    .with_method(EapMethod::Peap)
    .with_phase2(Phase2::Mschapv2);
```

> **Security:** Without certificate validation, your connection is vulnerable to evil-twin attacks. Always configure either `system_ca_certs` or `ca_cert_path` in production.

## Common Configurations

### Corporate PEAP/MSCHAPv2

The most common enterprise setup:

```rust
let eap = EapOptions::new("employee@corp.com", "password")
    .with_method(EapMethod::Peap)
    .with_phase2(Phase2::Mschapv2)
    .with_anonymous_identity("anonymous@corp.com")
    .with_domain_suffix_match("corp.com")
    .with_system_ca_certs(true);
```

### University EAP-TTLS/PAP

Common at educational institutions using eduroam:

```rust
let eap = EapOptions::new("student@university.edu", "password")
    .with_method(EapMethod::Ttls)
    .with_phase2(Phase2::Pap)
    .with_ca_cert_path("file:///etc/ssl/certs/university-ca.pem")
    .with_domain_suffix_match("university.edu");
```

## Full Example

```rust
use nmrs::{NetworkManager, WifiSecurity, EapOptions, EapMethod, Phase2};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let eap = EapOptions::builder()
        .identity("user@company.com")
        .password(
            std::env::var("WIFI_PASSWORD")
                .expect("Set WIFI_PASSWORD env var"),
        )
        .method(EapMethod::Peap)
        .phase2(Phase2::Mschapv2)
        .anonymous_identity("anonymous@company.com")
        .domain_suffix_match("company.com")
        .system_ca_certs(true)
        .build();

    nm.connect("CorpNetwork", None, WifiSecurity::WpaEap {
        opts: eap,
    }).await?;

    if let Some(ssid) = nm.current_ssid().await {
        println!("Connected to: {}", ssid);
    }

    Ok(())
}
```

## Troubleshooting

| Symptom | Likely Cause |
|---------|-------------|
| `AuthFailed` | Wrong username/password, or server rejected credentials |
| `SupplicantConfigFailed` | Misconfigured EAP method or phase2 |
| `SupplicantTimeout` | Server not responding — check CA cert and domain |
| `Timeout` | Authentication taking too long — try increasing timeout |

For enterprise networks, the authentication process can take longer than standard WPA-PSK connections. Consider using [custom timeouts](../advanced/timeouts.md):

```rust
use nmrs::{NetworkManager, TimeoutConfig};
use std::time::Duration;

let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(60));

let nm = NetworkManager::with_config(config).await?;
```

## Next Steps

- [Hidden Networks](./wifi-hidden.md) – enterprise networks are often hidden
- [Custom Timeouts](../advanced/timeouts.md) – increase timeout for slow auth servers
- [Error Handling](./error-handling.md) – handle enterprise auth errors
