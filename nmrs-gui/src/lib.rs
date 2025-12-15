pub mod file_lock;
pub mod objects;
pub mod style;
pub mod theme_config;
pub mod ui;

use clap::{ArgAction, Parser};
use gtk::prelude::*;
use gtk::Application;

use crate::file_lock::acquire_app_lock;
use crate::style::load_css;
use crate::ui::build_ui;

#[derive(Parser, Debug)]
#[command(name = "nmrs")]
#[command(disable_version_flag = true)]
#[command(version)]
struct Args {
    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue)]
    version: bool,
}

pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Args { version: true } = args {
        println!(
            "nmrs {}-beta ({})",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        );
        return Ok(());
    }

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
