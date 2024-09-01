use std::cell::Cell;

use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, CompositeTemplate};

#[derive(CompositeTemplate, Default)]
#[template(file = "src/window/window.blp")]
pub struct Window {
    pub number: Cell<i32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "ClapgrepWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl Window {}

impl ObjectImpl for Window {}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}

impl AdwApplicationWindowImpl for Window {}
