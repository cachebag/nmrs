use gtk::prelude::*;
use gtk::{Align, Box, Button, Label, Orientation, STYLE_PROVIDER_PRIORITY_USER};

use crate::ui::header::{THEMES, ThemeDef};

pub struct SettingsPage {
    root: gtk::Box,
}

impl SettingsPage {
    pub fn new(stack: &gtk::Stack, window: &gtk::ApplicationWindow) -> Self {
        let root = Box::new(Orientation::Vertical, 12);
        root.add_css_class("settings-page");
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(16);
        root.set_margin_end(16);

        let back = Button::with_label("← Back");
        back.add_css_class("back-button");
        back.set_halign(Align::Start);
        back.set_cursor_from_name(Some("pointer"));
        {
            let stack = stack.clone();
            back.connect_clicked(move |_| {
                stack.set_visible_child_name("networks");
            });
        }
        root.append(&back);

        let title = Label::new(Some("Settings"));
        title.add_css_class("section-header");
        title.set_halign(Align::Start);
        root.append(&title);

        Self::build_theme_section(&root, window);
        Self::build_light_dark_section(&root, window);

        Self { root }
    }

    fn build_theme_section(root: &gtk::Box, window: &gtk::ApplicationWindow) {
        let section = Box::new(Orientation::Vertical, 6);

        let label = Label::new(Some("Theme"));
        label.add_css_class("info-label");
        label.set_halign(Align::Start);
        section.append(&label);

        let names: Vec<&str> = THEMES.iter().map(|t| t.name).collect();
        let dropdown = gtk::DropDown::from_strings(&names);
        dropdown.set_halign(Align::Start);
        dropdown.set_hexpand(false);

        if let Some(saved) = crate::theme_config::load_theme()
            && let Some(idx) = THEMES.iter().position(|t| t.key == saved.as_str())
        {
            dropdown.set_selected(idx as u32);
        }

        let window_weak = window.downgrade();
        dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if idx >= THEMES.len() {
                return;
            }

            let theme = &THEMES[idx];

            if let Some(window) = window_weak.upgrade() {
                Self::apply_theme(theme, &window);
            }
        });

        section.append(&dropdown);
        root.append(&section);
    }

    fn build_light_dark_section(root: &gtk::Box, window: &gtk::ApplicationWindow) {
        let section = Box::new(Orientation::Vertical, 6);

        let label = Label::new(Some("Appearance"));
        label.add_css_class("info-label");
        label.set_halign(Align::Start);
        section.append(&label);

        let toggle_box = Box::new(Orientation::Horizontal, 8);

        let light_btn = Button::with_label("Light");
        light_btn.add_css_class("appearance-btn");

        let dark_btn = Button::with_label("Dark");
        dark_btn.add_css_class("appearance-btn");

        {
            let window_weak = window.downgrade();
            let dark_btn_clone = dark_btn.clone();
            light_btn.connect_clicked(move |btn| {
                if let Some(window) = window_weak.upgrade() {
                    window.remove_css_class("dark-theme");
                    window.add_css_class("light-theme");
                    btn.add_css_class("appearance-active");
                    dark_btn_clone.remove_css_class("appearance-active");
                    crate::theme_config::save_theme("dark");
                }
            });
        }

        {
            let window_weak = window.downgrade();
            let light_btn_clone = light_btn.clone();
            dark_btn.connect_clicked(move |btn| {
                if let Some(window) = window_weak.upgrade() {
                    window.remove_css_class("light-theme");
                    window.add_css_class("dark-theme");
                    btn.add_css_class("appearance-active");
                    light_btn_clone.remove_css_class("appearance-active");
                    crate::theme_config::save_theme("light");
                }
            });
        }

        if window.has_css_class("light-theme") {
            light_btn.add_css_class("appearance-active");
        } else {
            dark_btn.add_css_class("appearance-active");
        }

        toggle_box.append(&light_btn);
        toggle_box.append(&dark_btn);
        section.append(&toggle_box);
        root.append(&section);
    }

    fn apply_theme(theme: &ThemeDef, window: &gtk::ApplicationWindow) {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(theme.css);

        let display = gtk::prelude::RootExt::display(window);

        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );

        crate::theme_config::save_theme(theme.key);
        crate::style::load_user_css();
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}
