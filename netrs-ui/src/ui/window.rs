use gtk::prelude::*;
use gtk::{Align, Application, ApplicationWindow, Box as GtkBox, CenterBox};

pub fn create_window(app: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("netrs")
        .default_width(400)
        .default_height(300)
        .build()
}

pub fn setup_layout(window: &ApplicationWindow, sidebar: &GtkBox, central: &GtkBox) {
    let layout = CenterBox::new();
    layout.set_start_widget(Some(sidebar));
    layout.set_center_widget(Some(central));

    sidebar.set_valign(Align::Center);

    window.set_child(Some(&layout));
    window.present();
}
