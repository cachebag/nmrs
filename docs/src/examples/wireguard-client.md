# WireGuard VPN Client

This example demonstrates a complete WireGuard VPN client that connects, displays connection information, and cleanly disconnects.

## Features

- Builds VPN credentials with the builder pattern
- Connects and retrieves VPN details
- Displays IP configuration and DNS
- Cleanly disconnects on completion

## Code

```rust
use nmrs::{NetworkManager, VpnCredentials, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Check for existing VPN connections
    let existing = nm.list_vpn_connections().await?;
    if !existing.is_empty() {
        println!("Existing VPN connections:");
        for vpn in &existing {
            println!("  {} ({:?}) — {:?}", vpn.name, vpn.vpn_type, vpn.state);
        }
        println!();
    }

    // Build WireGuard peer configuration
    let peer = WireGuardPeer::new(
        std::env::var("WG_SERVER_PUBKEY")
            .expect("Set WG_SERVER_PUBKEY"),
        std::env::var("WG_ENDPOINT")
            .unwrap_or_else(|_| "vpn.example.com:51820".into()),
        vec!["0.0.0.0/0".into()],
    )
    .with_persistent_keepalive(25);

    // Build credentials
    let creds = VpnCredentials::builder()
        .name("ExampleVPN")
        .wireguard()
        .gateway(
            std::env::var("WG_ENDPOINT")
                .unwrap_or_else(|_| "vpn.example.com:51820".into()),
        )
        .private_key(
            std::env::var("WG_PRIVATE_KEY")
                .expect("Set WG_PRIVATE_KEY"),
        )
        .address(
            std::env::var("WG_ADDRESS")
                .unwrap_or_else(|_| "10.0.0.2/24".into()),
        )
        .add_peer(peer)
        .with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()])
        .with_mtu(1420)
        .build();

    // Connect
    println!("Connecting to VPN...");
    nm.connect_vpn(creds).await?;
    println!("Connected!\n");

    // Show VPN details
    let info = nm.get_vpn_info("ExampleVPN").await?;
    println!("VPN Connection Details:");
    println!("  Name:      {}", info.name);
    println!("  Type:      {:?}", info.vpn_type);
    println!("  State:     {:?}", info.state);
    println!("  Interface: {:?}", info.interface);
    println!("  Gateway:   {:?}", info.gateway);
    println!("  IPv4:      {:?}", info.ip4_address);
    println!("  IPv6:      {:?}", info.ip6_address);
    println!("  DNS:       {:?}", info.dns_servers);

    // Wait for user input before disconnecting
    println!("\nPress Enter to disconnect...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    // Disconnect
    nm.disconnect_vpn("ExampleVPN").await?;
    println!("VPN disconnected");

    // Optionally remove the profile
    // nm.forget_vpn("ExampleVPN").await?;

    Ok(())
}
```

## Running

```bash
WG_SERVER_PUBKEY="HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=" \
WG_PRIVATE_KEY="YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=" \
WG_ENDPOINT="vpn.example.com:51820" \
WG_ADDRESS="10.0.0.2/24" \
cargo run --example wireguard_client
```

## Sample Output

```
Connecting to VPN...
Connected!

VPN Connection Details:
  Name:      ExampleVPN
  Type:      WireGuard
  State:     Activated
  Interface: Some("wg-examplevpn")
  Gateway:   Some("vpn.example.com")
  IPv4:      Some("10.0.0.2/24")
  IPv6:      None
  DNS:       ["1.1.1.1", "8.8.8.8"]

Press Enter to disconnect...
```

## Split Tunnel Variation

Route only specific subnets through the VPN:

```rust
let peer = WireGuardPeer::new(
    "server_public_key",
    "vpn.example.com:51820",
    vec![
        "10.0.0.0/8".into(),
        "192.168.0.0/16".into(),
    ],
);
```

## Multiple Peers

Connect through multiple WireGuard servers:

```rust
let peer1 = WireGuardPeer::new(
    "peer1_pubkey",
    "us-east.vpn.example.com:51820",
    vec!["10.1.0.0/16".into()],
);

let peer2 = WireGuardPeer::new(
    "peer2_pubkey",
    "eu-west.vpn.example.com:51820",
    vec!["10.2.0.0/16".into()],
);

let creds = VpnCredentials::builder()
    .name("MultiPeerVPN")
    .wireguard()
    .gateway("us-east.vpn.example.com:51820")
    .private_key("client_private_key")
    .address("10.0.0.2/24")
    .add_peer(peer1)
    .add_peer(peer2)
    .build();
```
