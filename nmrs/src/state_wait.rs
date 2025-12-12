//! Connection state monitoring.
//!
//! Provides functions to wait for device state transitions during
//! connection establishment, with proper error mapping for failures.

use futures_timer::Delay;

use crate::Result;
use crate::constants::{device_state, retries, timeouts};
use crate::models::{ConnectionError, StateReason, reason_to_error};
use crate::proxies::NMDeviceProxy;

/// Waits for a device to reach the activated state or fail.
///
/// Polls the device state until activation succeeds, fails, or times out.
/// Returns a structured error indicating the specific failure reason.
pub(crate) async fn wait_for_connection_state(dev: &NMDeviceProxy<'_>) -> Result<()> {
    let mut config_stuck = 0;

    for i in 0..retries::CONNECTION_MAX_RETRIES {
        Delay::new(timeouts::connection_poll_interval()).await;

        let raw = dev.state().await?;

        if raw == device_state::ACTIVATED {
            return Ok(());
        }

        if raw == device_state::FAILED {
            return Err(match dev.state_reason().await {
                Ok((_, code)) => reason_to_error(code),
                Err(_) => ConnectionError::Failed(StateReason::Unknown),
            });
        }

        if raw == device_state::CONFIG {
            config_stuck += 1;
            if config_stuck > retries::CONNECTION_CONFIG_STUCK_THRESHOLD {
                return Err(ConnectionError::Stuck("config".into()));
            }
        } else {
            config_stuck = 0;
        }

        if i > retries::CONNECTION_STUCK_CHECK_START && raw == device_state::DISCONNECTED {
            return Err(ConnectionError::Stuck("disconnected".into()));
        }
    }

    Err(ConnectionError::Timeout)
}
