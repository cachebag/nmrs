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
    current_ssid: Option<&str>,
    current_band: Option<&str>,
) -> ListBox {
    let conn_threshold = 75;
    let list = ListBox::new();

    // Helper function to check if a network is connected (matches both SSID and band)
    let is_connected = |net: &models::Network| -> bool {
        if let Some(ssid) = current_ssid {
            if ssid != net.ssid {
                return false;
            }
            // If we have band info, check it matches
            if let Some(band) = current_band {
                let net_band = net.frequency.map(|freq| {
                    if (2400..=2500).contains(&freq) {
                        "2.4GHz"
                    } else if (5000..=6000).contains(&freq) {
                        "5GHz"
                    } else if (5925..=7125).contains(&freq) {
                        "6GHz"
                    } else {
                        "unknown"
                    }
                });
                return net_band == Some(band);
            }
            // If no band info, just match SSID (fallback)
            true
        } else {
            false
        }
    };

    // Sort networks: connected network first, then by signal strength (descending)
    let mut sorted_networks = networks.to_vec();
    sorted_networks.sort_by(|a, b| {
        let a_connected = is_connected(a);
        let b_connected = is_connected(b);

        match (a_connected, b_connected) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.strength.unwrap_or(0).cmp(&a.strength.unwrap_or(0)),
        }
    });

    for net in sorted_networks {
        let row = ListBoxRow::new();
        let hbox = Box::new(Orientation::Horizontal, 6);
        let gesture = GestureClick::new();

        row.add_css_class("network-selection");

        let connected = is_connected(&net);
        if connected {
            row.add_css_class("connected");
        }

        // Add band suffix for display only
        let display_name = if let Some(freq) = net.frequency {
            let band = if (2400..=2500).contains(&freq) {
                " (2.4GHz)"
            } else if (5000..=6000).contains(&freq) {
                " (5GHz)"
            } else if (5925..=7125).contains(&freq) {
                " (6GHz)"
            } else {
                ""
            };
            format!("{}{}", net.ssid, band)
        } else {
            net.ssid.clone()
        };

        let ssid = Label::new(Some(&display_name));
        hbox.append(&ssid);

        if connected {
            let connected_label = Label::new(Some("Connected"));
            connected_label.add_css_class("connected-label");
            hbox.append(&connected_label);
        }

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
                        let container = network_page(details, &stack);

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
                if n_press == 2 {
                    let ssid2 = ssid_str.clone();
                    let window = parent_window.clone();

                    glib::MainContext::default().spawn_local(async move {
                        match NetworkManager::new().await {
                            Ok(nm) => {
                                if secured {
                                    let have =
                                        nm.has_saved_connection(&ssid2).await.unwrap_or(false);

                                    if have {
                                        let creds = WifiSecurity::WpaPsk {
                                            psk: "".into(), // TODO: NM will use saved secrets
                                        };
                                        let _ = nm.connect(&ssid2, creds).await;
                                    } else {
                                        connect::connect_modal(&window, &ssid2, is_eap);
                                    }
                                } else {
                                    eprintln!("Connecting to open network: {ssid2}");
                                    let creds = WifiSecurity::Open;
                                    match nm.connect(&ssid2, creds).await {
                                        Ok(_) => eprintln!("Successfully connected to {ssid2}"),
                                        Err(e) => {
                                            eprintln!("Failed to connect to {ssid2}: {e}")
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("nm init fail: {e}"),
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
