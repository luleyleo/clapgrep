use crate::build::APP_VERSION;
use gtk::glib;
use std::path::PathBuf;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FullConfig {
    #[serde(default = "default_config_version")]
    pub version: u32,
    #[serde(default = "default_app_version")]
    pub last_version: String,
    pub search_path: PathBuf,

    pub window: WindowConfig,
    pub flags: SearchFlags,
    pub search: SearchConfig,
}

fn default_app_version() -> String {
    "24.03".to_string()
}

fn default_config_version() -> u32 {
    1
}

impl Default for FullConfig {
    fn default() -> Self {
        Self {
            version: 1,
            last_version: APP_VERSION.to_string(),
            search_path: glib::home_dir(),
            window: Default::default(),
            flags: Default::default(),
            search: Default::default(),
        }
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
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
pub struct SearchFlags {
    pub path_pattern_explicit: bool,
    pub case_sensitive: bool,
    pub include_hidden: bool,
    pub include_ignored: bool,
    pub disable_regex: bool,
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SearchConfig {
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
