//! Rust bindings for [ModemManager](https://modemmanager.org/) over D-Bus.
//!
//! This crate is in early development. The public surface includes the
//! high-level [`ModemManager`] entry point plus model types that describe
//! modems, SIMs, and packet-data bearers as exposed by ModemManager.
//!
//! # Modules
//!
//! - [`models`] re-exports every public data type under [`crate::api::models`].
//!   The same types are re-exported at the crate root, so `mmrs::ModemState`,
//!   `mmrs::models::ModemState`, and `mmrs::api::models::ModemState` refer to
//!   the same item.
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
//! ```no_run
//! use mmrs::{AccessTechnology, BearerConfig, IpType, ModemManager, ModemState};
//!
//! # async fn example() -> mmrs::Result<()> {
//! let mm = ModemManager::new().await?;
//! let modems = mm.list_modems().await?;
//! # let _ = modems;
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
//! # Ok(())
//! # }
//! ```

pub mod api;
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
    AccessTechnology, Bearer, BearerConfig, BearerStats, ConnectionStatus, Ip4Config, IpType,
    Modem, ModemError, ModemState, Result, Sim, SimLockState,
};
pub use api::{ModemManager, ModemScope};
