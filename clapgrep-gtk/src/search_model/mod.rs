mod imp;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use librusl::fileinfo::FileInfo;

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

    pub fn clear(&self) {
        let imp = self.imp();
        let len = imp.0.borrow().len();
        imp.0.borrow_mut().clear();
        self.items_changed(0, len as u32, 0)
    }

    pub fn append_file_info(&self, file_info: &FileInfo) {
        let search_results = file_info
            .matches
            .iter()
            .map(|m| SearchResult::new(&file_info.path, m.line, &m.content, &m.ranges));

        let mut data = self.imp().0.borrow_mut();
        let start = data.len() as u32;
        data.extend(search_results);
        let end = data.len() as u32;
        drop(data);

        self.items_changed(start, 0, end - start);
    }
}

impl Default for SearchModel {
    fn default() -> Self {
        Self::new()
    }
}
