use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Search {
    pub directory: PathBuf,
    pub pattern: String,
}

impl Default for Search {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("."),
            pattern: String::new(),
        }
    }
}
