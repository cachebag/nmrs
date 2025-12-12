use std::fs;
use std::path::PathBuf;

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("nmrs");
        fs::create_dir_all(&path).ok()?;
        path.push("theme");
        Some(path)
    })?
}

/// Save the selected theme *name* (e.g. "nord", "gruvbox", "dracula")
pub fn save_theme(name: &str) {
    if let Some(path) = get_config_path() {
        let _ = fs::write(path, name);
    }
}

/// Load the previously selected theme.
/// Returns Some("nord") or None if missing.
pub fn load_theme() -> Option<String> {
    get_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|s| s.trim().to_string())
}
