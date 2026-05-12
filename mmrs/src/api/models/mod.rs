//! Public data types for ModemManager.
//!
//! Each submodule mirrors one part of the ModemManager D-Bus surface:
//!
//! - [`modem`] — [`Modem`], [`ModemState`], [`AccessTechnology`]
//! - [`sim`] — [`Sim`], [`SimLockState`]
//!
//! The types are re-exported at the crate root for convenience.

mod modem;
mod sim;

pub use modem::{AccessTechnology, Modem, ModemState};
pub use sim::{Sim, SimLockState};
