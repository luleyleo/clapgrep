mod imp;

use glib::Object;
use gtk::glib;

use crate::search::SearchResult;

glib::wrapper! {
    pub struct ResultView(ObjectSubclass<imp::ResultView>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ResultView {
    pub fn new(result: &SearchResult) -> Self {
        Object::builder().property("result", result).build()
    }
}
