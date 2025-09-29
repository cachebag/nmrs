use gtk::Application;
use gtk::prelude::*;

mod style;
mod ui;

use crate::style::load_css;

#[tokio::main]
async fn main() {
    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    app.connect_activate(move |app| {
        load_css();
        ui::build_ui(app);
    });

    app.run();
}
