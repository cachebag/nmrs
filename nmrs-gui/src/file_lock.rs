use fs2::FileExt;
use std::fs::File;

pub fn acquire_app_lock() -> Result<File, String> {
    let mut lock_path = dirs::data_local_dir().unwrap_or(std::env::temp_dir());
    lock_path.push("my_app.lock");

    let file = File::create(&lock_path).map_err(|e| format!("Failed to create lock file: {e}"))?;

    // Exclusive lock; fails if another instance holds it
    file.try_lock_exclusive()
        .map_err(|_| "Another instance is already running".to_string())?;

    Ok(file)
}
