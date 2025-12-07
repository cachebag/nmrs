pub mod connect;
pub mod header;
pub mod network_page;
pub mod networks;

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ScrolledWindow, Spinner,
    Stack,
};
use std::cell::Cell;
use std::rc::Rc;

type Callback = Rc<dyn Fn()>;
type CallbackCell = Rc<std::cell::RefCell<Option<Callback>>>;

pub fn build_ui(app: &Application) {
    let win = ApplicationWindow::new(app);
    win.set_title(Some(""));
    win.set_default_size(400, 600);

    // Load and apply saved theme preference
    let is_light = crate::theme_config::load_theme();
    if is_light {
        win.add_css_class("light-theme");
    } else {
        win.add_css_class("dark-theme");
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

    stack.add_named(&list_container, Some("networks"));

    let status_clone = status.clone();
    let list_container_clone = list_container.clone();
    let stack_clone = stack.clone();
    let win_clone = win.clone();
    let is_scanning_clone = is_scanning.clone();
    let vbox_clone = vbox.clone();

    glib::MainContext::default().spawn_local(async move {
        match nmrs_core::NetworkManager::new().await {
            Ok(nm) => {
                let nm = Rc::new(nm);

                // Create the callback with a self-reference using std::cell::RefCell
                let on_success: Rc<dyn Fn()> = {
                    let list_container = list_container_clone.clone();
                    let is_scanning = is_scanning_clone.clone();
                    let nm = nm.clone();
                    let status = status_clone.clone();
                    let stack = stack_clone.clone();
                    let parent_window = win_clone.clone();

                    // Store a weak self-reference that we'll populate later
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

                        glib::MainContext::default().spawn_local(async move {
                            // Get the callback from the cell to pass to the refreshed context
                            let callback = on_success_cell.borrow().as_ref().map(|cb| cb.clone());
                            let refresh_ctx = Rc::new(networks::NetworksContext {
                                nm,
                                on_success: callback.unwrap_or_else(|| Rc::new(|| {})),
                                status,
                                stack,
                                parent_window,
                            });
                            header::refresh_networks(refresh_ctx, &list_container, &is_scanning)
                                .await;
                        });
                    }) as Rc<dyn Fn()>;

                    // Store the callback in the cell for self-reference
                    *on_success_cell_clone.borrow_mut() = Some(callback.clone());

                    callback
                };

                let ctx = Rc::new(networks::NetworksContext {
                    nm,
                    on_success,
                    status: status_clone.clone(),
                    stack: stack_clone.clone(),
                    parent_window: win_clone.clone(),
                });

                let header =
                    header::build_header(ctx, &list_container_clone, is_scanning_clone, &win_clone);
                vbox_clone.prepend(&header);
            }
            Err(err) => {
                status_clone.set_text(&format!("Failed to initialize: {err}"));
            }
        }
    });

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroller.set_propagate_natural_height(true);
    scroller.set_child(Some(&stack));
    vbox.append(&scroller);

    win.set_child(Some(&vbox));
    win.show();
}
