use adw::subclass::prelude::*;
use glib::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, CompositeTemplate};
use std::cell::RefCell;

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/window/window.blp")]
#[properties(wrapper_type = super::Window)]
pub struct Window {
    #[property(get, set)]
    pub file_search: RefCell<String>,
    #[property(get, set)]
    pub content_search: RefCell<String>,
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
impl Window {
    #[template_callback]
    fn on_search_button_clicked(&self, _: &gtk::Button) {
        println!("file_search = {}", self.file_search.borrow());
        println!("content_search = {}", self.content_search.borrow());
    }
}

#[glib::derived_properties]
impl ObjectImpl for Window {}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}

impl AdwApplicationWindowImpl for Window {}
