mod imp;

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
