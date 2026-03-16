use anyhow::Result;
use gtk::Align;
use gtk::GestureClick;
use gtk::prelude::*;
use gtk::{Box, Image, Label, ListBox, ListBoxRow, Orientation};
use nmrs::models::WifiSecurity;
use nmrs::{NetworkManager, models};
use std::rc::Rc;

use crate::ui::connect;
use crate::ui::network_page::NetworkPage;

pub struct NetworkRowController {
    pub row: gtk::ListBoxRow,
    pub arrow: gtk::Image,
    pub ctx: Rc<NetworksContext>,
    pub net: models::Network,
    pub details_page: Rc<NetworkPage>,
}

pub struct NetworksContext {
    pub nm: Rc<NetworkManager>,
    pub on_success: Rc<dyn Fn()>,
    pub status: Label,
    pub stack: gtk::Stack,
    pub parent_window: gtk::ApplicationWindow,
    pub details_page: Rc<NetworkPage>,
    pub wired_details_page: Rc<crate::ui::wired_page::WiredPage>,
}

impl NetworksContext {
    pub async fn new(
        on_success: Rc<dyn Fn()>,
        status: &Label,
        stack: &gtk::Stack,
        parent_window: &gtk::ApplicationWindow,
        details_page: Rc<NetworkPage>,
        wired_details_page: Rc<crate::ui::wired_page::WiredPage>,
    ) -> Result<Self> {
        let nm = Rc::new(NetworkManager::new().await?);

        Ok(Self {
            nm,
            on_success,
            status: status.clone(),
            stack: stack.clone(),
            parent_window: parent_window.clone(),
            details_page,
            wired_details_page,
        })
    }
}

impl NetworkRowController {
    pub fn new(
        row: gtk::ListBoxRow,
        arrow: gtk::Image,
        ctx: Rc<NetworksContext>,
        net: models::Network,
        details_page: Rc<NetworkPage>,
    ) -> Self {
        Self {
            row,
            arrow,
            ctx,
            net,
            details_page,
        }
    }

    pub fn attach(&self) {
        self.attach_arrow();
        self.attach_row_double();
    }

    fn attach_arrow(&self) {
        let click = GestureClick::new();

        let ctx = self.ctx.clone();
        let net = self.net.clone();
        let stack = self.ctx.stack.clone();
        let page = self.details_page.clone();

        click.connect_pressed(move |_, _, _, _| {
            let ctx_c = ctx.clone();
            let net_c = net.clone();
            let stack_c = stack.clone();
            let page_c = page.clone();

            glib::MainContext::default().spawn_local(async move {
                if let Ok(info) = ctx_c.nm.show_details(&net_c).await {
                    page_c.update(&info);
                    stack_c.set_visible_child_name("details");
                }
            });
        });

        self.arrow.add_controller(click);
    }

    fn attach_row_double(&self) {
        let click = GestureClick::new();

        let ctx = self.ctx.clone();
        let net = self.net.clone();
        let ssid = net.ssid.clone();
        let secured = net.secured;
        let is_eap = net.is_eap;

        let status = ctx.status.clone();
        let window = ctx.parent_window.clone();
        let on_success = ctx.on_success.clone();

        click.connect_pressed(move |_, n, _, _| {
            if n != 2 {
                return;
            }

            status.set_text(&format!("Connecting to {ssid}..."));

            let ssid_c = ssid.clone();
            let nm_c = ctx.nm.clone();
            let status_c = status.clone();
            let window_c = window.clone();
            let on_success_c = on_success.clone();

            glib::MainContext::default().spawn_local(async move {
                if secured {
                    let have = nm_c.has_saved_connection(&ssid_c).await.unwrap_or(false);

                    if have {
                        status_c.set_text(&format!("Connecting to {}...", ssid_c));
                        window_c.set_sensitive(false);
                        let creds = WifiSecurity::WpaPsk { psk: "".into() };
                        match nm_c.connect(&ssid_c, creds).await {
                            Ok(_) => {
                                status_c.set_text("");
                                on_success_c();
                            }
                            Err(e) => status_c.set_text(&format!("Failed to connect: {e}")),
                        }
                        window_c.set_sensitive(true);
                    } else {
                        connect::connect_modal(
                            nm_c.clone(),
                            &window_c,
                            &ssid_c,
                            is_eap,
                            on_success_c.clone(),
                        );
                    }
                } else {
                    status_c.set_text(&format!("Connecting to {}...", ssid_c));
                    window_c.set_sensitive(false);
                    let creds = WifiSecurity::Open;
                    match nm_c.connect(&ssid_c, creds).await {
                        Ok(_) => {
                            status_c.set_text("");
                            on_success_c();
                        }
                        Err(e) => status_c.set_text(&format!("Failed to connect: {e}")),
                    }
                    window_c.set_sensitive(true);
                }

                status_c.set_text("");
            });
        });

        self.row.add_controller(click);
    }
}

