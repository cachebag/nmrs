//! D-Bus proxy interfaces for ModemManager.
//!
//! This module contains low-level D-Bus proxy definitions for communicating
//! with ModemManager over the system bus.

// Re-exports are consumed by core/api layers that are not yet implemented.
#![allow(unused_imports)]

mod bearer;
mod manager;
mod modem;
mod modem_simple;
mod sim;

pub(crate) use bearer::MMBearerProxy;
pub(crate) use manager::MMManagerProxy;
pub(crate) use modem::MMModemProxy;
pub(crate) use modem_simple::MMModemSimpleProxy;
pub(crate) use sim::MMSimProxy;
