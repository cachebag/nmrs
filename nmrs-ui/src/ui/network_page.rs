use glib::clone;
use gtk::prelude::*;
use gtk::{Align, Box, Button, Image, Label, Orientation};
use nmrs_core::models::NetworkInfo;

pub fn network_page(info: &NetworkInfo, stack: &gtk::Stack) -> Box {
    let container = Box::new(Orientation::Vertical, 0);
    container.add_css_class("network-page");

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

    let header = Box::new(Orientation::Horizontal, 6);
    let icon = Image::from_icon_name("network-wireless-signal-excellent-symbolic");
    icon.set_pixel_size(22);
    let title = Label::new(Some(&info.ssid));
    title.add_css_class("network-title");
    header.append(&icon);
    header.append(&title);
    container.append(&header);

    let info_box = Box::new(Orientation::Vertical, 6);
    info_box.add_css_class("network-info");

    let fields = [
        ("BSSID", &info.bssid),
        ("Signal Strength", &format!("{}%", info.strength)),
        ("Security", &info.security),
    ];

    for (label, value) in fields {
        let row = Box::new(Orientation::Horizontal, 6);
        let key = Label::new(Some(label));
        key.add_css_class("info-label");
        key.set_halign(Align::Start);

        let val = Label::new(Some(value));
        val.add_css_class("info-value");
        val.set_halign(Align::Start);

        row.append(&key);
        row.append(&val);
        info_box.append(&row);
    }

    container.append(&info_box);
    container
}
