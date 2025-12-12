//! D-Bus proxy traits for NetworkManager interfaces.
//!
//! These traits define the NetworkManager D-Bus API surface used by this crate.
//! The `zbus::proxy` macro generates proxy implementations that handle
//! D-Bus communication automatically.
//!
//! # NetworkManager D-Bus Structure
//!
//! - `/org/freedesktop/NetworkManager` - Main NM object
//! - `/org/freedesktop/NetworkManager/Devices/*` - Device objects
//! - `/org/freedesktop/NetworkManager/AccessPoint/*` - Access point objects
//! - `/org/freedesktop/NetworkManager/ActiveConnection/*` - Active connection objects
//! - `/org/freedesktop/NetworkManager/Settings` - Connection settings
//!
//! # Signal-based State Monitoring
//!
//! This crate uses D-Bus signals for efficient state monitoring instead of polling:
//! - `NMDevice::StateChanged` - Emitted when device state changes
//! - `NMActiveConnection::StateChanged` - Emitted when connection activation state changes
//!
//! Use the generated `receive_device_state_changed()` and `receive_activation_state_changed()`
//! methods to get signal streams.

mod access_point;
mod active_connection;
mod device;
mod main_nm;
mod wireless;

pub use access_point::NMAccessPointProxy;
pub use active_connection::NMActiveConnectionProxy;
pub use device::NMDeviceProxy;
pub use main_nm::NMProxy;
pub use wireless::NMWirelessProxy;
