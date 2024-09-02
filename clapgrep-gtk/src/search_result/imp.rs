use std::cell::{Cell, RefCell};

use gtk::{
    gio,
    glib::{self, prelude::*},
    subclass::prelude::*,
};

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = super::SearchResult)]
pub struct SearchResult {
    #[property(get, set)]
    file: RefCell<String>,
    #[property(get, set)]
    line: Cell<u64>,
    #[property(get, set)]
    content: RefCell<String>,
    #[property(get, set, construct)]
    matches: RefCell<Option<gio::ListStore>>,
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for SearchResult {
    const NAME: &'static str = "ClapgrepSearchResult";
    type Type = super::SearchResult;
}

#[glib::derived_properties]
impl ObjectImpl for SearchResult {}
