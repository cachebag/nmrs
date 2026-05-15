//! High-level ModemManager entry point.

use std::collections::HashMap;
use std::net::Ipv4Addr;

use zbus::Connection;
use zvariant::{OwnedObjectPath, OwnedValue, Str, Value};

use crate::api::models::{
    AccessTechnology, Bearer, BearerConfig, BearerStats, ConnectionStatus, Ip4Config, Modem,
    ModemError, ModemState, Result, Sim,
};
use crate::api::modem_scope::ModemScope;
use crate::dbus::{MMBearerProxy, MMManagerProxy, MMModemProxy, MMModemSimpleProxy, MMSimProxy};

const MODEM_MANAGER_SERVICE: &str = "org.freedesktop.ModemManager1";
const MODEM_MANAGER_PATH: &str = "/org/freedesktop/ModemManager1";
const MODEM_INTERFACE: &str = "org.freedesktop.ModemManager1.Modem";

/// High-level interface to ModemManager over D-Bus.
///
/// This is the main entry point for enumerating modems, managing simple
/// packet-data connections, querying signal state, and working with SIM PINs.
#[derive(Debug, Clone)]
pub struct ModemManager {
    conn: Connection,
}

impl ModemManager {
    /// Connects to the system D-Bus and creates a new [`ModemManager`].
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        Self::with_connection(conn).await
    }

    /// Creates a [`ModemManager`] from an existing D-Bus connection.
    pub async fn with_connection(conn: Connection) -> Result<Self> {
        MMManagerProxy::new(&conn).await?;
        Ok(Self { conn })
    }

    /// Returns the underlying D-Bus connection.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Lists all modems currently managed by ModemManager.
    pub async fn list_modems(&self) -> Result<Vec<Modem>> {
        let paths = enumerate_modem_paths(&self.conn).await?;
        let mut modems = Vec::with_capacity(paths.len());

        for path in paths {
            modems.push(self.modem_info_for_path(path.as_str()).await?);
        }

        Ok(modems)
    }

    /// Returns the modem whose equipment identifier matches the given IMEI.
    pub async fn modem_by_imei(&self, imei: &str) -> Result<Modem> {
        self.list_modems()
            .await?
            .into_iter()
            .find(|modem| modem.equipment_identifier == imei)
            .ok_or_else(|| ModemError::ModemNotFound(format!("IMEI {imei}")))
    }

    /// Returns the first modem reported by ModemManager.
    pub async fn primary_modem(&self) -> Result<Modem> {
        let mut modems = self.list_modems().await?;
        if modems.is_empty() {
            return Err(ModemError::NoModems);
        }
        Ok(modems.remove(0))
    }

    /// Creates a scope for operating on a specific modem object path.
    #[must_use]
    pub fn modem(&self, path: &str) -> ModemScope<'_> {
        ModemScope::new(self, path)
    }

    /// Enables the primary modem.
    pub async fn enable(&self) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.enable_for_path(path.as_str()).await
    }

    /// Disables the primary modem.
    pub async fn disable(&self) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.disable_for_path(path.as_str()).await
    }

    /// Connects the primary modem using only an APN.
    ///
    /// Uses `Modem.Simple.Connect`, which lets ModemManager handle the
    /// one-shot enable, registration, and bearer connection flow.
    pub async fn connect_simple(&self, apn: &str) -> Result<Bearer> {
        self.connect(&BearerConfig::new(apn)).await
    }

    /// Connects the primary modem using a full bearer configuration.
    pub async fn connect(&self, config: &BearerConfig) -> Result<Bearer> {
        let path = self.primary_modem_path().await?;
        self.connect_for_path(path.as_str(), config).await
    }

    /// Disconnects all bearers on the primary modem.
    pub async fn disconnect(&self) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.disconnect_for_path(path.as_str()).await
    }

    /// Returns the primary modem's current connection status.
    pub async fn status(&self) -> Result<ConnectionStatus> {
        let path = self.primary_modem_path().await?;
        self.status_for_path(path.as_str()).await
    }

    /// Returns the primary modem's active SIM, if one is reported.
    pub async fn sim(&self) -> Result<Option<Sim>> {
        let path = self.primary_modem_path().await?;
        self.sim_for_path(path.as_str()).await
    }

    /// Sends a PIN to unlock the primary modem's SIM.
    pub async fn unlock_pin(&self, pin: &str) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.unlock_pin_for_path(path.as_str(), pin).await
    }

    /// Sends a PUK and new PIN to unlock the primary modem's SIM.
    pub async fn unlock_puk(&self, puk: &str, new_pin: &str) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.unlock_puk_for_path(path.as_str(), puk, new_pin).await
    }

    /// Enables or disables SIM PIN checking on the primary modem.
    pub async fn set_pin_enabled(&self, pin: &str, enabled: bool) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.set_pin_enabled_for_path(path.as_str(), pin, enabled)
            .await
    }

    /// Changes the primary modem SIM's PIN.
    pub async fn change_pin(&self, old: &str, new: &str) -> Result<()> {
        let path = self.primary_modem_path().await?;
        self.change_pin_for_path(path.as_str(), old, new).await
    }

    /// Returns the primary modem's current signal quality percentage.
    pub async fn signal_quality(&self) -> Result<u32> {
        let path = self.primary_modem_path().await?;
        self.signal_quality_for_path(path.as_str()).await
    }

    /// Returns the primary modem's current access technology bitmask.
    pub async fn access_technology(&self) -> Result<AccessTechnology> {
        let path = self.primary_modem_path().await?;
        self.access_technology_for_path(path.as_str()).await
    }

    pub(crate) async fn modem_info_for_path(&self, path: &str) -> Result<Modem> {
        let modem_path = modem_object_path(path)?;
        let proxy = MMModemProxy::builder(&self.conn)
            .path(modem_path.clone())?
            .build()
            .await?;

        let (signal_quality, _) = proxy.signal_quality().await?;
        let sim_path = proxy.sim().await?;
        let bearer_paths = proxy
            .bearers()
            .await?
            .into_iter()
            .map(|path| path.to_string())
            .collect();

        Ok(Modem {
            path: modem_path.to_string(),
            state: ModemState::from_raw(proxy.state().await?),
            manufacturer: proxy.manufacturer().await?,
            model: proxy.model().await?,
            equipment_identifier: proxy.equipment_identifier().await?,
            access_technologies: AccessTechnology::from(proxy.access_technologies().await?),
            signal_quality,
            primary_sim_path: object_path_option(&sim_path),
            bearer_paths,
        })
    }

    pub(crate) async fn enable_for_path(&self, path: &str) -> Result<()> {
        let proxy = modem_proxy(&self.conn, path).await?;
        proxy.enable(true).await?;
        Ok(())
    }

    pub(crate) async fn disable_for_path(&self, path: &str) -> Result<()> {
        let proxy = modem_proxy(&self.conn, path).await?;
        proxy.enable(false).await?;
        Ok(())
    }

    pub(crate) async fn connect_for_path(
        &self,
        path: &str,
        config: &BearerConfig,
    ) -> Result<Bearer> {
        if config.apn.trim().is_empty() {
            return Err(ModemError::InvalidApn(config.apn.clone()));
        }

        let proxy = modem_simple_proxy(&self.conn, path).await?;
        let bearer_path = proxy
            .connect(bearer_properties(config))
            .await
            .map_err(|e| ModemError::BearerCreationFailed(format!("Simple.Connect failed: {e}")))?;

        bearer_snapshot(&self.conn, &bearer_path).await
    }

    pub(crate) async fn disconnect_for_path(&self, path: &str) -> Result<()> {
        let proxy = modem_simple_proxy(&self.conn, path).await?;
        let all_bearers = OwnedObjectPath::try_from("/").map_err(|e| {
            ModemError::BearerDisconnectFailed(format!("invalid all-bearers path: {e}"))
        })?;

        proxy.disconnect(all_bearers).await.map_err(|e| {
            ModemError::BearerDisconnectFailed(format!("Simple.Disconnect failed: {e}"))
        })
    }

    pub(crate) async fn status_for_path(&self, path: &str) -> Result<ConnectionStatus> {
        let simple = modem_simple_proxy(&self.conn, path).await?;
        let status = simple.get_status().await?;
        let modem = self.modem_info_for_path(path).await?;

        let state = take_i32(&status, "state")
            .map(ModemState::from_raw)
            .unwrap_or(modem.state);
        let access_technology = take_u32(&status, "access-technology")
            .or_else(|| take_u32(&status, "access-technologies"))
            .map(AccessTechnology::from)
            .unwrap_or(modem.access_technologies);
        let signal_quality = take_u32(&status, "signal-quality").or(Some(modem.signal_quality));

        Ok(ConnectionStatus {
            modem_path: modem.path,
            state,
            connected: state.is_connected(),
            access_technology,
            signal_quality,
            bearer_paths: modem.bearer_paths,
        })
    }

    pub(crate) async fn sim_for_path(&self, path: &str) -> Result<Option<Sim>> {
        let modem = modem_proxy(&self.conn, path).await?;
        let sim_path = modem.sim().await?;
        if object_path_option(&sim_path).is_none() {
            return Ok(None);
        }

        let proxy = MMSimProxy::builder(&self.conn)
            .path(sim_path.clone())?
            .build()
            .await?;

        Ok(Some(Sim {
            path: sim_path.to_string(),
            active: proxy.active().await?,
            iccid: proxy.sim_identifier().await?,
            imsi: proxy.imsi().await?,
            operator_name: proxy.operator_name().await?,
        }))
    }

    pub(crate) async fn unlock_pin_for_path(&self, path: &str, pin: &str) -> Result<()> {
        let sim = sim_proxy_for_modem(&self.conn, path).await?;
        sim.send_pin(pin).await.map_err(|e| {
            if is_wrong_pin_error(&e) {
                ModemError::WrongPin
            } else {
                ModemError::Dbus(e)
            }
        })
    }

    pub(crate) async fn unlock_puk_for_path(
        &self,
        path: &str,
        puk: &str,
        new_pin: &str,
    ) -> Result<()> {
        let sim = sim_proxy_for_modem(&self.conn, path).await?;
        sim.send_puk(puk, new_pin).await.map_err(|e| {
            if is_wrong_puk_error(&e) {
                ModemError::WrongPuk
            } else {
                ModemError::Dbus(e)
            }
        })
    }

    pub(crate) async fn set_pin_enabled_for_path(
        &self,
        path: &str,
        pin: &str,
        enabled: bool,
    ) -> Result<()> {
        let sim = sim_proxy_for_modem(&self.conn, path).await?;
        sim.enable_pin(pin, enabled).await.map_err(|e| {
            if is_wrong_pin_error(&e) {
                ModemError::WrongPin
            } else {
                ModemError::Dbus(e)
            }
        })
    }

    pub(crate) async fn change_pin_for_path(&self, path: &str, old: &str, new: &str) -> Result<()> {
        let sim = sim_proxy_for_modem(&self.conn, path).await?;
        sim.change_pin(old, new).await.map_err(|e| {
            if is_wrong_pin_error(&e) {
                ModemError::WrongPin
            } else {
                ModemError::Dbus(e)
            }
        })
    }

    pub(crate) async fn signal_quality_for_path(&self, path: &str) -> Result<u32> {
        let proxy = modem_proxy(&self.conn, path).await?;
        let (quality, _) = proxy.signal_quality().await?;
        Ok(quality)
    }

    pub(crate) async fn access_technology_for_path(&self, path: &str) -> Result<AccessTechnology> {
        let proxy = modem_proxy(&self.conn, path).await?;
        Ok(AccessTechnology::from(proxy.access_technologies().await?))
    }

    async fn primary_modem_path(&self) -> Result<String> {
        enumerate_modem_paths(&self.conn)
            .await?
            .into_iter()
            .next()
            .ok_or(ModemError::NoModems)
    }
}

