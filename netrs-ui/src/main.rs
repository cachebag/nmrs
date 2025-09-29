use gtk::Application;
use gtk::prelude::*;
use netrs_core::NetworkManager;
use netrs_core::models::ConnectionError;
use std::sync::Arc;

mod ui;

use ui::{device_list, sidebar, window};

#[tokio::main]
async fn main() -> Result<(), ConnectionError> {
    let nm = NetworkManager::new().await?;
    let nm = Arc::new(nm);

    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    let nm_clone = nm.clone();
    app.connect_activate(move |app| {
        let main_window = window::create_window(app);

        let (central, device_list) = device_list::create_device_list();

        let sidebar_widget = sidebar::create_sidebar(nm_clone.clone(), &device_list, &main_window);

        window::setup_layout(&main_window, &sidebar_widget, &central);
    });

    app.run();
    Ok(())
}
