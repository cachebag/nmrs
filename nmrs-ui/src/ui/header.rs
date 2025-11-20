use glib::clone;
use gtk::prelude::*;
use gtk::{Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch};
use nmrs_core::NetworkManager;
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::ui::networks;

pub fn build_header(
    status: &Label,
    list_container: &GtkBox,
    parent_window: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
    is_scanning: Rc<Cell<bool>>,
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

    let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.add_css_class("refresh-btn");
    refresh_btn.set_valign(gtk::Align::Start);
    header.pack_end(&refresh_btn);
    refresh_btn.connect_clicked(clone!(
        #[weak]
        list_container,
        #[weak]
        status,
        #[weak]
        parent_window,
        #[weak]
        stack,
        #[strong]
        is_scanning,
        move |_| {
            glib::MainContext::default().spawn_local(clone!(
                #[strong]
                list_container,
                #[strong]
                status,
                #[strong]
                parent_window,
                #[strong]
                stack,
                #[strong]
                is_scanning,
                async move {
                    match NetworkManager::new().await {
                        Ok(nm) => {
                            refresh_networks(
                                &nm,
                                &list_container,
                                &status,
                                &parent_window,
                                &stack,
                                &is_scanning,
                            )
                            .await;
                        }
                        Err(err) => status.set_text(&format!("Error: {err}")),
                    }
                }
            ));
        }
    ));

    let wifi_switch = Switch::new();
    wifi_switch.set_valign(gtk::Align::Center);
    header.pack_end(&wifi_switch);
    wifi_switch.set_size_request(24, 24);

    header.pack_end(&status);

    // Initialize Wi-Fi state
    // This runs once on startup
    {
        let list_container_clone = list_container.clone();
        let status_clone = status.clone();
        let wifi_switch_clone = wifi_switch.clone();
        let pw = parent_window.clone();
        let stack_clone = stack.clone();
        let is_scanning_clone = is_scanning.clone();

        glib::MainContext::default().spawn_local(async move {
            stack_clone.set_visible_child_name("loading");
            clear_children(&list_container_clone);

            match NetworkManager::new().await {
                Ok(nm) => match nm.wifi_enabled().await {
                    Ok(enabled) => {
                        wifi_switch_clone.set_active(enabled);
                        if enabled {
                            refresh_networks(
                                &nm,
                                &list_container_clone,
                                &status_clone,
                                &pw,
                                &stack_clone,
                                &is_scanning_clone,
                            )
                            .await;
                        }
                    }
                    Err(err) => {
                        status_clone.set_text(&format!("Error fetching networks: {err}"));
                    }
                },
                Err(err) => status_clone.set_text(&format!("Error: {err}")),
            }
        })
    };

    // Handle Wi-Fi toggle changes
    // This runs whenever the user toggles the switch
    {
        let pw2 = parent_window.clone();
        let stack_clone = stack.clone();

        wifi_switch.connect_active_notify(move |sw| {
            let pw = pw2.clone();
            let list_container_clone = list_container.clone();
            let status_clone = status.clone();
            let sw = sw.clone();
            let stack_inner = stack_clone.clone();
            let is_scanning_clone = is_scanning.clone();

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
                                refresh_networks(
                                    &nm,
                                    &list_container_clone,
                                    &status_clone,
                                    &pw,
                                    &stack_inner,
                                    &is_scanning_clone,
                                )
                                .await;
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

async fn refresh_networks(
    nm: &NetworkManager,
    list_container: &GtkBox,
    status: &Label,
    pw: &gtk::ApplicationWindow,
    stack: &gtk::Stack,
    is_scanning: &Rc<Cell<bool>>,
) {
    if is_scanning.get() {
        status.set_text("Scan already in progress");
        return;
    }
    is_scanning.set(true);

    clear_children(list_container);
    status.set_text("Scanning...");

    if let Err(err) = nm.scan_networks().await {
        status.set_text(&format!("Scan failed: {err}"));
        is_scanning.set(false);
        return;
    }

    let mut last_len = 0;
    for _ in 0..5 {
        let nets = nm.list_networks().await.unwrap_or_default();
        if nets.len() == last_len && last_len > 0 {
            break;
        }
        last_len = nets.len();
        glib::timeout_future_seconds(1).await;
    }

    match nm.list_networks().await {
        Ok(mut nets) => {
            let current_conn = nm.current_connection_info().await;
            let (current_ssid, current_band) = if let Some((ssid, freq)) = current_conn {
                let ssid_str = ssid.clone();
                let band: Option<String> = freq.map(|f| {
                    if (2400..=2500).contains(&f) {
                        "2.4GHz".to_string()
                    } else if (5000..=6000).contains(&f) {
                        "5GHz".to_string()
                    } else if (5925..=7125).contains(&f) {
                        "6GHz".to_string()
                    } else {
                        "unknown".to_string()
                    }
                });
                (Some(ssid_str), band)
            } else {
                (None, None)
            };

            // Sort by signal strength (descending)
            nets.sort_by(|a, b| b.strength.unwrap_or(0).cmp(&a.strength.unwrap_or(0)));

            // Deduplicate by SSID + frequency band (not exact frequency)
            // This matches how networks are displayed (2.4GHz, 5GHz, 6GHz)
            let mut seen_combinations = HashSet::new();
            nets.retain(|net| {
                // Normalize frequency to band, matching the display logic
                let band = net.frequency.map(|freq| {
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
                let key = (net.ssid.clone(), band);
                seen_combinations.insert(key)
            });

            status.set_text("");
            let list: ListBox = networks::networks_view(
                &nets,
                pw,
                stack,
                current_ssid.as_deref(),
                current_band.as_deref(),
            );
            list_container.append(&list);
            stack.set_visible_child_name("networks");
        }
        Err(err) => status.set_text(&format!("Error fetching networks: {err}")),
    }

    is_scanning.set(false);
}

fn clear_children(container: &gtk::Box) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        container.remove(&widget);
    }
}
