//! Secret agent request and response types.

use std::collections::HashMap;

use log::warn;
use zvariant::{OwnedObjectPath, OwnedValue, Str};

use crate::ConnectionError;

bitflags::bitflags! {
    /// Flags passed by NetworkManager with a `GetSecrets` request.
    ///
    /// These correspond to `NMSecretAgentGetSecretsFlags` in the NetworkManager
    /// D-Bus API.
    ///
    /// Reference: <https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMSecretAgentGetSecretsFlags>
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SecretAgentFlags: u32 {
        /// The agent may interact with the user (e.g. show a dialog).
        const ALLOW_INTERACTION = 0x1;
        /// The agent should discard cached secrets and prompt again.
        const REQUEST_NEW = 0x2;
        /// The request was triggered by an explicit user action, not auto-connect.
        const USER_REQUESTED = 0x4;
        /// WPS push-button mode is active on the access point.
        const WPS_PBC_ACTIVE = 0x8;
    }
}

bitflags::bitflags! {
    /// Capabilities advertised when registering the agent with NetworkManager.
    ///
    /// These correspond to `NMSecretAgentCapabilities` in the NetworkManager
    /// D-Bus API.
    ///
    /// Reference: <https://networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMSecretAgentCapabilities>
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SecretAgentCapabilities: u32 {
        /// The agent supports VPN secret hints, allowing NetworkManager to
        /// send a list of required secret keys instead of the full setting.
        const VPN_HINTS = 0x1;
    }
}

/// Identifies which connection setting needs secrets.
///
/// NetworkManager sends the setting name as part of a `GetSecrets` request.
/// This enum parses common setting names and extracts relevant context from
/// the connection dictionary.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SecretSetting {
    /// 802.11 wireless security — typically a WPA/WPA2 PSK.
    WifiPsk {
        /// The SSID of the network requesting credentials.
        ssid: String,
    },
    /// 802.1X EAP authentication.
    WifiEap {
        /// The identity (username) if already configured.
        identity: Option<String>,
        /// The EAP method if already configured (e.g. `"peap"`, `"ttls"`).
        method: Option<String>,
    },
    /// VPN secrets (password, OTP, etc.).
    Vpn {
        /// The D-Bus service name of the VPN plugin
        /// (e.g. `"org.freedesktop.NetworkManager.openvpn"`).
        service_type: String,
        /// The VPN username if already configured.
        user_name: Option<String>,
    },
    /// GSM/mobile broadband secrets.
    Gsm,
    /// CDMA mobile broadband secrets.
    Cdma,
    /// PPPoE secrets.
    Pppoe,
    /// A setting name not recognized by this library.
    Other(String),
}

/// A request from NetworkManager for connection secrets.
///
/// When NetworkManager needs credentials it does not have (e.g. a Wi-Fi
/// password was forgotten, a VPN token expired), it calls the registered
/// secret agent's `GetSecrets` method. This struct is the parsed, high-level
/// representation of that call.
///
/// Respond using the [`responder`](Self::responder) field. If the responder is
/// dropped without a response method being called, the agent auto-replies with
/// `NoSecrets` and logs a warning.
///
/// # Example
///
/// ```no_run
/// use futures::StreamExt;
/// use nmrs::agent::{SecretAgent, SecretAgentFlags, SecretSetting};
///
/// # async fn example() -> nmrs::Result<()> {
/// let (handle, mut requests) = SecretAgent::builder().register().await?;
///
/// while let Some(req) = requests.next().await {
///     println!("secrets requested for {}", req.connection_id);
///     if let SecretSetting::WifiPsk { ref ssid } = req.setting {
///         req.responder.wifi_psk("hunter2").await?;
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
pub struct SecretRequest {
    /// UUID of the connection needing secrets.
    pub connection_uuid: String,
    /// Human-readable name of the connection (e.g. `"MyWiFi"`).
    pub connection_id: String,
    /// Connection type string (e.g. `"802-11-wireless"`, `"vpn"`).
    pub connection_type: String,
    /// D-Bus object path of the connection settings object.
    pub connection_path: OwnedObjectPath,
    /// Which setting section needs secrets.
    pub setting: SecretSetting,
    /// Optional hints from NetworkManager about which secrets are needed.
    pub hints: Vec<String>,
    /// Flags describing the context of the request.
    pub flags: SecretAgentFlags,
    /// The responder used to reply with secrets or cancel.
    pub responder: SecretResponder,
}

impl std::fmt::Debug for SecretRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretRequest")
            .field("connection_uuid", &self.connection_uuid)
            .field("connection_id", &self.connection_id)
            .field("connection_type", &self.connection_type)
            .field("connection_path", &self.connection_path)
            .field("setting", &self.setting)
            .field("hints", &self.hints)
            .field("flags", &self.flags)
            .finish_non_exhaustive()
    }
}

