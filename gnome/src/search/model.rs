use crate::search::SearchResult;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::{cell::RefCell, path::PathBuf};

glib::wrapper! {
    pub struct SearchModel(ObjectSubclass<SearchModelImp>)
        @implements gio::ListModel, gtk::SectionModel;
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
        imp.sections.borrow_mut().clear();
        self.items_changed(0, len as u32, 0)
    }

    fn append_file_info_impl(&self, file_info: clapgrep_core::SearchResult) -> Section {
        let base_path = self.imp().base_path.borrow();

        let mut data = self.imp().data.borrow_mut();
        let start = data.len() as u32;
        if file_info.entries.is_empty() {
            debug_assert!(
                !file_info.path_matches.is_empty(),
                "file_info.path_matches must not be empty"
            );

            data.push(SearchResult::new(
                base_path.clone(),
                file_info.path.clone(),
                &file_info.path_matches,
                0,
                0,
                "",
                &[],
            ));
        } else {
            let search_results = file_info.entries.into_iter().enumerate().map(|(i, m)| {
                let (line, page) = match m.location {
                    clapgrep_core::Location::Text { line } => (line, 0),
                    clapgrep_core::Location::Document { page, line } => (line, page),
                };

                SearchResult::new(
                    base_path.clone(),
                    file_info.path.clone(),
                    if i == 0 { &file_info.path_matches } else { &[] },
                    line,
                    page,
                    &m.content,
                    &m.matches,
                )
            });
            data.extend(search_results);
        }
        let end = data.len() as u32;

        let section = Section { start, end };
        self.imp().sections.borrow_mut().push(section);

        section
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
impl ObjectSubclass for SearchModelImp {
    const NAME: &'static str = "ClapgrepSearchModel";
    type Type = SearchModel;
    type Interfaces = (gio::ListModel, gtk::SectionModel);
}

impl ObjectImpl for SearchModelImp {}

impl ListModelImpl for SearchModelImp {
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

impl SectionModelImpl for SearchModelImp {
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