pub fn networks_view(
    ctx: Rc<NetworksContext>,
    networks: &[models::Network],
    current_ssid: Option<&str>,
    current_band: Option<&str>,
) -> ListBox {
    let conn_threshold = 75;
    let list = ListBox::new();

    let mut sorted_networks: Vec<_> = networks
        .iter()
        .filter(|net| !net.ssid.trim().is_empty())
        .cloned()
        .collect();

    sorted_networks.sort_by(|a, b| {
        let a_connected = is_current_network(a, current_ssid, current_band);
        let b_connected = is_current_network(b, current_ssid, current_band);

        match (a_connected, b_connected) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.strength.unwrap_or(0).cmp(&a.strength.unwrap_or(0)),
        }
    });

    for net in sorted_networks {
        let row = ListBoxRow::new();
        let hbox = Box::new(Orientation::Horizontal, 6);

        row.add_css_class("network-selection");

        if is_current_network(&net, current_ssid, current_band) {
            row.add_css_class("connected");
        }

        let display_name = match net.frequency.and_then(crate::ui::freq_to_band) {
            Some(band) => format!("{} ({band})", net.ssid),
            None => net.ssid.clone(),
        };

        hbox.append(&Label::new(Some(&display_name)));

        if is_current_network(&net, current_ssid, current_band) {
            let connected_label = Label::new(Some("Connected"));
            connected_label.add_css_class("connected-label");
            hbox.append(&connected_label);
        }

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        hbox.append(&spacer);

        if let Some(s) = net.strength {
            let icon_name = if net.secured {
                "network-wireless-encrypted-symbolic"
            } else {
                "network-wireless-signal-excellent-symbolic"
            };

            let image = Image::from_icon_name(icon_name);
            if net.secured {
                image.add_css_class("wifi-secure");
            } else {
                image.add_css_class("wifi-open");
            }

            let strength_label = Label::new(Some(&format!("{s}%")));
            hbox.append(&image);
            hbox.append(&strength_label);

            if s >= conn_threshold {
                strength_label.add_css_class("network-good");
            } else if s > 65 {
                strength_label.add_css_class("network-okay");
            } else {
                strength_label.add_css_class("network-poor");
            }
        }

        let arrow = Image::from_icon_name("go-next-symbolic");
        arrow.set_halign(Align::End);
        arrow.add_css_class("network-arrow");
        arrow.set_cursor_from_name(Some("pointer"));
        hbox.append(&arrow);

        row.set_child(Some(&hbox));

        let controller = NetworkRowController::new(
            row.clone(),
            arrow.clone(),
            ctx.clone(),
            net.clone(),
            ctx.details_page.clone(),
        );

        controller.attach();

        list.append(&row);
    }
    list
}

fn is_current_network(
    net: &models::Network,
    current_ssid: Option<&str>,
    current_band: Option<&str>,
) -> bool {
    let ssid = match current_ssid {
        Some(s) => s,
        None => return false,
    };

    if net.ssid != ssid {
        return false;
    }

    if let Some(band) = current_band {
        let net_band = net.frequency.and_then(crate::ui::freq_to_band);

        return net_band == Some(band);
    }

    true
}
