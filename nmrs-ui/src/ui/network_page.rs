use glib::clone;
use gtk::prelude::*;
use gtk::{Align, Box, Button, Image, Label, Orientation, Separator};
use nmrs_core::models::NetworkInfo;

pub fn network_page(info: &NetworkInfo, stack: &gtk::Stack) -> Box {
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
    header.append(&icon);
    header.append(&title);
    container.append(&header);

    // ------------------------------
    // Status section
    // ------------------------------
    let status_box = Box::new(Orientation::Vertical, 6);
    status_box.add_css_class("status-section");

    let status_header = Label::new(Some("Status"));
    status_header.add_css_class("section-header");
    status_box.append(&status_header);

    let status_fields = [
        ("Connection Status", info.status.as_str()),
        ("Signal Strength", &format!("{}%", info.strength)),
        ("Bars", info.bars.as_str()),
    ];

    for (label, value) in status_fields {
        let row = Box::new(Orientation::Vertical, 2);
        row.set_halign(Align::Start);

        let key = Label::new(Some(label));
        key.add_css_class("status-key");
        key.set_halign(Align::Start);

        let val = Label::new(Some(value));
        val.add_css_class("status-value");
        val.set_halign(Align::Start);

        row.append(&key);
        row.append(&val);
        status_box.append(&row);
    }

    container.append(&status_box);

    // Separator
    let sep = Separator::new(Orientation::Horizontal);
    sep.add_css_class("divider");
    container.append(&sep);

    // ------------------------------
    // Advanced Information
    // ------------------------------
    let advanced_box = Box::new(Orientation::Vertical, 8);
    advanced_box.add_css_class("advanced-section");

    let advanced_header = Label::new(Some("Advanced Information"));
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
