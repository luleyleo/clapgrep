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

mod imp {
    use glib::prelude::*;
    use gtk::{glib, subclass::prelude::*};
    use std::{cell::RefCell, path::PathBuf};

    use crate::APP_ID;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Config)]
    pub struct Config {
        #[property(name = "window-width", get, set, type = i32, member = width)]
        #[property(name = "window-height", get, set, type = i32, member = height)]
        #[property(name = "window-maximized", get, set, type = bool, member = maximized)]
        window: RefCell<WindowConfig>,

        #[property(name = "search-pdf", get, set, type = bool, member = pdf)]
        #[property(name = "search-office", get, set, type = bool, member = office)]
        search: RefCell<SearchConfig>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Config {
        const NAME: &'static str = "ClapgrepConfig";
        type Type = super::Config;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Config {
        fn constructed(&self) {
            self.load();

            self.obj().connect_notify(None, |obj, _| {
                obj.imp().save();
            });
        }
    }

    impl Config {
        fn config_path() -> PathBuf {
            let config_dir = glib::user_config_dir().join(APP_ID);
            std::fs::create_dir_all(&config_dir).unwrap();
            config_dir.join("config.toml")
        }

        pub fn load(&self) {
            let config_path = Self::config_path();
            let config = std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|str| toml::from_str::<FullConfig>(&str).ok())
                .unwrap_or_default();

            self.window.replace(config.window);
            self.search.replace(config.search);
        }

        pub fn save(&self) {
            let config = FullConfig {
                window: *self.window.borrow(),
                search: *self.search.borrow(),
            };
            let config_txt = toml::to_string(&config).unwrap();
            let config_path = Self::config_path();
            std::fs::write(config_path, config_txt).unwrap();
        }
    }

    #[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
    struct FullConfig {
        window: WindowConfig,
        search: SearchConfig,
    }

    #[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
    struct WindowConfig {
        width: i32,
        height: i32,
        maximized: bool,
    }
    impl Default for WindowConfig {
        fn default() -> Self {
            WindowConfig {
                width: 1600,
                height: 900,
                maximized: false,
            }
        }
    }

    #[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
    struct SearchConfig {
        pdf: bool,
        office: bool,
    }
    impl Default for SearchConfig {
        fn default() -> Self {
            SearchConfig {
                pdf: true,
                office: true,
            }
        }
    }
}
