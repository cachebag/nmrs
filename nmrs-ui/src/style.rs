use gtk::CssProvider;
use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk::gdk::Display;

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
