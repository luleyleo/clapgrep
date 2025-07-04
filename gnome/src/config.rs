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
    use crate::build::{APP_ID, APP_VERSION};
    use anyhow::Context;
    use glib::prelude::*;
    use gtk::{glib, subclass::prelude::*};
    use std::{cell::RefCell, path::PathBuf};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Config)]
    pub struct Config {
        #[property(get, set)]
        last_version: RefCell<String>,
        #[property(get, set)]
        search_path: RefCell<PathBuf>,

        #[property(name = "window-width", get, set, type = i32, member = width)]
        #[property(name = "window-height", get, set, type = i32, member = height)]
        #[property(name = "window-maximized", get, set, type = bool, member = maximized)]
        window: RefCell<WindowConfig>,

        #[property(name = "case-sensitive", get, set, type = bool, member = case_sensitive)]
        #[property(name = "include-hidden", get, set, type = bool, member = include_hidden)]
        #[property(name = "include-ignored", get, set, type = bool, member = include_ignored)]
        #[property(name = "disable-regex", get, set, type = bool, member = disable_regex)]
        flags: RefCell<SearchFlags>,

        #[property(name = "search-names", get, set, type = bool, member = names)]
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

        fn read_config() -> anyhow::Result<FullConfig> {
            let config_path = Self::config_path();
            if config_path.is_file() {
                let config_txt =
                    std::fs::read_to_string(&config_path).context("Failed to read config file")?;
                let config = toml::from_str::<FullConfig>(&config_txt)
                    .context("Failed to parse config file")?;

                Ok(config)
            } else {
                log::info!("No existing config file was found");
                Ok(FullConfig::default())
            }
        }

        pub fn load(&self) {
            let config = match Self::read_config() {
                Ok(config) => config,
                Err(err) => {
                    log::error!("Failed to read config file: {err}");
                    FullConfig::default()
                }
            };

            self.last_version.replace(config.last_version);
            self.search_path.replace(config.search_path);
            self.window.replace(config.window);
            self.flags.replace(config.flags);
            self.search.replace(config.search);
        }

        pub fn save(&self) {
            let config = FullConfig {
                last_version: self.last_version.borrow().clone(),
                search_path: self.search_path.borrow().clone(),
                window: *self.window.borrow(),
                flags: *self.flags.borrow(),
                search: *self.search.borrow(),
            };
            let config_txt = toml::to_string(&config).expect("Failed to serialize config");
            let config_path = Self::config_path();
            if let Err(err) = std::fs::write(&config_path, config_txt) {
                log::error!("Failed to write config ({config_path:?}): {err}");
            }
        }
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
    struct FullConfig {
        #[serde(default = "old_last_version")]
        last_version: String,
        search_path: PathBuf,

        window: WindowConfig,
        flags: SearchFlags,
        search: SearchConfig,
    }

    fn old_last_version() -> String {
        "24.03".to_string()
    }

    fn home_directory() -> PathBuf {
        glib::home_dir()
    }

    impl Default for FullConfig {
        fn default() -> Self {
            Self {
                last_version: APP_VERSION.to_string(),
                search_path: home_directory(),
                window: Default::default(),
                flags: Default::default(),
                search: Default::default(),
            }
        }
    }

    #[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
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

    #[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
    struct SearchFlags {
        pub case_sensitive: bool,
        pub include_hidden: bool,
        pub include_ignored: bool,
        pub disable_regex: bool,
    }

    #[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
    struct SearchConfig {
        names: bool,
        pdf: bool,
        office: bool,
    }
    impl Default for SearchConfig {
        fn default() -> Self {
            SearchConfig {
                names: true,
                pdf: true,
                office: true,
            }
        }
    }
}
