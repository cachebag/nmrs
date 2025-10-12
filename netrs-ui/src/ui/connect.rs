use gtk::{ApplicationWindow, prelude::*};
use gtk::{Box as GtkBox, Dialog, DialogFlags, Entry, Label, Orientation, ResponseType};

pub fn connect_modal(parent: &ApplicationWindow) {
    let dialog = Dialog::with_buttons(
        Some("Connect to Network"),
        Some(parent),
        DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Connect", ResponseType::Accept),
        ],
    );
    dialog.add_css_class("diag-buttons");

    let content_area = dialog.content_area();
    let vbox = GtkBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(32);
    vbox.set_margin_bottom(32);
    vbox.set_margin_start(48);
    vbox.set_margin_end(48);

    let label = Label::new(Some("Enter network password:"));
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Password"));
    entry.set_visibility(false); // hides characters

    vbox.append(&label);
    vbox.append(&entry);
    content_area.append(&vbox);

    dialog.set_default_response(ResponseType::Accept);
    dialog.show();

    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept {
            let pwd = entry.text();
            println!("User entered: {}", pwd);
            // insert connection logic here
        }
        d.close();
    });
}
