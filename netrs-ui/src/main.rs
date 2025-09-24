use netrs_core::NetworkManager;

#[tokio::main]
async fn main() {
    let nm = NetworkManager::new().await.expect("Failed to connect to NetworkManager");

    match nm.list_networks().await {
        Ok(networks) => {
            println!("Available networks:");
            for n in networks {
                println!("{} (strength: {}%) secure: {}", n.ssid, n.strength, n.secure);
            }
        }
        Err(e) => eprintln!("Error listing networks: {e}"),
    }
}
