use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, ListBox, Orientation};

pub fn create_device_list() -> (GtkBox, ListBox) {
    let central = GtkBox::new(Orientation::Vertical, 5);
    let device_list = ListBox::new();

    central.append(&device_list);
    central.set_halign(Align::Center);
    central.set_valign(Align::Center);

    (central, device_list)
}
