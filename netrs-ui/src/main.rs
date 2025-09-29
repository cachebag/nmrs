use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, Label};
use netrs_core::NetworkManager;
use netrs_core::models::ConnectionError;
use std::sync::Arc;

mod ui;
mod style;

use crate::style::load_css;
use crate::ui::header::build_header;

#[tokio::main]
async fn main() -> Result<(), ConnectionError> {
    let nm = NetworkManager::new().await?;
    let nm = Arc::new(nm);

    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    let _nm_clone = nm.clone();
    app.connect_activate(move |app| {
        load_css();

        let win = ApplicationWindow::builder()
            .application(app)
            .title("")
            .default_width(800)
            .default_height(600)
            .build();

        let root = GtkBox::new(Orientation::Vertical, 6);

        let status = Label::new(None);
        let header = build_header(&status);
        win.set_titlebar(Some(&header));

        root.append(&status);
        win.set_child(Some(&root));

        win.show();
    });

    app.run();
    Ok(())
}
