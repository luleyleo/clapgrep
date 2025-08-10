use crate::search::SearchMatch;
use clapgrep_core::Match;
use gtk::{
    gio,
    glib::{self, prelude::*},
    subclass::prelude::*,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};

use super::heading::SearchHeading;

glib::wrapper! {
    pub struct SearchResult(ObjectSubclass<SearchResultImp>);
}

impl Default for SearchResult {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchResult {
    pub fn new(
        heading: SearchHeading,
        line: u64,
        page: u64,
        content: &str,
        content_matches: &[Match],
    ) -> SearchResult {
        let content = if content.contains('\0') {
            println!("Found <NULL> in '{content}'");
            content.replace('\0', "<NULL>")
        } else {
            content.to_string()
        };

        let content_matches_store = gio::ListStore::new::<SearchMatch>();
        for m in content_matches {
            let sm = SearchMatch::new(m.start() as u32, m.end() as u32);
            content_matches_store.append(&sm);
        }

        glib::Object::builder()
            .property("heading", heading)
            .property("line", line)
            .property("page", page)
            .property("content", content)
            .property("content_matches", content_matches_store)
            .build()
    }

    pub fn matched_strings(&self) -> HashSet<String> {
        let matches = self.content_matches();
        let content = self.content();

        if let Some(matches) = matches {
            let mut matched_strings = HashSet::new();

            for m in matches.into_iter() {
                if let Ok(m) = m {
                    let search_match = m.downcast::<SearchMatch>().unwrap();
                    let (start, end) = (search_match.start() as usize, search_match.end() as usize);
                    let matched_string = content[start..end].to_string();
                    matched_strings.insert(matched_string);
                } else {
                    break;
                }
            }

            return matched_strings;
        }

        Default::default()
    }
}

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = SearchResult)]
pub struct SearchResultImp {
    #[property(get, set)]
    heading: RefCell<SearchHeading>,
    #[property(get, set)]
    line: Cell<u64>,
    #[property(get, set)]
    page: Cell<u64>,
    #[property(get, set)]
    content: RefCell<String>,
    #[property(get, set, construct)]
    content_matches: RefCell<Option<gio::ListStore>>,
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for SearchResultImp {
    const NAME: &'static str = "ClapgrepSearchResult";
    type Type = SearchResult;
}

#[glib::derived_properties]
impl ObjectImpl for SearchResultImp {}
