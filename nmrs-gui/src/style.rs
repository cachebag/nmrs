use gtk::gdk::Display;
use gtk::gio::File;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_USER};
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

thread_local! {
    static PROVIDER: RefCell<CssProvider> = RefCell::new(CssProvider::new());
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_default().join("nmrs")
}

fn style_path() -> PathBuf {
    config_dir().join("style.css")
}

fn custom_backup_path() -> PathBuf {
    config_dir().join("style.custom.css")
}

/// Register a single persistent CSS provider and load `~/.config/nmrs/style.css`.
/// If it doesn't exist, seeds it with the bundled default.
pub fn init(default_css: &str) {
    let display = Display::default().expect("No display found");

    PROVIDER.with(|p| {
        gtk::style_context_add_provider_for_display(
            &display,
            &*p.borrow(),
            STYLE_PROVIDER_PRIORITY_USER,
        );
    });

    ensure_dir();
    if !style_path().exists() {
        write_file(&style_path(), default_css);
    }
    reload();
}

/// Switch to a named theme: if the user was on "Custom", back up their
/// `style.css` to `style.custom.css` first. Then overwrite `style.css`
/// with the theme content and reload.
pub fn switch_to_theme(css: &str) {
    let current = crate::theme_config::load_theme().unwrap_or_default();
    if current == "custom" {
        backup_custom();
    }
    write_file(&style_path(), css);
    reload();
}

/// Switch to "Custom": restore `style.custom.css` back to `style.css`
/// if a backup exists, then reload.
pub fn switch_to_custom() {
    let backup = custom_backup_path();
    if backup.exists()
        && let Ok(contents) = fs::read_to_string(&backup)
    {
        write_file(&style_path(), &contents);
    }
    reload();
}

/// Reload `~/.config/nmrs/style.css` into the persistent provider.
pub fn reload() {
    let path = style_path();
    if path.exists() {
        let file = File::for_path(&path);
        PROVIDER.with(|p| {
            p.borrow().load_from_file(&file);
        });
    }
}

fn backup_custom() {
    let src = style_path();
    if src.exists() {
        fs::copy(&src, custom_backup_path()).ok();
    }
}

fn write_file(path: &PathBuf, contents: &str) {
    ensure_dir();
    let mut f = fs::File::create(path).expect("Failed to write CSS file");
    f.write_all(contents.as_bytes())
        .expect("Failed to write CSS file");
}

fn ensure_dir() {
    fs::create_dir_all(config_dir()).ok();
}
