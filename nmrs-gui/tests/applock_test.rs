#[cfg(test)]
mod tests {
    use nmrs_gui::file_lock::acquire_app_lock;
    use std::sync::Mutex;

    // Used so each test can be run independently.
    // Without that, with tests being run in parallel, the second test can cause the first one to fail.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_lock_file_is_created() {
        // Test if the lock file is properly created.
        let _guard = TEST_LOCK.lock().unwrap();
        let file = acquire_app_lock();
        assert!(file.is_ok(), "Failed to acquire lock: {:?}", file.err());

        let lock_dir = dirs::runtime_dir()
            .or_else(dirs::data_local_dir)
            .unwrap_or(std::env::temp_dir())
            .join("nmrs")
            .join("app.lock");

        assert!(
            lock_dir.exists(),
            "Lock file was not created at {:?}",
            lock_dir
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
}
