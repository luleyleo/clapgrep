use adw::subclass::prelude::*;
use glib::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, CompositeTemplate};
use std::cell::RefCell;

use crate::search_window::SearchWindow;

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/error_window/error_window.blp")]
#[properties(wrapper_type = super::ErrorWindow)]
pub struct ErrorWindow {
    #[property(get, set)]
    pub search_window: RefCell<Option<SearchWindow>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ErrorWindow {
    const NAME: &'static str = "ClapgrepErrorWindow";
    type Type = super::ErrorWindow;
    type ParentType = adw::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl ErrorWindow {
}

#[glib::derived_properties]
impl ObjectImpl for ErrorWindow {}

impl WidgetImpl for ErrorWindow {}

impl WindowImpl for ErrorWindow {}

impl AdwWindowImpl for ErrorWindow {}
