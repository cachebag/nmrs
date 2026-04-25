# WPA-PSK Networks

WPA-PSK (Wi-Fi Protected Access with Pre-Shared Key) is the most common security type for home and small-office Wi-Fi networks. You provide a password, and nmrs handles the WPA handshake.

## Connecting with a Password

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    nm.connect("HomeWiFi", None, WifiSecurity::WpaPsk {
        psk: "my_secure_password".into(),
    }).await?;

    println!("Connected!");
    Ok(())
}
```

The `WifiSecurity::WpaPsk` variant works with WPA, WPA2, and WPA3 Personal networks. NetworkManager negotiates the strongest supported protocol automatically.

## Password Requirements

- Must not be empty — `ConnectionError::MissingPassword` is returned for empty strings
- WPA-PSK passwords are typically 8–63 characters (ASCII passphrase) or exactly 64 hex characters (raw PSK)
- nmrs passes the password directly to NetworkManager, which handles validation

## Reading the Password at Runtime

Avoid hardcoding passwords. Read them from environment variables, user input, or a secrets manager:

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let password = std::env::var("WIFI_PASSWORD")
        .expect("Set WIFI_PASSWORD environment variable");

    nm.connect("HomeWiFi", None, WifiSecurity::WpaPsk {
        psk: password,
    }).await?;

    Ok(())
}
```

## Reconnecting to Saved Networks

After the first successful connection, NetworkManager saves the credentials in a connection profile. Subsequent connections to the same SSID will reuse the saved profile automatically — you don't need to provide the password again:

```rust
let nm = NetworkManager::new().await?;

if nm.has_saved_connection("HomeWiFi").await? {
    // Saved profile exists; password is stored in it.
    // The WifiSecurity value is ignored when a saved profile exists.
    nm.connect("HomeWiFi", None, WifiSecurity::Open).await?;
}
```

## Error Handling

The most common errors for WPA-PSK connections:

| Error | Meaning |
|-------|---------|
| `ConnectionError::AuthFailed` | Wrong password |
| `ConnectionError::MissingPassword` | Empty password string |
| `ConnectionError::NotFound` | Network not in range |
| `ConnectionError::Timeout` | Connection took too long |
| `ConnectionError::DhcpFailed` | Connected to AP but DHCP failed |

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionError};

let nm = NetworkManager::new().await?;

match nm.connect("HomeWiFi", None, WifiSecurity::WpaPsk {
    psk: "password".into(),
}).await {
    Ok(_) => println!("Connected!"),
    Err(ConnectionError::AuthFailed) => {
        eprintln!("Wrong password — check and try again");
    }
    Err(ConnectionError::MissingPassword) => {
        eprintln!("Password cannot be empty");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Next Steps

- [WPA-EAP (Enterprise)](./wifi-enterprise.md) – for corporate/university networks
- [Hidden Networks](./wifi-hidden.md) – connecting to non-broadcast SSIDs
- [Connection Profiles](./profiles.md) – managing saved connections
