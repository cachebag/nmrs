use gtk::gdk::Display;
use gtk::gio::File;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};
use std::fs;
use std::io::Write;

/// Load and apply the user's custom ~/.config/nmrs/style.css.
/// If the file does not exist it is created with the bundled defaults.
/// This must be called after any theme provider is registered so that
/// same-priority (STYLE_PROVIDER_PRIORITY_USER) rules resolve in favour of
/// the user's stylesheet.
pub fn load_user_css() {
    let path = dirs::config_dir()
        .unwrap_or_default()
        .join("nmrs/style.css");

    let display = Display::default().expect("No display found");

    if path.exists() {
        let provider = CssProvider::new();
        let file = File::for_path(&path);

        provider.load_from_file(&file);

        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }

        let default = include_str!("style.css");
        let mut f = fs::File::create(&path).expect("Failed to create CSS file");
        f.write_all(default.as_bytes())
            .expect("Failed to write default CSS");
    }
}

pub fn load_css() {
    let provider = CssProvider::new();

    let css = include_str!("style.css");
    provider.load_from_data(css);

    let display = Display::default().expect("No display found");

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
