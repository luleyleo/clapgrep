use std::cell::RefCell;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::search_result::SearchResult;

#[derive(Debug, Default)]
pub struct SearchModel(pub(super) RefCell<Vec<SearchResult>>);

#[glib::object_subclass]
impl ObjectSubclass for SearchModel {
    const NAME: &'static str = "ClapgrepSearchModel";
    type Type = super::SearchModel;
    type Interfaces = (gio::ListModel, gtk::SectionModel);
}

impl ObjectImpl for SearchModel {}

impl ListModelImpl for SearchModel {
    fn item_type(&self) -> glib::Type {
        SearchResult::static_type()
    }
    fn n_items(&self) -> u32 {
        self.0.borrow().len() as u32
    }
    fn item(&self, position: u32) -> Option<glib::Object> {
        self.0
            .borrow()
            .get(position as usize)
            .map(|o| o.clone().upcast::<glib::Object>())
    }
}

impl SectionModelImpl for SearchModel {
    fn section(&self, position: u32) -> (u32, u32) {
        let pivot = self.item(position).unwrap();
        let pivot = pivot.downcast::<SearchResult>().unwrap();

        let len = self.0.borrow().len();

        let start = self
            .0
            .borrow()
            .iter()
            .position(|sr| sr.file() == pivot.file())
            .unwrap();

        let end = len
            - self
                .0
                .borrow()
                .iter()
                .rev()
                .position(|sr| sr.file() == pivot.file())
                .unwrap();

        (start as u32, end as u32)
    }
}
