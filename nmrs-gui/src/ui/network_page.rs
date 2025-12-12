use glib::clone;
use gtk::prelude::*;
use gtk::{Align, Box, Button, Image, Label, Orientation};
use nmrs::NetworkManager;
use nmrs::models::NetworkInfo;
use std::cell::RefCell;
use std::rc::Rc;

pub struct NetworkPage {
    root: gtk::Box,

    title: gtk::Label,
    status: gtk::Label,
    strength: gtk::Label,
    bars: gtk::Label,

    bssid: gtk::Label,
    freq: gtk::Label,
    channel: gtk::Label,
    mode: gtk::Label,
    rate: gtk::Label,
    security: gtk::Label,

    current_ssid: Rc<RefCell<String>>,
}

impl NetworkPage {
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
        let icon = Image::from_icon_name("network-wireless-signal-excellent-symbolic");
        icon.set_pixel_size(24);

        let title = Label::new(None);
        title.add_css_class("network-title");

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let forget_btn = Button::with_label("Forget");
        forget_btn.add_css_class("forget-button");
        forget_btn.set_halign(Align::End);
        forget_btn.set_valign(Align::Center);
        forget_btn.set_cursor_from_name(Some("pointer"));

        let current_ssid = Rc::new(RefCell::new(String::new()));

        {
            let stack_clone = stack.clone();
            let current_ssid_clone = current_ssid.clone();

            forget_btn.connect_clicked(move |_| {
                let stack = stack_clone.clone();
                let ssid = current_ssid_clone.borrow().clone();

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(nm) = NetworkManager::new().await
                        && nm.forget(&ssid).await.is_ok()
                    {
                        stack.set_visible_child_name("networks");
                    }
                });
            });
        }

        header.append(&icon);
        header.append(&title);
        header.append(&spacer);
        header.append(&forget_btn);
        root.append(&header);

        let basic_box = Box::new(Orientation::Vertical, 6);
        basic_box.add_css_class("basic-section");

        let basic_header = Label::new(Some("Basic"));
        basic_header.add_css_class("section-header");
        basic_box.append(&basic_header);

        let status = Label::new(None);
        let strength = Label::new(None);
        let bars = Label::new(None);

        Self::add_row(&basic_box, "Connection Status", &status);
        Self::add_row(&basic_box, "Signal Strength", &strength);
        Self::add_row(&basic_box, "Bars", &bars);

        root.append(&basic_box);

        let advanced_box = Box::new(Orientation::Vertical, 8);
        advanced_box.add_css_class("advanced-section");

        let advanced_header = Label::new(Some("Advanced"));
        advanced_header.add_css_class("section-header");
        advanced_box.append(&advanced_header);

        let bssid = Label::new(None);
        let freq = Label::new(None);
        let channel = Label::new(None);
        let mode = Label::new(None);
        let rate = Label::new(None);
        let security = Label::new(None);

        Self::add_row(&advanced_box, "BSSID", &bssid);
        Self::add_row(&advanced_box, "Frequency", &freq);
        Self::add_row(&advanced_box, "Channel", &channel);
        Self::add_row(&advanced_box, "Mode", &mode);
        Self::add_row(&advanced_box, "Speed", &rate);
        Self::add_row(&advanced_box, "Security", &security);

        root.append(&advanced_box);

        Self {
            root,
            title,
            status,
            strength,
            bars,

            bssid,
            freq,
            channel,
            mode,
            rate,
            security,
            current_ssid,
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

    pub fn update(&self, info: &NetworkInfo) {
        self.current_ssid.replace(info.ssid.clone());
        self.title.set_text(&info.ssid);
        self.status.set_text(&info.status);
        self.strength.set_text(&format!("{}%", info.strength));
        self.bars.set_text(&info.bars);

        self.bssid.set_text(&info.bssid);
        self.freq.set_text(
            &info
                .freq
                .map(|f| format!("{:.1} GHz", f as f32 / 1000.0))
                .unwrap_or_else(|| "-".into()),
        );
        self.channel.set_text(
            &info
                .channel
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".into()),
        );
        self.mode.set_text(&info.mode);
        self.rate.set_text(
            &info
                .rate_mbps
                .map(|r| format!("{r:.2} Mbps"))
                .unwrap_or_else(|| "-".into()),
        );
        self.security.set_text(&info.security);
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}
