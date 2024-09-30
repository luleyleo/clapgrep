use cosmic_config::{ConfigGet, ConfigSet};
use glib::prelude::*;
use gtk::{glib, subclass::prelude::*};
use std::cell::{Cell, OnceCell};

use crate::APP_ID;

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = super::Config)]
pub struct Config {
    #[property(get, set = Self::set_window_width, default = 1600)]
    window_width: Cell<i32>,
    #[property(get, set = Self::set_window_height, default = 900)]
    window_height: Cell<i32>,
    #[property(get, set = Self::set_window_maximized, default = false)]
    window_maximized: Cell<bool>,

    config: OnceCell<cosmic_config::Config>,
}

impl Config {
    fn set_window_width(&self, width: i32) {
        let config = self.config.get().unwrap();
        let _ = config.set("window_width", width);
        self.window_width.set(width);
    }

    fn set_window_height(&self, height: i32) {
        let config = self.config.get().unwrap();
        let _ = config.set("window_height", height);
        self.window_height.set(height);
    }

    fn set_window_maximized(&self, maximized: bool) {
        let config = self.config.get().unwrap();
        let _ = config.set("window_maximized", maximized);
        self.window_maximized.set(maximized);
    }
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for Config {
    const NAME: &'static str = "ClapgrepConfig";
    type Type = super::Config;
}

#[glib::derived_properties]
impl ObjectImpl for Config {
    fn constructed(&self) {
        let config = cosmic_config::Config::new(APP_ID, 1).expect("failed to open config");

        if let Ok(window_width) = config.get("window_width") {
            self.window_width.set(window_width);
        }

        if let Ok(window_height) = config.get("window_height") {
            self.window_height.set(window_height);
        }

        if let Ok(window_maximized) = config.get("window_maximized") {
            self.window_maximized.set(window_maximized);
        }

        let _ = self.config.set(config);
    }
}
