use nmrs::models::BluetoothIdentity;
use nmrs::{NetworkManager, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let nm = NetworkManager::new().await?;

    println!("Scanning for Bluetooth devices...");
    let devices = nm.list_bluetooth_devices().await?;

    if devices.is_empty() {
        println!("No Bluetooth devices found.");
        println!("\nMake sure:");
        println!("  1. Bluetooth is enabled");
        println!("  2. Device is paired (use 'bluetoothctl')");
        return Ok(());
    }

    println!("\nAvailable Bluetooth devices:");
    for (i, device) in devices.iter().enumerate() {
        println!("  {}. {}", i + 1, device);
    }

    // Example: Connect to the first device
    if let Some(device) = devices.first {
        println!("\nConnecting to: {}", device);

        let settings = BluetoothIdentity {
            bdaddr: device.bdaddr.clone(),
            bt_device_type: device.bt_device_type.clone(),
        };

        let name = device
            .alias
            .as_ref()
            .or(device.name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Bluetooth Device");

        match nm.connect_bluetooth(name, &settings).await {
            Ok(_) => println!("✓ Successfully connected to {}", name),
            Err(e) => eprintln!("✗ Failed to connect: {}", e),
        }
    }

    Ok(())
}
