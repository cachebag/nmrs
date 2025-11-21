use gtk::gdk::Display;
use gtk::gio::File;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION, STYLE_PROVIDER_PRIORITY_USER};

fn load_user_css_if_exists(display: &Display) {
    let path = dirs::config_dir()
        .unwrap_or_default()
        .join("nmrs/style.css");

    if path.exists() {
        let provider = CssProvider::new();
        let file = File::for_path(&path);

        provider.load_from_file(&file);

        gtk::style_context_add_provider_for_display(
            display,
            &provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );
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

    load_user_css_if_exists(&display);
}
