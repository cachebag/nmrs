#[test]
fn style_css_loads() {
    if std::env::var("CI").is_ok() {
        return;
    }

    gtk::init().unwrap();
    nmrs_gui::style::load_css();
}
