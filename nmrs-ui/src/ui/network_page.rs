use glib::clone;
use gtk::prelude::*;
use gtk::{Align, Box, Button, Image, Label, Orientation};
use nmrs_core::NetworkManager;
use nmrs_core::models::NetworkInfo;

pub fn network_page(info: NetworkInfo, stack: &gtk::Stack) -> Box {
    let container = Box::new(Orientation::Vertical, 12);
    container.add_css_class("network-page");

    // Back button
    let back = Button::with_label("← Back");
    back.add_css_class("back-button");
    back.set_halign(Align::Start);
    back.set_cursor_from_name(Some("pointer"));
    back.connect_clicked(clone!(
        #[weak]
        stack,
        move |_| {
            stack.set_visible_child_name("networks");
        }
    ));
    container.append(&back);

    // Header
    let header = Box::new(Orientation::Horizontal, 6);
    let icon = Image::from_icon_name("network-wireless-signal-excellent-symbolic");
    icon.set_pixel_size(24);
    let title = Label::new(Some(&info.ssid));
    title.add_css_class("network-title");

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let forget_btn = Button::with_label("Forget");
    forget_btn.add_css_class("forget-button");
    forget_btn.set_halign(Align::End);
    forget_btn.set_valign(Align::Center);
    forget_btn.set_cursor_from_name(Some("pointer"));

    forget_btn.connect_clicked(clone!(
        #[strong]
        info,
        #[weak]
        stack,
        move |_| {
            let ssid = info.ssid.clone();
            glib::MainContext::default().spawn_local(async move {
                if let Ok(nm) = NetworkManager::new().await {
                    let _ = nm.forget(&ssid).await;
                    stack.set_visible_child_name("networks");
                }
            });
        }
    ));

    header.append(&icon);
    header.append(&title);
    header.append(&spacer);
    header.append(&forget_btn);
    container.append(&header);

    // Basic info section
    let basic_box = Box::new(Orientation::Vertical, 6);
    basic_box.add_css_class("basic-section");

    let basic_header = Label::new(Some("Basic"));
    basic_header.add_css_class("section-header");
    basic_box.append(&basic_header);

    let status_fields = [
        ("Connection Status", info.status.as_str()),
        ("Signal Strength", &format!("{}%", info.strength)),
        ("Bars", info.bars.as_str()),
    ];

    for (label, value) in status_fields {
        let row = Box::new(Orientation::Vertical, 2);
        row.set_halign(Align::Start);

        let key = Label::new(Some(label));
        key.add_css_class("basic-key");
        key.set_halign(Align::Start);

        let val = Label::new(Some(value));
        val.add_css_class("basic-value");
        val.set_halign(Align::Start);

        row.append(&key);
        row.append(&val);
        basic_box.append(&row);
    }

    container.append(&basic_box);

    // Advanced info section
    let advanced_box = Box::new(Orientation::Vertical, 8);
    advanced_box.add_css_class("advanced-section");

    let advanced_header = Label::new(Some("Advanced"));
    advanced_header.add_css_class("section-header");
    advanced_box.append(&advanced_header);

    let advanced_fields = [
        ("BSSID", info.bssid.as_str()),
        (
            "Frequency",
            &info
                .freq
                .map(|f| format!("{:.1} GHz", f as f32 / 1000.0))
                .unwrap_or_else(|| "-".into()),
        ),
        (
            "Channel",
            &info
                .channel
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".into()),
        ),
        ("Mode", info.mode.as_str()),
        (
            "Speed",
            &info
                .rate_mbps
                .map(|r| format!("{r:.2} Mbps"))
                .unwrap_or_else(|| "-".into()),
        ),
        ("Security", info.security.as_str()),
    ];

    for (label, value) in advanced_fields {
        let row = Box::new(Orientation::Vertical, 3);
        row.set_halign(Align::Start);

        let key = Label::new(Some(label));
        key.add_css_class("info-label");
        key.set_halign(Align::Start);

        let val = Label::new(Some(value));
        val.add_css_class("info-value");
        val.set_halign(Align::Start);

        row.append(&key);
        row.append(&val);

        advanced_box.append(&row);
    }

    container.append(&advanced_box);
    container
}
