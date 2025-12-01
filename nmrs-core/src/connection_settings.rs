use std::collections::HashMap;
use zbus::{Connection, Result};
use zvariant::{OwnedObjectPath, Value};

pub(crate) async fn get_saved_connection_path(
    conn: &Connection,
    ssid: &str,
) -> zbus::Result<Option<OwnedObjectPath>> {
    let settings = zbus::proxy::Proxy::new(
        conn,
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager/Settings",
        "org.freedesktop.NetworkManager.Settings",
    )
    .await?;

    let reply = settings.call_method("ListConnections", &()).await?;
    let conns: Vec<OwnedObjectPath> = reply.body().deserialize()?;

    for cpath in conns {
        let cproxy = zbus::proxy::Proxy::new(
            conn,
            "org.freedesktop.NetworkManager",
            cpath.as_str(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await?;

        let msg = cproxy.call_method("GetSettings", &()).await?;
        let body = msg.body();
        let all: HashMap<String, HashMap<String, Value>> = body.deserialize()?;

        if let Some(conn_section) = all.get("connection")
            && let Some(Value::Str(id)) = conn_section.get("id")
            && id == ssid
        {
            return Ok(Some(cpath));
        }
    }

    Ok(None)
}

pub(crate) async fn has_saved_connection(conn: &Connection, ssid: &str) -> zbus::Result<bool> {
    get_saved_connection_path(conn, ssid)
        .await
        .map(|p| p.is_some())
}

pub(crate) async fn delete_connection(conn: &Connection, conn_path: OwnedObjectPath) -> Result<()> {
    let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path(conn_path.clone())?
        .interface("org.freedesktop.NetworkManager.Settings.Connection")?
        .build()
        .await?;

    cproxy.call_method("Delete", &()).await?;
    eprintln!("Deleted connection: {}", conn_path.as_str());
    Ok(())
}
