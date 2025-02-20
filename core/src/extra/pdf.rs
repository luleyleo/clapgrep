use crate::search::SearchSink;
use grep::{regex::RegexMatcher, searcher::Searcher};
use poppler::Document;
use std::{error::Error, path::Path};

pub static EXTENSIONS: &[&str] = &["pdf"];

pub fn process(
    searcher: &mut Searcher,
    matcher: &RegexMatcher,
    path: &Path,
    sink: &mut SearchSink,
) -> Result<(), Box<dyn Error>> {
    let doc = Document::from_gfile(&gio::File::for_path(path), None, gio::Cancellable::NONE)?;
    for i in 0..doc.n_pages() {
        let page = doc.page(i).expect("out of range");
        if let Some(text) = page.text() {
            sink.page = Some(i as u64 + 1);
            searcher.search_slice(matcher, text.as_bytes(), &mut *sink)?
        }
    }
    Ok(())
}
