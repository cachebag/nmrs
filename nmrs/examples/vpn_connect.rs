use nmrs::{NetworkManager, VpnCredentials, VpnType, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    
    let nm = NetworkManager::new().await?;
    
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "ExampleVPN".into(),
        gateway: "vpn.example.com:51820".into(),
        private_key: std::env::var("WG_PRIVATE_KEY")
            .expect("Set WG_PRIVATE_KEY env var"),
        address: "10.0.0.2/24".into(),
        peers: vec![WireGuardPeer {
            public_key: std::env::var("WG_PUBLIC_KEY")
                .expect("Set WG_PUBLIC_KEY env var"),
            gateway: "vpn.example.com:51820".into(),
            allowed_ips: vec!["0.0.0.0/0".into()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".into()]),
        mtu: None,
        uuid: None,
    };
    
    println!("Connecting to VPN...");
    nm.connect_vpn(creds).await?;
    
    let info = nm.get_vpn_info("ExampleVPN").await?;
    println!("Connected! IP: {:?}", info.ip4_address);
    
    Ok(())
}