use std::path::PathBuf;

use grep::matcher::Match;

use crate::search::SearchId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Location {
    Text { line: u64 },
    Document { page: u64, line: u64 },
}

pub struct ResultEntry {
    pub location: Location,
    pub content: String,
    pub matches: Vec<Match>,
}

pub struct SearchResult {
    pub search: SearchId,
    pub path: PathBuf,
    pub path_matches: Vec<Match>,
    pub entries: Vec<ResultEntry>,
}

pub struct SearchError {
    pub search: SearchId,
    pub path: PathBuf,
    pub message: String,
}

pub enum SearchMessage {
    Result(SearchResult),
    Error(SearchError),
    Completed { search: SearchId },
}

impl SearchMessage {
    pub fn search(&self) -> SearchId {
        *match self {
            Self::Result(SearchResult { search, .. }) => search,
            Self::Error(SearchError { search, .. }) => search,
            Self::Completed { search, .. } => search,
        }
    }
}
