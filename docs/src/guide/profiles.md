# Connection Profiles

NetworkManager stores connection profiles for every network you've connected to. These profiles contain the configuration needed to reconnect — SSID, credentials, IP settings, and more. nmrs provides methods to list, query, and remove these profiles.

## Listing Saved Connections

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let connections = nm.list_saved_connections().await?;
    for name in &connections {
        println!("  {}", name);
    }

    Ok(())
}
```

`list_saved_connections()` returns the names of all saved connection profiles across all connection types — Wi-Fi, Ethernet, VPN, and Bluetooth.

## Checking for a Saved Connection

```rust
let nm = NetworkManager::new().await?;

if nm.has_saved_connection("HomeWiFi").await? {
    println!("Profile exists for HomeWiFi");
} else {
    println!("No saved profile — credentials will be needed");
}
```

## How Saved Profiles Affect Connection

When you call `connect()` with an SSID that has a saved profile, nmrs activates the saved profile directly. This means:

- **Credentials are already stored** — the `WifiSecurity` value you pass is ignored
- **Connection is faster** — no need to create a new profile
- **Settings are preserved** — autoconnect, priority, and IP configuration are retained

```rust
let nm = NetworkManager::new().await?;

// First connection — credentials are required and saved
nm.connect("HomeWiFi", WifiSecurity::WpaPsk {
    psk: "password".into(),
}).await?;

// Later reconnection — saved profile is used, security parameter is ignored
nm.connect("HomeWiFi", WifiSecurity::Open).await?;
```

## Forgetting (Deleting) Connections

### Wi-Fi Connections

```rust
let nm = NetworkManager::new().await?;

nm.forget("HomeWiFi").await?;
println!("Wi-Fi profile deleted");
```

If currently connected to that network, `forget()` disconnects first, then deletes all saved profiles matching the SSID.

### VPN Connections

```rust
nm.forget_vpn("MyVPN").await?;
```

### Bluetooth Connections

```rust
nm.forget_bluetooth("My Phone").await?;
```

## Getting the D-Bus Path

For advanced use cases, you can retrieve the D-Bus object path of a saved connection:

```rust
let nm = NetworkManager::new().await?;

if let Some(path) = nm.get_saved_connection_path("HomeWiFi").await? {
    println!("D-Bus path: {}", path.as_str());
}
```

## Profile Lifecycle

1. **Created** — when you first connect to a network, NetworkManager creates a profile
2. **Persisted** — profiles are saved to `/etc/NetworkManager/system-connections/`
3. **Reused** — subsequent connections to the same SSID use the saved profile
4. **Updated** — if you connect with different credentials, the profile may be updated
5. **Deleted** — calling `forget()`, `forget_vpn()`, or `forget_bluetooth()` removes it

## Next Steps

- [WiFi Management](./wifi.md) – scan and connect to Wi-Fi networks
- [VPN Management](./vpn-management.md) – manage VPN profiles
- [Bluetooth](./bluetooth.md) – Bluetooth connection profiles
