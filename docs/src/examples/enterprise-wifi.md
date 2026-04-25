# Enterprise WiFi

This example connects to a WPA-Enterprise (802.1X) network using PEAP/MSCHAPv2 — the most common configuration in corporate environments.

## Features

- Builds EAP options with the builder pattern
- Configures certificate validation
- Uses extended timeouts for enterprise authentication
- Handles authentication-specific errors

## Code

```rust
use nmrs::{
    EapMethod, EapOptions, NetworkManager, Phase2,
    TimeoutConfig, WifiSecurity, ConnectionError,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    // Enterprise auth can be slow — use a longer timeout
    let config = TimeoutConfig::new()
        .with_connection_timeout(Duration::from_secs(60));

    let nm = NetworkManager::with_config(config).await?;

    // Build EAP configuration
    let eap = EapOptions::builder()
        .identity(
            std::env::var("EAP_IDENTITY")
                .expect("Set EAP_IDENTITY (e.g., user@company.com)"),
        )
        .password(
            std::env::var("EAP_PASSWORD")
                .expect("Set EAP_PASSWORD"),
        )
        .method(EapMethod::Peap)
        .phase2(Phase2::Mschapv2)
        .anonymous_identity("anonymous@company.com")
        .domain_suffix_match("company.com")
        .system_ca_certs(true)
        .build();

    let ssid = std::env::var("EAP_SSID")
        .unwrap_or_else(|_| "CorpWiFi".into());

    println!("Connecting to enterprise network '{}'...", ssid);

    match nm.connect(&ssid, None, WifiSecurity::WpaEap { opts: eap }).await {
        Ok(_) => {
            println!("Connected!");

            if let Some((ssid, freq)) = nm.current_connection_info().await {
                let band = match freq {
                    Some(f) if f > 5000 => "5 GHz",
                    Some(_) => "2.4 GHz",
                    None => "unknown",
                };
                println!("  Network: {} ({})", ssid, band);
            }
        }
        Err(ConnectionError::AuthFailed) => {
            eprintln!("Authentication failed — check username and password");
        }
        Err(ConnectionError::SupplicantConfigFailed) => {
            eprintln!("Supplicant config error — check EAP method and phase2");
        }
        Err(ConnectionError::SupplicantTimeout) => {
            eprintln!("RADIUS server not responding — check CA cert and domain");
        }
        Err(ConnectionError::Timeout) => {
            eprintln!("Connection timed out — enterprise auth may need more time");
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
```

## Running

```bash
EAP_IDENTITY="user@company.com" \
EAP_PASSWORD="my_password" \
EAP_SSID="CorpWiFi" \
cargo run --example enterprise_wifi
```

## Variations

### TTLS/PAP Configuration

Some networks use TTLS with PAP instead of PEAP:

```rust
let eap = EapOptions::builder()
    .identity("student@university.edu")
    .password("password")
    .method(EapMethod::Ttls)
    .phase2(Phase2::Pap)
    .ca_cert_path("file:///etc/ssl/certs/university-ca.pem")
    .build();
```

### Custom CA Certificate

If your organization provides a specific CA certificate:

```rust
let eap = EapOptions::builder()
    .identity("user@company.com")
    .password("password")
    .method(EapMethod::Peap)
    .phase2(Phase2::Mschapv2)
    .ca_cert_path("file:///usr/local/share/ca-certificates/corp-ca.pem")
    .domain_suffix_match("company.com")
    .build();
```

## Common Issues

| Problem | Solution |
|---------|----------|
| `AuthFailed` | Verify username format (email vs plain username) and password |
| `SupplicantConfigFailed` | Check EAP method — ask IT which to use |
| `SupplicantTimeout` | Verify CA cert path and domain suffix match |
| Connection is slow | Increase timeout with `TimeoutConfig` |