async fn enumerate_modem_paths(conn: &Connection) -> Result<Vec<String>> {
    let manager = zbus::fdo::ObjectManagerProxy::builder(conn)
        .destination(MODEM_MANAGER_SERVICE)?
        .path(MODEM_MANAGER_PATH)?
        .build()
        .await?;

    let objects = manager.get_managed_objects().await?;
    let mut paths: Vec<String> = objects
        .into_iter()
        .filter(|(_, ifaces)| ifaces.contains_key(MODEM_INTERFACE))
        .map(|(path, _)| path.to_string())
        .collect();
    paths.sort();
    Ok(paths)
}

async fn modem_proxy<'a>(conn: &'a Connection, path: &str) -> Result<MMModemProxy<'a>> {
    Ok(MMModemProxy::builder(conn)
        .path(modem_object_path(path)?)?
        .build()
        .await?)
}

async fn modem_simple_proxy<'a>(
    conn: &'a Connection,
    path: &str,
) -> Result<MMModemSimpleProxy<'a>> {
    Ok(MMModemSimpleProxy::builder(conn)
        .path(modem_object_path(path)?)?
        .build()
        .await?)
}

async fn sim_proxy_for_modem<'a>(conn: &'a Connection, path: &str) -> Result<MMSimProxy<'a>> {
    let modem = modem_proxy(conn, path).await?;
    let sim_path = modem.sim().await?;
    if object_path_option(&sim_path).is_none() {
        return Err(ModemError::NoSim);
    }

    Ok(MMSimProxy::builder(conn).path(sim_path)?.build().await?)
}

