//! Connection state monitoring using D-Bus signals.
//!
//! Provides functions to wait for device and connection state transitions
//! using NetworkManager's signal-based API instead of polling. This approach
//! is more efficient and provides faster response times.
//!
//! # Signal-Based Monitoring
//!
//! Instead of polling device state in a loop, these functions subscribe to
//! D-Bus signals that NetworkManager emits when state changes occur:
//!
//! - `NMDevice.StateChanged` - Emitted when device state changes
//! - `NMActiveConnection.StateChanged` - Emitted when connection activation state changes
//!
//! This provides a few benefits:
//! - Immediate response to state changes (no polling delay)
//! - Lower CPU usage (no spinning loops)
//! - More reliable; at least in the sense that we won't miss rapid state transitions.
//! - Better error messages with specific failure reasons

use futures::{FutureExt, StreamExt, select};
use futures_timer::Delay;
use log::{debug, warn};
use std::pin::pin;
use std::time::Duration;
use zbus::Connection;

use crate::Result;
use crate::api::models::{
    ActiveConnectionState, ConnectionError, ConnectionStateReason, connection_state_reason_to_error,
};
use crate::dbus::{NMActiveConnectionProxy, NMDeviceProxy};
use crate::types::constants::{device_state, timeouts};

/// Default timeout for connection activation (30 seconds).
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for device disconnection (10 seconds).
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Waits for an active connection to reach the activated state.
///
/// Monitors the connection activation process by subscribing to the
/// `StateChanged` signal on the active connection object. This provides
/// more detailed error information than device-level monitoring.
pub(crate) async fn wait_for_connection_activation(
    conn: &Connection,
    active_conn_path: &zvariant::OwnedObjectPath,
) -> Result<()> {
    let active_conn = NMActiveConnectionProxy::builder(conn)
        .path(active_conn_path.clone())?
        .build()
        .await?;

    // Subscribe to signals FIRST to avoid race condition
    let mut stream = active_conn.receive_activation_state_changed().await?;
    debug!("Subscribed to ActiveConnection StateChanged signal");

    // Check current state - if already terminal, return immediately
    let current_state = active_conn.state().await?;
    let state = ActiveConnectionState::from(current_state);
    debug!("Current active connection state: {state}");

    match state {
        ActiveConnectionState::Activated => {
            debug!("Connection already activated");
            return Ok(());
        }
        ActiveConnectionState::Deactivated => {
            warn!("Connection already deactivated");
            return Err(ConnectionError::ConnectionFailed(
                ConnectionStateReason::Unknown,
            ));
        }
        _ => {}
    }

    // Wait for state change with timeout (runtime-agnostic)
    let mut timeout_delay = pin!(Delay::new(CONNECTION_TIMEOUT).fuse());

    loop {
        select! {
            _ = timeout_delay => {
                warn!("Connection activation timed out after {:?}", CONNECTION_TIMEOUT);
                return Err(ConnectionError::Timeout);
            }
            signal_opt = stream.next() => {
                match signal_opt {
                    Some(signal) => {
                        match signal.args() {
                            Ok(args) => {
                                let new_state = ActiveConnectionState::from(args.state);
                                let reason = ConnectionStateReason::from(args.reason);
                                debug!("Active connection state changed to: {new_state} (reason: {reason})");

                                match new_state {
                                    ActiveConnectionState::Activated => {
                                        debug!("Connection activation successful");
                                        return Ok(());
                                    }
                                    ActiveConnectionState::Deactivated => {
                                        debug!("Connection activation failed: {reason}");
                                        return Err(connection_state_reason_to_error(args.reason));
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse StateChanged signal args: {e}");
                            }
                        }
                    }
                    None => {
                        return Err(ConnectionError::Stuck("signal stream ended".into()));
                    }
                }
            }
        }
    }
}

/// Waits for a device to reach the disconnected state using D-Bus signals.
pub(crate) async fn wait_for_device_disconnect(dev: &NMDeviceProxy<'_>) -> Result<()> {
    // Subscribe to signals FIRST to avoid race condition
    let mut stream = dev.receive_device_state_changed().await?;
    debug!("Subscribed to device StateChanged signal for disconnect");

    let current_state = dev.state().await?;
    debug!("Current device state for disconnect: {current_state}");

    if current_state == device_state::DISCONNECTED || current_state == device_state::UNAVAILABLE {
        debug!("Device already disconnected");
        return Ok(());
    }

    // Wait for disconnect with timeout (runtime-agnostic)
    let mut timeout_delay = pin!(Delay::new(DISCONNECT_TIMEOUT).fuse());

    loop {
        select! {
            _ = timeout_delay => {
                // Check final state - might have reached target during the last moments
                let final_state = dev.state().await?;
                if final_state == device_state::DISCONNECTED || final_state == device_state::UNAVAILABLE {
                    return Ok(());
                } else {
                    warn!("Disconnect timed out, device still in state: {final_state}");
                    return Err(ConnectionError::Stuck(format!("state {final_state}")));
                }
            }
            signal_opt = stream.next() => {
                match signal_opt {
                    Some(signal) => {
                        match signal.args() {
                            Ok(args) => {
                                let new_state = args.new_state;
                                debug!("Device state during disconnect: {new_state}");

                                if new_state == device_state::DISCONNECTED
                                    || new_state == device_state::UNAVAILABLE
                                {
                                    debug!("Device reached disconnected state");
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse StateChanged signal args: {e}");
                            }
                        }
                    }
                    None => {
                        return Err(ConnectionError::Stuck("signal stream ended".into()));
                    }
                }
            }
        }
    }
}

/// Waits for a Wi-Fi device to be ready (Disconnected or Activated state).
pub(crate) async fn wait_for_wifi_device_ready(dev: &NMDeviceProxy<'_>) -> Result<()> {
    // Subscribe to signals FIRST to avoid race condition
    let mut stream = dev.receive_device_state_changed().await?;
    debug!("Subscribed to device StateChanged signal for ready check");

    let current_state = dev.state().await?;
    debug!("Current device state for ready check: {current_state}");

    if current_state == device_state::DISCONNECTED || current_state == device_state::ACTIVATED {
        debug!("Device already ready");
        return Ok(());
    }

    let ready_timeout = timeouts::wifi_ready_timeout();
    let mut timeout_delay = pin!(Delay::new(ready_timeout).fuse());

    loop {
        select! {
            _ = timeout_delay => {
                // Check final state
                let final_state = dev.state().await?;
                if final_state == device_state::DISCONNECTED || final_state == device_state::ACTIVATED {
                    return Ok(());
                } else {
                    warn!("Wi-Fi device not ready after timeout, state: {final_state}");
                    return Err(ConnectionError::WifiNotReady);
                }
            }
            signal_opt = stream.next() => {
                match signal_opt {
                    Some(signal) => {
                        match signal.args() {
                            Ok(args) => {
                                let new_state = args.new_state;
                                debug!("Device state during ready wait: {new_state}");

                                if new_state == device_state::DISCONNECTED
                                    || new_state == device_state::ACTIVATED
                                {
                                    debug!("Device is now ready");
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse StateChanged signal args: {e}");
                            }
                        }
                    }
                    None => {
                        return Err(ConnectionError::WifiNotReady);
                    }
                }
            }
        }
    }
}
