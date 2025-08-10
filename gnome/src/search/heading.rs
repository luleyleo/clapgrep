use crate::search::SearchMatch;
use clapgrep_core::Match;
use gtk::{
    gio,
    glib::{self, prelude::*},
    subclass::prelude::*,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

glib::wrapper! {
    pub struct SearchHeading(ObjectSubclass<SearchHeadingImp>);
}

impl Default for SearchHeading {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchHeading {
    pub fn new(search_path: &Path, file_path: &Path, file_name_matches: &[Match]) -> SearchHeading {
        let file_path = file_path
            .strip_prefix(&search_path)
            .expect("failed to strip file_path prefix");

        let file_name_matches_store = (!file_name_matches.is_empty()).then(|| {
            let file_name_offset = file_path
                .to_string_lossy()
                .find(file_path.file_name().unwrap().to_string_lossy().as_ref())
                .unwrap();

            let file_name_matches_store = gio::ListStore::new::<SearchMatch>();
            for m in file_name_matches {
                let m = m.offset(file_name_offset);
                let sm = SearchMatch::new(m.start() as u32, m.end() as u32);
                file_name_matches_store.append(&sm);
            }
            file_name_matches_store
        });

        glib::Object::builder()
            .property("search_path", search_path)
            .property("file_path", file_path)
            .property("file_name_matches", file_name_matches_store)
            .build()
    }

    pub fn absolute_path(&self) -> PathBuf {
        let search_path = self.search_path();
        let file_path = self.file_path();

        search_path.join(file_path)
    }
}

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = SearchHeading)]
pub struct SearchHeadingImp {
    #[property(get, set)]
    search_path: RefCell<PathBuf>,
    #[property(get, set)]
    file_path: RefCell<PathBuf>,
    #[property(get, set, nullable)]
    file_name_matches: RefCell<Option<gio::ListStore>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchHeadingImp {
    const NAME: &'static str = "ClapgrepSearchHeading";
    type Type = SearchHeading;
}

#[glib::derived_properties]
impl ObjectImpl for SearchHeadingImp {}
