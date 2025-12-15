use glib::clone;
use gtk::prelude::*;
use gtk::STYLE_PROVIDER_PRIORITY_USER;
use gtk::{glib, Align, Box as GtkBox, HeaderBar, Label, ListBox, Orientation, Switch};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

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
    window: &gtk::ApplicationWindow,
) -> HeaderBar {
    let header = HeaderBar::new();
    header.set_show_title_buttons(false);

    let list_container = list_container.clone();

    let wifi_box = GtkBox::new(Orientation::Horizontal, 6);
    let wifi_label = Label::new(Some("Wi-Fi"));
    wifi_label.set_halign(gtk::Align::Start);
    wifi_label.add_css_class("wifi-label");

    let names: Vec<&str> = THEMES.iter().map(|t| t.name).collect();
    let dropdown = gtk::DropDown::from_strings(&names);

    if let Some(saved) = crate::theme_config::load_theme() {
        if let Some(idx) = THEMES.iter().position(|t| t.key == saved.as_str()) {
            dropdown.set_selected(idx as u32);
        }
    }

    dropdown.set_valign(gtk::Align::Center);
    dropdown.add_css_class("dropdown");

    let window_weak = window.downgrade();

    dropdown.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        if idx >= THEMES.len() {
            return;
        }

        let theme = &THEMES[idx];

        if let Some(window) = window_weak.upgrade() {
            let provider = gtk::CssProvider::new();
            provider.load_from_data(theme.css);

            let display = gtk::prelude::RootExt::display(&window);

            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                STYLE_PROVIDER_PRIORITY_USER,
            );

            crate::theme_config::save_theme(theme.key);
        }
    });

    wifi_box.append(&wifi_label);
    wifi_box.append(&dropdown);
    header.pack_start(&wifi_box);

    let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.add_css_class("refresh-btn");
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

    let theme_btn = gtk::Button::new();
    theme_btn.add_css_class("theme-toggle-btn");
    theme_btn.set_valign(gtk::Align::Center);
    theme_btn.set_has_frame(false);

    let is_light = window.has_css_class("light-theme");
    let initial_icon = if is_light {
        "weather-clear-night-symbolic"
    } else {
        "weather-clear-symbolic"
    };
    theme_btn.set_icon_name(initial_icon);

    let window_weak = window.downgrade();
    theme_btn.connect_clicked(move |btn| {
        if let Some(window) = window_weak.upgrade() {
            let is_light = window.has_css_class("light-theme");

            if is_light {
                window.remove_css_class("light-theme");
                window.add_css_class("dark-theme");
                btn.set_icon_name("weather-clear-symbolic");
                crate::theme_config::save_theme("light");
            } else {
                window.remove_css_class("dark-theme");
                window.add_css_class("light-theme");
                btn.set_icon_name("weather-clear-night-symbolic");
                crate::theme_config::save_theme("dark");
            }
        }
    });

    header.pack_end(&theme_btn);

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

                // Create a wired devices context
                let wired_ctx = wired_devices::WiredDevicesContext {
                    nm: ctx.nm.clone(),
                    on_success: ctx.on_success.clone(),
                    status: ctx.status.clone(),
                    stack: ctx.stack.clone(),
                    parent_window: ctx.parent_window.clone(),
                };
                let wired_ctx = Rc::new(wired_ctx);

                let wired_list = wired_devices::wired_devices_view(
                    wired_ctx,
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
                let band: Option<String> = freq
                    .and_then(crate::ui::freq_to_band)
                    .map(|s| s.to_string());
                (Some(ssid_str), band)
            } else {
                (None, None)
            };

            nets.sort_by(|a, b| b.strength.unwrap_or(0).cmp(&a.strength.unwrap_or(0)));

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

pub fn clear_children(container: &gtk::Box) {
    let mut child = container.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        container.remove(&widget);
    }
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
        // Don't interfere with an ongoing scan or refresh
        return;
    }

    // Set flag to prevent concurrent refreshes
    is_scanning.set(true);

    clear_children(list_container);

    // Fetch wired devices first
    if let Ok(wired_devices) = ctx.nm.list_wired_devices().await {
        // eprintln!("Found {} wired devices total", wired_devices.len());

        // Filter out unavailable devices to reduce clutter
        let available_devices: Vec<_> = wired_devices
            .into_iter()
            .filter(|dev| {
                let show = matches!(
                    dev.state,
                    models::DeviceState::Activated
                        | models::DeviceState::Disconnected
                        | models::DeviceState::Prepare
                        | models::DeviceState::Config
                        | models::DeviceState::Unmanaged
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
        );*/

        if !available_devices.is_empty() {
            let wired_header = Label::new(Some("Wired"));
            wired_header.add_css_class("section-header");
            wired_header.add_css_class("wired-section-header");
            wired_header.set_halign(Align::Start);
            wired_header.set_margin_top(8);
            wired_header.set_margin_bottom(4);
            wired_header.set_margin_start(12);
            list_container.append(&wired_header);

            let wired_ctx = wired_devices::WiredDevicesContext {
                nm: ctx.nm.clone(),
                on_success: ctx.on_success.clone(),
                status: ctx.status.clone(),
                stack: ctx.stack.clone(),
                parent_window: ctx.parent_window.clone(),
            };
            let wired_ctx = Rc::new(wired_ctx);

            let wired_list = wired_devices::wired_devices_view(
                wired_ctx,
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

    match ctx.nm.list_networks().await {
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

            nets.sort_by(|a, b| b.strength.unwrap_or(0).cmp(&a.strength.unwrap_or(0)));

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
            );
            list_container.append(&list);
            ctx.stack.set_visible_child_name("networks");
        }
        Err(err) => {
            ctx.status
                .set_text(&format!("Error fetching networks: {err}"));
        }
    }

    // Release the lock
    is_scanning.set(false);
}
