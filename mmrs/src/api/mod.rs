//! Public-facing API surface for the `mmrs` crate.
//!
//! Exposes the high-level [`ModemManager`] entry point, scoped
//! [`ModemScope`] operations, and the [`models`] sub-module.

pub mod models;
mod modem_manager;
mod modem_scope;

pub use modem_manager::ModemManager;
pub use modem_scope::ModemScope;
