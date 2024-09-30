mod imp;

use gtk::glib;

glib::wrapper! {
    pub struct Config(ObjectSubclass<imp::Config>);
}

impl Config {
    pub fn new() -> Config {
        glib::Object::new()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
