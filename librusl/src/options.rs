#[derive(Clone, Debug)]
pub struct Options {
    pub sort: Sort,
    pub last_dir: String,
    pub name_history: Vec<String>,
    pub content_history: Vec<String>,
    pub name: NameOptions,
    pub content: ContentOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sort: Sort::None,
            last_dir: ".".to_string(),
            name_history: vec![],
            content_history: vec![],
            name: Default::default(),
            content: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NameOptions {
    pub case_sensitive: bool,
    pub file_types: FTypes,
    pub same_filesystem: bool,
    pub follow_links: bool,
    pub ignore_dot: bool,
    pub use_gitignore: bool,
}

impl Default for NameOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            file_types: FTypes::All,
            same_filesystem: false,
            follow_links: false,
            ignore_dot: true,
            use_gitignore: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ContentOptions {
    pub case_sensitive: bool,
    pub extended: bool,
    pub nonregex: bool, //--fixed-string
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Sort {
    #[default]
    None,
    Path,
    Name,
    Extension,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum FTypes {
    Files,
    Directories,
    #[default]
    All,
}
