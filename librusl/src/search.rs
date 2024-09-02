#[derive(Clone, Debug)]
pub struct Search {
    pub directory: String,
    pub pattern: String,
}

impl Default for Search {
    fn default() -> Self {
        Self {
            directory: ".".to_string(),
            pattern: String::new(),
        }
    }
}
