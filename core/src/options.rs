use crate::extended::ExtendedType;

#[derive(Clone, Debug)]
pub struct Options {
    pub sort: Sort,
    pub case_sensitive: bool,
    pub same_filesystem: bool,
    pub follow_links: bool,
    pub ignore_dot: bool,
    pub use_gitignore: bool,
    pub extended: Vec<ExtendedType>,
    pub fixed_string: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Sort {
    #[default]
    None,
    Path,
    Name,
    Extension,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sort: Sort::default(),
            case_sensitive: false,
            same_filesystem: false,
            follow_links: false,
            ignore_dot: true,
            use_gitignore: true,
            extended: Vec::new(),
            fixed_string: false,
        }
    }
}
