use crate::extended::ExtendedType;
use std::{borrow::Cow, path::PathBuf};

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub matches: Vec<Match>,
    pub plugin: Option<ExtendedType>,
}

impl FileInfo {
    pub fn content(&self, max_count: usize, max_length: usize) -> String {
        self.matches
            .iter()
            .take(max_count)
            .map(|x| FileInfo::limited_match(x, max_length, true))
            .collect::<Vec<String>>()
            .join("\n")
    }
    pub fn limited_match(x: &Match, max_length: usize, line_number: bool) -> String {
        //limit content line length
        let fixed = match x.content.char_indices().nth(max_length) {
            None => Cow::from(&x.content),
            Some((idx, _)) => Cow::from(format!("{}...", &x.content[..idx])),
        };
        let num = if line_number {
            format!("{}: ", x.line)
        } else {
            String::new()
        };
        format!("{}{}", num, fixed.trim_end())
    }
}

#[derive(Clone, Debug)]
pub struct Match {
    pub line: usize,
    pub content: String,
    pub ranges: Vec<std::ops::Range<usize>>,
}
