use futures_util::StreamExt;
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch};
use netrs_core::NetworkManager;
use netrs_core::dbus::{NMProxy, NMWirelessProxy};
use zbus::Connection;

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

                            // subscribe for live updates
                            spawn_signal_listeners(
                                list_container_clone.clone(),
                                status_clone.clone(),
                            )
                            .await;
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

                            if nm.wait_for_wifi_ready().await.is_ok() {
                                match nm.list_networks().await {
                                    Ok(nets) => {
                                        status_clone.set_text("");
                                        let list: ListBox = networks::networks_view(&nets);
                                        list_container.append(&list);

                                        spawn_signal_listeners(
                                            list_container.clone(),
                                            status_clone.clone(),
                                        )
                                        .await;
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

async fn spawn_signal_listeners(list_container: GtkBox, status: Label) {
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(e) => {
            status.set_text(&format!("DBus conn error: {e}"));
            return;
        }
    };

    let nm_proxy = match NMProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => {
            status.set_text(&format!("NM proxy error: {e}"));
            return;
        }
    };

    let devices = match nm_proxy.get_devices().await {
        Ok(d) => d,
        Err(e) => {
            status.set_text(&format!("Get devices error: {e}"));
            return;
        }
    };

    for dev_path in devices {
        let builder = match NMWirelessProxy::builder(&conn).path(dev_path.clone()) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let wifi = match builder.build().await {
            Ok(proxy) => proxy,
            Err(_) => continue,
        };

        let mut added = match wifi.receive_access_point_added().await {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut removed = match wifi.receive_access_point_removed().await {
            Ok(s) => s,
            Err(_) => continue,
        };

        let list_container_signal = list_container.clone();
        let status_signal = status.clone();

        glib::MainContext::default().spawn_local(async move {
            loop {
                tokio::select! {
                    _ = added.next() => {
                        refresh_networks(&list_container_signal, &status_signal).await;
                    }
                    _ = removed.next() => {
                        refresh_networks(&list_container_signal, &status_signal).await;
                    }
                }
            }
        });
    }
}

async fn refresh_networks(list_container: &GtkBox, status: &Label) {
    match NetworkManager::new().await {
        Ok(nm) => match nm.list_networks().await {
            Ok(nets) => {
                clear_children(list_container);
                let list: ListBox = networks::networks_view(&nets);
                list_container.append(&list);
            }
            Err(e) => status.set_text(&format!("Error refreshing networks: {e}")),
        },
        Err(e) => status.set_text(&format!("Error refreshing networks: {e}")),
    }
}
