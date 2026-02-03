//! Saved connection profile management.
//!
//! Provides functions for querying and deleting saved NetworkManager
//! connection profiles. Saved connections persist across reboots and
//! store credentials for automatic reconnection.

use log::debug;
use std::collections::HashMap;
use zbus::Connection;
use zvariant::{OwnedObjectPath, Value};

use crate::api::models::ConnectionError;
use crate::util::utils::{connection_settings_proxy, settings_proxy};
use crate::util::validation::validate_ssid;
use crate::Result;

/// Finds the D-Bus path of a saved connection by SSID or connection name.
///
/// Iterates through all saved connections in NetworkManager's settings
/// and returns the path of the first one whose connection ID matches
/// the given SSID or name.
///
/// Note: This function is used for both WiFi SSIDs and VPN connection names.
/// The validation enforces WiFi SSID rules (max 32 bytes), which is also
/// reasonable for VPN connection names.
///
/// Returns `None` if no saved connection exists for this SSID/name.
pub(crate) async fn get_saved_connection_path(
    conn: &Connection,
    ssid: &str,
) -> Result<Option<OwnedObjectPath>> {
    // Validate the connection name/SSID
    if ssid.trim().is_empty() {
        return Ok(None);
    }

    // Validate using SSID rules (max 32 bytes, no special chars)
    // This applies to both WiFi SSIDs and connection names
    validate_ssid(ssid)?;

    let settings = settings_proxy(conn).await?;

    let reply = settings
        .call_method("ListConnections", &())
        .await
        .map_err(|e| ConnectionError::DbusOperation {
            context: "failed to list saved connections".to_string(),
            source: e,
        })?;

    let conns: Vec<OwnedObjectPath> = reply.body().deserialize()?;

    for cpath in conns {
        let cproxy = connection_settings_proxy(conn, cpath.clone()).await?;

        let msg = cproxy.call_method("GetSettings", &()).await.map_err(|e| {
            ConnectionError::DbusOperation {
                context: format!("failed to get settings for {}", cpath.as_str()),
                source: e,
            }
        })?;

        let body = msg.body();
        let all: HashMap<String, HashMap<String, Value>> = body.deserialize()?;

        if let Some(conn_section) = all.get("connection") {
            if let Some(Value::Str(id)) = conn_section.get("id") {
                if id == ssid {
                    return Ok(Some(cpath));
                }
            }
        }
    }

    Ok(None)
}

/// Checks whether a saved connection exists for the given SSID.
pub(crate) async fn has_saved_connection(conn: &Connection, ssid: &str) -> Result<bool> {
    get_saved_connection_path(conn, ssid)
        .await
        .map(|p| p.is_some())
}

/// Deletes a saved connection by its D-Bus path.
///
/// Calls the Delete method on the connection settings object.
/// This permanently removes the saved connection from NetworkManager.
pub(crate) async fn delete_connection(conn: &Connection, conn_path: OwnedObjectPath) -> Result<()> {
    let cproxy = connection_settings_proxy(conn, conn_path.clone()).await?;

    cproxy
        .call_method("Delete", &())
        .await
        .map_err(|e| ConnectionError::DbusOperation {
            context: format!("failed to delete connection {}", conn_path.as_str()),
            source: e,
        })?;

    debug!("Deleted connection: {}", conn_path.as_str());
    Ok(())
}

/// Lists all saved connection profiles.
///
/// Returns a vector of connection names (IDs) for all saved profiles
/// in NetworkManager. This includes WiFi, Ethernet, VPN, and other connection types.
pub(crate) async fn list_saved_connections(conn: &Connection) -> Result<Vec<String>> {
    let settings = settings_proxy(conn).await?;

    let reply = settings
        .call_method("ListConnections", &())
        .await
        .map_err(|e| ConnectionError::DbusOperation {
            context: "failed to list saved connections".to_string(),
            source: e,
        })?;

    let conns: Vec<OwnedObjectPath> = reply.body().deserialize()?;

    let mut connection_names = Vec::new();

    for cpath in conns {
        let cproxy = connection_settings_proxy(conn, cpath.clone()).await?;

        if let Ok(msg) = cproxy.call_method("GetSettings", &()).await {
            let body = msg.body();
            if let Ok(all) = body.deserialize::<HashMap<String, HashMap<String, Value>>>() {
                if let Some(conn_section) = all.get("connection") {
                    if let Some(Value::Str(id)) = conn_section.get("id") {
                        connection_names.push(id.to_string());
                    }
                }
            }
        }
    }

    debug!("Found {} saved connection(s)", connection_names.len());
    Ok(connection_names)
}
