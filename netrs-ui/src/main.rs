use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Label, Button, Orientation};

fn main() {
    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("netrs")
            .default_width(400)
            .default_height(300)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 10);

        let label = Label::new(Some("Hello from netrs!"));

        let button = Button::with_label("Quit");
        let win_clone = window.clone();
        button.connect_clicked(move |_| {
            win_clone.close();
        });

        vbox.append(&label);
        vbox.append(&button);

        window.set_child(Some(&vbox));
        window.present();
    });

    app.run();
}