async fn bearer_snapshot(conn: &Connection, path: &OwnedObjectPath) -> Result<Bearer> {
    let proxy = MMBearerProxy::builder(conn)
        .path(path.clone())?
        .build()
        .await?;
    let ip4 = proxy.ip4_config().await?;
    let stats = proxy.stats().await?;

    Ok(Bearer {
        path: path.to_string(),
        interface: proxy.interface().await?,
        connected: proxy.connected().await?,
        ip4_config: decode_ip4_config(&ip4),
        stats: decode_bearer_stats(&stats),
    })
}

fn bearer_properties(config: &BearerConfig) -> HashMap<&str, Value<'_>> {
    let mut properties = HashMap::new();
    properties.insert("apn", Value::from(config.apn.as_str()));
    properties.insert("ip-type", Value::from(config.ip_type.as_raw()));
    properties.insert("allow-roaming", Value::from(config.allow_roaming));

    if let Some(user) = &config.user {
        properties.insert("user", Value::from(user.as_str()));
    }
    if let Some(password) = &config.password {
        properties.insert("password", Value::from(password.as_str()));
    }

    properties
}

fn modem_object_path(path: &str) -> Result<OwnedObjectPath> {
    OwnedObjectPath::try_from(path)
        .map_err(|e| ModemError::ModemNotFound(format!("{path} (invalid D-Bus object path: {e})")))
}

