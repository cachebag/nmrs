use glib::clone;
use gtk::GestureClick;
use gtk::prelude::*;
use gtk::{Box, Image, Label, ListBox, ListBoxRow, Orientation};
use netrs_core::{models, NetworkManager};

use crate::ui::connect;

pub fn networks_view(
    networks: &[models::Network],
    parent_window: &gtk::ApplicationWindow,
) -> ListBox {
    let conn_threshold = 75;
    let list = ListBox::new();

    for net in networks {
        let row = ListBoxRow::new();
        let hbox = Box::new(Orientation::Horizontal, 6);
        let gesture = GestureClick::new();

        row.add_css_class("network-selection");
        let ssid = Label::new(Some(&net.ssid));

        hbox.append(&ssid);

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        hbox.append(&spacer);

        if let Some(s) = net.strength {
            let icon_name = if net.secured {
                "network-wireless-encrypted-symbolic"
            } else {
                "network-wireless-signal-excellent-symbolic"
            };

            let image = Image::from_icon_name(icon_name);
            if net.secured {
                image.add_css_class("wifi-secure");
            } else {
                image.add_css_class("wifi-open");
            }
            let strength_label = Label::new(Some(&format!("{s}%")));

            hbox.append(&image);
            hbox.append(&strength_label);

            if s >= conn_threshold {
                strength_label.add_css_class("network-good");
            } else if s > 65 {
                strength_label.add_css_class("network-okay");
            } else {
                strength_label.add_css_class("network-poor");
            }
        }

        // debouncing is not needed here unless we add logic for single clicks
        let ssid_str = net.ssid.clone();
        let secured = net.secured;
        gesture.connect_pressed(clone!(
            #[weak]
            parent_window,
            move |_, n_press, _x, _y| {
                if n_press == 2 && secured {
                    println!("Double click");
                    connect::connect_modal(&parent_window, &ssid_str);
                } else if n_press == 2 {
                    eprintln!("Connecting to {ssid_str}");
                    glib::MainContext::default().spawn_local({
                        let ssid = ssid_str.clone();
                        async move {
                            match NetworkManager::new().await {
                                Ok(nm) => {
                                    if let Err(err) = nm.connect(&ssid, "").await {
                                        eprintln!("Failed to connect network: {err}");
                                    }
                                }
                                Err(err) => eprintln!("Failed to init NetworkManager: {err}"),
                            }
                        }
                    });
                }
            }
        ));

        row.add_controller(gesture);

        row.set_child(Some(&hbox));
        list.append(&row);
    }

    list
}
