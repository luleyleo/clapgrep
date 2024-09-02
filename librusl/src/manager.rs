use crate::{
    extended::ExtendedType,
    fileinfo::{FileInfo, Match},
    options::{Options, Sort},
    rgtools::{self, EXTENSION_SEPARATOR, SEPARATOR},
    search::Search,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

pub enum Message {
    File(FileInfo, usize),
    Done(usize, Duration, bool), //id, elapsed, stopped
    ContentFiles(Vec<FileInfo>, usize, Duration),
    StartSearch(usize),
    FileErrors(Vec<String>),
    SearchCount(usize), //number of files we went through
    Quit,
}

#[derive(Debug)]
pub enum SearchResult {
    FinalResults(FinalResults),
    InterimResult(FileInfo),
    SearchErrors(Vec<String>),
    SearchCount(usize),
}

#[derive(Debug, Clone)]
pub struct FinalResults {
    pub data: Vec<FileInfo>,
    pub duration: Duration,
    pub id: usize,
    pub stopped: bool,
}

pub struct Manager {
    internal_sender: Sender<Message>,     //send internal messages
    current_search_id: Arc<AtomicUsize>,  //we keep track of searches, and stop old searches
    total_search_count: Arc<AtomicUsize>, //we keep track of total number of files searched
    stopped: Arc<AtomicBool>,             //if we stopped the search
    options: Arc<Mutex<Options>>,
}

// Manager has an internal receiver channel to receive internal messages.
// These get processed and sent to the external sender channel.

impl Manager {
    pub fn new(external_sender: Sender<SearchResult>) -> Self {
        Self::new_with_options(external_sender, Options::default())
    }

    pub fn new_with_options(external_sender: Sender<SearchResult>, options: Options) -> Self {
        let options = Arc::new(Mutex::new(options));

        //internal channel that sends results inside
        let (internal_sender, internal_receiver) = std::sync::mpsc::channel();
        let options_for_receiver = options.clone();
        thread::spawn(move || {
            message_receiver(internal_receiver, external_sender, options_for_receiver);
        });

        Self {
            internal_sender,
            current_search_id: Arc::new(AtomicUsize::new(0)),
            options,
            total_search_count: Arc::new(AtomicUsize::new(0)),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&mut self) {
        //incrementing search id will stop any ongoing searches
        self.current_search_id.fetch_add(1, Ordering::Relaxed);
        self.stopped.store(true, Ordering::Relaxed);
    }

    pub fn search(&mut self, search: &Search) {
        self.stop();
        self.stopped.store(false, Ordering::Relaxed);
        self.spawn_search(&search);
    }

    pub fn quit(&self) {
        self.internal_sender.send(Message::Quit).expect("Quit");
    }

    pub fn dir_is_valid(&self, dir: &str) -> bool {
        PathBuf::from(dir).exists()
    }

    pub fn set_options(&mut self, ops: Options) {
        *self.options.lock().unwrap() = ops;
    }

    pub fn get_options(&self) -> Options {
        self.options.lock().unwrap().clone()
    }

    pub fn set_sort(&mut self, sort: Sort) {
        self.options.lock().unwrap().sort = sort;
    }

    fn spawn_search(&self, search: &Search) {
        let search = search.to_owned();
        self.total_search_count.store(0, Ordering::Relaxed);
        let file_sender = self.internal_sender.clone();
        let start_search_id = self.current_search_id.load(Ordering::Relaxed);
        let stopped = self.stopped.clone();

        //reset search, and send type
        let res = file_sender.send(Message::StartSearch(start_search_id));
        if let Result::Err(err) = res {
            eprintln!("Error starting search: {err}");
        }

        //send total count regularly
        let total_sender = file_sender.clone();
        let total_count = self.total_search_count.clone();
        let current_search_id0 = self.current_search_id.clone();

        thread::spawn(move || {
            let total_count = total_count.clone();
            loop {
                let _ =
                    total_sender.send(Message::SearchCount(total_count.load(Ordering::Relaxed)));
                thread::sleep(Duration::from_millis(100));
                if current_search_id0.load(Ordering::Relaxed) != start_search_id {
                    break;
                }
            }
        });

        if !search.pattern.is_empty() {
            let current_search_id2 = self.current_search_id.clone();
            let options2 = self.options.lock().unwrap().clone();
            let total_search_count2 = self.total_search_count.clone();

            thread::spawn(move || {
                let start = Instant::now();
                let files = Manager::find_contents(
                    &search.pattern,
                    &search.directory,
                    &HashSet::new(),
                    options2,
                    current_search_id2,
                    start_search_id,
                    Some(total_search_count2),
                );
                let stopped = stopped.load(Ordering::Relaxed);
                file_sender
                    .send(Message::ContentFiles(
                        files.results,
                        start_search_id,
                        start.elapsed(),
                    ))
                    .unwrap();
                file_sender.send(Message::FileErrors(files.errors)).unwrap();
                file_sender
                    .send(Message::Done(start_search_id, start.elapsed(), stopped))
                    .unwrap();
                eprintln!("Done content search");
            });
        }
    }

    fn find_contents(
        text: &str,
        dir: &str,
        allowed_files: &HashSet<String>,
        options: Options,
        global_search_id: Arc<AtomicUsize>,
        start_search_id: usize,
        total_search_count: Option<Arc<AtomicUsize>>, //only Some if find contents, else None because we would have counted the file in the name search
    ) -> ContentFileInfoResults {
        let re = regex::RegexBuilder::new(text)
            .case_insensitive(!options.case_sensitive)
            .build();

        let content_results = rgtools::search_contents(
            text,
            &[OsString::from_str(dir).unwrap()],
            allowed_files,
            options,
            global_search_id,
            start_search_id,
            total_search_count,
        );
        let strings = content_results.results;
        let errors = content_results.errors;

        let file_line_content: Vec<Vec<&str>> = strings
            .iter()
            .map(|x| x.split(&SEPARATOR).collect::<Vec<&str>>())
            .filter(|x| x.len() == 3)
            .collect();
        let mut hm: HashMap<String, FileInfo> = HashMap::new();
        for f in file_line_content.iter() {
            let (path, extended): (String, Option<ExtendedType>) =
                match f[0].split_once(EXTENSION_SEPARATOR) {
                    Some((a, b)) => (a.to_string(), Some(b.into())),
                    None => (f[0].to_string(), None),
                };

            let entry = hm.entry(path.clone()).or_insert(FileInfo {
                path: path.clone(),
                matches: vec![],
                plugin: extended,
            });
            let regex_matches = re
                .clone()
                .unwrap()
                .find_iter(f[2])
                .map(|a| a.range())
                .collect::<Vec<_>>();
            entry.matches.push(Match {
                line: f[1].parse().unwrap_or(0),
                content: f[2].to_owned(),
                ranges: regex_matches,
            });
        }
        ContentFileInfoResults {
            results: hm.into_values().collect(),
            errors,
        }
    }

    pub fn do_sort(vec: &mut [FileInfo], sort: Sort) {
        match sort {
            Sort::None => (),
            Sort::Path => vec.sort_by(|a, b| a.path.cmp(&b.path)),
            Sort::Name => unimplemented!(),
            Sort::Extension => unimplemented!(),
        };
    }
}

#[derive(Default)]
pub struct ContentFileInfoResults {
    pub results: Vec<FileInfo>,
    pub errors: Vec<String>,
}

fn message_receiver(
    internal_receiver: Receiver<Message>,
    external_sender: Sender<SearchResult>,
    ops: Arc<Mutex<Options>>,
) {
    let mut final_names = vec![];
    let mut latest_number = 0;
    let mut tot_elapsed = Duration::from_secs(0);
    loop {
        let message = internal_receiver.recv();
        if message.is_err() {
            continue;
        }
        let message = message.unwrap();
        match message {
            Message::StartSearch(id) => {
                latest_number = id;
                tot_elapsed = Duration::from_secs(0);
                final_names.clear();
            }
            Message::ContentFiles(files, number, elapsed) => {
                if number == latest_number {
                    //only update if new update (old updates are discarded)
                    for f in files {
                        final_names.push(f);
                    }
                    tot_elapsed += elapsed;
                }
            }
            Message::File(file, number) => {
                //only update if new update (old updates are discarded)
                if number == latest_number {
                    //send to output
                    final_names.push(file.clone());
                    //quietly ignore if no receiver because probably closed
                    let _ = external_sender.send(SearchResult::InterimResult(file));
                }
            }
            Message::Done(number, elapsed, stopped) => {
                if number == latest_number {
                    tot_elapsed += elapsed.to_owned();

                    let sort_type = ops.lock().unwrap().sort;
                    Manager::do_sort(&mut final_names, sort_type);
                    let results = SearchResult::FinalResults(FinalResults {
                        id: latest_number,
                        data: final_names.to_vec(),
                        duration: tot_elapsed,
                        stopped,
                    });

                    //send out to whoever is listening
                    let _ = external_sender.send(results);
                }
            }

            Message::Quit => break,
            Message::FileErrors(err) => {
                // eprintln!("Err: {err:?}");
                let _ = external_sender.send(SearchResult::SearchErrors(err));
            }
            Message::SearchCount(count) => {
                let _ = external_sender.send(SearchResult::SearchCount(count));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn find_names() {
        let file1 = add_demo_file();

        let (s, r) = channel();
        let mut man = Manager::new(s);
        let search = Search {
            directory: file1.parent().unwrap().to_string_lossy().to_string(),
            pattern: "41".to_string(),
        };
        println!("using search {search:?}");
        man.search(&search);

        //first get interim
        let mess = r.recv();
        println!("mess {mess:?}");
        if let Ok(mess) = mess {
            println!("{mess:?}");
            match mess {
                SearchResult::InterimResult(fi) => {
                    assert_eq!(fi.matches.len(), 1);
                }
                _ => panic!("should be interim"),
            }
        }

        let mess = r.recv();
        println!("mess {mess:?}");
        if let Ok(mess) = mess {
            println!("{mess:?}");
            match mess {
                SearchResult::FinalResults(fr) => {
                    assert_eq!(fr.data.len(), 1);
                }
                _ => panic!("should be final"),
            }
        }
    }

    fn add_demo_file() -> PathBuf {
        let mut dir = std::env::temp_dir();
        println!("Using Temporary directory: {}", dir.display());

        //create new directory in here, and create a file with the relevant text
        dir.push("rusltestdir");
        if dir.exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        if std::fs::create_dir_all(&dir).is_err() {
            panic!("could not create temp dir");
        }

        let mut file1 = dir.clone();
        file1.push("temp.csv");

        if std::fs::write(&file1, "hello\nthere 41 go").is_err() {
            panic!("could not create file");
        } else {
            println!("writing to {file1:?}")
        }
        file1
    }
}
