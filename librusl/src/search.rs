#[derive(Clone, Debug)]
pub struct Search {
    pub directory: String,
    pub pattern: String,
    pub glob: String,
}

impl Default for Search {
    fn default() -> Self {
        Self {
            directory: ".".to_string(),
            pattern: String::new(),
            glob: String::new(),
        }
    }
}
