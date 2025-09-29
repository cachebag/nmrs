use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch};
use netrs_core::NetworkManager;

use crate::ui::networks;

pub fn build_header(status: &Label, list_container: &GtkBox) -> HeaderBar {
    let header = HeaderBar::new();

    let wifi_box = GtkBox::new(Orientation::Horizontal, 6);
    let wifi_label = Label::new(Some("Wi-Fi"));
    let wifi_switch = Switch::new();

    wifi_box.append(&wifi_label);
    wifi_box.append(&wifi_switch);

    {
        let list_container_clone = list_container.clone();
        let status_clone = status.clone();
        let wifi_switch_clone = wifi_switch.clone();

        glib::MainContext::default().spawn_local(async move {
            clear_children(&list_container_clone);

            match NetworkManager::new().await {
                Ok(nm) => match nm.wifi_enabled().await {
                    Ok(enabled) => {
                        wifi_switch_clone.set_active(enabled);

                        if enabled {
                            status_clone.set_text("");
                            match nm.list_networks().await {
                                Ok(nets) => {
                                    let list: ListBox = networks::networks_view(&nets);
                                    list_container_clone.append(&list);
                                }
                                Err(err) => {
                                    status_clone
                                        .set_text(&format!("Error fetching networks: {err}"));
                                }
                            }
                        } else {
                            let disabled_label = Label::new(Some("Wi-Fi is disabled"));
                            disabled_label.set_halign(Align::Center);
                            disabled_label.set_valign(Align::Center);
                            list_container_clone.append(&disabled_label);
                        }
                    }
                    Err(err) => status_clone.set_text(&format!("Error: {err}")),
                },
                Err(err) => status_clone.set_text(&format!("Error: {err}")),
            }
        });
    }

    {
        let list_container = list_container.clone();
        let status_clone = status.clone();

        wifi_switch.connect_active_notify(move |sw| {
            let list_container = list_container.clone();
            let status_clone = status_clone.clone();
            let sw = sw.clone();

            glib::MainContext::default().spawn_local(async move {
                clear_children(&list_container);

                match NetworkManager::new().await {
                    Ok(nm) => {
                        if let Err(err) = nm.set_wifi_enabled(sw.is_active()).await {
                            status_clone.set_text(&format!("Error setting Wi-Fi: {err}"));
                            return;
                        }

                        if sw.is_active() {
                            status_clone.set_text("Enabling...");

                            if nm.wait_for_wifi_ready(&nm).await.is_ok() {
                                if let Err(err) = nm.scan_networks().await {
                                    status_clone.set_text(&format!("Error scanning: {err}"));
                                }

                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                                match nm.list_networks().await {
                                    Ok(nets) => {
                                        status_clone.set_text("");
                                        let list: ListBox = networks::networks_view(&nets);
                                        list_container.append(&list);
                                    }
                                    Err(err) => {
                                        status_clone
                                            .set_text(&format!("Error fetching networks: {err}"));
                                    }
                                }
                            } else {
                                status_clone.set_text("Wi-Fi failed to initialize");
                            }
                        } else {
                            status_clone.set_text("Wi-Fi is disabled");
                            let disabled_label = Label::new(Some("Wi-Fi is disabled"));
                            disabled_label.set_halign(Align::Center);
                            disabled_label.set_valign(Align::Center);
                            list_container.append(&disabled_label);
                        }
                    }
                    Err(err) => status_clone.set_text(&format!("Error: {err}")),
                }
            });
        });
    }

    header.pack_start(&wifi_box);
    header
}

fn clear_children(container: &gtk::Box) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        container.remove(&widget);
    }
}
