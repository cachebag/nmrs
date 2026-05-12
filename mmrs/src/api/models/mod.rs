//! Public data types for ModemManager.
//!
//! Each submodule mirrors one part of the ModemManager D-Bus surface:
//!
//! - [`sim`] — [`Sim`], [`SimLockState`]
//!
//! The types are re-exported at the crate root for convenience.

mod sim;

pub use sim::{Sim, SimLockState};
