use glib::Propagation;
use gtk::{
    prelude::*, ApplicationWindow, Box as GtkBox, Button, CheckButton, Dialog, Entry,
    EventControllerKey, FileChooserAction, FileChooserDialog, Label, Orientation, ResponseType,
};
use log::{debug, error};
use nmrs::{
    models::{EapMethod, EapOptions, Phase2, WifiSecurity},
    NetworkManager,
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
        if let Some(current) = nm.current_ssid().await {
            if current == ssid_owned {
                debug!("Already connected to {current}, skipping modal");
                return;
            }
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

    let (cert_entry, use_system_certs, browse_btn) = if is_eap {
        let cert_label = Label::new(Some("CA Certificate (optional):"));
        cert_label.set_margin_top(8);
        let cert_entry = Entry::new();
        cert_entry.add_css_class("pw-entry");
        cert_entry.set_placeholder_text(Some("/path/to/ca-cert.pem"));

        let cert_hbox = GtkBox::new(Orientation::Horizontal, 8);
        let browse_btn = Button::with_label("Browse...");
        browse_btn.add_css_class("cert-browse-btn");
        cert_hbox.append(&cert_entry);
        cert_hbox.append(&browse_btn);

        vbox.append(&cert_label);
        vbox.append(&cert_hbox);

        let system_certs_check = CheckButton::with_label("Use system CA certificates");
        system_certs_check.set_active(true);
        system_certs_check.set_margin_top(4);
        vbox.append(&system_certs_check);

        (Some(cert_entry), Some(system_certs_check), Some(browse_btn))
    } else {
        (None, None, None)
    };

    content_area.append(&vbox);

    let dialog_rc = Rc::new(dialog);
    let ssid_owned = ssid.to_string();
    let user_entry_clone = user_entry.clone();

    let status_label = Label::new(Some(""));
    status_label.add_css_class("status-label");
    vbox.append(&status_label);

    if let Some(browse_btn) = browse_btn {
        let cert_entry_for_browse = cert_entry.clone();
        let dialog_weak = dialog_rc.downgrade();
        browse_btn.connect_clicked(move |_| {
            if let Some(parent_dialog) = dialog_weak.upgrade() {
                let file_dialog = FileChooserDialog::new(
                    Some("Select CA Certificate"),
                    Some(&parent_dialog),
                    FileChooserAction::Open,
                    &[
                        ("Cancel", ResponseType::Cancel),
                        ("Open", ResponseType::Accept),
                    ],
                );

                let cert_entry = cert_entry_for_browse.clone();
                file_dialog.connect_response(move |dialog, response| {
                    if response == ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                cert_entry
                                    .as_ref()
                                    .unwrap()
                                    .set_text(&path.to_string_lossy());
                            }
                        }
                    }
                    dialog.close();
                });

                file_dialog.show();
            }
        });
    }

    {
        let dialog_rc = dialog_rc.clone();
        let status_label = status_label.clone();
        let refresh_callback = on_connection_success.clone();
        let nm = nm.clone();
        let cert_entry_clone = cert_entry.clone();
        let use_system_certs_clone = use_system_certs.clone();

        entry.connect_activate(move |entry| {
            let pwd = entry.text().to_string();

            let username = user_entry_clone
                .as_ref()
                .map(|e| e.text().to_string())
                .unwrap_or_default();

            let cert_path = cert_entry_clone.as_ref().and_then(|e| {
                let text = e.text().to_string();
                if text.trim().is_empty() {
                    None
                } else {
                    Some(text)
                }
            });

            let use_system_ca = use_system_certs_clone
                .as_ref()
                .map(|cb| cb.is_active())
                .unwrap_or(true);

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
                    let mut opts = EapOptions::new(username, pwd)
                        .with_method(EapMethod::Peap)
                        .with_phase2(Phase2::Mschapv2)
                        .with_system_ca_certs(use_system_ca);

                    if let Some(cert) = cert_path {
                        opts = opts.with_ca_cert_path(format!("file://{}", cert));
                    }

                    WifiSecurity::WpaEap { opts }
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
