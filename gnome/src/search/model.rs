use crate::search::SearchResult;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::{cell::RefCell, path::PathBuf};

use super::SearchHeading;

glib::wrapper! {
    pub struct SearchModel(ObjectSubclass<SearchModelImp>)
        @implements gio::ListModel;
}

impl Default for SearchModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchModel {
    pub fn new() -> SearchModel {
        glib::Object::new()
    }

    pub fn set_base_path(&self, path: impl Into<PathBuf>) {
        *self.imp().base_path.borrow_mut() = path.into();
    }

    pub fn clear(&self) {
        let imp = self.imp();
        let len = imp.data.borrow().len();
        imp.data.borrow_mut().clear();
        self.items_changed(0, len as u32, 0)
    }

    fn append_file_info_impl(&self, file_info: clapgrep_core::SearchResult) -> Section {
        let base_path = self.imp().base_path.borrow();

        let mut data = self.imp().data.borrow_mut();

        let heading = SearchHeading::new(
            base_path.clone(),
            file_info.path.clone(),
            &file_info.path_matches,
        );

        let start = data.len() as u32;
        data.push(heading.clone().upcast::<glib::Object>());
        let search_results = file_info.entries.into_iter().map(|m| {
            let (line, page) = match m.location {
                clapgrep_core::Location::Text { line } => (line, 0),
                clapgrep_core::Location::Document { page, line } => (line, page),
            };

            SearchResult::new(heading.clone(), line, page, &m.content, &m.matches)
                .upcast::<glib::Object>()
        });
        data.extend(search_results);
        let end = data.len() as u32;

        Section { start, end }
    }

    pub fn append_file_info(&self, file_info: clapgrep_core::SearchResult) {
        let Section { start, end } = self.append_file_info_impl(file_info);
        self.items_changed(start, 0, end - start);
    }

    pub fn extend_with_results(&self, results: impl Iterator<Item = clapgrep_core::SearchResult>) {
        let start = self.imp().data.borrow().len() as u32;
        for file_info in results {
            if !file_info.entries.is_empty() || !file_info.path_matches.is_empty() {
                self.append_file_info_impl(file_info);
            }
        }
        let end = self.imp().data.borrow().len() as u32;

        self.items_changed(start, 0, end - start);
    }
}

#[derive(Debug, Default)]
pub struct SearchModelImp {
    pub base_path: RefCell<PathBuf>,
    pub data: RefCell<Vec<glib::Object>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub start: u32,
    pub end: u32,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchModelImp {
    const NAME: &'static str = "ClapgrepSearchModel";
    type Type = SearchModel;
    type Interfaces = (gio::ListModel,);
}

impl ObjectImpl for SearchModelImp {}

impl ListModelImpl for SearchModelImp {
    fn item_type(&self) -> glib::Type {
        glib::Object::static_type()
    }

    fn n_items(&self) -> u32 {
        self.data.borrow().len() as u32
    }

    fn item(&self, position: u32) -> Option<glib::Object> {
        self.data.borrow().get(position as usize).cloned()
    }
}
