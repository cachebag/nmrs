pub mod connect;
pub mod header;
pub mod network_page;
pub mod networks;
pub mod wired_devices;
pub mod wired_page;

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ScrolledWindow, Spinner,
    Stack, STYLE_PROVIDER_PRIORITY_USER,
};
use std::cell::Cell;
use std::rc::Rc;

use crate::ui::header::THEMES;

type Callback = Rc<dyn Fn()>;
type CallbackCell = Rc<std::cell::RefCell<Option<Callback>>>;

pub fn freq_to_band(freq: u32) -> Option<&'static str> {
    match freq {
        2400..=2500 => Some("2.4GHz"),
        5150..=5925 => Some("5GHz"),
        5926..=7125 => Some("6GHz"),
        _ => None,
    }
}

pub fn build_ui(app: &Application) {
    let win = ApplicationWindow::new(app);
    win.set_title(Some(""));
    win.set_default_size(100, 600);

    if let Some(key) = crate::theme_config::load_theme() {
        if let Some(theme) = THEMES.iter().find(|t| t.key == key.as_str()) {
            let provider = gtk::CssProvider::new();
            provider.load_from_data(theme.css);

            let display = gtk::prelude::RootExt::display(&win);
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                STYLE_PROVIDER_PRIORITY_USER,
            );

            win.add_css_class("dark-theme");
        }
    }

    let vbox = GtkBox::new(Orientation::Vertical, 0);
    let status = Label::new(None);
    let list_container = GtkBox::new(Orientation::Vertical, 0);
    let stack = Stack::new();
    let is_scanning = Rc::new(Cell::new(false));

    let spinner = Spinner::new();
    spinner.set_halign(gtk::Align::Center);
    spinner.set_valign(gtk::Align::Center);
    spinner.set_property("width-request", 24i32);
    spinner.set_property("height-request", 24i32);
    spinner.add_css_class("loading-spinner");
    spinner.start();

    stack.add_named(&spinner, Some("loading"));
    stack.set_visible_child_name("loading");

    let status_clone = status.clone();
    let list_container_clone = list_container.clone();
    let stack_clone = stack.clone();
    let win_clone = win.clone();
    let is_scanning_clone = is_scanning.clone();
    let vbox_clone = vbox.clone();

    glib::MainContext::default().spawn_local(async move {
        match nmrs::NetworkManager::new().await {
            Ok(nm) => {
                let nm = Rc::new(nm);

                let details_page = Rc::new(network_page::NetworkPage::new(&stack_clone));
                let details_scroller = ScrolledWindow::new();
                details_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
                details_scroller.set_child(Some(details_page.widget()));
                stack_clone.add_named(&details_scroller, Some("details"));

                let wired_details_page = Rc::new(wired_page::WiredPage::new(&stack_clone));
                let wired_details_scroller = ScrolledWindow::new();
                wired_details_scroller
                    .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
                wired_details_scroller.set_child(Some(wired_details_page.widget()));
                stack_clone.add_named(&wired_details_scroller, Some("wired-details"));

                let on_success: Rc<dyn Fn()> = {
                    let list_container = list_container_clone.clone();
                    let is_scanning = is_scanning_clone.clone();
                    let nm = nm.clone();
                    let status = status_clone.clone();
                    let stack = stack_clone.clone();
                    let parent_window = win_clone.clone();
                    let details_page = details_page.clone();
                    let wired_details_page = wired_details_page.clone();

                    let on_success_cell: CallbackCell = Rc::new(std::cell::RefCell::new(None));
                    let on_success_cell_clone = on_success_cell.clone();

                    let callback = Rc::new(move || {
                        let list_container = list_container.clone();
                        let is_scanning = is_scanning.clone();
                        let nm = nm.clone();
                        let status = status.clone();
                        let stack = stack.clone();
                        let parent_window = parent_window.clone();
                        let on_success_cell = on_success_cell.clone();
                        let details_page = details_page.clone();
                        let wired_details_page = wired_details_page.clone();

                        glib::MainContext::default().spawn_local(async move {
                            let callback = on_success_cell.borrow().as_ref().map(|cb| cb.clone());
                            let refresh_ctx = Rc::new(networks::NetworksContext {
                                nm,
                                on_success: callback.unwrap_or_else(|| Rc::new(|| {})),
                                status,
                                stack,
                                parent_window,
                                details_page: details_page.clone(),
                                wired_details_page: wired_details_page.clone(),
                            });
                            header::refresh_networks(refresh_ctx, &list_container, &is_scanning)
                                .await;
                        });
                    }) as Rc<dyn Fn()>;

                    *on_success_cell_clone.borrow_mut() = Some(callback.clone());

                    callback
                };

                let ctx = Rc::new(networks::NetworksContext {
                    nm: nm.clone(),
                    on_success: on_success.clone(),
                    status: status_clone.clone(),
                    stack: stack_clone.clone(),
                    parent_window: win_clone.clone(),
                    details_page: details_page.clone(),
                    wired_details_page,
                });

                details_page.set_on_success(on_success);

                let header = header::build_header(
                    ctx.clone(),
                    &list_container_clone,
                    is_scanning_clone.clone(),
                    &win_clone,
                );
                vbox_clone.prepend(&header);

                {
                    let nm_device_monitor = nm.clone();
                    let list_container_device = list_container_clone.clone();
                    let is_scanning_device = is_scanning_clone.clone();
                    let ctx_device = ctx.clone();
                    let pending_device_refresh = Rc::new(std::cell::RefCell::new(false));

                    glib::MainContext::default().spawn_local(async move {
                        loop {
                            let ctx_device_clone = ctx_device.clone();
                            let list_container_clone = list_container_device.clone();
                            let is_scanning_clone = is_scanning_device.clone();
                            let pending_device_refresh_clone = pending_device_refresh.clone();

                            let result = nm_device_monitor
                                .monitor_device_changes(move || {
                                    let ctx = ctx_device_clone.clone();
                                    let list_container = list_container_clone.clone();
                                    let is_scanning = is_scanning_clone.clone();
                                    let pending_refresh = pending_device_refresh_clone.clone();

                                    if pending_refresh.replace(true) {
                                        return;
                                    }

                                    glib::MainContext::default().spawn_local(async move {
                                        glib::timeout_future_seconds(3).await;
                                        *pending_refresh.borrow_mut() = false;

                                        let current_page = ctx.stack.visible_child_name();
                                        let on_networks_page =
                                            current_page.as_deref() == Some("networks");

                                        if !is_scanning.get() && on_networks_page {
                                            header::refresh_networks_no_scan(
                                                ctx,
                                                &list_container,
                                                &is_scanning,
                                            )
                                            .await;
                                        }
                                    });
                                })
                                .await;

                            if let Err(e) = result {
                                eprintln!("Device monitoring error: {}, restarting in 5s...", e)
                            }
                            glib::timeout_future_seconds(5).await;
                        }
                    });
                }

                {
                    let nm_network_monitor = nm.clone();
                    let list_container_network = list_container_clone.clone();
                    let is_scanning_network = is_scanning_clone.clone();
                    let ctx_network = ctx.clone();
                    let pending_network_refresh = Rc::new(std::cell::RefCell::new(false));

                    glib::MainContext::default().spawn_local(async move {
                        loop {
                            let ctx_network_clone = ctx_network.clone();
                            let list_container_clone = list_container_network.clone();
                            let is_scanning_clone = is_scanning_network.clone();
                            let pending_network_refresh_clone = pending_network_refresh.clone();

                            let result = nm_network_monitor
                                .monitor_network_changes(move || {
                                    let ctx = ctx_network_clone.clone();
                                    let list_container = list_container_clone.clone();
                                    let is_scanning = is_scanning_clone.clone();
                                    let pending_refresh = pending_network_refresh_clone.clone();

                                    if pending_refresh.replace(true) {
                                        return;
                                    }

                                    glib::MainContext::default().spawn_local(async move {
                                        glib::timeout_future_seconds(8).await;
                                        *pending_refresh.borrow_mut() = false;

                                        let current_page = ctx.stack.visible_child_name();
                                        let on_networks_page =
                                            current_page.as_deref() == Some("networks");

                                        if !is_scanning.get() && on_networks_page {
                                            header::refresh_networks_no_scan(
                                                ctx,
                                                &list_container,
                                                &is_scanning,
                                            )
                                            .await;
                                        }
                                    });
                                })
                                .await;

                            if let Err(e) = result {
                                eprintln!("Network monitoring error: {}, restarting in 5s...", e)
                            }
                            glib::timeout_future_seconds(5).await;
                        }
                    });
                }
            }
            Err(err) => {
                status_clone.set_text(&format!("Failed to initialize: {err}"));
            }
        }
    });

    let networks_scroller = ScrolledWindow::new();
    networks_scroller.set_vexpand(true);
    networks_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    networks_scroller.set_child(Some(&list_container));

    stack.add_named(&networks_scroller, Some("networks"));

    stack.set_vexpand(true);
    vbox.append(&stack);

    win.set_child(Some(&vbox));
    win.show();
}
