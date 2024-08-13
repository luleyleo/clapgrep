//hide windows console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use iced::{
    event, executor,
    keyboard::{key::Named, Event, Key},
    widget::scrollable,
    widget::Button,
    widget::Column,
    widget::Row,
    widget::{self, Text},
    widget::{mouse_area, Space},
    widget::{Container, TextInput},
    window::icon,
    Application, Color, Command, Element, Length, Settings, Subscription, Theme,
};

use librusl::{
    fileinfo::FileInfo,
    manager::{Manager, SearchResult},
    search::Search,
};

struct App {
    name: String,
    contents: String,
    directory: String,
    results: Vec<FileInfo>,
    manager: Manager,
    receiver: Receiver<SearchResult>,
    message: String,
    found: usize,
    searching: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    FindPressed,
    NameChanged(String),
    ContentsChanged(String),
    DirectoryChanged(String),
    OpenDirectory,
    CheckExternal,
    Event(iced::event::Event),
    CopyToClipboard(Vec<String>),
}

pub fn main() {
    let mut sets = Settings::<()> {
        default_text_size: iced::Pixels::from(18.),
        // default_font: Font::MONOSPACE,
        antialiasing: true,
        ..Default::default()
    };

    let image = image::load_from_memory_with_format(include_bytes!("icons/icon.png"), image::ImageFormat::Png)
        .unwrap()
        .into_rgba8();
    let (wid, hei) = image.dimensions();
    let icon = image.into_raw();
    sets.window.icon = Some(icon::from_rgba(icon, wid, hei).unwrap());

    App::run(sets).unwrap();
}

impl Application for App {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let (s, r) = channel();
        let man = Manager::new(s);

