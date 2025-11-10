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

pub fn build_ui(app: &Application) {
    let win = ApplicationWindow::new(app);
    win.set_title(Some(""));
    win.set_default_size(400, 600);

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

    let header = header::build_header(&status, &list_container, &win, &stack, is_scanning);
    vbox.append(&header);

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_child(Some(&stack));
    vbox.append(&scroller);

    win.set_child(Some(&vbox));
    win.show();
}