/// Sends secrets (or a refusal) back to NetworkManager.
///
/// Each `SecretResponder` must be consumed exactly once by calling one of its
/// response methods. If dropped without being consumed, it auto-replies with
/// `NoSecrets` and logs a warning.
///
/// The response methods consume `self` to enforce single-use semantics.
pub struct SecretResponder {
    reply_tx: Option<futures::channel::oneshot::Sender<SecretReply>>,
    setting_name: String,
}

impl std::fmt::Debug for SecretResponder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretResponder")
            .field("setting_name", &self.setting_name)
            .field("consumed", &self.reply_tx.is_none())
            .finish()
    }
}

/// A cancellation notification from NetworkManager.
///
/// Emitted when NetworkManager calls `CancelGetSecrets` for an in-flight
/// request. By the time this is received, the agent has already replied to
/// NetworkManager on the consumer's behalf.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CancelReason {
    /// D-Bus object path of the cancelled connection.
    pub connection_path: OwnedObjectPath,
    /// The setting section that was being requested.
    pub setting_name: String,
}

/// A save or delete event from NetworkManager.
///
/// NetworkManager sends `SaveSecrets` and `DeleteSecrets` so agents can
/// persist or remove secrets from a keyring. Since `nmrs` delegates
/// persistence to the consumer, these are exposed as optional events.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SecretStoreEvent {
    /// NetworkManager asked the agent to persist secrets for a connection.
    Save {
        /// D-Bus object path of the connection.
        connection_path: OwnedObjectPath,
    },
    /// NetworkManager asked the agent to delete stored secrets.
    Delete {
        /// D-Bus object path of the connection.
        connection_path: OwnedObjectPath,
    },
}

pub(crate) type ConnectionDict = HashMap<String, HashMap<String, OwnedValue>>;

pub(crate) enum SecretReply {
    Secrets(ConnectionDict),
    UserCanceled,
    NoSecrets,
}

impl SecretResponder {
    pub(crate) fn new(
        reply_tx: futures::channel::oneshot::Sender<SecretReply>,
        setting_name: String,
    ) -> Self {
        Self {
            reply_tx: Some(reply_tx),
            setting_name,
        }
    }

    /// Respond with a Wi-Fi PSK (pre-shared key / password).
    ///
    /// This is the most common response for WPA/WPA2-Personal networks.
    ///
    /// # Errors
    ///
    /// Returns an error if the reply channel is already closed (e.g. the
    /// request was cancelled by NetworkManager).
    pub async fn wifi_psk(mut self, psk: impl Into<String>) -> crate::Result<()> {
        let mut inner = HashMap::new();
        inner.insert("psk".to_owned(), OwnedValue::from(Str::from(psk.into())));
        let mut outer = HashMap::new();
        outer.insert("802-11-wireless-security".to_owned(), inner);
        self.send_reply(SecretReply::Secrets(outer))
    }

    /// Respond with 802.1X EAP credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the reply channel is already closed.
    pub async fn wifi_eap(
        mut self,
        identity: Option<String>,
        password: String,
    ) -> crate::Result<()> {
        let mut inner = HashMap::new();
        inner.insert("password".to_owned(), OwnedValue::from(Str::from(password)));
        if let Some(id) = identity {
            inner.insert("identity".to_owned(), OwnedValue::from(Str::from(id)));
        }
        let mut outer = HashMap::new();
        outer.insert("802-1x".to_owned(), inner);
        self.send_reply(SecretReply::Secrets(outer))
    }

    /// Respond with VPN secrets.
    ///
    /// The keys depend on the VPN plugin (e.g. `"password"` for OpenVPN,
    /// `"Xauth password"` for vpnc). Consult the VPN plugin's documentation
    /// for the expected keys.
    ///
    /// # Errors
    ///
    /// Returns an error if the reply channel is already closed.
    pub async fn vpn_secrets(mut self, secrets: HashMap<String, String>) -> crate::Result<()> {
        let mut inner = HashMap::new();
        inner.insert("secrets".to_owned(), OwnedValue::from(secrets));
        let mut outer = HashMap::new();
        outer.insert("vpn".to_owned(), inner);
        self.send_reply(SecretReply::Secrets(outer))
    }

    /// Respond with a raw setting sub-dictionary.
    ///
    /// This is an escape hatch for setting types not covered by the
    /// convenience methods. The `setting_name` must match the setting
    /// NetworkManager requested (e.g. `"802-11-wireless-security"`).
    ///
    /// # Errors
    ///
    /// Returns an error if the reply channel is already closed.
    pub async fn raw(
        mut self,
        setting_name: impl Into<String>,
        data: HashMap<String, OwnedValue>,
    ) -> crate::Result<()> {
        let mut outer = HashMap::new();
        outer.insert(setting_name.into(), data);
        self.send_reply(SecretReply::Secrets(outer))
    }

