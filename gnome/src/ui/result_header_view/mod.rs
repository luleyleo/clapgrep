mod imp;

use glib::Object;
use gtk::glib;

use crate::search::SearchResult;

glib::wrapper! {
    pub struct ResultHeaderView(ObjectSubclass<imp::ResultHeaderView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ResultHeaderView {
    pub fn new(result: &SearchResult) -> Self {
        Object::builder().property("result", result).build()
    }
}
