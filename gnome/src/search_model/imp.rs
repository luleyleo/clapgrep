use crate::search_result::SearchResult;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::{cell::RefCell, path::PathBuf};

#[derive(Debug, Default)]
pub struct SearchModel {
    pub data: RefCell<Vec<SearchResult>>,
    pub sections: RefCell<Vec<Section>>,
    pub base_path: RefCell<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub start: u32,
    pub end: u32,
}

impl Section {
    pub fn size(&self) -> u32 {
        self.end - self.start
    }
}

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
        self.data.borrow().len() as u32
    }

    fn item(&self, position: u32) -> Option<glib::Object> {
        self.data
            .borrow()
            .get(position as usize)
            .map(|o| o.clone().upcast::<glib::Object>())
    }
}

impl SectionModelImpl for SearchModel {
    fn section(&self, position: u32) -> (u32, u32) {
        let mut total = 0;
        for section in self.sections.borrow().iter() {
            total += section.size();

            if total > position {
                return (section.start, section.end);
            }
        }

        panic!("missing section")
    }
}
