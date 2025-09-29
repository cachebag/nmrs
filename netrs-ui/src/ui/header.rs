use gtk::prelude::*;
use gtk::{Box as GtkBox, HeaderBar, Label, Orientation, Switch};

pub fn build_header(status: &Label) -> HeaderBar {
    let header = HeaderBar::new();

    let wifi_box = GtkBox::new(Orientation::Horizontal, 6);
    let wifi_label = Label::new(Some("Wi-Fi"));
    let wifi_switch = Switch::new();

    wifi_box.append(&wifi_label);
    wifi_box.append(&wifi_switch);

    let status_clone = status.clone();
    wifi_switch.connect_active_notify(move |sw| {
        if sw.is_active() {
            status_clone.set_text("");
        } else {
            status_clone.set_text("Wi-Fi is disabled");
        }
    });

    header.pack_start(&wifi_box);
    header
}
