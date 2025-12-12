use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

mod imp {
    use super::*;
    use crate::objects::theme::glib::subclass::prelude::ObjectImpl;
    use crate::objects::theme::glib::subclass::prelude::ObjectSubclass;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct Theme {
        pub label: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Theme {
        const NAME: &'static str = "ThemeObject";
        type Type = super::Theme;
    }

    impl ObjectImpl for Theme {}
}

glib::wrapper! {
    pub struct Theme(ObjectSubclass<imp::Theme>);
}

impl Theme {
    pub fn new(label: &str) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().label.replace(label.to_string());
        obj
    }

    pub fn label(&self) -> String {
        self.imp().label.borrow().clone()
    }
}
