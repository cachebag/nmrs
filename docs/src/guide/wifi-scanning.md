# Scanning Networks

nmrs provides two approaches to discovering Wi-Fi networks: triggering an active scan and listing cached results.

## Triggering a Scan

`scan_networks()` instructs all wireless devices to perform an active 802.11 probe scan. This sends probe requests on each channel and waits for responses from nearby access points.

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Trigger an active scan on all wireless devices
    nm.scan_networks().await?;

    Ok(())
}
```

> **Note:** Scanning is asynchronous at the hardware level. After `scan_networks()` returns, NetworkManager continues to receive beacon frames and probe responses for a short period. You may want to add a brief delay before listing networks if you need the freshest results.

## Listing Networks

`list_networks()` returns all Wi-Fi networks currently known to NetworkManager. This includes results from the most recent scan as well as networks that NetworkManager has cached from prior scans.

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let networks = nm.list_networks().await?;
    for net in &networks {
        println!("{:30} {}%", net.ssid, net.strength.unwrap_or(0));
    }

    Ok(())
}
```

## The Network Struct

Each discovered network is represented by the [`Network`](../api/models.md) struct:

| Field | Type | Description |
|-------|------|-------------|
| `device` | `String` | Interface name (e.g., `"wlan0"`) |
| `ssid` | `String` | Network name |
| `bssid` | `Option<String>` | Access point MAC address |
| `strength` | `Option<u8>` | Signal strength (0–100) |
| `frequency` | `Option<u32>` | Frequency in MHz |
| `secured` | `bool` | Whether the network requires authentication |
| `is_psk` | `bool` | WPA-PSK (password) authentication |
| `is_eap` | `bool` | WPA-EAP (enterprise) authentication |
| `ip4_address` | `Option<String>` | IPv4 address if connected |
| `ip6_address` | `Option<String>` | IPv6 address if connected |

## Getting Detailed Information

For richer details about a specific network, use `show_details()`:

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let networks = nm.list_networks().await?;
    if let Some(network) = networks.first() {
        let info = nm.show_details(network).await?;

        println!("SSID:      {}", info.ssid);
        println!("BSSID:     {}", info.bssid);
        println!("Signal:    {} {}", info.strength, info.bars);
        println!("Frequency: {:?} MHz", info.freq);
        println!("Channel:   {:?}", info.channel);
        println!("Mode:      {}", info.mode);
        println!("Speed:     {:?} Mbps", info.rate_mbps);
        println!("Security:  {}", info.security);
        println!("Status:    {}", info.status);
    }

    Ok(())
}
```

The `NetworkInfo` struct returned by `show_details()` includes:

- **bars** – a visual signal-strength indicator (e.g., `"▂▄▆█"`)
- **channel** – the Wi-Fi channel number derived from frequency
- **rate_mbps** – link speed when connected
- **security** – human-readable security description
- **status** – connection status string

## Scan + List Pattern

The most common pattern is to trigger a scan, then list the results:

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    nm.scan_networks().await?;
    let networks = nm.list_networks().await?;

    for net in &networks {
        let security = if net.is_eap {
            "EAP"
        } else if net.is_psk {
            "PSK"
        } else {
            "Open"
        };

        let band = match net.frequency {
            Some(f) if f > 5900 => "6 GHz",
            Some(f) if f > 5000 => "5 GHz",
            Some(_) => "2.4 GHz",
            None => "?",
        };

        println!(
            "{:30} {:>3}%  {:>7}  {}",
            net.ssid,
            net.strength.unwrap_or(0),
            band,
            security,
        );
    }

    Ok(())
}
```

## Network Deduplication

When multiple access points broadcast the same SSID (common in mesh or enterprise deployments), nmrs merges them into a single `Network` entry. The entry retains the strongest signal, while security flags are combined with a logical OR. This means a single SSID entry might show both `is_psk` and `is_eap` as `true` if different APs advertise different capabilities.

## Next Steps

- [Connecting to Networks](./wifi-connecting.md) – use scan results to connect
- [WPA-PSK Networks](./wifi-wpa-psk.md) – password-based authentication
- [WPA-EAP (Enterprise)](./wifi-enterprise.md) – 802.1X authentication
