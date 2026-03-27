use fs2::FileExt;
use std::fs;
use std::fs::File;

pub fn acquire_app_lock() -> Result<File, String> {
    // Prefer XDG_RUNTIME_DIR for ephemeral lock files (should be /run/usr/<USR_ID>). Fall back to local data dir.
    let mut lock_dir = dirs::runtime_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or(std::env::temp_dir());

    lock_dir.push("nmrs");

    fs::create_dir_all(&lock_dir).map_err(|e| format!("Failed to create lock directory: {e}"))?;

    let lock_path = lock_dir.join("app.lock");

    let file = File::create(&lock_path).map_err(|e| format!("Failed to create lock file: {e}"))?;

    // Exclusive lock; fails if another instance holds it
    file.try_lock_exclusive()
        .map_err(|_| "Another instance is already running".to_string())?;

    Ok(file)
}

// Used so each test can be run independently.
// Without that, with tests being run in parallel, the second test can cause the first one to fail.
#[cfg(test)]
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_lock_file_is_created() {
    // Test if the lock file is properly created.
    let _guard = TEST_LOCK.lock().unwrap();
    let file = acquire_app_lock();
    assert!(file.is_ok(), "Failed to acquire lock: {:?}", file.err());

    let lock_path = dirs::runtime_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or(std::env::temp_dir())
        .join("nmrs")
        .join("app.lock");

    assert!(
        lock_path.exists(),
        "Lock file was not created at {:?}",
        lock_path
    );
}

#[test]
fn test_second_instance_is_rejected() {
    // Test if acquire_app_lock fails as it should when called twice.
    let _guard = TEST_LOCK.lock().unwrap();
    let _first = acquire_app_lock().expect("First lock should succeed");
    let second = acquire_app_lock();
    assert!(second.is_err(), "Second instance should have been rejected");
    assert_eq!(second.unwrap_err(), "Another instance is already running");
}
