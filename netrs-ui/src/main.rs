use gtk::Application;
use gtk::prelude::*;
use netrs_core::{NetworkManager, models::ConnectionError};
use std::sync::Arc;

mod style;
mod ui;

use crate::style::load_css;

#[tokio::main]
async fn main() -> Result<(), ConnectionError> {
    let nm = Arc::new(NetworkManager::new().await?);
    let networks = nm.list_networks().await?;

    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    app.connect_activate(move |app| {
        load_css();
        ui::build_ui(app, networks.clone());
    });

    app.run();
    Ok(())
}
