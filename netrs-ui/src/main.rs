use glib::MainContext;
use gtk::prelude::*;
use gtk::{Align, Application, ApplicationWindow, Box as GtkBox, Button, CenterBox, Orientation};
use netrs_core::NetworkManager;
use netrs_core::models::ConnectionError;

#[tokio::main]
async fn main() -> Result<(), ConnectionError> {
    let nm = NetworkManager::new().await?;
    // move nm into the closure by putting it in an Arc
    let nm = std::sync::Arc::new(nm);

    let app = Application::builder()
        .application_id("org.netrs.ui")
        .build();

    let nm_clone = nm.clone();
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("netrs")
            .default_width(400)
            .default_height(300)
            .build();

        let sidebar = GtkBox::new(Orientation::Vertical, 5);

        let grid = gtk::Grid::new();
        let quit = Button::with_label("Quit");
        let show_devices = Button::with_label("Show available devices");

        grid.attach(&show_devices, 0, 0, 1, 1);
        grid.attach(&quit, 0, 2, 3, 1);

        let win_clone = window.clone();
        quit.connect_clicked(move |_| {
            win_clone.close();
        });

        let nm_inner = nm_clone.clone();
        show_devices.connect_clicked(move |_| {
            // run async inside GTK main loop
            let nm2 = nm_inner.clone();
            MainContext::default().spawn_local(async move {
                match nm2.list_devices().await {
                    Ok(devs) => {
                        for d in devs {
                            eprintln!("Device: {:?}", d);
                        }
                    }
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            });
        });

        sidebar.append(&grid);

        let layout = CenterBox::new();
        layout.set_start_widget(Some(&sidebar));
        sidebar.set_valign(Align::Center);

        window.set_child(Some(&layout));
        window.present();
    });

    app.run();
    Ok(())
}
