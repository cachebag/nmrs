//! Secret agent builder, handle, and lifecycle management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use futures::channel::mpsc;
use log::debug;
use zbus::Connection;

use crate::ConnectionError;
use crate::dbus::AgentManagerProxy;

use super::iface::SecretAgentInterface;
use super::request::{CancelReason, SecretAgentCapabilities, SecretRequest, SecretStoreEvent};

const DEFAULT_IDENTIFIER: &str = "com.system76.CosmicApplets.nmrs.secret_agent";
const DEFAULT_OBJECT_PATH: &str = "/com/system76/CosmicApplets/nmrs/SecretAgent";
const DEFAULT_QUEUE_DEPTH: usize = 32;

/// Entry point for creating a NetworkManager secret agent.
///
/// A secret agent receives credential requests from NetworkManager over D-Bus
/// whenever a connection needs secrets the system does not already have (Wi-Fi
/// password forgotten, VPN token expired, etc.).
///
/// Use [`SecretAgent::builder()`] to configure and register the agent.
///
/// # Example
///
/// ```no_run
/// use futures::StreamExt;
/// use nmrs::agent::{SecretAgent, SecretSetting};
///
/// # async fn example() -> nmrs::Result<()> {
/// let (handle, mut requests) = SecretAgent::builder().register().await?;
///
/// while let Some(req) = requests.next().await {
///     if let SecretSetting::WifiPsk { ref ssid } = req.setting {
///         println!("password requested for {ssid}");
///         req.responder.wifi_psk("my-password").await?;
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct SecretAgent;

impl SecretAgent {
    /// Returns a builder for configuring and registering a secret agent.
    #[must_use]
    pub fn builder() -> SecretAgentBuilder {
        SecretAgentBuilder::default()
    }
}

/// Builder for configuring and registering a [`SecretAgent`].
///
/// Use the `with_*` methods to override defaults, then call
/// [`register()`](Self::register) to connect to the system bus and start
/// serving.
///
/// # Defaults
///
/// | Setting | Default |
/// |---------|---------|
/// | identifier | `com.system76.CosmicApplets.nmrs.secret_agent` |
/// | capabilities | [`SecretAgentCapabilities::VPN_HINTS`] |
/// | object path | `/com/system76/CosmicApplets/nmrs/SecretAgent` |
/// | queue depth | 32 |
#[derive(Debug)]
pub struct SecretAgentBuilder {
    identifier: String,
    capabilities: SecretAgentCapabilities,
    object_path: String,
    queue_depth: usize,
}

impl Default for SecretAgentBuilder {
    fn default() -> Self {
        Self {
            identifier: DEFAULT_IDENTIFIER.into(),
            capabilities: SecretAgentCapabilities::VPN_HINTS,
            object_path: DEFAULT_OBJECT_PATH.into(),
            queue_depth: DEFAULT_QUEUE_DEPTH,
        }
    }
}

impl SecretAgentBuilder {
    /// Sets the D-Bus well-known name the agent will own.
    ///
    /// This must be unique on the system bus. If another process already owns
    /// the name, registration will fail with
    /// [`ConnectionError::AgentAlreadyRegistered`].
    #[must_use]
    pub fn with_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.identifier = identifier.into();
        self
    }

    /// Sets the capabilities advertised to NetworkManager.
    ///
    /// Defaults to [`SecretAgentCapabilities::VPN_HINTS`].
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: SecretAgentCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Sets the D-Bus object path where the agent interface is served.
    #[must_use]
    pub fn with_object_path(mut self, path: impl Into<String>) -> Self {
        self.object_path = path.into();
        self
    }

    /// Sets the maximum number of `GetSecrets` requests to buffer before
    /// back-pressure kicks in. Defaults to 32.
    #[must_use]
    pub fn with_queue_depth(mut self, depth: usize) -> Self {
        self.queue_depth = depth;
        self
    }

    /// Connects to the system bus, registers the agent with NetworkManager,
    /// and returns a handle and a stream of incoming secret requests.
    ///
    /// The returned [`mpsc::Receiver`](futures::channel::mpsc::Receiver)
    /// implements [`Stream`](futures::Stream) and yields
    /// [`SecretRequest`] items as they arrive from NetworkManager.
    ///
    /// # Errors
    ///
    /// - [`ConnectionError::AgentAlreadyRegistered`] if another process
    ///   already owns the requested bus name.
    /// - [`ConnectionError::AgentRegistration`] if the bus name could not
    ///   be acquired or NetworkManager rejected the registration.
    /// - [`ConnectionError::Dbus`] for other D-Bus failures.
    pub async fn register(
        self,
    ) -> crate::Result<(SecretAgentHandle, mpsc::Receiver<SecretRequest>)> {
        let (request_tx, request_rx) = mpsc::channel(self.queue_depth);
        let (cancel_tx, cancel_rx) = mpsc::unbounded();
        let (store_tx, store_rx) = mpsc::unbounded();

        let iface = SecretAgentInterface {
            request_tx,
            cancel_tx,
            store_tx,
            pending: Arc::new(Mutex::new(HashMap::new())),
        };

        let conn = Connection::system()
            .await
            .map_err(|e| ConnectionError::DbusOperation {
                context: "connecting to system bus for secret agent".into(),
                source: e,
            })?;

        conn.object_server()
            .at(&*self.object_path, iface)
            .await
            .map_err(|e| ConnectionError::DbusOperation {
                context: format!("serving SecretAgent interface at {}", self.object_path),
                source: e,
            })?;

        conn.request_name(&*self.identifier).await.map_err(|e| {
            ConnectionError::AgentRegistration {
                context: format!("bus name '{}': {e}", self.identifier),
            }
        })?;

        debug!(
            "Acquired bus name '{}', serving at '{}'",
            self.identifier, self.object_path
        );

        let agent_proxy =
            AgentManagerProxy::new(&conn)
                .await
                .map_err(|e| ConnectionError::DbusOperation {
                    context: "creating AgentManager proxy".into(),
                    source: e,
                })?;

        agent_proxy
            .register_with_capabilities(&self.identifier, self.capabilities.bits())
            .await
            .map_err(|e| ConnectionError::DbusOperation {
                context: "registering secret agent with NetworkManager".into(),
                source: e,
            })?;

        debug!(
            "Registered secret agent '{}' with capabilities {:?}",
            self.identifier, self.capabilities
        );

        let handle = SecretAgentHandle {
            conn,
            identifier: self.identifier,
            capabilities: self.capabilities,
            object_path: self.object_path,
            cancel_rx,
            store_rx,
        };

        Ok((handle, request_rx))
    }
}

