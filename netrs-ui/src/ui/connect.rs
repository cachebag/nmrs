use glib::Propagation;
use gtk::{
    ApplicationWindow, Box as GtkBox, Dialog, Entry, EventControllerKey, Label, Orientation,
    prelude::*,
};
use netrs_core::NetworkManager;
use std::rc::Rc;

pub fn connect_modal(parent: &ApplicationWindow, ssid: &str) {
    let dialog = Dialog::new();
    dialog.set_title(Some("Connect to Network"));
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.add_css_class("diag-buttons");

    let content_area = dialog.content_area();
    let vbox = GtkBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(32);
    vbox.set_margin_bottom(32);
    vbox.set_margin_start(48);
    vbox.set_margin_end(48);

    let label = Label::new(Some("Enter network password:"));
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Password"));
    entry.set_visibility(false);

    vbox.append(&label);
    vbox.append(&entry);
    content_area.append(&vbox);
    
    let dialog_rc = Rc::new(dialog);
    let ssid_owned = ssid.to_string();
    {
        let dialog_rc = dialog_rc.clone();
        entry.connect_activate(move |entry| {
            let pwd = entry.text();
            println!("User entered: {pwd}");

            let ssid = ssid_owned.clone();
            glib::MainContext::default().spawn_local(async move {
                match NetworkManager::new().await {
                    Ok(nm) => {
                        if let Err(err) = nm.connect(&ssid, &pwd).await {
                            eprintln!("Failed to connect: {err}");
                        }
                    }
                    Err(err) => eprintln!("Failed to init NetworkManager: {err}"),
                }
            });

            dialog_rc.close();
        });
    }

    {
        let dialog_rc = dialog_rc.clone();
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                dialog_rc.close();
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        entry.add_controller(key_controller);
    }

    dialog_rc.show();
}
