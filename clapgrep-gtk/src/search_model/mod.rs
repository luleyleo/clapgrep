mod imp;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::search_result::SearchResult;

glib::wrapper! {
    pub struct SearchModel(ObjectSubclass<imp::SearchModel>)
        @implements gio::ListModel, gtk::SectionModel;
}

impl SearchModel {
    pub fn new() -> SearchModel {
        glib::Object::new()
    }

    pub fn append(&self, obj: &SearchResult) {
        let imp = self.imp();
        let index = {
            // Borrow the data only once and ensure the borrow guard is dropped
            // before we emit the items_changed signal because the view
            // could call get_item / get_n_item from the signal handler to update its state
            let mut data = imp.0.borrow_mut();
            data.push(obj.clone());
            data.len() - 1
        };
        self.items_changed(index as u32, 0, 1);
    }

    pub fn remove(&self, index: u32) {
        let imp = self.imp();
        imp.0.borrow_mut().remove(index as usize);
        self.items_changed(index, 1, 0);
    }
}

impl Default for SearchModel {
    fn default() -> Self {
        Self::new()
    }
}