/// Handle to a running secret agent.
///
/// Provides methods to re-register after a NetworkManager restart, access
/// the cancellation and store-event streams, and shut the agent down.
pub struct SecretAgentHandle {
    conn: Connection,
    identifier: String,
    capabilities: SecretAgentCapabilities,
    object_path: String,
    cancel_rx: mpsc::UnboundedReceiver<CancelReason>,
    store_rx: mpsc::UnboundedReceiver<SecretStoreEvent>,
}

impl std::fmt::Debug for SecretAgentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretAgentHandle")
            .field("identifier", &self.identifier)
            .field("object_path", &self.object_path)
            .finish_non_exhaustive()
    }
}

impl SecretAgentHandle {
    /// Re-registers the agent with NetworkManager.
    ///
    /// Call this after detecting that NetworkManager restarted (e.g. its
    /// D-Bus name owner changed). The call is idempotent while the bus
    /// connection is healthy.
    ///
    /// # Errors
    ///
    /// Returns an error if the D-Bus call to `RegisterWithCapabilities` fails.
    pub async fn reregister(&self) -> crate::Result<()> {
        let proxy = AgentManagerProxy::new(&self.conn).await.map_err(|e| {
            ConnectionError::DbusOperation {
                context: "creating AgentManager proxy for re-registration".into(),
                source: e,
            }
        })?;
        proxy
            .register_with_capabilities(&self.identifier, self.capabilities.bits())
            .await
            .map_err(|e| ConnectionError::DbusOperation {
                context: "re-registering secret agent with NetworkManager".into(),
                source: e,
            })?;
        debug!("Re-registered secret agent '{}'", self.identifier);
        Ok(())
    }

    /// Unregisters the agent from NetworkManager and releases the bus name.
    ///
    /// After this call, the request stream returned by
    /// [`SecretAgentBuilder::register`] will complete.
    ///
    /// # Errors
    ///
    /// Returns an error if the D-Bus `Unregister` call fails.
    pub async fn unregister(self) -> crate::Result<()> {
        let proxy = AgentManagerProxy::new(&self.conn).await.map_err(|e| {
            ConnectionError::DbusOperation {
                context: "creating AgentManager proxy for unregistration".into(),
                source: e,
            }
        })?;
        proxy
            .unregister()
            .await
            .map_err(|e| ConnectionError::DbusOperation {
                context: "unregistering secret agent".into(),
                source: e,
            })?;
        self.conn
            .release_name(&*self.identifier)
            .await
            .map_err(|e| ConnectionError::DbusOperation {
                context: format!("releasing bus name '{}'", self.identifier),
                source: e,
            })?;
        debug!("Unregistered secret agent '{}'", self.identifier);
        Ok(())
    }

    /// Returns a mutable reference to the cancellation stream.
    ///
    /// Yields [`CancelReason`] items when NetworkManager calls
    /// `CancelGetSecrets` for an in-flight request. By the time the
    /// consumer receives this event, the agent has already replied to
    /// NetworkManager.
    ///
    /// Use with [`StreamExt::next()`](futures::StreamExt::next):
    ///
    /// ```ignore
    /// while let Some(reason) = handle.cancellations().next().await {
    ///     println!("cancelled: {}", reason.setting_name);
    /// }
    /// ```
    pub fn cancellations(&mut self) -> &mut mpsc::UnboundedReceiver<CancelReason> {
        &mut self.cancel_rx
    }

    /// Returns a mutable reference to the save/delete event stream.
    ///
    /// Yields [`SecretStoreEvent`] items when NetworkManager sends
    /// `SaveSecrets` or `DeleteSecrets`. These are informational — the agent
    /// always acknowledges them immediately.
    pub fn store_events(&mut self) -> &mut mpsc::UnboundedReceiver<SecretStoreEvent> {
        &mut self.store_rx
    }
}
