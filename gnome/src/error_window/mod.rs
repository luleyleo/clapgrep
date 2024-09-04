mod imp;

use glib::Object;
use gtk::{gio, glib, prelude::*};

use crate::window::Window;

glib::wrapper! {
    pub struct ErrorWindow(ObjectSubclass<imp::ErrorWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ErrorWindow {
    pub fn new(parent: &Window) -> Self {
        Object::builder()
            .property("application", parent.application().unwrap())
            .property("modal", true)
            .property("transient-for", parent)
            .property("search_window", parent)
            .build()
    }
}
