//! Saved connection profile management.
//!
//! Provides functions for querying and deleting saved NetworkManager
//! connection profiles. Saved connections persist across reboots and
//! store credentials for automatic reconnection.

use log::debug;
use std::collections::HashMap;
use zbus::Connection;
use zvariant::{OwnedObjectPath, Value};

use crate::util::utils::{connection_settings_proxy, settings_proxy};
use crate::Result;

/// Finds the D-Bus path of a saved connection by SSID.
///
/// Iterates through all saved connections in NetworkManager's settings
/// and returns the path of the first one whose connection ID matches
/// the given SSID.
///
/// Returns `None` if no saved connection exists for this SSID.
pub(crate) async fn get_saved_connection_path(
    conn: &Connection,
    ssid: &str,
) -> Result<Option<OwnedObjectPath>> {
    let settings = settings_proxy(conn).await?;

    let reply = settings.call_method("ListConnections", &()).await?;
    let conns: Vec<OwnedObjectPath> = reply.body().deserialize()?;

    for cpath in conns {
        let cproxy = connection_settings_proxy(conn, cpath.clone()).await?;

        let msg = cproxy.call_method("GetSettings", &()).await?;
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

    cproxy.call_method("Delete", &()).await?;
    debug!("Deleted connection: {}", conn_path.as_str());
    Ok(())
}
