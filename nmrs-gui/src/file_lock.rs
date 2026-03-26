use fs2::FileExt;
use std::fs;
use std::fs::File;

pub fn acquire_app_lock() -> Result<File, String> {
    // Prefer XDG_RUNTIME_DIR for ephemeral lock files (should be /run/usr/<USR_ID>). Fall back to local data dir.
    let mut lock_dir = dirs::runtime_dir()
        .or_else(|| dirs::data_local_dir())
        .unwrap_or(std::env::temp_dir());

    lock_dir.push("nmrs");

    fs::create_dir_all(&lock_dir)
        .map_err(|e| format!("Failed to create lock directory: {e}"))?;

    let lock_path = lock_dir.join("app.lock");

    let file = File::create(&lock_path)
        .map_err(|e| format!("Failed to create lock file: {e}"))?;

    // Exclusive lock; fails if another instance holds it
    file.try_lock_exclusive()
        .map_err(|_| "Another instance is already running".to_string())?;

    Ok(file)
}