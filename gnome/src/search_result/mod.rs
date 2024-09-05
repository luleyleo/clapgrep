mod imp;

use std::path::{Path, PathBuf};

use gtk::{
    gio, glib, pango,
    prelude::{Cast, ListModelExt},
};

use crate::search_match::SearchMatch;

glib::wrapper! {
    pub struct SearchResult(ObjectSubclass<imp::SearchResult>);
}

impl SearchResult {
    pub fn new(
        file: impl Into<PathBuf>,
        absolute_file: impl AsRef<Path>,
        line: usize,
        content: &str,
        matches: &[std::ops::Range<usize>],
    ) -> SearchResult {
        let file = file.into();

        let matches_store = gio::ListStore::new::<SearchMatch>();
        for m in matches {
            let sm = SearchMatch::new(m.start as u32, m.end as u32);
            matches_store.append(&sm);
        }

        let uri = format!("file://{}", absolute_file.as_ref().to_string_lossy());

        glib::Object::builder()
            .property("file", file)
            .property("uri", uri)
            .property("line", line as u64)
            .property("content", content)
            .property("matches", matches_store)
            .build()
    }

    pub fn get_highlights(&self) -> Vec<pango::AttrColor> {
        let matches = self.matches();

        if let Some(matches) = matches {
            let mut attrs = Vec::with_capacity(matches.n_items() as usize);

            for m in matches.into_iter() {
                if let Ok(m) = m {
                    let search_match = m.downcast::<SearchMatch>().unwrap();
                    let mut attr = pango::AttrColor::new_foreground(255, 0, 0);
                    attr.set_start_index(search_match.start());
                    attr.set_end_index(search_match.end());
                    attrs.push(attr);
                } else {
                    break;
                }
            }

            return attrs;
        }

        Vec::new()
    }
}
