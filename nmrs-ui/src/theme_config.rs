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

pub fn save_theme(is_light: bool) {
    if let Some(path) = get_config_path() {
        let _ = fs::write(path, if is_light { "light" } else { "dark" });
    }
}

pub fn load_theme() -> bool {
    get_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|content| content.trim() == "light")
        .unwrap_or(false) // Default to dark theme
}
