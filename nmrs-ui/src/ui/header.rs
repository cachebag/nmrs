use glib::clone;
use gtk::prelude::*;
use gtk::{Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::ui::networks;
use crate::ui::networks::NetworksContext;

pub fn build_header(
    ctx: Rc<NetworksContext>,
    list_container: &GtkBox,
    is_scanning: Rc<Cell<bool>>,
) -> HeaderBar {
    let header = HeaderBar::new();
    header.set_show_title_buttons(false);

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
        #[strong]
        ctx,
        #[strong]
        is_scanning,
        move |_| {
            let ctx = ctx.clone();
            let list_container = list_container.clone();
            let is_scanning = is_scanning.clone();

            glib::MainContext::default().spawn_local(async move {
                refresh_networks(ctx, &list_container, &is_scanning).await;
            });
        }
    ));

    let wifi_switch = Switch::new();
    wifi_switch.set_valign(gtk::Align::Center);
    header.pack_end(&wifi_switch);
    wifi_switch.set_size_request(24, 24);

    header.pack_end(&ctx.status);

    {
        let list_container = list_container.clone();
        let wifi_switch = wifi_switch.clone();
        let ctx = ctx.clone();
        let is_scanning = is_scanning.clone();

        glib::MainContext::default().spawn_local(async move {
            ctx.stack.set_visible_child_name("loading");
            clear_children(&list_container);

            match ctx.nm.wifi_enabled().await {
                Ok(enabled) => {
                    wifi_switch.set_active(enabled);
                    if enabled {
                        refresh_networks(ctx, &list_container, &is_scanning).await;
                    }
                }
                Err(err) => {
                    ctx.status
                        .set_text(&format!("Error fetching networks: {err}"));
                }
            }
        })
    };

    {
        let ctx = ctx.clone();

        wifi_switch.connect_active_notify(move |sw| {
            let ctx = ctx.clone();
            let list_container = list_container.clone();
            let sw = sw.clone();
            let is_scanning = is_scanning.clone();

            glib::MainContext::default().spawn_local(async move {
                clear_children(&list_container);

                if let Err(err) = ctx.nm.set_wifi_enabled(sw.is_active()).await {
                    ctx.status.set_text(&format!("Error setting Wi-Fi: {err}"));
                    return;
                }

                if sw.is_active() {
                    if ctx.nm.wait_for_wifi_ready().await.is_ok() {
                        refresh_networks(ctx, &list_container, &is_scanning).await;
                    } else {
                        ctx.status.set_text("Wi-Fi failed to initialize");
                    }
                }
            });
        });
    }

    header
}

pub async fn refresh_networks(
    ctx: Rc<NetworksContext>,
    list_container: &GtkBox,
    is_scanning: &Rc<Cell<bool>>,
) {
    if is_scanning.get() {
        ctx.status.set_text("Scan already in progress");
        return;
    }
    is_scanning.set(true);

    clear_children(list_container);
    ctx.status.set_text("Scanning...");

    if let Err(err) = ctx.nm.scan_networks().await {
        ctx.status.set_text(&format!("Scan failed: {err}"));
        is_scanning.set(false);
        return;
    }

    let mut last_len = 0;
    for _ in 0..5 {
        let nets = ctx.nm.list_networks().await.unwrap_or_default();
        if nets.len() == last_len && last_len > 0 {
            break;
        }
        last_len = nets.len();
        glib::timeout_future_seconds(1).await;
    }

    match ctx.nm.list_networks().await {
        Ok(mut nets) => {
            let current_conn = ctx.nm.current_connection_info().await;
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

            ctx.status.set_text("");

            let list: ListBox = networks::networks_view(
                ctx.clone(),
                &nets,
                current_ssid.as_deref(),
                current_band.as_deref(),
            );
            list_container.append(&list);
            ctx.stack.set_visible_child_name("networks");
        }
        Err(err) => ctx
            .status
            .set_text(&format!("Error fetching networks: {err}")),
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
