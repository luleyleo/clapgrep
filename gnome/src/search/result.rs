use crate::search::SearchMatch;
use clapgrep_core::Match;
use gtk::{gio, glib, prelude::*};
use std::{collections::HashSet, path::PathBuf};

glib::wrapper! {
    pub struct SearchResult(ObjectSubclass<imp::SearchResult>);
}

impl Default for SearchResult {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl SearchResult {
    pub fn new(
        search_path: PathBuf,
        file_path: PathBuf,
        file_name_matches: &[Match],
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
            .property("line", line)
            .property("page", page)
            .property("content", content)
            .property("content_matches", content_matches_store)
            .build()
    }

    pub fn absolute_path(&self) -> PathBuf {
        let search_path = self.search_path();
        let file_path = self.file_path();

        search_path.join(file_path)
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

mod imp {
    use gtk::{
        gio,
        glib::{self, prelude::*},
        subclass::prelude::*,
    };
    use std::{
        cell::{Cell, RefCell},
        path::PathBuf,
    };

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::SearchResult)]
    pub struct SearchResult {
        // search relate properties
        #[property(get, set)]
        search_path: RefCell<PathBuf>,

        // file related properties
        #[property(get, set)]
        file_path: RefCell<PathBuf>,
        #[property(get, set, nullable)]
        file_name_matches: RefCell<Option<gio::ListStore>>,

        // entry related propertie
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
    impl ObjectSubclass for SearchResult {
        const NAME: &'static str = "ClapgrepSearchResult";
        type Type = super::SearchResult;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SearchResult {}
}
