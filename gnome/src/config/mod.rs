use gtk::glib;

mod v1;
mod v2;

thread_local! {
    static INSTANCE: Config = Config::new();
}

glib::wrapper! {
    pub struct Config(ObjectSubclass<imp::Config>);
}

impl Config {
    fn new() -> Config {
        glib::Object::new()
    }
}

impl Default for Config {
    fn default() -> Self {
        INSTANCE.with(|i| i.clone())
    }
}

mod imp {
    use crate::build::APP_ID;
    use anyhow::Context;
    use glib::prelude::*;
    use gtk::{glib, subclass::prelude::*};
    use std::{cell::RefCell, path::PathBuf};

    use super::{
        v1,
        v2::{self, Config as InnerConfig},
    };

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Config)]
    pub struct Config {
        #[property(name = "version", get, type = u32, member = version)]
        #[property(name = "last-app-version", get, set, type = String, member = last_app_version)]
        // Search
        #[property(name = "search-path", get, set, type = PathBuf, member = search_path)]
        #[property(name = "max-search-results", get, set, type = u32, member = max_search_results)]
        // Window
        #[property(name = "window-width", get, set, type = i32, member = window_width)]
        #[property(name = "window-height", get, set, type = i32, member = window_height)]
        #[property(name = "window-maximized", get, set, type = bool, member = window_maximized)]
        // Flags
        #[property(name = "path-pattern-explicit", get, set, type = bool, member = flag_path_pattern_explicit)]
        #[property(name = "case-sensitive", get, set, type = bool, member = flag_case_sensitive)]
        #[property(name = "include-hidden", get, set, type = bool, member = flag_include_hidden)]
        #[property(name = "include-ignored", get, set, type = bool, member = flag_include_ignored)]
        #[property(name = "disable-regex", get, set, type = bool, member = flag_disable_regex)]
        // File Types
        #[property(name = "search-names", get, set, type = bool, member = search_names)]
        #[property(name = "search-pdf", get, set, type = bool, member = search_pdf)]
        #[property(name = "search-office", get, set, type = bool, member = search_office)]
        inner: RefCell<InnerConfig>,
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

        fn read_config() -> anyhow::Result<InnerConfig> {
            let config_path = Self::config_path();
            if !config_path.is_file() {
                log::info!("No existing config file was found");
                return Ok(InnerConfig::default());
            }

            loop {
                let config_txt =
                    std::fs::read_to_string(&config_path).context("Failed to read config file")?;

                let version = toml::from_str::<VersionOnly>(&config_txt)
                    .context("Failed to parse version of config file")?
                    .version;

                match version {
                    1 => {
                        let config_old = toml::from_str::<v1::Config>(&config_txt)
                            .context("Failed to parse v1 config file")?;

                        let config_new = v2::Config::from(config_old);
                        let config_txt = toml::to_string(&config_new).unwrap();
                        std::fs::write(&config_path, config_txt).unwrap();
                    }
                    2 => {
                        return toml::from_str::<v2::Config>(&config_txt)
                            .context("Failed to parse v2 config file");
                    }
                    _ => unreachable!(),
                }
            }
        }

        pub fn load(&self) {
            let config = match Self::read_config() {
                Ok(config) => config,
                Err(err) => {
                    log::error!("Failed to read config file: {err}");
                    InnerConfig::default()
                }
            };

            self.inner.replace(config);
        }

        pub fn save(&self) {
            let config = self.inner.borrow();
            let config_txt = toml::to_string(&*config).expect("Failed to serialize config");
            let config_path = Self::config_path();
            if let Err(err) = std::fs::write(&config_path, config_txt) {
                log::error!("Failed to write config ({config_path:?}): {err}");
            }
        }
    }

    #[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
    struct VersionOnly {
        #[serde(default = "default_config_version")]
        pub version: u32,
    }

    fn default_config_version() -> u32 {
        1
    }
}
