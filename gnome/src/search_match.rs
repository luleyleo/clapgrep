use gtk::glib;

glib::wrapper! {
    pub struct SearchMatch(ObjectSubclass<imp::SearchMatch>);
}

impl SearchMatch {
    pub fn new(start: u32, end: u32) -> SearchMatch {
        glib::Object::builder()
            .property("start", start)
            .property("end", end)
            .build()
    }
}

mod imp {
    use glib::prelude::*;
    use gtk::{glib, subclass::prelude::*};
    use std::cell::Cell;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::SearchMatch)]
    pub struct SearchMatch {
        #[property(get, set)]
        start: Cell<u32>,
        #[property(get, set)]
        end: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchMatch {
        const NAME: &'static str = "ClapgrepSearchMatch";
        type Type = super::SearchMatch;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SearchMatch {}
}
