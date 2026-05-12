//! Public data types for ModemManager.
//!
//! Each submodule mirrors one part of the ModemManager D-Bus surface:
//!
//! - [`modem`] — [`Modem`], [`ModemState`], [`AccessTechnology`]
//! - [`sim`] — [`Sim`], [`SimLockState`]
//! - [`bearer`] — [`Bearer`], [`BearerConfig`], [`BearerStats`], [`Ip4Config`], [`IpType`]
//!
//! The types are re-exported at the crate root for convenience.

mod bearer;
mod modem;
mod sim;

pub use bearer::{Bearer, BearerConfig, BearerStats, Ip4Config, IpType};
pub use modem::{AccessTechnology, Modem, ModemState};
pub use sim::{Sim, SimLockState};
