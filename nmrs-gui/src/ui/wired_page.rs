use glib::clone;
use gtk::prelude::*;
use gtk::{Align, Box, Button, Image, Label, Orientation};
use nmrs::models::Device;

pub struct WiredPage {
    root: gtk::Box,

    title: gtk::Label,
    state_label: gtk::Label,

    interface: gtk::Label,
    device_type: gtk::Label,
    mac_address: gtk::Label,
    driver: gtk::Label,
    managed: gtk::Label,
}

impl WiredPage {
    pub fn new(stack: &gtk::Stack) -> Self {
        let root = Box::new(Orientation::Vertical, 12);
        root.add_css_class("network-page");

        let back = Button::with_label("← Back");
        back.add_css_class("back-button");
        back.set_halign(Align::Start);
        back.set_cursor_from_name(Some("pointer"));
        back.connect_clicked(clone![
            #[weak]
            stack,
            move |_| {
                stack.set_visible_child_name("networks");
            }
        ]);
        root.append(&back);

        let header = Box::new(Orientation::Horizontal, 6);
        let icon = Image::from_icon_name("network-wired-symbolic");
        icon.set_pixel_size(24);

        let title = Label::new(None);
        title.add_css_class("network-title");

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        header.append(&icon);
        header.append(&title);
        header.append(&spacer);
        root.append(&header);

        let basic_box = Box::new(Orientation::Vertical, 6);
        basic_box.add_css_class("basic-section");

        let basic_header = Label::new(Some("Basic"));
        basic_header.add_css_class("section-header");
        basic_box.append(&basic_header);

        let state_label = Label::new(None);
        let interface = Label::new(None);

        Self::add_row(&basic_box, "Connection State", &state_label);
        Self::add_row(&basic_box, "Interface", &interface);

        root.append(&basic_box);

        let advanced_box = Box::new(Orientation::Vertical, 8);
        advanced_box.add_css_class("advanced-section");

        let advanced_header = Label::new(Some("Advanced"));
        advanced_header.add_css_class("section-header");
        advanced_box.append(&advanced_header);

        let device_type = Label::new(None);
        let mac_address = Label::new(None);
        let driver = Label::new(None);
        let managed = Label::new(None);

        Self::add_row(&advanced_box, "Device Type", &device_type);
        Self::add_row(&advanced_box, "MAC Address", &mac_address);
        Self::add_row(&advanced_box, "Driver", &driver);
        Self::add_row(&advanced_box, "Managed", &managed);

        root.append(&advanced_box);

        Self {
            root,
            title,
            state_label,

            interface,
            device_type,
            mac_address,
            driver,
            managed,
        }
    }

    fn add_row(parent: &gtk::Box, key_text: &str, val_widget: &gtk::Label) {
        let row = Box::new(Orientation::Vertical, 3);
        row.set_halign(Align::Start);

        let key = Label::new(Some(key_text));
        key.add_css_class("info-label");
        key.set_halign(Align::Start);

        val_widget.add_css_class("info-value");
        val_widget.set_halign(Align::Start);

        row.append(&key);
        row.append(val_widget);
        parent.append(&row);
    }

    pub fn update(&self, device: &Device) {
        self.title
            .set_text(&format!("Wired Device: {}", device.interface));
        self.state_label.set_text(&format!("{}", device.state));
        self.interface.set_text(&device.interface);
        self.device_type
            .set_text(&format!("{}", device.device_type));
        self.mac_address.set_text(&device.identity.current_mac);
        self.driver
            .set_text(&device.driver.clone().unwrap_or_else(|| "-".into()));
        self.managed.set_text(
            device
                .managed
                .map(|m| if m { "Yes" } else { "No" })
                .unwrap_or("-"),
        );
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}
