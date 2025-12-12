use glib::Propagation;
use gtk::{
    ApplicationWindow, Box as GtkBox, Dialog, Entry, EventControllerKey, Label, Orientation,
    prelude::*,
};
use log::{debug, error};
use nmrs::{
    NetworkManager,
    models::{EapMethod, EapOptions, Phase2, WifiSecurity},
};
use std::rc::Rc;

pub fn connect_modal(
    nm: Rc<NetworkManager>,
    parent: &ApplicationWindow,
    ssid: &str,
    is_eap: bool,
    on_connection_success: Rc<dyn Fn()>,
) {
    let ssid_owned = ssid.to_string();
    let parent_weak = parent.downgrade();

    glib::MainContext::default().spawn_local(async move {
        if let Some(current) = nm.current_ssid().await
            && current == ssid_owned
        {
            debug!("Already connected to {current}, skipping modal");
            return;
        }

        if let Some(parent) = parent_weak.upgrade() {
            draw_connect_modal(nm, &parent, &ssid_owned, is_eap, on_connection_success);
        }
    });
}

fn draw_connect_modal(
    nm: Rc<NetworkManager>,
    parent: &ApplicationWindow,
    ssid: &str,
    is_eap: bool,
    on_connection_success: Rc<dyn Fn()>,
) {
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
        user_entry.add_css_class("pw-entry");
        user_entry.set_placeholder_text(Some("email, username, id..."));
        vbox.append(&user_label);
        vbox.append(&user_entry);
        Some(user_entry)
    } else {
        None
    };

    let label = Label::new(Some("Password:"));
    let entry = Entry::new();
    entry.add_css_class("pw-entry");
    entry.set_placeholder_text(Some("Password"));
    entry.set_visibility(false);
    vbox.append(&label);
    vbox.append(&entry);

    content_area.append(&vbox);

    let dialog_rc = Rc::new(dialog);
    let ssid_owned = ssid.to_string();
    let user_entry_clone = user_entry.clone();

    let status_label = Label::new(Some(""));
    status_label.add_css_class("status-label");
    vbox.append(&status_label);

    {
        let dialog_rc = dialog_rc.clone();
        let status_label = status_label.clone();
        let refresh_callback = on_connection_success.clone();
        let nm = nm.clone();

        entry.connect_activate(move |entry| {
            let pwd = entry.text().to_string();

            let username = user_entry_clone
                .as_ref()
                .map(|e| e.text().to_string())
                .unwrap_or_default();
            let ssid = ssid_owned.clone();
            let dialog = dialog_rc.clone();
            let status = status_label.clone();
            let entry = entry.clone();
            let user_entry = user_entry_clone.clone();
            let on_success = refresh_callback.clone();
            let nm = nm.clone();

            entry.set_sensitive(false);
            if let Some(ref user_entry) = user_entry {
                user_entry.set_sensitive(false);
            }
            status.set_text("Connecting...");

            glib::MainContext::default().spawn_local(async move {
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

                debug!("Calling nm.connect() for '{ssid}'");
                match nm.connect(&ssid, creds).await {
                    Ok(_) => {
                        debug!("nm.connect() succeeded!");
                        status.set_text("✓ Connected!");
                        on_success();
                        glib::timeout_future_seconds(1).await;
                        dialog.close();
                    }
                    Err(err) => {
                        error!("nm.connect() failed: {err}");
                        let err_str = err.to_string().to_lowercase();
                        if err_str.contains("authentication")
                            || err_str.contains("supplicant")
                            || err_str.contains("password")
                            || err_str.contains("psk")
                            || err_str.contains("wrong")
                        {
                            status.set_text("Wrong password, try again");
                            entry.set_text("");
                            entry.grab_focus();
                        } else {
                            status.set_text(&format!("✗ Failed: {err}"));
                        }
                        entry.set_sensitive(true);
                        if let Some(ref user_entry) = user_entry {
                            user_entry.set_sensitive(true);
                        }
                    }
                }
            });
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
