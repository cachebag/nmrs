# Hidden Networks

Hidden networks do not broadcast their SSID in beacon frames. To connect, you must know the exact SSID and provide the correct credentials. nmrs handles hidden network connections the same way as visible networks — if the SSID is not found during the scan, NetworkManager will attempt a directed probe.

## Connecting to a Hidden Network

The API for connecting to hidden networks is identical to visible networks. Simply provide the SSID and security credentials:

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Hidden open network
    nm.connect("HiddenCafe", WifiSecurity::Open).await?;

    // Hidden WPA-PSK network
    nm.connect("SecretLab", WifiSecurity::WpaPsk {
        psk: "lab_password".into(),
    }).await?;

    Ok(())
}
```

## How It Works

When you call `connect()` with an SSID:

1. nmrs first checks if there is a **saved connection profile** for that SSID — if so, it activates the saved profile directly
2. If no saved profile exists, it searches the **visible access point list**
3. If the network is not visible (hidden), NetworkManager creates a connection profile with the hidden flag set and performs a **directed probe request** for the specific SSID

This means hidden networks work transparently. The first connection may take slightly longer as NetworkManager performs the directed scan.

## Hidden Enterprise Networks

Hidden networks can also use WPA-EAP authentication:

```rust
use nmrs::{NetworkManager, WifiSecurity, EapOptions, EapMethod, Phase2};

let nm = NetworkManager::new().await?;

let eap = EapOptions::new("user@company.com", "password")
    .with_method(EapMethod::Peap)
    .with_phase2(Phase2::Mschapv2)
    .with_system_ca_certs(true);

nm.connect("HiddenCorpNet", WifiSecurity::WpaEap {
    opts: eap,
}).await?;
```

## Reconnecting

After the first successful connection, NetworkManager saves the profile with the hidden flag. Subsequent connections to the same SSID will reconnect automatically using the saved profile, even though the network doesn't appear in scan results.

## Considerations

- **Privacy:** Hidden networks are not truly hidden — the SSID is transmitted during the association process. They provide obscurity, not security.
- **Battery impact:** Devices probing for hidden networks transmit more frequently, which can reduce battery life on mobile devices.
- **First connection:** The initial connection may be slower than visible networks because NetworkManager must perform a directed probe.

## Next Steps

- [WPA-PSK Networks](./wifi-wpa-psk.md) – password-protected networks
- [WPA-EAP (Enterprise)](./wifi-enterprise.md) – corporate authentication
- [Connection Profiles](./profiles.md) – managing saved hidden network profiles
