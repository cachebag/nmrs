# Models Module

The `models` module contains all data types used by nmrs. These are re-exported at the crate root and through `nmrs::models`.

## Device Models

### Device

Represents a network device managed by NetworkManager.

```rust
pub struct Device {
    pub path: String,           // D-Bus object path
    pub interface: String,      // e.g., "wlan0", "eth0"
    pub identity: DeviceIdentity,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub managed: Option<bool>,
    pub driver: Option<String>,
    pub ip4_address: Option<String>,
    pub ip6_address: Option<String>,
}
```

Methods: `is_wireless()`, `is_wired()`, `is_bluetooth()`

### DeviceIdentity

```rust
pub struct DeviceIdentity {
    pub permanent_mac: String,
    pub current_mac: String,
}
```

### DeviceType

```rust
pub enum DeviceType {
    Ethernet,
    Wifi,
    WifiP2P,
    Loopback,
    Bluetooth,
    Other(u32),
}
```

Methods: `supports_scanning()`, `requires_specific_object()`, `has_global_enabled_state()`, `connection_type_str()`, `to_code()`

### DeviceState

```rust
pub enum DeviceState {
    Unmanaged, Unavailable, Disconnected,
    Prepare, Config, NeedAuth, IpConfig, IpCheck, Secondaries,
    Activated, Deactivating, Failed,
    Other(u32),
}
```

Methods: `is_transitional()`

## Wi-Fi Models

### Network

A discovered Wi-Fi network.

```rust
pub struct Network {
    pub device: String,
    pub ssid: String,
    pub bssid: Option<String>,
    pub strength: Option<u8>,
    pub frequency: Option<u32>,
    pub secured: bool,
    pub is_psk: bool,
    pub is_eap: bool,
    pub ip4_address: Option<String>,
    pub ip6_address: Option<String>,
}
```

### NetworkInfo

Detailed network information from `show_details()`.

```rust
pub struct NetworkInfo {
    pub ssid: String,
    pub bssid: String,
    pub strength: u8,
    pub freq: Option<u32>,
    pub channel: Option<u16>,
    pub mode: String,
    pub rate_mbps: Option<u32>,
    pub bars: String,         // e.g., "▂▄▆█"
    pub security: String,
    pub status: String,
    pub ip4_address: Option<String>,
    pub ip6_address: Option<String>,
}
```

### WifiSecurity

```rust
pub enum WifiSecurity {
    Open,
    WpaPsk { psk: String },
    WpaEap { opts: EapOptions },
}
```

Methods: `secured()`, `is_psk()`, `is_eap()`

### EapOptions

Enterprise Wi-Fi configuration.

```rust
pub struct EapOptions {
    pub identity: String,
    pub password: String,
    pub anonymous_identity: Option<String>,
    pub domain_suffix_match: Option<String>,
    pub ca_cert_path: Option<String>,
    pub system_ca_certs: bool,
    pub method: EapMethod,
    pub phase2: Phase2,
}
```

Constructors: `new(identity, password)`, `builder()`

### EapMethod / Phase2

```rust
pub enum EapMethod { Peap, Ttls }
pub enum Phase2 { Mschapv2, Pap }
```

## VPN Models

### VpnCredentials

```rust
pub struct VpnCredentials {
    pub vpn_type: VpnType,
    pub name: String,
    pub gateway: String,
    pub private_key: String,
    pub address: String,
    pub peers: Vec<WireGuardPeer>,
    pub dns: Option<Vec<String>>,
    pub mtu: Option<u32>,
    pub uuid: Option<Uuid>,
}
```

Constructors: `new(...)`, `builder()`

### WireGuardPeer

```rust
pub struct WireGuardPeer {
    pub public_key: String,
    pub gateway: String,
    pub allowed_ips: Vec<String>,
    pub preshared_key: Option<String>,
    pub persistent_keepalive: Option<u32>,
}
```

### VpnConnection / VpnConnectionInfo

```rust
pub struct VpnConnection {
    pub name: String,
    pub vpn_type: VpnType,
    pub state: DeviceState,
    pub interface: Option<String>,
}

pub struct VpnConnectionInfo {
    pub name: String,
    pub vpn_type: VpnType,
    pub state: DeviceState,
    pub interface: Option<String>,
    pub gateway: Option<String>,
    pub ip4_address: Option<String>,
    pub ip6_address: Option<String>,
    pub dns_servers: Vec<String>,
}
```

## Bluetooth Models

### BluetoothDevice

```rust
pub struct BluetoothDevice {
    pub bdaddr: String,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub bt_caps: u32,
    pub state: DeviceState,
}
```

### BluetoothIdentity

```rust
pub struct BluetoothIdentity {
    pub bdaddr: String,
    pub bt_device_type: BluetoothNetworkRole,
}
```

### BluetoothNetworkRole

```rust
pub enum BluetoothNetworkRole { PanU, Dun }
```

## Configuration Models

### TimeoutConfig

```rust
pub struct TimeoutConfig {
    pub connection_timeout: Duration,  // default: 30s
    pub disconnect_timeout: Duration,  // default: 10s
}
```

### ConnectionOptions

```rust
pub struct ConnectionOptions {
    pub autoconnect: bool,
    pub autoconnect_priority: Option<i32>,
    pub autoconnect_retries: Option<i32>,
}
```

## Non-Exhaustive Types

All enums and structs in nmrs are marked `#[non_exhaustive]`. Always include a wildcard arm in match expressions and don't construct structs directly (use constructors/builders).

## Full API Reference

For complete documentation with all method signatures and trait implementations, see [docs.rs/nmrs](https://docs.rs/nmrs).
