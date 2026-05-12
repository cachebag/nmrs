//! Error types for ModemManager operations.

use thiserror::Error;

use super::sim::SimLockState;

/// Errors that can occur during ModemManager operations.
///
/// All fallible operations in `mmrs` return [`Result<T, ModemError>`].
///
/// # Examples
///
/// ```rust
/// use mmrs::{ModemError, SimLockState};
///
/// fn handle(err: ModemError) {
///     match err {
///         ModemError::NoModems => eprintln!("no modems available"),
///         ModemError::SimLocked(lock) => {
///             eprintln!("SIM is locked: {:?}", lock);
///         }
///         ModemError::WrongPin => eprintln!("incorrect PIN"),
///         other => eprintln!("modem error: {other}"),
///     }
/// }
/// # handle(ModemError::SimLocked(SimLockState::SimPin));
/// ```
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ModemError {
    /// A D-Bus communication error occurred.
    #[error("d-bus error: {0}")]
    Dbus(#[from] zbus::Error),

    /// A D-Bus operation failed, with context about what was being attempted.
    #[error("{context}: {source}")]
    DbusOperation {
        /// Human-readable description of the operation that failed.
        context: String,
        /// The underlying `zbus` error.
        #[source]
        source: zbus::Error,
    },

    /// ModemManager reported no managed modems.
    #[error("no modems found")]
    NoModems,

    /// No modem was found at the requested D-Bus path.
    #[error("modem not found: {0}")]
    ModemNotFound(String),

    /// The modem is in the failed state and cannot be used.
    #[error("modem in failed state: {0}")]
    ModemFailed(String),

    /// The modem is currently disabled (call `enable` first).
    #[error("modem is disabled")]
    ModemDisabled,

    /// No SIM is inserted in the modem (or the slot is empty).
    #[error("no SIM present")]
    NoSim,

    /// The SIM is locked and requires unlocking before use.
    #[error("sim locked: {0:?}")]
    SimLocked(SimLockState),

    /// The supplied PIN was incorrect.
    #[error("wrong pin")]
    WrongPin,

    /// The supplied PUK was incorrect (PIN remains locked).
    #[error("wrong puk")]
    WrongPuk,

    /// The PIN format was invalid (e.g. non-digit or wrong length).
    #[error("invalid pin format: {0}")]
    InvalidPin(String),

    /// Bearer creation failed with the given reason.
    #[error("bearer creation failed: {0}")]
    BearerCreationFailed(String),

    /// No bearer was found at the requested D-Bus path.
    #[error("bearer not found: {0}")]
    BearerNotFound(String),

    /// Activating the bearer (bringing up the data connection) failed.
    #[error("bearer connect failed: {0}")]
    BearerConnectFailed(String),

    /// Deactivating the bearer failed.
    #[error("bearer disconnect failed: {0}")]
    BearerDisconnectFailed(String),

    /// The configured APN was rejected or invalid.
    #[error("invalid apn: {0}")]
    InvalidApn(String),

    /// The operation timed out waiting for the modem.
    #[error("modem operation timed out")]
    Timeout,

    /// The operation is not supported by this modem or firmware.
    #[error("operation not supported: {0}")]
    Unsupported(String),

    /// A string property returned by ModemManager was not valid UTF-8.
    #[error("invalid utf-8 in modem property: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    /// An integer property could not be parsed.
    #[error("invalid integer in modem property: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
}

/// Convenience alias for `Result<T, ModemError>`.
pub type Result<T> = std::result::Result<T, ModemError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_context() {
        let err = ModemError::ModemNotFound("/org/freedesktop/ModemManager1/Modem/0".into());
        let rendered = err.to_string();
        assert!(rendered.contains("modem not found"));
        assert!(rendered.contains("Modem/0"));
    }

    #[test]
    fn sim_locked_carries_state() {
        let err = ModemError::SimLocked(SimLockState::SimPuk);
        assert!(err.to_string().contains("SimPuk"));
    }

    #[test]
    fn dbus_operation_chains_source() {
        let err = ModemError::DbusOperation {
            context: "calling Enable".into(),
            source: zbus::Error::InvalidReply,
        };
        let rendered = err.to_string();
        assert!(rendered.contains("calling Enable"));
        assert!(std::error::Error::source(&err).is_some());
    }
}
