use anyhow::Result;
use gtk::Align;
use gtk::GestureClick;
use gtk::prelude::*;
use gtk::{Box, Image, Label, ListBox, ListBoxRow, Orientation};
use nmrs_core::models::WifiSecurity;
use nmrs_core::{NetworkManager, models};
use std::rc::Rc;

use crate::ui::connect;
use crate::ui::network_page::network_page;

pub struct NetworksContext {
    pub nm: Rc<NetworkManager>,
    pub on_success: Rc<dyn Fn()>,
    pub status: Label,
    pub stack: gtk::Stack,
    pub parent_window: gtk::ApplicationWindow,
}

impl NetworksContext {
    pub async fn new(
        on_success: Rc<dyn Fn()>,
        status: &Label,
        stack: &gtk::Stack,
        parent_window: &gtk::ApplicationWindow,
    ) -> Result<Self> {
        let nm = Rc::new(NetworkManager::new().await?);

        Ok(Self {
            nm,
            on_success,
            status: status.clone(),
            stack: stack.clone(),
            parent_window: parent_window.clone(),
        })
    }
}

pub fn networks_view(
    ctx: Rc<NetworksContext>,
    networks: &[models::Network],
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
                let net_band = freq_to_band(net.frequency.unwrap_or_default());
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

        row.add_css_class("network-selection");

        if is_connected(&net) {
            row.add_css_class("connected");
        }

        // Add band suffix for display only
        let display_name = if let Some(freq) = net.frequency {
            if let Some(band) = freq_to_band(freq) {
                format!("{} ({})", net.ssid, band)
            } else {
                net.ssid.clone()
            }
        } else {
            net.ssid.clone()
        };

        hbox.append(&Label::new(Some(&display_name)));

        if is_connected(&net) {
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

        let ctx_details = ctx.clone();
        let stack_for_details = ctx.stack.clone();

        let arrow_click = GestureClick::new();
        let net_clone = net.clone();

        arrow_click.connect_pressed(move |_, _, _, _| {
            let ctx = ctx_details.clone();
            let stack = stack_for_details.clone();
            let net = net_clone.clone();

            glib::MainContext::default().spawn_local(async move {
                let nm = ctx.nm.clone();

                if let Ok(details) = nm.show_details(&net).await {
                    let container = network_page(details, &stack);

                    if let Some(old) = stack.child_by_name("details") {
                        stack.remove(&old);
                    }
                    stack.add_named(&container, Some("details"));
                    stack.set_visible_child_name("details");
                }
            });
        });
        arrow.add_controller(arrow_click);

        // Double-click row to connect / open modal for secured networks
        let ctx_click = ctx.clone();
        let list_clone = list.clone();
        let ssid_str = net.ssid.clone();
        let secured = net.secured;
        let is_eap = net.is_eap;

        let gesture = GestureClick::new();
        gesture.connect_pressed(move |_, n_press, _, _| {
            if n_press == 2 {
                let ssid = ssid_str.clone();
                let nm = ctx_click.nm.clone();
                let status = ctx_click.status.clone();
                let window = ctx_click.parent_window.clone();
                let list = list_clone.clone();
                let on_success = ctx_click.on_success.clone();

                status.set_text(&format!("Connecting to {ssid}..."));
                list.set_sensitive(false);

                glib::MainContext::default().spawn_local(async move {
                    if secured {
                        let have = nm.has_saved_connection(&ssid).await.unwrap_or(false);

                        if have {
                            let creds = WifiSecurity::WpaPsk { psk: "".into() };
                            match nm.connect(&ssid, creds).await {
                                Ok(_) => on_success(),
                                Err(e) => status.set_text(&format!("Failed to connect: {e}")),
                            }
                            status.set_text("");
                            list.set_sensitive(true);
                        } else {
                            list.set_sensitive(true);
                            status.set_text("");
                            connect::connect_modal(
                                nm.clone(),
                                &window,
                                &ssid,
                                is_eap,
                                on_success.clone(),
                            );
                        }
                    } else {
                        eprintln!("Connecting to open network: {ssid}");
                        let creds = WifiSecurity::Open;
                        match nm.connect(&ssid, creds).await {
                            Ok(_) => on_success(),
                            Err(e) => status.set_text(&format!("Failed to connect: {e}")),
                        }
                        status.set_text("");
                        list.set_sensitive(true);
                    }
                });
            }
        });
        row.add_controller(gesture);

        hbox.append(&arrow);
        row.set_child(Some(&hbox));
        list.append(&row);
    }
    list
}

fn freq_to_band(freq: u32) -> Option<&'static str> {
    match freq {
        2400..=2500 => Some("2.4GHz"),
        5000..=5900 => Some("5GHz"),
        5901..=7125 => Some("6GHz"),
        _ => None,
    }
}
