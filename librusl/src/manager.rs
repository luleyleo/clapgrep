use crate::{
    extended::ExtendedType,
    fileinfo::{FileInfo, Match},
    options::{FTypes, Options, Sort},
    rgtools::{self, EXTENSION_SEPARATOR, SEPARATOR},
    search::Search,
};
use clipboard::{ClipboardContext, ClipboardProvider};
use ignore::WalkBuilder;
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
        let ops = load_options();
        let ops = Arc::new(Mutex::new(ops));

        //internal channel that sends results inside
        let (s, r) = std::sync::mpsc::channel();
        let ops_for_receiver = ops.clone();
        thread::spawn(move || {
            message_receiver(r, external_sender, ops_for_receiver);
        });
        Self {
            internal_sender: s,
            current_search_id: Arc::new(AtomicUsize::new(0)),
            options: ops,
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

        let mut ops = self.options.lock().unwrap();
        ops.last_dir = search.dir.clone();
        if !search.name_text.is_empty() && !ops.name_history.contains(&search.name_text) {
            ops.name_history.push(search.name_text.clone());
        }
        if !search.contents_text.is_empty() && !ops.content_history.contains(&search.contents_text)
        {
            ops.content_history.push(search.contents_text.clone());
        }
        drop(ops);

        self.spawn_search(&search);
    }

    pub fn save_and_quit(&self) {
        save_settings(&self.options.lock().unwrap());
        self.internal_sender.send(Message::Quit).expect("Quit");
    }

    pub fn save(&self) {
        save_settings(&self.options.lock().unwrap());
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

    pub fn export(&self, paths: Vec<String>) {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

        let r = ctx.set_contents(paths.join("\n"));
        if let Err(err) = r {
            eprintln!("Clip error: {}", err);
        }
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

        //do name search
        let current_search_id1 = self.current_search_id.clone();
        let search1 = search.clone();
        let file_sender1 = file_sender.clone();
        let options1 = self.options.lock().unwrap().clone();
        let total_search_count1 = self.total_search_count.clone();

        if !search.name_text.is_empty() {
            let counter_search_id = current_search_id1.clone();
            thread::spawn(move || {
                let start = Instant::now();
                Manager::find_names(
                    &search1,
                    options1,
                    file_sender1.clone(),
                    current_search_id1,
                    start_search_id,
                    total_search_count1,
                );
                let stopped = stopped.load(Ordering::Relaxed);
                if let Err(err) =
                    file_sender1.send(Message::Done(start_search_id, start.elapsed(), stopped))
                {
                    eprintln!("Manager: Could not send result {start_search_id} {err:?}:{err}");
                }
                counter_search_id.fetch_add(1, Ordering::Relaxed); //stop counting
            });
        }
        //do content search (only if name is empty, otherwise it will be spawned after)
        else if !search.contents_text.is_empty() && search.name_text.is_empty() {
            let current_search_id2 = self.current_search_id.clone();
            let options2 = self.options.lock().unwrap().clone();
            let total_search_count2 = self.total_search_count.clone();

            thread::spawn(move || {
                let start = Instant::now();
                let files = Manager::find_contents(
                    &search.contents_text,
                    &search.dir,
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

    fn find_names(
        search: &Search,
        options: Options,
        file_sender: Sender<Message>,
        global_search_id: Arc<AtomicUsize>, // current global id
        start_search_id: usize,             //id when starting this search
        total_search_count: Arc<AtomicUsize>,
    ) {
        let text = &search.name_text;
        let dir = &search.dir;
        let ftype = options.name.file_types;
        let sens = options.name.case_sensitive;
        let re = regex::RegexBuilder::new(text)
            .case_insensitive(!sens)
            .build();
        if re.is_err() {
            return;
        }
        let re = re.unwrap();
        let re = Arc::new(re);

        let walker = WalkBuilder::new(dir)
            .follow_links(options.name.follow_links)
            .same_file_system(options.name.same_filesystem)
            .threads(num_cpus::get())
            .hidden(options.name.ignore_dot)
            .git_ignore(options.name.use_gitignore)
            .build_parallel();

        //walk dir
        walker.run(|| {
            let file_sender = file_sender.clone();
            let re = re.clone();
            let global_search_id = global_search_id.clone();
            let options = options.clone();
            let total_search_count = total_search_count.clone();
            Box::new(move |result| {
                if global_search_id.load(Ordering::Relaxed) != start_search_id {
                    return ignore::WalkState::Quit;
                }
                //dont include root directory name itself
                if let Ok(dent) = &result {
                    if dent.depth() == 0 {
                        return ignore::WalkState::Continue;
                    }
                }

                total_search_count.fetch_add(1, Ordering::Relaxed);
                let dent = match result {
                    Ok(dent) => dent,
                    Err(err) => {
                        let _ = file_sender.send(Message::FileErrors(vec![err.to_string()]));
                        return ignore::WalkState::Continue;
                    }
                };

                let fs_type = dent.file_type();
                if fs_type.is_none() {
                    return ignore::WalkState::Continue;
                }
                let fs_type = fs_type.unwrap();

                //skip files if we dont want them
                match ftype {
                    FTypes::Files => {
                        if !fs_type.is_file() {
                            return ignore::WalkState::Continue;
                        }
                    }
                    FTypes::Directories => {
                        if !fs_type.is_dir() {
                            return ignore::WalkState::Continue;
                        }
                    }
                    _ => (),
                }
                let is_match = re
                    .clone()
                    .is_match(dent.file_name().to_str().unwrap_or_default());

                if is_match {
                    let mut must_add = true;
                    let mut matches = vec![];
                    if !search.contents_text.is_empty() {
                        if fs_type.is_dir() {
                            must_add = false;
                        } else {
                            //check if contents match
                            let cont = Manager::find_contents(
                                &search.contents_text,
                                dir,
                                &HashSet::from_iter([dent.path().to_string_lossy().to_string()]),
                                options.clone(),
                                global_search_id.clone(),
                                start_search_id,
                                None,
                            );
                            if cont.results.is_empty() {
                                must_add = false;
                            } else {
                                matches = cont.results[0].matches.clone();
                            }

                            if !cont.errors.is_empty() {
                                file_sender.send(Message::FileErrors(cont.errors)).unwrap();
                            }
                        }
                    }
                    if global_search_id.load(Ordering::Relaxed) != start_search_id {
                        eprintln!("New search started, stopping current search");
                        return ignore::WalkState::Quit;
                    }
                    if must_add {
                        //find all matches on name
                        let regex_matches = re
                            .find_iter(dent.file_name().to_str().unwrap_or_default())
                            .map(|a| a.range())
                            .collect::<Vec<_>>();

                        let res = file_sender.send(Message::File(
                            FileInfo {
                                path: dent.path().to_string_lossy().to_string(),
                                name: dent.file_name().to_string_lossy().to_string(),
                                ext: PathBuf::from(dent.path())
                                    .extension()
                                    .unwrap_or(&OsString::from(""))
                                    .to_str()
                                    .unwrap_or_default()
                                    .into(),
                                matches,
                                is_folder: dent.file_type().unwrap().is_dir(),
                                plugin: None,
                                ranges: regex_matches,
                            },
                            start_search_id,
                        ));
                        //receiver closed, so we quit
                        if res.is_err() {
                            eprintln!("receiver closed, stopping search");
                            return ignore::WalkState::Quit;
                        }
                    }
                }

                ignore::WalkState::Continue
            })
        });
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
            .case_insensitive(!options.content.case_sensitive)
            .build();

        let content_results = rgtools::search_contents(
            text,
            &[OsString::from_str(dir).unwrap()],
            allowed_files,
            options.content,
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
            let pb = PathBuf::from(&path);

            let entry = hm.entry(path.clone()).or_insert(FileInfo {
                path: path.clone(),
                matches: vec![],
                ext: pb
                    .extension()
                    .unwrap_or(&OsString::from(""))
                    .to_str()
                    .unwrap_or_default()
                    .into(),
                name: PathBuf::from(f[0])
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .into(),
                is_folder: pb.is_dir(),
                plugin: extended,
                ranges: vec![],
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
            Sort::Name => vec.sort_by(|a, b| a.name.cmp(&b.name)),
            Sort::Extension => vec.sort_by(|a, b| a.ext.cmp(&b.ext)),
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

fn get_or_create_settings_path() -> Option<String> {
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("rusl");
        if dir.exists() {
            dir.push("config.toml");
            if dir.exists() {
                if let Some(file) = dir.to_str() {
                    return Some(file.to_string());
                }
            }
        }
    }
    //try create
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("rusl");

        if std::fs::create_dir_all(&dir).is_ok() {
            dir.push("config.toml");
            if std::fs::File::create(&dir).is_ok() {
                if let Some(file) = dir.to_str() {
                    return Some(file.to_string());
                }
            }
        }
    }
    //else not able to
    None
}

fn load_options() -> Options {
    let mut ops = Options::default();

    if let Some(file) = get_or_create_settings_path() {
        if let Ok(data) = std::fs::read_to_string(file) {
            if let Ok(new_ops) = toml::from_str(&data) {
                ops = new_ops;
            } else {
                eprintln!("Could not load settings");
            }
        } else {
            eprintln!("Could read settings");
        }
    } else {
        eprintln!("Could not access settings file");
    }

    ops
}

fn save_settings(ops: &Options) {
    let mut save_ops = ops.clone();
    //only save last few history
    let count = 20;
    save_ops.name_history = save_ops
        .name_history
        .into_iter()
        .rev()
        .take(count)
        .rev()
        .collect();
    save_ops.content_history = save_ops
        .content_history
        .into_iter()
        .rev()
        .take(count)
        .rev()
        .collect();

    if let Some(file) = get_or_create_settings_path() {
        let result = toml::to_string_pretty(&save_ops);
        if let Ok(data) = result {
            let write_result = std::fs::write(file, data);
            if write_result.is_err() {
                eprintln!("Could not write settings {write_result:?}");
            }
        } else {
            eprintln!("Could not convert {result:?}");
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
            dir: file1.parent().unwrap().to_string_lossy().to_string(),
            name_text: file1.file_name().unwrap().to_string_lossy().to_string(),
            contents_text: "41".to_string(),
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
