use futures_util::StreamExt;
use glib::ControlFlow;
use gtk::prelude::*;
use gtk::{Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch};
use nmrs_core::NetworkManager;
use nmrs_core::dbus::{NMProxy, NMWirelessProxy};
use std::cell::Cell;
use zbus::Connection;

thread_local! {
    static REFRESH_SCHEDULED: Cell<bool> = const { Cell::new(false) };
}

use crate::ui::networks;

pub fn build_header(
    status: &Label,
    list_container: &GtkBox,
    parent_window: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
) -> HeaderBar {
    let header = HeaderBar::new();
    header.set_show_title_buttons(false);

    let parent_window = parent_window.clone();
    let status = status.clone();
    let list_container = list_container.clone();

    let wifi_box = GtkBox::new(Orientation::Horizontal, 6);
    let wifi_label = Label::new(Some("Wi-Fi"));
    wifi_label.set_halign(gtk::Align::Start);
    wifi_label.add_css_class("wifi-label");

    wifi_box.append(&wifi_label);
    header.pack_start(&wifi_box);

    let wifi_switch = Switch::new();
    wifi_switch.set_valign(gtk::Align::Center);
    header.pack_end(&wifi_switch);

    header.pack_end(&status);

    {
        let list_container_clone = list_container.clone();
        let status_clone = status.clone();
        let wifi_switch_clone = wifi_switch.clone();
        let pw = parent_window.clone();
        let stack_clone = stack.clone();

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
                                    let list: ListBox =
                                        networks::networks_view(&nets, &pw, &stack_clone);
                                    list_container_clone.append(&list);
                                }
                                Err(err) => {
                                    status_clone
                                        .set_text(&format!("Error fetching networks: {err}"));
                                }
                            }
                        }
                    }
                    Err(err) => status_clone.set_text(&format!("Error: {err}")),
                },
                Err(err) => status_clone.set_text(&format!("Error: {err}")),
            }
        });
    }

    {
        let pw2 = parent_window.clone();
        let stack_clone = stack.clone();

        wifi_switch.connect_active_notify(move |sw| {
            let pw = pw2.clone();
            let list_container_clone = list_container.clone();
            let status_clone = status.clone();
            let sw = sw.clone();
            let stack_inner = stack_clone.clone();

            glib::MainContext::default().spawn_local(async move {
                clear_children(&list_container_clone);

                match NetworkManager::new().await {
                    Ok(nm) => {
                        if let Err(err) = nm.set_wifi_enabled(sw.is_active()).await {
                            status_clone.set_text(&format!("Error setting Wi-Fi: {err}"));
                            return;
                        }

                        if sw.is_active() {
                            if nm.wait_for_wifi_ready().await.is_ok() {
                                let _ = nm.scan_networks().await;
                                status_clone.set_text("Scanning...");

                                spawn_signal_listeners(
                                    list_container_clone.clone(),
                                    status_clone.clone(),
                                    &pw,
                                    &stack_inner,
                                )
                                .await;

                                match nm.list_networks().await {
                                    Ok(nets) => {
                                        status_clone.set_text("");
                                        let list: ListBox =
                                            networks::networks_view(&nets, &pw, &stack_inner);
                                        list_container_clone.append(&list);
                                    }
                                    Err(err) => {
                                        status_clone
                                            .set_text(&format!("Error fetching networks: {err}"));
                                    }
                                }
                            } else {
                                status_clone.set_text("Wi-Fi failed to initialize");
                            }
                        }
                    }
                    Err(err) => status_clone.set_text(&format!("Error: {err}")),
                }
            });
        });
    }

    header
}

fn clear_children(container: &gtk::Box) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        container.remove(&widget);
    }
}

async fn spawn_signal_listeners(
    list_container: GtkBox,
    status: Label,
    parent_window: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
) {
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
        let pw = parent_window.clone();
        let stack_signal = stack.clone();

        glib::MainContext::default().spawn_local(async move {
            loop {
                tokio::select! {
                    event = added.next() => match event {
                        Some(_) => {
                            schedule_refresh(list_container_signal.clone(), status_signal.clone(), &pw, &stack_signal).await;
                        }
                        None => break,
                    },
                    event = removed.next() => match event {
                        Some(_) => {
                            schedule_refresh(list_container_signal.clone(), status_signal.clone(), &pw, &stack_signal).await;
                        }
                        None => break,
                    },
                }
            }
        });
    }
}

async fn refresh_networks(
    list_container: &GtkBox,
    status: &Label,
    parent_window: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
) {
    match NetworkManager::new().await {
        Ok(nm) => match nm.list_networks().await {
            Ok(nets) => {
                clear_children(list_container);
                let list: ListBox = networks::networks_view(&nets, parent_window, stack);
                list_container.append(&list);
            }
            Err(e) => status.set_text(&format!("Error refreshing networks: {e}")),
        },
        Err(e) => status.set_text(&format!("Error refreshing networks: {e}")),
    }
}

async fn schedule_refresh(
    list_container: GtkBox,
    status: Label,
    parent_window: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
) {
    REFRESH_SCHEDULED.with(|flag| {
        if flag.get() {
            return;
        }
        flag.set(true);

        let list_container_clone = list_container.clone();
        let status_clone = status.clone();
        let pw = parent_window.clone();
        let stack_clone = stack.clone();

        glib::timeout_add_seconds_local(1, move || {
            let list_container_inner = list_container_clone.clone();
            let status_inner = status_clone.clone();
            let pw2 = pw.clone();
            let stack_inner = stack_clone.clone();

            glib::MainContext::default().spawn_local(async move {
                refresh_networks(&list_container_inner, &status_inner, &pw2, &stack_inner).await;
                REFRESH_SCHEDULED.with(|f| f.set(false));
            });

            ControlFlow::Break
        });
    });
}