fn object_path_option(path: &OwnedObjectPath) -> Option<String> {
    let path = path.to_string();
    if path == "/" { None } else { Some(path) }
}

fn decode_ip4_config(values: &HashMap<String, OwnedValue>) -> Option<Ip4Config> {
    if values.is_empty() {
        return None;
    }

    Some(Ip4Config {
        method: take_str(values, "method")
            .or_else(|| take_u32(values, "method").map(|value| value.to_string()))
            .unwrap_or_default(),
        address: take_str(values, "address").and_then(|value| value.parse().ok()),
        prefix: take_u32(values, "prefix").unwrap_or_default(),
        gateway: take_str(values, "gateway").and_then(|value| value.parse().ok()),
        dns: take_ipv4_vec(values, "dns"),
        mtu: take_u32(values, "mtu"),
    })
}

fn decode_bearer_stats(values: &HashMap<String, OwnedValue>) -> BearerStats {
    BearerStats {
        rx_bytes: take_u64(values, "rx-bytes").unwrap_or_default(),
        tx_bytes: take_u64(values, "tx-bytes").unwrap_or_default(),
        duration_seconds: take_u32(values, "duration").unwrap_or_default(),
        attempts: take_u32(values, "attempts").unwrap_or_default(),
        failed_attempts: take_u32(values, "failed-attempts").unwrap_or_default(),
        total_duration_seconds: take_u32(values, "total-duration").unwrap_or_default(),
        total_rx_bytes: take_u64(values, "total-rx-bytes").unwrap_or_default(),
        total_tx_bytes: take_u64(values, "total-tx-bytes").unwrap_or_default(),
    }
}

fn take_str(values: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    values.get(key).and_then(owned_to_str)
}

fn take_u32(values: &HashMap<String, OwnedValue>, key: &str) -> Option<u32> {
    values.get(key).and_then(owned_to_u32)
}

fn take_i32(values: &HashMap<String, OwnedValue>, key: &str) -> Option<i32> {
    values.get(key).and_then(owned_to_i32)
}

fn take_u64(values: &HashMap<String, OwnedValue>, key: &str) -> Option<u64> {
    values.get(key).and_then(owned_to_u64)
}

fn take_ipv4_vec(values: &HashMap<String, OwnedValue>, key: &str) -> Vec<Ipv4Addr> {
    let Some(value) = values.get(key) else {
        return Vec::new();
    };

    if let Ok(strings) = Vec::<String>::try_from(value.clone()) {
        return strings
            .into_iter()
            .filter_map(|value| value.parse().ok())
            .collect();
    }

    if let Ok(numbers) = Vec::<u32>::try_from(value.clone()) {
        return numbers.into_iter().map(Ipv4Addr::from).collect();
    }

    Vec::new()
}

fn owned_to_str(value: &OwnedValue) -> Option<String> {
    Str::try_from(value.clone())
        .ok()
        .map(|value| value.to_string())
        .or_else(|| String::try_from(value.clone()).ok())
}

fn owned_to_u32(value: &OwnedValue) -> Option<u32> {
    u32::try_from(value.clone()).ok().or_else(|| {
        i32::try_from(value.clone())
            .ok()
            .and_then(|value| value.try_into().ok())
    })
}

fn owned_to_i32(value: &OwnedValue) -> Option<i32> {
    i32::try_from(value.clone()).ok().or_else(|| {
        u32::try_from(value.clone())
            .ok()
            .and_then(|value| value.try_into().ok())
    })
}

fn owned_to_u64(value: &OwnedValue) -> Option<u64> {
    u64::try_from(value.clone())
        .ok()
        .or_else(|| u32::try_from(value.clone()).ok().map(u64::from))
}

fn is_wrong_pin_error(error: &zbus::Error) -> bool {
    let rendered = error.to_string().to_ascii_lowercase();
    rendered.contains("wrong") && rendered.contains("pin")
}

fn is_wrong_puk_error(error: &zbus::Error) -> bool {
    let rendered = error.to_string().to_ascii_lowercase();
    rendered.contains("wrong") && rendered.contains("puk")
}
