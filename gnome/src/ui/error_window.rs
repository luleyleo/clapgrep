use adw::subclass::prelude::*;
use glib::{subclass::InitializingObject, Object};
use gtk::{gio, glib, prelude::*, CompositeTemplate};
use std::cell::RefCell;

use crate::ui::SearchWindow;

glib::wrapper! {
    pub struct ErrorWindow(ObjectSubclass<ErrorWindowImp>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ErrorWindow {
    pub fn new(parent: &SearchWindow) -> Self {
        Object::builder()
            .property("application", parent.application().unwrap())
            .property("modal", true)
            .property("transient-for", parent)
            .property("search_window", parent)
            .build()
    }
}

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/ui/error_window.blp")]
#[properties(wrapper_type = ErrorWindow)]
pub struct ErrorWindowImp {
    #[property(get, set)]
    pub search_window: RefCell<Option<SearchWindow>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ErrorWindowImp {
    const NAME: &'static str = "ClapgrepErrorWindow";
    type Type = ErrorWindow;
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
impl ErrorWindowImp {}

#[glib::derived_properties]
impl ObjectImpl for ErrorWindowImp {}

impl WidgetImpl for ErrorWindowImp {}

impl WindowImpl for ErrorWindowImp {}

impl AdwWindowImpl for ErrorWindowImp {}
