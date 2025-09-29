use glib::MainContext;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, ListBox, Orientation};
use netrs_core::NetworkManager;
use std::sync::Arc;

pub fn create_sidebar(
    nm: Arc<NetworkManager>,
    device_list: &ListBox,
    window: &gtk::ApplicationWindow,
) -> GtkBox {
    let sidebar = GtkBox::new(Orientation::Vertical, 5);
    let grid = gtk::Grid::new();

    let quit = Button::with_label("Quit");
    let show_devices = Button::with_label("Show available devices");

    // Event handler for show devices button
    let nm_clone = nm.clone();
    let device_list_clone = device_list.clone();
    show_devices.connect_clicked(move |_| {
        let nm2 = nm_clone.clone();
        let device_list_ref = device_list_clone.clone();

        MainContext::default().spawn_local(async move {
            match nm2.list_devices().await {
                Ok(devs) => {
                    glib::idle_add_local_once(move || {
                        while let Some(child) = device_list_ref.first_child() {
                            device_list_ref.remove(&child);
                        }

                        for d in devs {
                            let row = gtk::ListBoxRow::new();
                            let label = gtk::Label::new(Some(&format!(
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

    // Event handler for quit button
    let window_clone = window.clone();
    quit.connect_clicked(move |_| {
        window_clone.close();
    });

    grid.attach(&show_devices, 0, 0, 1, 1);
    grid.attach(&quit, 0, 2, 3, 1);

    sidebar.append(&grid);

    sidebar
}
