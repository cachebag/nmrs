//! Shared test utilities.
//!
//! This module provides common test helpers that need to be shared across
//! multiple test modules to avoid race conditions.

use std::sync::Mutex;

/// Global mutex for tests that manipulate environment variables.
///
/// Any test that sets `XDG_DATA_HOME` or other env vars must hold this lock
/// to avoid race conditions with other tests running in parallel.
pub static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Runs a test closure with a fake `XDG_DATA_HOME` pointing to a unique temp directory.
///
/// This function:
/// 1. Acquires the global `ENV_LOCK` to serialize env var access
/// 2. Creates a unique temp directory
/// 3. Sets `XDG_DATA_HOME` to that directory
/// 4. Runs the provided closure
/// 5. Cleans up the env var and temp directory
///
/// If the closure panics, cleanup still happens (via Drop) but the mutex
/// will be poisoned. Use `ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())`
/// if you need to recover from poisoned state.
pub fn with_fake_xdg<R>(f: impl FnOnce() -> R) -> R {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|poisoned| {
        // Recover from poisoned mutex (previous test panicked)
        poisoned.into_inner()
    });

    let base = std::env::temp_dir().join(format!("nmrs-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base).expect("failed to create temp directory for test");

    // SAFETY: tests are serialized on ENV_LOCK; no other thread modifies env concurrently.
    unsafe {
        std::env::set_var("XDG_DATA_HOME", &base);
    }

    // Use a guard struct to ensure cleanup happens even on panic
    struct Cleanup {
        base: std::path::PathBuf,
    }

    impl Drop for Cleanup {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var("XDG_DATA_HOME");
            }
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    let _cleanup = Cleanup { base };
    f()
}
