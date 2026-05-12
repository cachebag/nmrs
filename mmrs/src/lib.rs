//! Rust bindings for [ModemManager](https://modemmanager.org/) over D-Bus.
//!
//! Currently in early development; the public surface is being grown
//! incrementally. See [`models`] for the available data types.

mod api;
pub mod core;
pub mod dbus;
pub mod monitoring;
pub mod types;

/// Public data types for ModemManager.
///
/// Every item in this module is also re-exported at the crate root.
pub mod models {
    pub use crate::api::models::*;
}

pub use api::models::{Sim, SimLockState};
