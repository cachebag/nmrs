pub mod file_lock;
pub mod style;
pub mod ui;

use gtk::Application;
use gtk::prelude::*;

use crate::file_lock::acquire_app_lock;
use crate::style::load_css;
use crate::ui::build_ui;

pub fn run() -> anyhow::Result<()> {
    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    let _lock = match acquire_app_lock() {
        Ok(lock) => lock,
        Err(e) => {
            eprintln!("Failed to start: {e}");
            std::process::exit(1);
        }
    };

    app.connect_activate(|app| {
        load_css();
        build_ui(app);
    });

    app.run();
    Ok(())
}
