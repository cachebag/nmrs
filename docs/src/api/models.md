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

### AccessPoint

A single AP with per-BSSID details.

```rust
pub struct AccessPoint {
    pub ssid: String,
    pub bssid: String,
    pub strength: u8,
    pub frequency_mhz: u32,
    pub device: String,
    // ... security flags and device state
}
```

### WifiDevice

A Wi-Fi device discovered by `list_wifi_devices()`.

```rust
pub struct WifiDevice {
    pub interface: String,
    pub mac: String,
    pub state: DeviceState,
    pub active_ssid: Option<String>,
}
```

### WifiScope

Per-interface operations scope returned by `nm.wifi("wlan1")`.

Methods: `interface()`, `scan()`, `list_networks()`, `list_access_points()`, `connect(ssid, creds)`, `connect_to_bssid(ssid, bssid, creds)`, `disconnect()`, `set_enabled(bool)`, `forget(ssid)`

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

## Radio / Airplane-Mode Models

### RadioState

```rust
pub struct RadioState {
    pub enabled: bool,           // software toggle
    pub hardware_enabled: bool,  // rfkill state
}
```

### AirplaneModeState

Aggregated state across Wi-Fi, WWAN, and Bluetooth radios. Methods: `is_airplane_mode()`.

## VPN Models

### WireGuardConfig

WireGuard VPN configuration (replaces the deprecated `VpnCredentials`).

```rust
pub struct WireGuardConfig {
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

Constructor: `new(name, gateway, private_key, address, peers)`
Builder methods: `.with_dns(vec)`, `.with_mtu(u32)`, `.with_uuid(uuid)`

### OpenVpnConfig

OpenVPN configuration.

```rust
pub struct OpenVpnConfig {
    pub name: String,
    pub remote: String,
    pub port: u16,
    pub tcp: bool,
    pub auth_type: Option<OpenVpnAuthType>,
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub compression: Option<OpenVpnCompression>,
    pub proxy: Option<OpenVpnProxy>,
    // ... many more TLS and routing fields
}
```

Constructor: `new(name, remote, port, tcp)`
Builder methods: `.with_auth_type()`, `.with_username()`, `.with_password()`, `.with_ca_cert()`, `.with_client_cert()`, `.with_client_key()`, `.with_dns()`, `.with_mtu()`, `.with_compression()`, `.with_proxy()`, `.with_tls_auth()`, `.with_tls_crypt()`, `.with_redirect_gateway()`, `.with_routes()`, and many more.

### OpenVpnAuthType

```rust
pub enum OpenVpnAuthType { Password, Tls, PasswordTls, StaticKey }
```

### OpenVpnCompression

```rust
pub enum OpenVpnCompression { No, Lzo, Lz4, Lz4V2, Yes }
```

### OpenVpnProxy

```rust
pub enum OpenVpnProxy {
    Http { server, port, username, password, retry },
    Socks { server, port, retry },
}
```

### VpnRoute

```rust
pub struct VpnRoute {
    pub dest: String,
    pub prefix: u32,
    pub next_hop: Option<String>,
    pub metric: Option<u32>,
}
```

Constructor: `new(dest, prefix)`. Builder methods: `.next_hop(gw)`, `.metric(m)`.

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

### VpnType

Protocol-specific metadata decoded from NM settings (data-carrying enum):

```rust
pub enum VpnType {
    WireGuard { private_key, peer_public_key, endpoint, allowed_ips, ... },
    OpenVpn { remote, connection_type, user_name, ca, cert, key, ... },
    OpenConnect { gateway, user_name, protocol, ... },
    StrongSwan { address, method, user_name, ... },
    Pptp { gateway, user_name, ... },
    L2tp { gateway, user_name, ipsec_enabled, ... },
    Generic { service_type, data, secrets, ... },
}
```

### VpnKind

```rust
pub enum VpnKind { Plugin, WireGuard }
```

### VpnConnection

A saved or active VPN connection with rich metadata.

```rust
pub struct VpnConnection {
    pub uuid: String,
    pub id: String,
    pub name: String,
    pub vpn_type: VpnType,
    pub state: DeviceState,
    pub interface: Option<String>,
    pub active: bool,
    pub user_name: Option<String>,
    pub password_flags: VpnSecretFlags,
    pub service_type: String,
    pub kind: VpnKind,
}
```

### VpnConnectionInfo

Detailed active VPN information.

```rust
pub struct VpnConnectionInfo {
    pub name: String,
    pub vpn_kind: VpnKind,
    pub state: DeviceState,
    pub interface: Option<String>,
    pub gateway: Option<String>,
    pub ip4_address: Option<String>,
    pub ip6_address: Option<String>,
    pub dns_servers: Vec<String>,
    pub details: Option<VpnDetails>,
}
```

### VpnDetails

```rust
pub enum VpnDetails {
    WireGuard { public_key, endpoint },
    OpenVpn { remote, port, protocol, cipher, auth, compression },
}
```

## Saved Connection Models

### SavedConnection

Full decoded saved profile from `list_saved_connections()`.

Fields: `uuid`, `id`, `connection_type`, `interface_name`, `autoconnect`, `timestamp`, `settings`.

### SavedConnectionBrief

Lightweight: `uuid`, `id`, `connection_type`.

### SettingsPatch

Partial update for `update_saved_connection`.

## Connectivity Models

### ConnectivityState

```rust
pub enum ConnectivityState { Unknown, None, Portal, Limited, Full }
```

### ConnectivityReport

```rust
pub struct ConnectivityReport {
    pub state: ConnectivityState,
    pub check_enabled: bool,
    pub check_uri: Option<String>,
    pub captive_portal_url: Option<String>,
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
