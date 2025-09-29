use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, CssProvider, ListBox, Orientation};

pub fn create_device_list() -> (GtkBox, ListBox) {
    let central = GtkBox::new(Orientation::Vertical, 5);
    let device_list = ListBox::new();

    // Add subtle styling
    let css_provider = CssProvider::new();
    css_provider.load_from_data(
        "
        .device-list {
            background: alpha(@theme_base_color, 0.95);
            border-radius: 8px;
            border: 1px solid alpha(@borders, 0.3);
            box-shadow: 0 1px 3px alpha(black, 0.05);
        }
        .device-list row {
            padding: 8px 12px;
            border-radius: 4px;
            margin: 2px 4px;
        }
        .device-list row:hover {
            background: alpha(@theme_selected_bg_color, 0.1);
        }
        .device-list row:selected {
            background: alpha(@theme_selected_bg_color, 0.15);
            color: @theme_selected_fg_color;
        }
        .device-container {
            padding: 16px;
            background: alpha(@theme_base_color, 0.5);
            border-radius: 12px;
        }
        "
    );

    device_list.add_css_class("device-list");
    central.add_css_class("device-container");

    let style_context = device_list.style_context();
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    
    let central_style_context = central.style_context();
    central_style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    central.append(&device_list);
    central.set_halign(Align::Center);
    central.set_valign(Align::Center);

    (central, device_list)
}
