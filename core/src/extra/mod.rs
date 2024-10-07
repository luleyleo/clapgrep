use crate::search::SearchSink;
use grep::{regex::RegexMatcher, searcher::Searcher};
use std::{error::Error, path::Path};

pub mod office;
pub mod pdf;

pub type ExtraFn =
    fn(&mut Searcher, &RegexMatcher, &Path, &mut SearchSink) -> Result<(), Box<dyn Error>>;
