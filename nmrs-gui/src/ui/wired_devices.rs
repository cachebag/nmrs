use gtk::Align;
use gtk::GestureClick;
use gtk::prelude::*;
use gtk::{Box, Image, Label, ListBox, ListBoxRow, Orientation};
use nmrs::{NetworkManager, models};
use std::rc::Rc;

use crate::ui::wired_page::WiredPage;

pub struct WiredDeviceRowController {
    pub row: gtk::ListBoxRow,
    pub arrow: gtk::Image,
    pub ctx: Rc<WiredDevicesContext>,
    pub device: models::Device,
    pub details_page: Rc<WiredPage>,
}

pub struct WiredDevicesContext {
    pub nm: Rc<NetworkManager>,
    pub on_success: Rc<dyn Fn()>,
    pub status: Label,
    pub stack: gtk::Stack,
    pub parent_window: gtk::ApplicationWindow,
}

impl WiredDeviceRowController {
    pub fn new(
        row: gtk::ListBoxRow,
        arrow: gtk::Image,
        ctx: Rc<WiredDevicesContext>,
        device: models::Device,
        details_page: Rc<WiredPage>,
    ) -> Self {
        Self {
            row,
            arrow,
            ctx,
            device,
            details_page,
        }
    }

    pub fn attach(&self) {
        self.attach_arrow();
        self.attach_row_double();
    }

    fn attach_arrow(&self) {
        let click = GestureClick::new();

        let device = self.device.clone();
        let stack = self.ctx.stack.clone();
        let page = self.details_page.clone();

        click.connect_pressed(move |_, _, _, _| {
            let device_c = device.clone();
            let stack_c = stack.clone();
            let page_c = page.clone();

            glib::MainContext::default().spawn_local(async move {
                page_c.update(&device_c);
                stack_c.set_visible_child_name("wired-details");
            });
        });

        self.arrow.add_controller(click);
    }

    fn attach_row_double(&self) {
        let click = GestureClick::new();

        let ctx = self.ctx.clone();
        let device = self.device.clone();
        let interface = device.interface.clone();

        let status = ctx.status.clone();
        let window = ctx.parent_window.clone();
        let on_success = ctx.on_success.clone();

        click.connect_pressed(move |_, n, _, _| {
            if n != 2 {
                return;
            }

            status.set_text(&format!("Connecting to {interface}..."));

            let nm_c = ctx.nm.clone();
            let status_c = status.clone();
            let window_c = window.clone();
            let on_success_c = on_success.clone();

            glib::MainContext::default().spawn_local(async move {
                window_c.set_sensitive(false);
                match nm_c.connect_wired().await {
                    Ok(_) => {
                        status_c.set_text("");
                        on_success_c();
                    }
                    Err(e) => status_c.set_text(&format!("Failed to connect: {e}")),
                }
                window_c.set_sensitive(true);
                status_c.set_text("");
            });
        });

        self.row.add_controller(click);
    }
}

pub fn wired_devices_view(
    ctx: Rc<WiredDevicesContext>,
    devices: &[models::Device],
    details_page: Rc<WiredPage>,
) -> ListBox {
    let list = ListBox::new();

    for device in devices {
        let row = ListBoxRow::new();
        let hbox = Box::new(Orientation::Horizontal, 6);

        row.add_css_class("network-selection");

        if device.state == models::DeviceState::Activated {
            row.add_css_class("connected");
        }

        let display_name = format!("{} ({})", device.interface, device.device_type);
        hbox.append(&Label::new(Some(&display_name)));

        if device.state == models::DeviceState::Activated {
            let connected_label = Label::new(Some("Connected"));
            connected_label.add_css_class("connected-label");
            hbox.append(&connected_label);
        }

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        hbox.append(&spacer);

        // Only show state for meaningful states (not transitional ones)
        let state_text = match device.state {
            models::DeviceState::Activated => Some("Connected"),
            models::DeviceState::Disconnected => Some("Disconnected"),
            models::DeviceState::Unavailable => Some("Unavailable"),
            models::DeviceState::Failed => Some("Failed"),
            // Hide transitional states (Unmanaged, Prepare, Config, etc)
            _ => None,
        };

        if let Some(text) = state_text {
            let state_label = Label::new(Some(text));
            state_label.add_css_class(match device.state {
                models::DeviceState::Activated => "network-good",
                models::DeviceState::Unavailable
                | models::DeviceState::Disconnected
                | models::DeviceState::Failed => "network-poor",
                _ => "network-okay",
            });
            hbox.append(&state_label);
        }

        let icon = Image::from_icon_name("network-wired-symbolic");
        icon.add_css_class("wired-icon");
        hbox.append(&icon);

        let arrow = Image::from_icon_name("go-next-symbolic");
        arrow.set_halign(Align::End);
        arrow.add_css_class("network-arrow");
        arrow.set_cursor_from_name(Some("pointer"));
        hbox.append(&arrow);

        row.set_child(Some(&hbox));

        let controller = WiredDeviceRowController::new(
            row.clone(),
            arrow.clone(),
            ctx.clone(),
            device.clone(),
            details_page.clone(),
        );

        controller.attach();

        list.append(&row);
    }
    list
}