        let d = Self {
            name: "".to_string(),
            contents: "".to_string(),
            message: "".to_string(),
            directory: man.get_options().last_dir.clone(),
            results: vec![],
            manager: man,
            receiver: r,
            found: 0,
            searching: false,
        };
        (d, widget::focus_next())
    }

    fn title(&self) -> String {
        "rusl".into()
    }

    fn view(&self) -> Element<Self::Message> {
        let name = TextInput::new("Find file name", &self.name)
            .padding(4)
            .on_input(Message::NameChanged)
            .on_submit(Message::FindPressed);
        let contents = TextInput::new("Find contents", &self.contents)
            .on_input(Message::ContentsChanged)
            .padding(4)
            .on_submit(Message::FindPressed);
        let clipboard = if self.results.is_empty() {
            Container::new(Text::new(""))
        } else {
            Container::new(
                Button::new(Text::new("Clipboard")).on_press(Message::CopyToClipboard(self.results.iter().map(|x| x.path.clone()).collect())),
            )
        };
        let dir = TextInput::new("", &self.directory).on_input(Message::DirectoryChanged).padding(4);
        let res = Column::with_children(self.results.iter().map(|x| {
            let max = 50;
            let maxlen = 200;

            let file = Text::new(&x.path).color(Color::from_rgb8(120, 120, 255));
            let mut col = Column::new().push(file);
            for mat in x.matches.iter().take(max) {
                let content = format!("{}", FileInfo::limited_match(mat, maxlen, false));
                let num = Text::new(format!("{}:", mat.line)).width(50.).color(Color::from_rgb8(0, 200, 0));
                let details = Text::new(content).width(Length::Fill).color(Color::from_rgb8(200, 200, 200));
                let row = Row::new().push(num).push(details);
                col = col.push(row);
            }
            if x.matches.len() > max {
                col = col.push(Text::new(format!("and {} other lines", x.matches.len() - max)));
            }
            let mou = mouse_area(col)
                .on_press(Message::CopyToClipboard(vec![x.path.clone()]))
                .interaction(iced::mouse::Interaction::Grabbing);
            Row::new().spacing(10).push(mou).into()
        }));
        let res = scrollable::Scrollable::new(res);
        Column::new()
            .padding(10)
            .spacing(10)
            .push(
                Row::new()
                    .push(Text::new("File name").width(Length::Fixed(100.)))
                    .push(Space::new(iced::Length::Fixed(10.), iced::Length::Shrink))
                    .push(name),
            )
            .push(
                Row::new()
                    .push(Text::new("Contents").width(Length::Fixed(100.)))
                    .push(Space::new(iced::Length::Fixed(10.), iced::Length::Shrink))
                    .push(contents),
            )
            .push(
                Row::new()
                    .push(Text::new("Directory").width(Length::Fixed(100.)))
                    .push(Button::new(Text::new("+")).on_press(Message::OpenDirectory))
                    .push(Space::new(iced::Length::Fixed(10.), iced::Length::Shrink))
                    .push(dir),
            )
            .push(
                Row::new()
                    .spacing(15)
                    .align_items(iced::Alignment::End)
                    .push(if self.searching {
                        Button::new(Text::new("Stop")).on_press(Message::FindPressed)
                    } else {
                        Button::new(Text::new("Find")).on_press(Message::FindPressed)
                    })
                    .push(Text::new(&self.message))
                    .push(clipboard),
            )
            .push(res)
            .into()
    }
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::FindPressed => {
                if self.searching {
                    self.manager.stop();
                    self.message = format!("Found {} items. Stopped", self.found);

                    self.searching = false;
                } else {
                    self.results.clear();
                    self.searching = true;
                    self.found = 0;
                    self.message = "Searching...".to_string();
                    self.manager.search(&Search {
                        dir: self.directory.clone(),
                        name_text: self.name.clone(),
                        contents_text: self.contents.clone(),
                    })
                }
            }
            Message::NameChanged(nn) => self.name = nn,
            Message::ContentsChanged(con) => self.contents = con,
            Message::DirectoryChanged(dir) => {
                self.directory = dir.clone();
                if !self.manager.dir_is_valid(&dir) {
                    self.message = "Invalid directory".to_string();
                } else {
                    self.message = "".to_string();
                }
            }
            Message::CheckExternal => {
                while let Ok(res) = self.receiver.try_recv() {
                    match res {
                        SearchResult::FinalResults(res) => {
                            self.searching = false;
                            self.message = format!("Found {} items in {:.2}s", res.data.len(), res.duration.as_secs_f64());
                            //self.results = res.data.iter().take(1000).cloned().collect();
                            if res.data.len() > 1000 {
                                self.results.push(FileInfo {
                                    path: format!("...and {} others", res.data.len() - 1000),
                                    matches: vec![],
                                    ext: "".into(),
                                    name: "".into(),
                                    is_folder: false,
                                    plugin: None,
                                    ranges: vec![],
                                });
                            }
                        }
                        SearchResult::InterimResult(res) => {
                            if self.results.len() < 1000 {
                                self.results.push(res)
                            }
                            self.found += 1;
                            self.message = format!("Found {}, searching...", self.found);
                        }
                        SearchResult::SearchErrors(_) => {}
                        SearchResult::SearchCount(_) => {}
                    }
                }
                if let Err(std::sync::mpsc::TryRecvError::Disconnected) = self.receiver.try_recv() {
                    return Command::none();
                }
            }
            Message::OpenDirectory => {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.directory = path.to_string_lossy().to_string()
                }
            }
            Message::Event(iced::Event::Keyboard(Event::KeyPressed {
                key: Key::Named(Named::Tab),
                modifiers,
                ..
            })) => {
                return if modifiers.shift() {
                    widget::focus_previous()
                } else {
                    widget::focus_next()
                };
            }
            Message::Event(iced::Event::Window(_, iced::window::Event::CloseRequested)) => {
                self.manager.save_and_quit();
            }

            Message::Event(_) => {}
            Message::CopyToClipboard(str) => {
                self.manager.export(str);
                self.message = "Copied to clipboard".to_string();
            }
        }

        Command::none()
    }
    fn theme(&self) -> Theme {
        Theme::Dark
    }
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch(vec![
            //keep looking for external messages.
            //this is a hack and polls receiver.
            //TODO: notify gui only if necessary (once results received) - dont know if possible with ICED
            iced::time::every(Duration::from_millis(10)).map(|_| Message::CheckExternal),
            //keyboard events
            event::listen().map(Message::Event),
        ])
    }
}
