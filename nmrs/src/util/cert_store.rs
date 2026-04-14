//! Persist inline PEM material from `.ovpn` profiles to disk for NetworkManager
//!
//! # Connection-rename caveat
//!
//! The cert directory is keyed by `connection_name` at import time. If a user
//! later renames the NM connection (e.g. via `nmcli`), `forget_vpn` will look
//! for `certs/<new_name>/` which won't exist, and `certs/<old_name>/` will
//! linger on disk. A future improvement could store the cert directory name in
//! a custom `vpn.data` key so cleanup remains correct after renames.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{ConnectionError, util::validation::validate_connection_name};

/// Writes PEM bytes for one material type and returns an **absolute** path for `vpn.data`.
///
/// `cert_type`: `"ca"`, `"cert"`, `"key"`, or `"ta"` (tls-auth static key).
///
/// The write is atomic: data is flushed to a temporary file in the same
/// directory and then renamed into place, so readers never see a half-written
/// PEM file.
pub fn store_inline_cert(
    connection_name: &str,
    cert_type: &str,
    pem_data: &str,
) -> Result<PathBuf, ConnectionError> {
    let dir = connection_cert_dir(connection_name)?;
    fs::create_dir_all(&dir).map_err(|e| {
        ConnectionError::VpnFailed(format!(
            "cert store: create directory {}: {e}",
            dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
            ConnectionError::VpnFailed(format!(
                "cert store: chmod directory {}: {e}",
                dir.display()
            ))
        })?;
    }

    let filename = filename_for(cert_type)?;
    let path = dir.join(filename);
    let tmp_path = dir.join(format!(".{filename}.tmp"));

    {
        let mut opts = OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut file = opts.open(&tmp_path).map_err(|e| {
            ConnectionError::VpnFailed(format!(
                "cert store: open {} for write: {e}",
                tmp_path.display(),
            ))
        })?;
        file.write_all(pem_data.as_bytes()).map_err(|e| {
            ConnectionError::VpnFailed(format!("cert store: write {}: {e}", tmp_path.display(),))
        })?;
        file.sync_all().map_err(|e| {
            ConnectionError::VpnFailed(format!("cert store: sync {}: {e}", tmp_path.display()))
        })?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            ConnectionError::VpnFailed(format!("cert store: chmod {}: {e}", tmp_path.display()))
        })?;
    }

    fs::rename(&tmp_path, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        ConnectionError::VpnFailed(format!(
            "cert store: rename {} -> {}: {e}",
            tmp_path.display(),
            path.display()
        ))
    })?;

    path.canonicalize().map_err(|e| {
        ConnectionError::VpnFailed(format!("cert store: canonicalize {}: {e}", path.display()))
    })
}

/// Removes all stored cert files for this connection.
///
/// **Idempotent:** if the directory does not exist, returns `Ok(())`.
pub fn cleanup_certs(connection_name: &str) -> Result<(), ConnectionError> {
    let dir = connection_cert_dir(connection_name)?;
    match fs::remove_dir_all(&dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ConnectionError::VpnFailed(format!(
            "cert store: remove {}: {e}",
            dir.display()
        ))),
    }
}

/// Resolved XDG data home: `$XDG_DATA_HOME`, or `$HOME/.local/share` if unset or empty.
fn xdg_data_home() -> Result<PathBuf, ConnectionError> {
    match std::env::var_os("XDG_DATA_HOME") {
        Some(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => {
            let home = std::env::var_os("HOME").ok_or_else(|| {
                ConnectionError::VpnFailed(
                    "cert store: HOME is not set (cannot resolve XDG data directory)".into(),
                )
            })?;
            Ok(Path::new(&home).join(".local/share"))
        }
    }
}

/// `$XDG_DATA_HOME/nmrs/certs/<connection_name>/`
fn connection_cert_dir(connection_name: &str) -> Result<PathBuf, ConnectionError> {
    validate_connection_name(connection_name)?;
    if connection_name.contains('/') || connection_name.contains('\\') {
        return Err(ConnectionError::InvalidAddress(
            "connection name must not contain path separators".into(),
        ));
    }
    if connection_name == "." || connection_name == ".." {
        return Err(ConnectionError::InvalidAddress(
            "invalid connection name".into(),
        ));
    }
    Ok(xdg_data_home()?
        .join("nmrs")
        .join("certs")
        .join(connection_name))
}

fn filename_for(cert_type: &str) -> Result<&'static str, ConnectionError> {
    match cert_type {
        "ca" => Ok("ca.pem"),
        "cert" => Ok("cert.pem"),
        "key" => Ok("key.pem"),
        "ta" => Ok("ta.key"),
        "tls-crypt" => Ok("tls-crypt.key"),
        _ => Err(ConnectionError::InvalidAddress(format!(
            "unknown cert_type {cert_type:?} (expected ca, cert, key, ta, tls-crypt)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_fake_xdg<R>(f: impl FnOnce() -> R) -> R {
        let _g = ENV_LOCK.lock().unwrap();
        let base = std::env::temp_dir().join(format!("nmrs-cert-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();
        // SAFETY: tests are serialized on this mutex; no other thread reads env concurrently.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", &base);
        }
        let out = f();
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
        let _ = std::fs::remove_dir_all(&base);
        out
    }

    #[test]
    fn write_read_cleanup_cycle() {
        with_fake_xdg(|| {
            let pem = "-----BEGIN CERTIFICATE-----\nABC\n-----END CERTIFICATE-----\n";
            let p = store_inline_cert("MyVPN", "ca", pem).unwrap();
            let got = std::fs::read_to_string(&p).unwrap();
            assert_eq!(got, pem);
            cleanup_certs("MyVPN").unwrap();
            assert!(!p.exists());
        });
    }

    #[test]
    fn cleanup_nonexistent_is_ok() {
        with_fake_xdg(|| {
            cleanup_certs("does-not-exist").unwrap();
        });
    }

    #[test]
    fn double_cleanup_ok() {
        with_fake_xdg(|| {
            store_inline_cert("x", "ca", "pem").unwrap();
            cleanup_certs("x").unwrap();
            cleanup_certs("x").unwrap();
        });
    }

    #[cfg(unix)]
    #[test]
    fn permissions_are_rw_for_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        with_fake_xdg(|| {
            let p = store_inline_cert("perm", "key", "secret").unwrap();
            let mode = std::fs::metadata(&p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        });
    }
}
