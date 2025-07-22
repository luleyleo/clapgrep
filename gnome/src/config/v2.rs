use crate::build::APP_VERSION;
use gtk::glib;
use std::path::PathBuf;

use super::v1;

impl Config {
    pub fn version() -> u32 {
        2
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub last_app_version: String,
    pub search_path: PathBuf,

    pub window_width: i32,
    pub window_height: i32,
    pub window_maximized: bool,

    pub flag_path_pattern_explicit: bool,
    pub flag_case_sensitive: bool,
    pub flag_include_hidden: bool,
    pub flag_include_ignored: bool,
    pub flag_disable_regex: bool,

    pub search_names: bool,
    pub search_pdf: bool,
    pub search_office: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: Self::version(),
            last_app_version: APP_VERSION.to_string(),
            search_path: glib::home_dir(),

            window_width: 1600,
            window_height: 900,
            window_maximized: false,

            flag_path_pattern_explicit: false,
            flag_case_sensitive: false,
            flag_include_hidden: false,
            flag_include_ignored: false,
            flag_disable_regex: false,

            search_names: true,
            search_pdf: true,
            search_office: true,
        }
    }
}

impl From<v1::Config> for Config {
    fn from(old: v1::Config) -> Self {
        Config {
            version: Self::version(),
            last_app_version: old.last_version,
            search_path: old.search_path,

            window_width: old.window.width,
            window_height: old.window.height,
            window_maximized: old.window.maximized,

            flag_path_pattern_explicit: old.flags.path_pattern_explicit,
            flag_case_sensitive: old.flags.case_sensitive,
            flag_include_hidden: old.flags.include_hidden,
            flag_include_ignored: old.flags.include_ignored,
            flag_disable_regex: old.flags.disable_regex,

            search_names: old.search.names,
            search_pdf: old.search.pdf,
            search_office: old.search.office,
        }
    }
}
