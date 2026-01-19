/// Connect to a WireGuard VPN using NetworkManager and print the assigned IP address.
///
/// This example demonstrates using the builder pattern for creating VPN credentials,
/// which provides a more ergonomic and readable API compared to the traditional constructor.
use nmrs::{NetworkManager, VpnCredentials, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Create a WireGuard peer with keepalive
    let peer = WireGuardPeer::new(
        std::env::var("WG_PUBLIC_KEY").expect("Set WG_PUBLIC_KEY env var"),
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".into()],
    )
    .with_persistent_keepalive(25);

    // Use the builder pattern for a more readable configuration
    let creds = VpnCredentials::builder()
        .name("ExampleVPN")
        .wireguard()
        .gateway("vpn.example.com:51820")
        .private_key(std::env::var("WG_PRIVATE_KEY").expect("Set WG_PRIVATE_KEY env var"))
        .address("10.0.0.2/24")
        .add_peer(peer)
        .with_dns(vec!["1.1.1.1".into()])
        .build();

    println!("Connecting to VPN...");
    nm.connect_vpn(creds).await?;

    let info = nm.get_vpn_info("ExampleVPN").await?;
    println!("Connected! IP: {:?}", info.ip4_address);

    Ok(())
}
