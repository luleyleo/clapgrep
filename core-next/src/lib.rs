mod engine;
mod result;
mod search;
mod utils;

pub use engine::SearchEngine;
pub use result::{Location, ResultEntry, SearchMessage, SearchResult};
pub use search::{SearchFlags, SearchParameters};

pub use grep::matcher::Match;
