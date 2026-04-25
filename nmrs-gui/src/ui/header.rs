use glib::clone;
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch, glib};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use nmrs::ConnectivityState;
use nmrs::models;

use crate::ui::networks;
use crate::ui::networks::NetworksContext;
use crate::ui::wired_devices;

pub struct ThemeDef {
    pub key: &'static str,
    pub name: &'static str,
    pub css: &'static str,
}

pub static THEMES: &[ThemeDef] = &[
    ThemeDef {
        key: "gruvbox",
        name: "Gruvbox",
        css: include_str!("../themes/gruvbox.css"),
    },
    ThemeDef {
        key: "nord",
        name: "Nord",
        css: include_str!("../themes/nord.css"),
    },
    ThemeDef {
        key: "dracula",
        name: "Dracula",
        css: include_str!("../themes/dracula.css"),
    },
    ThemeDef {
        key: "catppuccin",
        name: "Catppuccin",
        css: include_str!("../themes/catppuccin.css"),
    },
    ThemeDef {
        key: "tokyo",
        name: "Tokyo Night",
        css: include_str!("../themes/tokyo.css"),
    },
];

pub fn build_header(
    ctx: Rc<NetworksContext>,
    list_container: &GtkBox,
    is_scanning: Rc<Cell<bool>>,
) -> HeaderBar {
    let header = HeaderBar::new();
    header.set_show_title_buttons(false);

    let list_container = list_container.clone();

    // Left side: status label
    ctx.status.set_hexpand(true);
    ctx.status.set_halign(Align::Start);
    header.pack_start(&ctx.status);

    // Right side: settings gear
    let settings_btn = gtk::Button::from_icon_name("emblem-system-symbolic");
    settings_btn.set_has_frame(false);
    settings_btn.set_valign(Align::Center);
    settings_btn.set_tooltip_text(Some("Settings"));
    settings_btn.add_css_class("settings-btn");
    {
        let stack = ctx.stack.clone();
        settings_btn.connect_clicked(move |_| {
            stack.set_visible_child_name("settings");
        });
    }
    header.pack_end(&settings_btn);

    // Right side: radio controls (airplane + wifi switch)
    let airplane_btn = gtk::Button::new();
    airplane_btn.set_valign(Align::Center);
    airplane_btn.set_has_frame(false);
    airplane_btn.set_icon_name("airplane-mode-symbolic");
    airplane_btn.set_tooltip_text(Some("Toggle Airplane Mode"));
    airplane_btn.add_css_class("airplane-btn");
    header.pack_end(&airplane_btn);

    let wifi_switch = Switch::new();
    wifi_switch.set_valign(Align::Center);
    wifi_switch.set_size_request(24, 24);
    header.pack_end(&wifi_switch);

    // Right side: refresh
    let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.add_css_class("refresh-btn");
    refresh_btn.set_has_frame(false);
    refresh_btn.set_tooltip_text(Some("Refresh networks and devices"));
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

    {
        let list_container = list_container.clone();
        let wifi_switch = wifi_switch.clone();
        let airplane_btn = airplane_btn.clone();
        let ctx = ctx.clone();
        let is_scanning = is_scanning.clone();

        glib::MainContext::default().spawn_local(async move {
            ctx.stack.set_visible_child_name("loading");
            clear_children(&list_container);

            apply_airplane_icon(&airplane_btn, &ctx).await;
            apply_connectivity_status(&ctx).await;

            match ctx.nm.wifi_state().await.map(|s| s.enabled) {
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
        airplane_btn.connect_clicked(clone!(
            #[weak]
            list_container,
            #[strong]
            wifi_switch,
            #[strong]
            is_scanning,
            move |btn| {
                let ctx = ctx.clone();
                let list_container = list_container.clone();
                let is_scanning = is_scanning.clone();
                let wifi_switch = wifi_switch.clone();
                let btn = btn.clone();

                glib::MainContext::default().spawn_local(async move {
                    let currently_airplane = ctx
                        .nm
                        .airplane_mode_state()
                        .await
                        .map(|s| s.is_airplane_mode())
                        .unwrap_or(false);

                    let new_state = !currently_airplane;

                    if let Err(err) = ctx.nm.set_airplane_mode(new_state).await {
                        ctx.status.set_text(&format!("Airplane mode error: {err}"));
                        return;
                    }

                    apply_airplane_icon(&btn, &ctx).await;

                    if new_state {
                        wifi_switch.set_active(false);
                        clear_children(&list_container);
                        ctx.status.set_text("Airplane mode on");
                    } else {
                        let wifi_on = ctx
                            .nm
                            .wifi_state()
                            .await
                            .map(|s| s.enabled)
                            .unwrap_or(false);
                        wifi_switch.set_active(wifi_on);
                        if wifi_on && ctx.nm.wait_for_wifi_ready().await.is_ok() {
                            refresh_networks(ctx, &list_container, &is_scanning).await;
                        }
                    }
                });
            }
        ));
    }

    {
        let ctx = ctx.clone();

        wifi_switch.connect_active_notify(move |sw| {
            let ctx = ctx.clone();
            let list_container = list_container.clone();
            let sw = sw.clone();
            let is_scanning = is_scanning.clone();

            glib::MainContext::default().spawn_local(async move {
                clear_children(&list_container);

                if let Err(err) = ctx.nm.set_wireless_enabled(sw.is_active()).await {
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

async fn apply_airplane_icon(btn: &gtk::Button, ctx: &NetworksContext) {
    match ctx.nm.airplane_mode_state().await {
        Ok(state) => {
            if state.is_airplane_mode() {
                btn.set_icon_name("airplane-mode-symbolic");
                btn.set_tooltip_text(Some("Airplane Mode is ON — click to disable"));
                btn.add_css_class("airplane-active");
            } else {
                btn.set_icon_name("network-wireless-symbolic");
                btn.set_tooltip_text(Some("Airplane Mode is OFF — click to enable"));
                btn.remove_css_class("airplane-active");
            }

            if state.any_hardware_killed() {
                btn.set_tooltip_text(Some("A hardware radio kill switch is active"));
            }
        }
        Err(_) => {
            btn.set_icon_name("airplane-mode-symbolic");
            btn.set_sensitive(false);
            btn.set_tooltip_text(Some("Airplane mode unavailable"));
        }
    }
}

async fn apply_connectivity_status(ctx: &NetworksContext) {
    if let Ok(report) = ctx.nm.connectivity_report().await {
        let text = connectivity_label(&report.state, report.captive_portal_url.as_deref());
        if !text.is_empty() {
            ctx.status.set_text(&text);
        }
    }
}

fn connectivity_label(state: &ConnectivityState, portal_url: Option<&str>) -> String {
    match state {
        ConnectivityState::Full => String::new(),
        ConnectivityState::Portal => match portal_url {
            Some(url) => format!("Captive portal: {url}"),
            None => "Captive portal detected".to_string(),
        },
        ConnectivityState::Limited => "Limited connectivity".to_string(),
        ConnectivityState::None => "No internet".to_string(),
        ConnectivityState::Unknown => String::new(),
        _ => String::new(),
    }
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

    // Fetch wired devices first
    match ctx.nm.list_wired_devices().await {
        Ok(wired_devices) => {
            // eprintln!("Found {} wired devices total", wired_devices.len());

            let available_devices: Vec<_> = wired_devices
                .into_iter()
                .filter(|dev| {
                    let show = matches!(
                        dev.state,
                        models::DeviceState::Activated
                            | models::DeviceState::Disconnected
                            | models::DeviceState::Prepare
                            | models::DeviceState::Config
                    );
                    /* eprintln!(
                        "  - {} ({}): {} -> {}",
                        dev.interface,
                        dev.device_type,
                        dev.state,
                        if show { "SHOW" } else { "HIDE" }
                    ); */
                    show
                })
                .collect();

            /* eprintln!(
                "Showing {} available wired devices",
                available_devices.len()
            ); */

            if !available_devices.is_empty() {
                let wired_header = Label::new(Some("Wired"));
                wired_header.add_css_class("section-header");
                wired_header.add_css_class("wired-section-header");
                wired_header.set_halign(Align::Start);
                wired_header.set_margin_top(8);
                wired_header.set_margin_bottom(4);
                wired_header.set_margin_start(12);
                list_container.append(&wired_header);

                let wired_list = wired_devices::wired_devices_view(
                    ctx.clone(),
                    &available_devices,
                    ctx.wired_details_page.clone(),
                );
                wired_list.add_css_class("wired-devices-list");
                list_container.append(&wired_list);

                let separator = gtk::Separator::new(Orientation::Horizontal);
                separator.add_css_class("device-separator");
                separator.set_margin_top(12);
                separator.set_margin_bottom(12);
                list_container.append(&separator);
            }
        }
        Err(e) => {
            eprintln!("Failed to list wired devices: {}", e);
        }
    }

    let wireless_header = Label::new(Some("Wireless"));
    wireless_header.add_css_class("section-header");
    wireless_header.add_css_class("wireless-section-header");
    wireless_header.set_halign(Align::Start);
    wireless_header.set_margin_top(8);
    wireless_header.set_margin_bottom(4);
    wireless_header.set_margin_start(12);
    list_container.append(&wireless_header);

    if let Err(err) = ctx.nm.scan_networks(None).await {
        ctx.status.set_text(&format!("Scan failed: {err}"));
        is_scanning.set(false);
        return;
    }

    let mut last_len = 0;
    for _ in 0..5 {
        let nets = ctx.nm.list_networks(None).await.unwrap_or_default();
        if nets.len() == last_len && last_len > 0 {
            break;
        }
        last_len = nets.len();
        glib::timeout_future_seconds(1).await;
    }

    let saved_ssids = saved_network_ids(&ctx).await;

    match ctx.nm.list_networks(None).await {
        Ok(mut nets) => {
            let current_conn = ctx.nm.current_connection_info().await;
            let (current_ssid, current_band) = if let Some((ssid, freq)) = current_conn {
                let ssid_str = ssid.clone();
                let band: Option<String> = freq
                    .and_then(crate::ui::freq_to_band)
                    .map(|s| s.to_string());
                (Some(ssid_str), band)
            } else {
                (None, None)
            };

            nets.sort_by_key(|b| std::cmp::Reverse(b.strength.unwrap_or(0)));

            let mut seen_combinations = HashSet::new();
            nets.retain(|net| {
                let band = net.frequency.and_then(crate::ui::freq_to_band);
                let key = (net.ssid.clone(), band);
                seen_combinations.insert(key)
            });

            ctx.status.set_text("");

            let list: ListBox = networks::networks_view(
                ctx.clone(),
                &nets,
                current_ssid.as_deref(),
                current_band.as_deref(),
                &saved_ssids,
            );
            list_container.append(&list);
            ctx.stack.set_visible_child_name("networks");
        }
        Err(err) => ctx
            .status
            .set_text(&format!("Error fetching networks: {err}")),
    }

    apply_connectivity_status(&ctx).await;

    is_scanning.set(false);
}

pub fn clear_children(container: &gtk::Box) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        container.remove(&widget);
    }
}

async fn saved_network_ids(ctx: &NetworksContext) -> HashSet<String> {
    ctx.nm
        .list_saved_connections_brief()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.connection_type == "802-11-wireless")
        .map(|c| c.id)
        .collect()
}

/// Refresh the network list WITHOUT triggering a new scan.
/// This is useful for live updates when the network list changes
/// (e.g., wired device state changes, AP added/removed).
pub async fn refresh_networks_no_scan(
    ctx: Rc<NetworksContext>,
    list_container: &GtkBox,
    is_scanning: &Rc<Cell<bool>>,
) {
    if is_scanning.get() {
        return;
    }

    is_scanning.set(true);

    clear_children(list_container);

    if let Ok(wired_devices) = ctx.nm.list_wired_devices().await {
        let available_devices: Vec<_> = wired_devices
            .into_iter()
            .filter(|dev| {
                matches!(
                    dev.state,
                    models::DeviceState::Activated
                        | models::DeviceState::Disconnected
                        | models::DeviceState::Prepare
                        | models::DeviceState::Config
                        | models::DeviceState::Unmanaged
                )
            })
            .collect();

        if !available_devices.is_empty() {
            let wired_header = Label::new(Some("Wired"));
            wired_header.add_css_class("section-header");
            wired_header.add_css_class("wired-section-header");
            wired_header.set_halign(Align::Start);
            wired_header.set_margin_top(8);
            wired_header.set_margin_bottom(4);
            wired_header.set_margin_start(12);
            list_container.append(&wired_header);

            let wired_list = wired_devices::wired_devices_view(
                ctx.clone(),
                &available_devices,
                ctx.wired_details_page.clone(),
            );
            wired_list.add_css_class("wired-devices-list");
            list_container.append(&wired_list);

            let separator = gtk::Separator::new(Orientation::Horizontal);
            separator.add_css_class("device-separator");
            separator.set_margin_top(12);
            separator.set_margin_bottom(12);
            list_container.append(&separator);
        }
    }

    let wireless_header = Label::new(Some("Wireless"));
    wireless_header.add_css_class("section-header");
    wireless_header.add_css_class("wireless-section-header");
    wireless_header.set_halign(Align::Start);
    wireless_header.set_margin_top(8);
    wireless_header.set_margin_bottom(4);
    wireless_header.set_margin_start(12);
    list_container.append(&wireless_header);

    let saved_ssids = saved_network_ids(&ctx).await;

    match ctx.nm.list_networks(None).await {
        Ok(mut nets) => {
            let current_conn = ctx.nm.current_connection_info().await;
            let (current_ssid, current_band) = if let Some((ssid, freq)) = current_conn {
                let ssid_str = ssid.clone();
                let band: Option<String> = freq
                    .and_then(crate::ui::freq_to_band)
                    .map(|s| s.to_string());
                (Some(ssid_str), band)
            } else {
                (None, None)
            };

            nets.sort_by_key(|b| std::cmp::Reverse(b.strength.unwrap_or(0)));

            let mut seen_combinations = HashSet::new();
            nets.retain(|net| {
                let band = net.frequency.and_then(crate::ui::freq_to_band);
                let key = (net.ssid.clone(), band);
                seen_combinations.insert(key)
            });

            let list: ListBox = networks::networks_view(
                ctx.clone(),
                &nets,
                current_ssid.as_deref(),
                current_band.as_deref(),
                &saved_ssids,
            );
            list_container.append(&list);
            ctx.stack.set_visible_child_name("networks");
        }
        Err(err) => {
            ctx.status
                .set_text(&format!("Error fetching networks: {err}"));
        }
    }

    apply_connectivity_status(&ctx).await;

    is_scanning.set(false);
}
