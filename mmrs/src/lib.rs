//! Rust bindings for [ModemManager](https://modemmanager.org/) over D-Bus.
//!
//! This crate is in early development. The currently stable surface is the
//! set of public **model types** that describe modems, SIMs, and packet-data
//! bearers as exposed by ModemManager. Higher-level helpers
//! (connect / disconnect, monitoring, builders) will land on top of these
//! types in subsequent releases.
//!
//! # Modules
//!
//! - [`models`] re-exports every public data type. The same types are
//!   re-exported at the crate root for convenience (so
//!   `mmrs::ModemState` and `mmrs::models::ModemState` refer to the same
//!   item).
//!
//! # Quick reference
//!
//! - **Modem** — [`Modem`], [`ModemState`], [`AccessTechnology`]
//! - **SIM** — [`Sim`], [`SimLockState`]
//! - **Bearer** — [`Bearer`], [`BearerConfig`], [`BearerStats`],
//!   [`Ip4Config`], [`IpType`]
//! - **Errors** — [`ModemError`], [`Result`]
//!
//! # Example
//!
//! ```rust
//! use mmrs::{AccessTechnology, BearerConfig, IpType, ModemState};
//!
//! let state = ModemState::from_raw(11);
//! assert!(state.is_connected());
//!
//! let tech = AccessTechnology::from(0x4000); // MM_MODEM_ACCESS_TECHNOLOGY_LTE
//! assert!(tech.has_lte());
//!
//! let cfg = BearerConfig::new("internet")
//!     .with_ip_type(IpType::Ipv4v6)
//!     .with_user("user")
//!     .with_password("hunter2");
//! assert_eq!(cfg.apn, "internet");
//! ```

mod api;
pub mod core;
pub mod dbus;
pub mod monitoring;
pub mod types;

/// Public data types for ModemManager (modems, SIMs, bearers, errors).
///
/// Every item in this module is also re-exported at the crate root.
pub mod models {
    pub use crate::api::models::*;
}

pub use api::models::{
    AccessTechnology, Bearer, BearerConfig, BearerStats, Ip4Config, IpType, Modem, ModemError,
    ModemState, Result, Sim, SimLockState,
};
