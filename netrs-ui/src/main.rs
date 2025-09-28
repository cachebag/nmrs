use glib::MainContext;
use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CenterBox, Label, ListBox,
    ListBoxRow, Orientation,
};
use netrs_core::NetworkManager;
use netrs_core::models::ConnectionError;

#[tokio::main]
async fn main() -> Result<(), ConnectionError> {
    let nm = NetworkManager::new().await?;
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
        let central = GtkBox::new(Orientation::Vertical, 5);

        let grid = gtk::Grid::new();
        let quit = Button::with_label("Quit");
        let show_devices = Button::with_label("Show available devices");

        let device_list = ListBox::new();

        // Handler for showing devices
        let nm_inner = nm_clone.clone();
        let device_list_clone = device_list.clone();
        show_devices.connect_clicked(move |_| {
            let nm2 = nm_inner.clone();
            let device_list_ref = device_list_clone.clone();

            MainContext::default().spawn_local(async move {
                match nm2.list_devices().await {
                    Ok(devs) => {
                        // Schedule UI update on GTK main thread
                        glib::idle_add_local_once(move || {
                            while let Some(child) = device_list_ref.first_child() {
                                device_list_ref.remove(&child);
                            }

                            for d in devs {
                                let row = ListBoxRow::new();
                                let label = Label::new(Some(&format!(
                                    "{} ({}) - {} [{}]",
                                    d.interface,
                                    d.device_type,
                                    d.state,
                                    d.driver.as_deref().unwrap_or("unknown"),
                                )));
                                row.set_child(Some(&label));
                                device_list_ref.append(&row);
                            }
                            device_list_ref.show();
                        });
                    }
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            });
        });

        grid.attach(&show_devices, 0, 0, 1, 1);
        grid.attach(&quit, 0, 2, 3, 1);

        let win_clone = window.clone();
        quit.connect_clicked(move |_| {
            win_clone.close();
        });

        sidebar.append(&grid);
        central.append(&device_list);

        let layout = CenterBox::new();
        layout.set_start_widget(Some(&sidebar));
        layout.set_center_widget(Some(&central));
        sidebar.set_valign(Align::Center);
        central.set_halign(Align::Center);
        central.set_valign(Align::Center);

        window.set_child(Some(&layout));
        window.present();
    });

    app.run();
    Ok(())
}
