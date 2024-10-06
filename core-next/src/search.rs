use crate::{
    result::{Location, SearchError},
    utils, ResultEntry, SearchEngine, SearchMessage, SearchResult,
};
use grep::{
    matcher::Match,
    regex::{RegexMatcher, RegexMatcherBuilder},
    searcher::SearcherBuilder,
};
use ignore::{WalkBuilder, WalkState};
use std::{
    io,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Debug, Clone)]
pub struct SearchParameter {
    pub base_directory: PathBuf,
    pub pattern: String,
    pub flags: SearchFlags,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SearchFlags {
    pub case_sensitive: bool,
    pub fixed_string: bool,
    pub same_filesystem: bool,
    pub follow_links: bool,
    pub search_hidden: bool,
    pub search_ignored: bool,
}

pub type SharedSearchId = Arc<AtomicUsize>;
pub type SearchId = usize;

/// Blocking content search
pub fn run(engine: SearchEngine, params: SearchParameter) {
    let search = engine.current_search_id.load(Ordering::Acquire);

    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!params.flags.case_sensitive)
        .fixed_strings(params.flags.fixed_string)
        .build(&params.pattern);

    if let Err(err) = matcher {
        _ = engine.sender.send(SearchMessage::Error(SearchError {
            search,
            path: params.base_directory,
            message: format!("Failed to start search: {}", err),
        }));
        _ = engine.sender.send(SearchMessage::Completed { search });
        return;
    }
    let matcher = matcher.unwrap();

    let walker = WalkBuilder::new(&params.base_directory)
        .git_ignore(!params.flags.search_ignored)
        .ignore(!params.flags.search_ignored)
        .hidden(params.flags.search_hidden)
        .follow_links(params.flags.follow_links)
        .same_file_system(params.flags.same_filesystem)
        .build_parallel();

    walker.run(move || {
        let engine = engine.clone();

        let matcher = matcher.clone();

        let mut searcher = SearcherBuilder::new()
            .binary_detection(grep::searcher::BinaryDetection::quit(b'\x01'))
            .line_number(true)
            .build();

        let mut sink = SearchSink::new(matcher.clone());

        Box::new(move |entry: Result<ignore::DirEntry, ignore::Error>| {
            if engine.current_search_id.load(Ordering::Relaxed) != search {
                return WalkState::Quit;
            }

            if entry.is_err() {
                return WalkState::Continue;
            }
            let entry = entry.unwrap();

            if !entry.file_type().unwrap().is_file() {
                return WalkState::Continue;
            }

            if let Err(err) = searcher.search_path(&matcher, entry.path(), &mut sink) {
                _ = engine.send_error(
                    search,
                    entry.path().to_path_buf(),
                    format!("failed to search file: {}", err),
                );
                return WalkState::Continue;
            }

            let result = SearchResult {
                search,
                path: entry.path().to_path_buf(),
                entries: sink.take_entries(),
            };

            if engine.sender.send(SearchMessage::Result(result)).is_err() {
                return WalkState::Quit;
            }

            WalkState::Continue
        })
    });
}

struct SearchSink {
    matcher: RegexMatcher,
    entries: Vec<ResultEntry>,
}

impl SearchSink {
    pub fn new(matcher: RegexMatcher) -> Self {
        SearchSink {
            matcher,
            entries: Vec::new(),
        }
    }

    pub fn take_entries(&mut self) -> Vec<ResultEntry> {
        std::mem::take(&mut self.entries)
    }

    fn extract_matches(
        &self,
        searcher: &grep::searcher::Searcher,
        bytes: &[u8],
        range: std::ops::Range<usize>,
    ) -> io::Result<Vec<Match>> {
        let mut matches = Vec::new();

        utils::find_iter_at_in_context(searcher, &self.matcher, bytes, range.clone(), |m| {
            let (s, e) = (m.start() - range.start, m.end() - range.start);
            matches.push(Match::new(s, e));
            true
        })?;

        Ok(matches)
    }
}

impl grep::searcher::Sink for SearchSink {
    type Error = io::Error;

    fn matched(
        &mut self,
        searcher: &grep::searcher::Searcher,
        mat: &grep::searcher::SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        let matches = self.extract_matches(searcher, mat.buffer(), mat.bytes_range_in_buffer())?;
        let content = String::from_utf8_lossy(mat.bytes()).to_string();
        let line = mat.line_number().unwrap();

        self.entries.push(ResultEntry {
            location: Location::Text { line },
            content,
            matches,
        });

        Ok(true)
    }
}
