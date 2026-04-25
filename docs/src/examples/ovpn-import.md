# .ovpn File Import

This example shows how to import an existing `.ovpn` configuration file into NetworkManager using nmrs.

## Features

- Import `.ovpn` files with a single method call
- Builder approach for more control over the import
- Automatic handling of inline certificates
- Error handling for parse failures

## One-Liner Import

The simplest way to import an `.ovpn` file:

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Import with credentials
    nm.import_ovpn("client.ovpn", Some("alice"), Some("hunter2")).await?;
    println!("VPN imported and connected!");

    Ok(())
}
```

If the `.ovpn` file uses certificate-only authentication, pass `None` for username and password:

```rust
nm.import_ovpn("client.ovpn", None, None).await?;
```

## Builder Approach

For more control over the import process, use `OpenVpnBuilder`:

```rust
use nmrs::{NetworkManager, OpenVpnBuilder};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let config = OpenVpnBuilder::from_ovpn_file("client.ovpn")?
        .username("alice")
        .password("hunter2")
        .build()?;

    nm.connect_vpn(config).await?;
    println!("VPN connected!");

    Ok(())
}
```

The builder extracts remote, port, protocol, certificates, and other settings from the `.ovpn` file automatically.

## Inline Certificates

Many `.ovpn` files embed certificates directly rather than referencing external files:

```
<ca>
-----BEGIN CERTIFICATE-----
MIIBxTCCAWugAwIBAgIJAJ...
-----END CERTIFICATE-----
</ca>

<cert>
-----BEGIN CERTIFICATE-----
MIICCzCCAZGgAwIBAgIRAP...
-----END CERTIFICATE-----
</cert>

<key>
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w...
-----END PRIVATE KEY-----
</key>
```

`from_ovpn_file` handles these automatically — inline certificates are extracted and written to temporary files that NetworkManager can reference. No extra steps needed:

```rust
let config = OpenVpnBuilder::from_ovpn_file("inline-certs.ovpn")?
    .username("alice")
    .password("hunter2")
    .build()?;
```

## Error Handling

Handle parse failures and missing fields gracefully:

```rust
use nmrs::{OpenVpnBuilder, ConnectionError};

fn import_config(path: &str) -> nmrs::Result<()> {
    let config = match OpenVpnBuilder::from_ovpn_file(path) {
        Ok(builder) => builder.build()?,
        Err(ConnectionError::InvalidConfig(msg)) => {
            eprintln!("Failed to parse {path}: {msg}");
            return Err(ConnectionError::InvalidConfig(msg));
        }
        Err(ConnectionError::NotFound) => {
            eprintln!("File not found: {path}");
            return Err(ConnectionError::NotFound);
        }
        Err(e) => {
            eprintln!("Import error: {e}");
            return Err(e);
        }
    };

    println!("Successfully parsed configuration");
    Ok(())
}
```

Common parse errors:

| Error | Cause |
|-------|-------|
| `InvalidConfig` | Missing `remote` directive, malformed options, or invalid certificate data |
| `NotFound` | The `.ovpn` file does not exist at the given path |
| `AuthFailed` | Credentials required but not provided |

## Complete Example

```rust
use nmrs::{NetworkManager, OpenVpnBuilder};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let ovpn_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "client.ovpn".into());

    let username = std::env::var("OVPN_USER").ok();
    let password = std::env::var("OVPN_PASS").ok();

    println!("Importing {ovpn_path}...");

    // Builder approach for maximum control
    let mut builder = OpenVpnBuilder::from_ovpn_file(&ovpn_path)?;

    if let Some(user) = &username {
        builder = builder.username(user);
    }
    if let Some(pass) = &password {
        builder = builder.password(pass);
    }

    let config = builder.build()?;
    nm.connect_vpn(config).await?;

    println!("Connected! Checking VPN info...");
    let vpns = nm.list_vpn_connections().await?;
    for vpn in &vpns {
        if vpn.active {
            println!("  Active: {} ({:?})", vpn.name, vpn.vpn_type);
        }
    }

    Ok(())
}
```

## Next Steps

- [OpenVPN Client Example](./openvpn-client.md) — build OpenVPN config from scratch
- [VPN Connections Guide](../guide/vpn.md) — comprehensive VPN overview
- [OpenVPN Setup](../guide/vpn-openvpn.md) — detailed OpenVPN configuration
