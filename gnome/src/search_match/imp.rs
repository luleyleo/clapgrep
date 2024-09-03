use std::cell::Cell;

use glib::prelude::*;
use gtk::{glib, subclass::prelude::*};

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = super::SearchMatch)]
pub struct SearchMatch {
    #[property(get, set)]
    start: Cell<u32>,
    #[property(get, set)]
    end: Cell<u32>,
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for SearchMatch {
    const NAME: &'static str = "ClapgrepSearchMatch";
    type Type = super::SearchMatch;
}

#[glib::derived_properties]
impl ObjectImpl for SearchMatch {}
