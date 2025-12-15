//! Core internal logic for connection management.
//!
//! This module contains the internal implementation details for managing
//! network connections, devices, scanning, and state monitoring.

pub(crate) mod connection;
pub(crate) mod connection_settings;
pub(crate) mod device;
pub(crate) mod scan;
pub(crate) mod state_wait;
pub(crate) mod vpn;
