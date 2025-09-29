pub mod header;
pub mod networks;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation, ScrolledWindow};
use netrs_core::models::Network;

pub fn build_ui(app: &Application, networks: Vec<Network>) {
    let win = ApplicationWindow::new(app);
    win.set_title(Some("netrs"));
    win.set_default_size(400, 600);

    let vbox = Box::new(Orientation::Vertical, 0);

    let status = Label::new(None);
    let header = header::build_header(&status);
    vbox.append(&header);

    let list = networks::networks_view(&networks);
    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_child(Some(&list));
    vbox.append(&scroller);

    win.set_child(Some(&vbox));
    win.show();
}
