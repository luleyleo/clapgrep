use crate::{
    extra,
    result::{Location, SearchError},
    utils, ResultEntry, SearchEngine, SearchMessage, SearchResult,
};
use grep::{
    matcher::{Match, Matcher},
    regex::{RegexMatcher, RegexMatcherBuilder},
    searcher::SearcherBuilder,
};
use ignore::{WalkBuilder, WalkState};
use std::{
    error::Error,
    io,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[derive(Debug, Clone)]
pub struct SearchParameters {
    pub base_directory: PathBuf,
    pub content_pattern: String,
    pub path_pattern: String,
    pub flags: SearchFlags,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SearchFlags {
    pub case_sensitive: bool,
    pub fixed_string: bool,

    pub search_names: bool,
    pub search_pdf: bool,
    pub search_office: bool,

    pub search_hidden: bool,
    pub search_ignored: bool,

    pub same_filesystem: bool,
    pub follow_links: bool,
}

pub type SharedSearchId = Arc<AtomicUsize>;
pub type SearchId = usize;

/// Blocking content search
pub fn run(engine: SearchEngine, params: SearchParameters) {
    let search = engine.current_search_id.load(Ordering::Acquire);

    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!params.flags.case_sensitive)
        .fixed_strings(params.flags.fixed_string)
        .build(&params.content_pattern);

    if let Err(err) = matcher {
        _ = engine.sender.send(SearchMessage::Error(SearchError {
            search,
            path: params.base_directory,
            message: format!("Failed to start search: {err}"),
        }));
        _ = engine.sender.send(SearchMessage::Completed { search });
        return;
    }
    let matcher = matcher.unwrap();

    let threads = match std::thread::available_parallelism() {
        Ok(cores) => cores.get(),
        Err(_) => 2,
    };

    let pattern = if !params.path_pattern.is_empty() {
        // Validity of path patterns should be checked externally.
        glob::Pattern::new(&params.path_pattern).ok()
    } else {
        None
    };

    let base_directory = params.base_directory;
    let walker = WalkBuilder::new(&base_directory)
        .git_ignore(!params.flags.search_ignored)
        .ignore(!params.flags.search_ignored)
        .hidden(params.flags.search_hidden)
        .follow_links(params.flags.follow_links)
        .same_file_system(params.flags.same_filesystem)
        .threads(threads)
        .filter_entry(move |dir| match (dir.path().is_file(), pattern.as_ref()) {
            (true, Some(pattern)) => {
                let relative_dir = dir.path().strip_prefix(&base_directory).unwrap();
                pattern.matches_path(relative_dir)
            }
            _ => true,
        })
        .build_parallel();

    let mut preprocessors: Vec<(_, extra::ExtraFn)> = Vec::new();
    if params.flags.search_pdf {
        preprocessors.push((extra::pdf::EXTENSIONS, extra::pdf::process));
    }
    if params.flags.search_office {
        preprocessors.push((extra::office::EXTENSIONS, extra::office::process));
    }

    walker.run(|| {
        let engine = engine.clone();
        let matcher = matcher.clone();
        let preprocessors = preprocessors.clone();
        let mut sink = SearchSink::new(matcher.clone());
        let mut searcher = SearcherBuilder::new()
            .binary_detection(grep::searcher::BinaryDetection::quit(b'\x01'))
            .line_number(true)
            .build();

        Box::new(move |entry: Result<ignore::DirEntry, ignore::Error>| {
            if engine.current_search_id.load(Ordering::Relaxed) != search {
                return WalkState::Quit;
            }

            if entry.is_err() {
                return WalkState::Continue;
            }
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();

            if file_type.is_symlink() {
                return WalkState::Continue;
            }

            let mut path_matches = Vec::new();
            if params.flags.search_names {
                let file_name = entry.file_name();
                matcher
                    .find_iter(file_name.as_bytes(), |m| {
                        path_matches.push(m);
                        true
                    })
                    .expect("RegexMatcher should never throw an error");
            }

            if !file_type.is_file() {
                return WalkState::Continue;
            }

            let extension = entry
                .path()
                .extension()
                .unwrap_or_default()
                .to_string_lossy();

            let pre_processor = preprocessors
                .iter()
                .find(|(exts, _)| exts.contains(&extension.as_ref()))
                .map(|(_, extract_fn)| extract_fn);

            let search_result = match pre_processor {
                Some(process) => process(&mut searcher, &matcher, entry.path(), &mut sink),
                None => searcher.search_path(&matcher, entry.path(), &mut sink),
            };

            if let Err(err) = search_result {
                _ = engine.send_error(
                    search,
                    entry.path().to_path_buf(),
                    format!("failed to search file: {err}"),
                );
                return WalkState::Continue;
            }

            let result = SearchResult {
                search,
                path: entry.path().to_path_buf(),
                path_matches,
                entries: sink.take_entries(),
            };

            if engine.sender.send(SearchMessage::Result(result)).is_err() {
                return WalkState::Quit;
            }

            WalkState::Continue
        })
    });

    _ = engine.sender.send(SearchMessage::Completed { search });
}

pub struct SearchSink {
    pub page: Option<u64>,
    matcher: RegexMatcher,
    entries: Vec<ResultEntry>,
}

impl SearchSink {
    pub fn new(matcher: RegexMatcher) -> Self {
        SearchSink {
            page: None,
            matcher,
            entries: Vec::new(),
        }
    }

    pub fn take_entries(&mut self) -> Vec<ResultEntry> {
        self.page = None;
        std::mem::take(&mut self.entries)
    }

    pub fn extract_matches(
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
    type Error = Box<dyn Error>;

    fn matched(
        &mut self,
        searcher: &grep::searcher::Searcher,
        mat: &grep::searcher::SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        let matches = self.extract_matches(searcher, mat.buffer(), mat.bytes_range_in_buffer())?;
        let content = String::from_utf8_lossy(mat.bytes())
            .trim_ascii_end()
            .to_string();

        let line = mat.line_number().unwrap();
        let location = match self.page {
            None => Location::Text { line },
            Some(page) => Location::Document { page, line },
        };

        self.entries.push(ResultEntry {
            location,
            content,
            matches,
        });

        Ok(true)
    }
}
