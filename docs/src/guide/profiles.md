# Connection Profiles

NetworkManager stores connection profiles for every network you've connected to. These profiles contain the configuration needed to reconnect — SSID, credentials, IP settings, and more. nmrs provides methods to list, query, and remove these profiles.

## Listing Saved Connections

`nmrs` exposes three flavors of "list the saved profiles", trading off
detail for cost:

| Method | Cost | Returns |
|--------|------|---------|
| [`list_saved_connections`](../api/network-manager.md#connection-profile-methods) | One `GetSettings` call per profile, full decode | `Vec<SavedConnection>` |
| [`list_saved_connections_brief`](../api/network-manager.md#connection-profile-methods) | One `GetSettings` per profile, minimal decode | `Vec<SavedConnectionBrief>` |
| [`list_saved_connection_ids`](../api/network-manager.md#connection-profile-methods) | Same calls as `_brief`, only the names | `Vec<String>` |

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Full decode — useful for showing IP / autoconnect / security in a UI.
    for conn in nm.list_saved_connections().await? {
        println!("  {:<32} {:<12} {}", conn.id, conn.connection_type, conn.uuid);
    }

    // Lightweight — just names and types, useful for menus.
    for brief in nm.list_saved_connections_brief().await? {
        println!("  {:<32} ({})", brief.id, brief.connection_type);
    }

    Ok(())
}
```

Each `SavedConnection` includes the profile `id` (display name), `uuid`,
`connection_type` (`"802-11-wireless"`, `"vpn"`, `"wireguard"`, `"bluetooth"`,
…), and a decoded [`SettingsSummary`](../api/models.md#settingspatch).

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
nm.connect("HomeWiFi", None, WifiSecurity::WpaPsk {
    psk: "password".into(),
}).await?;

// Later reconnection — saved profile is used, security parameter is ignored
nm.connect("HomeWiFi", None, WifiSecurity::Open).await?;
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

## Loading a Single Profile by UUID

```rust
let nm = NetworkManager::new().await?;

let profile = nm.get_saved_connection("a1b2c3d4-...").await?;
println!("{} ({}) autoconnect={}", profile.id, profile.connection_type, profile.autoconnect);
```

For the raw `GetSettings` map (advanced consumers building their own
decoder), use `nm.get_saved_connection_raw(uuid)`.

## Updating a Profile

`update_saved_connection` merges a [`SettingsPatch`](../api/models.md#settingspatch)
into an existing profile via NM's `Update` / `UpdateUnsaved` methods.
This is the right call to flip `autoconnect`, change a priority, or
update DNS without rebuilding the entire profile.

## Deleting by UUID

When the profile UUID is known, you can delete it directly:

```rust
nm.delete_saved_connection("a1b2c3d4-...").await?;
```

For SSID-based deletion (which also disconnects first if active), use
`forget`, `forget_vpn`, or `forget_bluetooth` as shown above.

## Reloading from Disk

If you've edited keyfiles in `/etc/NetworkManager/system-connections/`
out-of-band, ask NetworkManager to re-read them:

```rust
nm.reload_saved_connections().await?;
```

## Getting the D-Bus Path

For advanced use cases, you can retrieve the D-Bus object path of a saved
connection by SSID:

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
