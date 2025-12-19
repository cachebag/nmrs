use nmrs::{
    models::{BluetoothIdentity, BluetoothNetworkRole},
    NetworkManager, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let nm = NetworkManager::new().await?;

    println!("Scanning for Bluetooth devices...");
    let devices = nm.list_bluetooth_devices().await?;

    let mut bucket = Vec::new();
    // List bluetooth devices
    for d in devices {
        println!("{d}");
        bucket.push(d);
        nm.connect_bluetooth(
            "unknown",
            &BluetoothIdentity {
                bdaddr: "00:00:00:00:00".into(),
                bt_device_type: BluetoothNetworkRole::Dun,
            },
        )
        .await?;
    }

    Ok(())
}
