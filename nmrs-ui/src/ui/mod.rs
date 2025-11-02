pub mod connect;
pub mod header;
pub mod network_page;
pub mod networks;

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ScrolledWindow, Stack,
};

pub fn build_ui(app: &Application) {
    let win = ApplicationWindow::new(app);
    win.set_title(Some(""));
    win.set_default_size(400, 600);

    let vbox = GtkBox::new(Orientation::Vertical, 0);
    let status = Label::new(None);
    let list_container = GtkBox::new(Orientation::Vertical, 0);

    let stack = Stack::new();
    stack.add_named(&list_container, Some("networks"));
    stack.set_visible_child_name("networks");

    let header = header::build_header(&status, &list_container, &win, &stack);
    vbox.append(&header);

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_child(Some(&stack));
    vbox.append(&scroller);

    win.set_child(Some(&vbox));
    win.show();
}
