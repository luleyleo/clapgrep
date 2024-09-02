// SPDX-License-Identifier: {{LICENSE}}

use crate::{config::Config, fl};
use cosmic::{
    app::{Command, Core},
    cosmic_config::{self, CosmicConfigEntry},
    cosmic_theme,
    iced::{self, keyboard, Alignment, Length, Padding, Subscription},
    theme,
    widget::{self, menu},
    Application, ApplicationExt, Apply, Element,
};
use librusl::{
    fileinfo::FileInfo,
    manager::{Manager, SearchResult},
    search::Search,
};
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

const REPOSITORY: &str = "https://github.com/luleyleo/clapgrep";
const APP_ICON: &[u8] =
    include_bytes!("../res/icons/hicolor/scalable/apps/de.leopoldluley.Clapgrep.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: Config,

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

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    FindPressed,
    NameChanged(String),
    ContentsChanged(String),
    DirectoryChanged(String),
    OpenDirectory,
    CheckExternal,
    Event(cosmic::iced::event::Event),
    CopyToClipboard(Vec<String>),

    OpenRepositoryUrl,
    UpdateConfig(Config),
}

/// Create a COSMIC application from the app model
impl Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "de.leopoldluley.Clapgrep";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (s, r) = channel();
        let man = Manager::new(s);

        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => {
                    // for why in errors {
                    //     tracing::error!(%why, "error loading app config");
                    // }

                    config
                }
            })
            .unwrap_or_default();

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            key_binds: HashMap::new(),
            config,

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

        // Create a startup command that sets the window title.
        let command = app.set_window_title("Clapgrep".to_string());

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let name = widget::text_input("Find file name", &self.name)
            .padding(4)
            .on_input(Message::NameChanged)
            .on_submit(Message::FindPressed);
        let contents = widget::text_input("Find contents", &self.contents)
            .on_input(Message::ContentsChanged)
            .padding(4)
            .on_submit(Message::FindPressed);
        let dir = widget::text_input("", &self.directory)
            .on_input(Message::DirectoryChanged)
            .padding(4);

        let clipboard = if self.results.is_empty() {
            widget::Container::new(widget::Text::new(""))
        } else {
            widget::Container::new(widget::button(widget::Text::new("Clipboard")).on_press(
                Message::CopyToClipboard(self.results.iter().map(|x| x.path.clone()).collect()),
            ))
        };

        let results =
            widget::Column::with_children(self.results.iter().map(|r| self.result_view(r)))
                .apply(widget::scrollable);

        let controls = widget::column()
            .padding(10)
            .spacing(10)
            .push(
                widget::Row::new()
                    .push(widget::Text::new("File name").width(Length::Fixed(100.)))
                    .push(widget::Space::new(
                        iced::Length::Fixed(10.),
                        iced::Length::Shrink,
                    ))
                    .push(name),
            )
            .push(
                widget::Row::new()
                    .push(widget::Text::new("Contents").width(Length::Fixed(100.)))
                    .push(widget::Space::new(
                        iced::Length::Fixed(10.),
                        iced::Length::Shrink,
                    ))
                    .push(contents),
            )
            .push(
                widget::Row::new()
                    .push(widget::Text::new("Directory").width(Length::Fixed(100.)))
                    .push(widget::Space::new(
                        iced::Length::Fixed(10.),
                        iced::Length::Shrink,
                    ))
                    .push(dir)
                    .push(widget::Space::new(
                        iced::Length::Fixed(10.),
                        iced::Length::Shrink,
                    ))
                    .push(
                        widget::button(widget::Text::new("open")).on_press(Message::OpenDirectory),
                    ),
            )
            .push(
                widget::Row::new()
                    .spacing(15)
                    .align_items(iced::Alignment::End)
                    .push(if self.searching {
                        widget::button(widget::Text::new("Stop")).on_press(Message::FindPressed)
                    } else {
                        widget::button(widget::Text::new("Find")).on_press(Message::FindPressed)
                    })
                    .push(widget::Text::new(&self.message))
                    .push(clipboard),
            )
            .apply(widget::container)
            .style(cosmic::style::Container::Card);

        widget::Column::new()
            .padding(10)
            .spacing(10)
            .push(controls.height(Length::Shrink))
            .push(results.height(Length::Fill))
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            //keep looking for external messages.
            //this is a hack and polls receiver.
            //TODO: notify gui only if necessary (once results received) - dont know if possible with ICED
            iced::time::every(Duration::from_millis(100)).map(|_| Message::CheckExternal),
            //keyboard events
            iced::event::listen().map(Message::Event),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Commands may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

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
                        directory: self.directory.clone(),
                        glob: self.name.clone(),
                        pattern: self.contents.clone(),
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
                            self.message = format!(
                                "Found {} items in {:.2}s",
                                res.data.len(),
                                res.duration.as_secs_f64()
                            );
                            let number_of_results = res.data.len();

                            self.results = res.data;
                            self.results.truncate(1000);

                            if number_of_results > 1000 {
                                self.results.push(FileInfo {
                                    path: format!("...and {} others", number_of_results - 1000),
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
                // if let Some(path) = rfd::FileDialog::new().pick_folder() {
                //     self.directory = path.to_string_lossy().to_string()
                // }
            }
            Message::Event(iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Tab),
                modifiers,
                ..
            })) => {
                return if modifiers.shift() {
                    iced::widget::focus_previous()
                } else {
                    iced::widget::focus_next()
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
}

impl AppModel {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = widget::text::title3(fl!("app-title"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn result_view<'s>(&'s self, result: &'s FileInfo) -> Element<'s, Message> {
        let max = 50;
        let maxlen = 200;

        let file = widget::Text::new(&result.path).style(iced::Color::from_rgb8(120, 120, 255));

        let mut col = widget::column().padding(5).spacing(3).push(file);

        let max_num = result
            .matches
            .iter()
            .take(max)
            .map(|mat| mat.line)
            .max()
            .unwrap_or_default()
            .to_string()
            .len();

        for mat in result.matches.iter().take(max) {
            let num = widget::text::monotext(format!("{:width$}: ", mat.line, width = max_num));

            let content = format!("{}", FileInfo::limited_match(mat, maxlen, false));
            let details = widget::text::monotext(content).width(Length::Fill);

            col = col.push(
                widget::row()
                    .padding(Padding::from([0, 5]))
                    .push(num)
                    .push(details),
            );
        }

        if result.matches.len() > max {
            col = col.push(widget::Text::new(format!(
                "and {} other lines",
                result.matches.len() - max
            )));
        }

        let mou = iced::widget::MouseArea::new(col)
            // .interaction(iced::mouse::Interaction::Grabbing)
            .on_press(Message::CopyToClipboard(vec![result.path.clone()]));

        widget::Row::new().spacing(10).push(mou).into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::OpenRepositoryUrl,
        }
    }
}
