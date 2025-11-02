use glib::clone;
use gtk::Align;
use gtk::GestureClick;
use gtk::prelude::*;
use gtk::{Box, Image, Label, ListBox, ListBoxRow, Orientation};
use nmrs_core::models::WifiSecurity;
use nmrs_core::{NetworkManager, models};

use crate::ui::connect;
use crate::ui::network_page::network_page;

pub fn networks_view(
    networks: &[models::Network],
    parent_window: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
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

        let arrow = Image::from_icon_name("go-next-symbolic");
        arrow.set_halign(Align::End);
        arrow.add_css_class("network-arrow");
        arrow.set_cursor_from_name(Some("pointer"));

        let arrow_click = GestureClick::new();
        let net_clone = net.clone();

        arrow_click.connect_pressed(clone!(
            #[weak]
            stack,
            move |_, _, _, _| {
                let net_data = net_clone.clone();
                glib::MainContext::default().spawn_local(async move {
                    if let Ok(nm) = NetworkManager::new().await
                        && let Ok(details) = nm.show_details(&net_data).await
                    {
                        let container = network_page(&details, &stack);

                        if let Some(old) = stack.child_by_name("details") {
                            stack.remove(&old);
                        }
                        stack.add_named(&container, Some("details"));
                        stack.set_visible_child_name("details");
                    }
                });
            }
        ));

        arrow.add_controller(arrow_click);

        // Double-click row to connect / open modal for secured networks
        let ssid_str = net.ssid.clone();
        let secured = net.secured;
        let is_eap = net.is_eap;

        gesture.connect_pressed(clone!(
            #[weak]
            parent_window,
            move |_, n_press, _x, _y| {
                if n_press == 2 && secured {
                    connect::connect_modal(&parent_window, &ssid_str, is_eap);
                } else if n_press == 2 {
                    glib::MainContext::default().spawn_local({
                        let ssid = ssid_str.clone();
                        async move {
                            match NetworkManager::new().await {
                                Ok(nm) => {
                                    let creds = WifiSecurity::Open;
                                    if let Err(err) = nm.connect(&ssid, creds).await {
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
        hbox.append(&arrow);
        row.set_child(Some(&hbox));
        list.append(&row);
    }

    list
}
