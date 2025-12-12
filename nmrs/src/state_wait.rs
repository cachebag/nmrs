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

use futures::StreamExt;
use log::{debug, warn};
use std::time::Duration;
use tokio::time::timeout;
use zbus::Connection;

use crate::Result;
use crate::constants::{device_state, timeouts};
use crate::models::{
    ActiveConnectionState, ConnectionError, ConnectionStateReason, connection_state_reason_to_error,
};
use crate::proxies::{NMActiveConnectionProxy, NMDeviceProxy};

/// Default timeout for connection activation (30 seconds).
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for device disconnection (10 seconds).
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Waits for an active connection to reach the activated state.
///
/// Monitors the connection activation process by subscribing to the
/// `StateChanged` signal on the active connection object. This provides
/// more detailed error information than device-level monitoring.
///
/// # Arguments
///
/// * `conn` - The D-Bus connection
/// * `active_conn_path` - Path to the active connection object
///
/// # Errors
///
/// Returns an error if:
/// - The connection enters the `Deactivated` state (with the specific failure reason)
/// - The timeout expires before activation completes
/// - A D-Bus communication error occurs
///
/// # Example
///
/// ```ignore
/// let (_, active_conn) = nm.add_and_activate_connection(settings, device, ap).await?;
/// wait_for_connection_activation(&conn, &active_conn).await?;
/// ```
pub(crate) async fn wait_for_connection_activation(
    conn: &Connection,
    active_conn_path: &zvariant::OwnedObjectPath,
) -> Result<()> {
    let active_conn = NMActiveConnectionProxy::builder(conn)
        .path(active_conn_path.clone())?
        .build()
        .await?;

    // Check current state first
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

    // Subscribe to state change signals
    let mut stream = active_conn.receive_activation_state_changed().await?;
    debug!("Subscribed to ActiveConnection StateChanged signal");

    // Wait for state change with timeout
    let result = timeout(CONNECTION_TIMEOUT, async {
        while let Some(signal) = stream.next().await {
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
                        _ => {
                            // Still in progress (Activating/Deactivating)
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse StateChanged signal args: {e}");
                }
            }
        }
        // Stream ended unexpectedly
        Err(ConnectionError::Stuck("signal stream ended".into()))
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            warn!(
                "Connection activation timed out after {:?}",
                CONNECTION_TIMEOUT
            );
            Err(ConnectionError::Timeout)
        }
    }
}

/// Waits for a device to reach the disconnected state using D-Bus signals.
///
/// Used when disconnecting from a network to ensure the device has fully
/// released the connection before attempting a new one.
///
/// # Arguments
///
/// * `dev` - The device proxy to monitor
///
/// # Errors
///
/// Returns an error if:
/// - The timeout expires before disconnection completes
/// - A D-Bus communication error occurs
pub(crate) async fn wait_for_device_disconnect(dev: &NMDeviceProxy<'_>) -> Result<()> {
    // Check current state first
    let current_state = dev.state().await?;
    debug!("Current device state for disconnect: {current_state}");

    if current_state == device_state::DISCONNECTED || current_state == device_state::UNAVAILABLE {
        debug!("Device already disconnected");
        return Ok(());
    }

    // Subscribe to state change signals
    let mut stream = dev.receive_device_state_changed().await?;
    debug!("Subscribed to device StateChanged signal for disconnect");

    // Wait for disconnect with timeout
    let result = timeout(DISCONNECT_TIMEOUT, async {
        while let Some(signal) = stream.next().await {
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
        Err(ConnectionError::Stuck("signal stream ended".into()))
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            // Check final state - might have reached target during the last moments
            let final_state = dev.state().await?;
            if final_state == device_state::DISCONNECTED || final_state == device_state::UNAVAILABLE
            {
                Ok(())
            } else {
                warn!("Disconnect timed out, device still in state: {final_state}");
                Err(ConnectionError::Stuck(format!("state {final_state}")))
            }
        }
    }
}

/// Waits for a Wi-Fi device to be ready (Disconnected or Activated state).
///
/// Used after enabling Wi-Fi to wait for the device to initialize before
/// performing operations like scanning.
///
/// # Arguments
///
/// * `dev` - The device proxy to monitor
///
/// # Errors
///
/// Returns `WifiNotReady` if the device doesn't become ready within the timeout.
pub(crate) async fn wait_for_wifi_device_ready(dev: &NMDeviceProxy<'_>) -> Result<()> {
    // Check current state first
    let current_state = dev.state().await?;
    debug!("Current device state for ready check: {current_state}");

    if current_state == device_state::DISCONNECTED || current_state == device_state::ACTIVATED {
        debug!("Device already ready");
        return Ok(());
    }

    // Subscribe to state change signals
    let mut stream = dev.receive_device_state_changed().await?;
    debug!("Subscribed to device StateChanged signal for ready check");

    let ready_timeout = timeouts::wifi_ready_timeout();

    let result = timeout(ready_timeout, async {
        while let Some(signal) = stream.next().await {
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
        Err(ConnectionError::WifiNotReady)
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => {
            // Check final state
            let final_state = dev.state().await?;
            if final_state == device_state::DISCONNECTED || final_state == device_state::ACTIVATED {
                Ok(())
            } else {
                warn!("Wi-Fi device not ready after timeout, state: {final_state}");
                Err(ConnectionError::WifiNotReady)
            }
        }
    }
}
