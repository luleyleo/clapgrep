use glib::prelude::*;
use gtk::{glib, subclass::prelude::*};
use std::cell::Cell;

glib::wrapper! {
    pub struct SearchMatch(ObjectSubclass<SearchMatchImp>);
}

impl SearchMatch {
    pub fn new(start: u32, end: u32) -> SearchMatch {
        glib::Object::builder()
            .property("start", start)
            .property("end", end)
            .build()
    }
}

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = SearchMatch)]
pub struct SearchMatchImp {
    #[property(get, set)]
    start: Cell<u32>,
    #[property(get, set)]
    end: Cell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchMatchImp {
    const NAME: &'static str = "ClapgrepSearchMatch";
    type Type = SearchMatch;
}

#[glib::derived_properties]
impl ObjectImpl for SearchMatchImp {}
