use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation};
use nmrs_core::models::NetworkInfo;
use relm4::RelmWidgetExt;

pub fn network_page(info: &NetworkInfo) -> Box {
    let container = Box::new(Orientation::Vertical, 16);
    container.add_css_class("network-page");
    container.set_margin_all(20);

    let header = Box::new(Orientation::Horizontal, 8);
    let icon = Image::from_icon_name("network-wireless-signal-excellent-symbolic");
    icon.set_pixel_size(32);
    let title = Label::new(Some(&info.ssid));
    title.add_css_class("network-title");
    header.append(&icon);
    header.append(&title);
    container.append(&header);

    let details = Box::new(Orientation::Vertical, 4);
    let bssid = Label::new(Some(&format!("BSSID: {}", info.bssid)));
    let strength = Label::new(Some(&format!("Signal: {}%", info.strength)));
    let security = Label::new(Some(&format!("Security: {}", info.security)));
    details.append(&bssid);
    details.append(&strength);
    details.append(&security);
    container.append(&details);

    let actions = Box::new(Orientation::Horizontal, 12);
    actions.set_halign(gtk::Align::End);

    let disconnect = Button::with_label("Disconnect");
    disconnect.add_css_class("disconnect-btn");
    actions.append(&disconnect);

    let forget = Button::with_label("Forget");
    forget.add_css_class("forget-btn");
    actions.append(&forget);

    container.append(&actions);

    container
}
