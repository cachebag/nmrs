# OpenVPN Client

This example demonstrates a complete OpenVPN client that authenticates with password+TLS, connects, displays VPN details, and cleanly disconnects.

## Features

- Builds OpenVPN config with password+TLS authentication
- Reads credentials from environment variables
- Connects and retrieves VPN details
- Displays IP configuration, DNS, and gateway
- Cleanly disconnects on completion

## Code

```rust
use nmrs::{NetworkManager, OpenVpnConfig, OpenVpnAuthType};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Read credentials from environment
    let remote = std::env::var("OVPN_REMOTE")
        .unwrap_or_else(|_| "vpn.example.com".into());
    let port: u16 = std::env::var("OVPN_PORT")
        .unwrap_or_else(|_| "1194".into())
        .parse()
        .expect("OVPN_PORT must be a valid port number");
    let username = std::env::var("OVPN_USER")
        .expect("Set OVPN_USER");
    let password = std::env::var("OVPN_PASS")
        .expect("Set OVPN_PASS");
    let ca_path = std::env::var("OVPN_CA")
        .unwrap_or_else(|_| "/etc/openvpn/ca.crt".into());
    let cert_path = std::env::var("OVPN_CERT")
        .unwrap_or_else(|_| "/etc/openvpn/client.crt".into());
    let key_path = std::env::var("OVPN_KEY")
        .unwrap_or_else(|_| "/etc/openvpn/client.key".into());

    // Build OpenVPN config (password+TLS)
    let config = OpenVpnConfig::new("CorpVPN", &remote, port, false)
        .with_auth_type(OpenVpnAuthType::PasswordTls)
        .with_username(&username)
        .with_password(&password)
        .with_ca_cert(&ca_path)
        .with_client_cert(&cert_path)
        .with_client_key(&key_path);

    // Connect
    println!("Connecting to OpenVPN ({remote}:{port})...");
    nm.connect_vpn(config).await?;
    println!("Connected!\n");

    // Show VPN details
    let info = nm.get_vpn_info("CorpVPN").await?;
    println!("VPN Connection Details:");
    println!("  Name:      {}", info.name);
    println!("  Kind:      {:?}", info.vpn_kind);
    println!("  State:     {:?}", info.state);
    println!("  Interface: {:?}", info.interface);
    println!("  Gateway:   {:?}", info.gateway);
    println!("  IPv4:      {:?}", info.ip4_address);
    println!("  IPv6:      {:?}", info.ip6_address);
    println!("  DNS:       {:?}", info.dns_servers);

    if let Some(details) = &info.details {
        println!("  Details:   {:?}", details);
    }

    // Wait for user input before disconnecting
    println!("\nPress Enter to disconnect...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    // Disconnect
    nm.disconnect_vpn("CorpVPN").await?;
    println!("VPN disconnected");

    Ok(())
}
```

## Running

```bash
OVPN_REMOTE="vpn.example.com" \
OVPN_PORT="1194" \
OVPN_USER="alice" \
OVPN_PASS="hunter2" \
OVPN_CA="/etc/openvpn/ca.crt" \
OVPN_CERT="/etc/openvpn/client.crt" \
OVPN_KEY="/etc/openvpn/client.key" \
cargo run --example openvpn_client
```

## Sample Output

```
Connecting to OpenVPN (vpn.example.com:1194)...
Connected!

VPN Connection Details:
  Name:      CorpVPN
  Kind:      Plugin
  State:     Activated
  Interface: Some("tun0")
  Gateway:   Some("vpn.example.com")
  IPv4:      Some("10.8.0.2")
  IPv6:      None
  DNS:       ["10.8.0.1"]

Press Enter to disconnect...
```

## Error Handling

```rust
use nmrs::{NetworkManager, OpenVpnConfig, OpenVpnAuthType, ConnectionError};

async fn connect_with_retry(nm: &NetworkManager) -> nmrs::Result<()> {
    let config = OpenVpnConfig::new("CorpVPN", "vpn.example.com", 1194, false)
        .with_auth_type(OpenVpnAuthType::PasswordTls)
        .with_username("alice")
        .with_password("hunter2")
        .with_ca_cert("/etc/openvpn/ca.crt")
        .with_client_cert("/etc/openvpn/client.crt")
        .with_client_key("/etc/openvpn/client.key");

    match nm.connect_vpn(config).await {
        Ok(_) => {
            println!("VPN connected");
            Ok(())
        }
        Err(ConnectionError::AuthFailed) => {
            eprintln!("Authentication failed — check username/password and certificates");
            Err(ConnectionError::AuthFailed)
        }
        Err(ConnectionError::Timeout) => {
            eprintln!("Connection timed out — check server address and port");
            Err(ConnectionError::Timeout)
        }
        Err(ConnectionError::VpnFailed) => {
            eprintln!("VPN activation failed — verify OpenVPN plugin is installed");
            Err(ConnectionError::VpnFailed)
        }
        Err(e) => {
            eprintln!("Unexpected error: {e}");
            Err(e)
        }
    }
}
```

## TLS-Only Variation

For certificate-only authentication (no username/password):

```rust
use nmrs::{OpenVpnConfig, OpenVpnAuthType};

let config = OpenVpnConfig::new("TlsOnlyVPN", "vpn.example.com", 1194, false)
    .with_auth_type(OpenVpnAuthType::Tls)
    .with_ca_cert("/etc/openvpn/ca.crt")
    .with_client_cert("/etc/openvpn/client.crt")
    .with_client_key("/etc/openvpn/client.key");

nm.connect_vpn(config).await?;
```

## Password-Only Variation

For username/password authentication without client certificates:

```rust
use nmrs::{OpenVpnConfig, OpenVpnAuthType};

let config = OpenVpnConfig::new("PassVPN", "vpn.example.com", 1194, false)
    .with_auth_type(OpenVpnAuthType::Password)
    .with_username("alice")
    .with_password("hunter2")
    .with_ca_cert("/etc/openvpn/ca.crt");

nm.connect_vpn(config).await?;
```

## Next Steps

- [`.ovpn` File Import](./ovpn-import.md) — import existing OpenVPN configurations
- [VPN Connections Guide](../guide/vpn.md) — comprehensive VPN overview
- [OpenVPN Setup](../guide/vpn-openvpn.md) — detailed OpenVPN configuration
- [WireGuard VPN Client](./wireguard-client.md) — WireGuard example
