use gtk::prelude::*;
use gtk::{Box, Label, ListBox, ListBoxRow, Orientation};
use netrs_core::models;

pub fn networks_view(networks: &[models::Network]) -> ListBox {
    let list = ListBox::new();

    for net in networks {
        let row = ListBoxRow::new();
        let hbox = Box::new(Orientation::Horizontal, 6);

        row.add_css_class("network-selection");
        let ssid = Label::new(Some(&net.ssid));

        hbox.append(&ssid);
        if let Some(s) = net.strength {
            let strength_label = Label::new(Some(&format!("{s}%")));
            hbox.append(&strength_label);
        }

        row.set_child(Some(&hbox));
        list.append(&row);
    }

    list
}
