pub mod style;
pub mod ui;

use gtk::Application;
use gtk::prelude::*;

pub fn run() {
    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    app.connect_activate(|app| {
        crate::style::load_css();
        crate::ui::build_ui(app);
    });

    app.run();
}
