pub mod header;
pub mod networks;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ScrolledWindow};
use netrs_core::NetworkManager;

pub fn build_ui(app: &Application) {
    let win = ApplicationWindow::new(app);
    win.set_title(Some("netrs"));
    win.set_default_size(400, 600);

    let vbox = GtkBox::new(Orientation::Vertical, 0);

    let status = Label::new(None);

    let list_container = GtkBox::new(Orientation::Vertical, 0);

    let header = header::build_header(&status, &list_container);
    vbox.append(&header);

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_child(Some(&list_container));
    vbox.append(&scroller);

    win.set_child(Some(&vbox));
    win.show();

    let list_container_clone = list_container.clone();
    let status_clone = status.clone();
    glib::MainContext::default().spawn_local(async move {
        match NetworkManager::new().await {
            Ok(nm) => match nm.wifi_enabled().await {
                Ok(true) => {
                    status_clone.set_text("");
                    match nm.list_networks().await {
                        Ok(nets) => {
                            let list = crate::ui::networks::networks_view(&nets);
                            list_container_clone.append(&list);
                        }
                        Err(err) => {
                            status_clone.set_text(&format!("Error fetching networks: {err}"));
                        }
                    }
                }
                Ok(false) => {
                    let disabled_label = Label::new(Some("Wi-Fi is disabled"));
                    disabled_label.set_halign(gtk::Align::Center);
                    disabled_label.set_valign(gtk::Align::Center);
                    list_container_clone.append(&disabled_label);
                }
                Err(err) => status_clone.set_text(&format!("Error: {err}")),
            },
            Err(err) => status_clone.set_text(&format!("Error: {err}")),
        }
    });
}