    /// Tell NetworkManager the user canceled the secret request.
    ///
    /// This raises `org.freedesktop.NetworkManager.SecretAgent.UserCanceled`
    /// on the D-Bus side, which typically aborts the connection attempt.
    ///
    /// # Errors
    ///
    /// Returns an error if the reply channel is already closed.
    pub async fn cancel(mut self) -> crate::Result<()> {
        self.send_reply(SecretReply::UserCanceled)
    }

    /// Tell NetworkManager no secrets are available.
    ///
    /// Unlike [`cancel`](Self::cancel), this signals that the agent simply
    /// doesn't have the requested secrets. NetworkManager will not retry
    /// after receiving this.
    ///
    /// # Errors
    ///
    /// Returns an error if the reply channel is already closed.
    pub async fn no_secrets(mut self) -> crate::Result<()> {
        self.send_reply(SecretReply::NoSecrets)
    }

    fn send_reply(&mut self, reply: SecretReply) -> crate::Result<()> {
        let tx = self
            .reply_tx
            .take()
            .ok_or(ConnectionError::AgentNotRegistered)?;
        let _ = tx.send(reply);
        Ok(())
    }
}

impl Drop for SecretResponder {
    fn drop(&mut self) {
        if let Some(tx) = self.reply_tx.take() {
            warn!("SecretResponder dropped without responding; auto-replying NoSecrets");
            let _ = tx.send(SecretReply::NoSecrets);
        }
    }
}

/// Extracts a string value from a nested connection settings dictionary.
pub(crate) fn extract_setting_string(
    connection: &ConnectionDict,
    section: &str,
    key: &str,
) -> Option<String> {
    let section_dict = connection.get(section)?;
    let value = section_dict.get(key)?;
    <&str>::try_from(value).ok().map(String::from)
}

/// Extracts the SSID from the wireless setting. The SSID is stored as a byte
/// array (`ay`) in NetworkManager's connection dict.
pub(crate) fn extract_ssid(connection: &ConnectionDict) -> Option<String> {
    let wireless = connection.get("802-11-wireless")?;
    let ssid_value = wireless.get("ssid")?;
    // SSID is stored as `ay` (byte array) by NetworkManager
    if let Ok(bytes) = <Vec<u8>>::try_from(ssid_value.clone()) {
        return Some(String::from_utf8_lossy(&bytes).into_owned());
    }
    <&str>::try_from(ssid_value).ok().map(String::from)
}

/// Parses the raw `GetSecrets` arguments into a [`SecretSetting`].
pub(crate) fn parse_secret_setting(
    connection: &ConnectionDict,
    setting_name: &str,
) -> SecretSetting {
    match setting_name {
        "802-11-wireless-security" => SecretSetting::WifiPsk {
            ssid: extract_ssid(connection).unwrap_or_default(),
        },
        "802-1x" => SecretSetting::WifiEap {
            identity: extract_setting_string(connection, "802-1x", "identity"),
            method: extract_setting_string(connection, "802-1x", "eap"),
        },
        "vpn" => SecretSetting::Vpn {
            service_type: extract_setting_string(connection, "vpn", "service-type")
                .unwrap_or_default(),
            user_name: extract_setting_string(connection, "vpn", "user-name"),
        },
        "gsm" => SecretSetting::Gsm,
        "cdma" => SecretSetting::Cdma,
        "pppoe" => SecretSetting::Pppoe,
        other => SecretSetting::Other(other.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_from_bits() {
        let flags = SecretAgentFlags::from_bits_truncate(0x5);
        assert!(flags.contains(SecretAgentFlags::ALLOW_INTERACTION));
        assert!(flags.contains(SecretAgentFlags::USER_REQUESTED));
        assert!(!flags.contains(SecretAgentFlags::REQUEST_NEW));
    }

    #[test]
    fn capabilities_bits_round_trip() {
        let caps = SecretAgentCapabilities::VPN_HINTS;
        assert_eq!(caps.bits(), 0x1);
    }

    #[test]
    fn parse_wifi_psk_setting() {
        let connection = HashMap::new();
        let setting = parse_secret_setting(&connection, "802-11-wireless-security");
        assert!(matches!(setting, SecretSetting::WifiPsk { .. }));
    }

    #[test]
    fn parse_vpn_setting() {
        let connection = HashMap::new();
        let setting = parse_secret_setting(&connection, "vpn");
        assert!(matches!(setting, SecretSetting::Vpn { .. }));
    }

    #[test]
    fn parse_unknown_setting() {
        let connection = HashMap::new();
        let setting = parse_secret_setting(&connection, "some-custom-thing");
        assert!(matches!(setting, SecretSetting::Other(s) if s == "some-custom-thing"));
    }

    #[test]
    fn responder_drop_sends_no_secrets() {
        let (tx, mut rx) = futures::channel::oneshot::channel();
        let responder = SecretResponder::new(tx, "test".into());
        drop(responder);
        let reply = rx.try_recv().expect("should have received a reply");
        assert!(reply.is_some(), "drop should have sent a reply");
        assert!(matches!(reply.unwrap(), SecretReply::NoSecrets));
    }
}
