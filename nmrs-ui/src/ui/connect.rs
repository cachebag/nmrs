use glib::Propagation;
use gtk::{
    ApplicationWindow, Box as GtkBox, Dialog, Entry, EventControllerKey, Label, Orientation,
    prelude::*,
};
use nmrs_core::{
    NetworkManager,
    models::{EapMethod, EapOptions, Phase2, WifiSecurity},
};
use std::rc::Rc;

pub fn connect_modal(parent: &ApplicationWindow, ssid: &str, is_eap: bool) {
    let ssid_owned = ssid.to_string();
    let parent_weak = parent.downgrade();

    glib::MainContext::default().spawn_local(async move {
        if let Ok(nm) = NetworkManager::new().await
            && let Some(current) = nm.current_ssid().await
            && current == ssid_owned
        {
            println!("Already connected to {current}, skipping modal");
            return;
        }

        if let Some(parent) = parent_weak.upgrade() {
            draw_connect_modal(&parent, &ssid_owned, is_eap);
        }
    });
}

fn draw_connect_modal(parent: &ApplicationWindow, ssid: &str, is_eap: bool) {
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

    let user_entry = if is_eap {
        let user_label = Label::new(Some("Username:"));
        let user_entry = Entry::new();
        user_entry.set_placeholder_text(Some("student@university.edu"));
        vbox.append(&user_label);
        vbox.append(&user_entry);
        Some(user_entry)
    } else {
        None
    };

    let label = Label::new(Some("Password:"));
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Password"));
    entry.set_visibility(false);
    vbox.append(&label);
    vbox.append(&entry);

    content_area.append(&vbox);

    let dialog_rc = Rc::new(dialog);
    let ssid_owned = ssid.to_string();
    let user_entry_clone = user_entry.clone();

    {
        let dialog_rc = dialog_rc.clone();
        entry.connect_activate(move |entry| {
            let pwd = entry.text().to_string();
            let username = user_entry_clone
                .as_ref()
                .map(|e| e.text().to_string())
                .unwrap_or_default();
            let ssid = ssid_owned.clone();

            println!("User entered username={username}, password={pwd}");

            glib::MainContext::default().spawn_local(async move {
                match NetworkManager::new().await {
                    Ok(nm) => {
                        let creds = if is_eap {
                            WifiSecurity::WpaEap {
                                opts: EapOptions {
                                    identity: username,
                                    password: pwd,
                                    anonymous_identity: None,
                                    domain_suffix_match: None,
                                    ca_cert_path: None,
                                    system_ca_certs: true,
                                    method: EapMethod::Peap,
                                    phase2: Phase2::Mschapv2,
                                },
                            }
                        } else {
                            WifiSecurity::WpaPsk { psk: pwd }
                        };

                        if let Err(err) = nm.connect(&ssid, creds).await {
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
