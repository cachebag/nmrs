#[test]
fn app_initializes_without_panic() {
    // Skip when no display (e.g. in CI)
    if std::env::var("CI").is_ok() {
        return;
    }

    gtk::init().unwrap();
    let result = std::panic::catch_unwind(|| {
        nmrs_ui::run();
    });
    assert!(result.is_ok(), "UI startup panicked");
}
