use crate::{
    config::Config,
    i18n::gettext_f,
    search::{SearchModel, SearchResult},
    ui::{preview::Preview, ErrorWindow, ResultHeaderView, ResultView},
    APP_ID,
};
use adw::{prelude::PreferencesGroupExt, subclass::prelude::*};
use clapgrep_core::{SearchEngine, SearchFlags, SearchMessage, SearchParameters};
use gettextrs::gettext;
use glib::subclass::InitializingObject;
use gtk::{
    gio::{self, Cancellable},
    glib::{self, clone},
    prelude::*,
    CompositeTemplate, FileDialog, StringList,
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/ui/search_window/search_window.blp")]
#[properties(wrapper_type = super::SearchWindow)]
pub struct SearchWindow {
    #[property(get, set)]
    pub search_path: RefCell<PathBuf>,
    #[property(get)]
    pub search_directory: RefCell<String>,
    #[property(get, set)]
    pub content_search: RefCell<String>,
    #[property(get)]
    pub results: SearchModel,

    #[property(get, set)]
    pub case_sensitive: Cell<bool>,
    #[property(get, set)]
    pub include_hidden: Cell<bool>,
    #[property(get, set)]
    pub include_ignored: Cell<bool>,
    #[property(get, set)]
    pub disable_regex: Cell<bool>,

    #[property(get, set)]
    pub search_names: Cell<bool>,
    #[property(get, set)]
    pub search_pdf: Cell<bool>,
    #[property(get, set)]
    pub search_office: Cell<bool>,

    #[property(get, set)]
    pub search_running: Cell<bool>,
    #[property(get, set)]
    pub searched_files: Cell<u32>,
    #[property(get)]
    pub number_of_matches: Cell<u32>,

    #[property(get, set)]
    pub search_progress_visible: Cell<bool>,
    #[property(get, set)]
    pub search_progress_notification: RefCell<String>,
    #[property(get, set)]
    pub search_progress_action: RefCell<String>,

    #[property(get)]
    pub errors: StringList,
    #[property(get)]
    pub number_of_errors: Cell<u32>,
    #[property(get)]
    pub has_errors: Cell<bool>,
    #[property(get)]
    pub search_errors_notification: RefCell<String>,

    #[template_child]
    pub update_banner: TemplateChild<adw::PreferencesGroup>,
    #[template_child]
    pub progress_banner: TemplateChild<adw::Banner>,
    #[template_child]
    pub results_stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub no_search_page: TemplateChild<gtk::StackPage>,
    #[template_child]
    pub no_results_page: TemplateChild<gtk::StackPage>,
    #[template_child]
    pub results_page: TemplateChild<gtk::StackPage>,
    #[template_child]
    pub split_view: TemplateChild<adw::NavigationSplitView>,
    #[template_child]
    pub inner_split_view: TemplateChild<adw::NavigationSplitView>,
    #[template_child]
    pub preview_navigation_page: TemplateChild<adw::NavigationPage>,
    #[template_child]
    pub preview: TemplateChild<Preview>,

    pub engine: SearchEngine,
    pub config: Config,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchWindow {
    const NAME: &'static str = "ClapgrepSearchWindow";
    type Type = super::SearchWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        ResultView::static_type();
        ResultHeaderView::static_type();

        klass.bind_template();
        klass.bind_template_callbacks();

        klass.install_action("win.start-search", None, |win, _, _| {
            win.imp().start_search();
        });
        klass.install_action("win.stop-search", None, |win, _, _| {
            win.imp().stop_search();
        });
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl SearchWindow {
    #[template_callback]
    fn on_search(&self, _: &adw::ActionRow) {
        self.start_search();
    }

    #[template_callback]
    fn on_entry_activated(&self, _: &adw::EntryRow) {
        self.start_search();
    }

    #[template_callback]
    fn on_search_progress_action(&self, _: &adw::Banner) {
        if self.search_running.get() {
            self.stop_search();
        } else {
            self.obj().set_search_progress_visible(false);
        }
    }

    #[template_callback]
    fn on_cd(&self, _: &gtk::Button) {
        let obj = self.obj();
        let initial_folder = gio::File::for_path(self.search_path.borrow().as_path());

        FileDialog::builder()
            .title(gettext("Choose Search Path"))
            .initial_folder(&initial_folder)
            .modal(true)
            .build()
            .select_folder(
                Some(self.obj().as_ref()),
                Cancellable::NONE,
                clone!(
                    #[weak]
                    obj,
                    move |result| {
                        if let Ok(result) = result {
                            obj.set_search_path(result.path().unwrap());
                        }
                    }
                ),
            );
    }

    #[template_callback]
    fn on_show_errors(&self, _: &adw::Banner) {
        let error_window = ErrorWindow::new(&self.obj());
        error_window.present();
    }

    #[template_callback]
    fn on_result_activated(&self, position: u32) {
        if let Some(result) = self.results.item(position) {
            let result = result.downcast::<SearchResult>().unwrap();
            if !result.content().is_empty() {
                self.preview.set_result(&result);
                self.preview_navigation_page.set_visible(true);
                self.inner_split_view.set_show_content(true);
            }
        }
    }

    #[template_callback]
    fn on_hide_update_banner(&self) {
        self.update_banner.set_visible(false);
    }
}

impl SearchWindow {
    fn init_manager(&self) {
        let app = self.obj().clone();
        let model = self.results.clone();
        glib::MainContext::default().spawn_local(async move {
            let imp = app.imp();
            let receiver = imp.engine.receiver();

            // Prevents GTK from getting overloaded with too many updates on the GUI thread.
            const BUFFER_SIZE: usize = 1024;
            const BUFFER_DURATION: Duration = Duration::from_millis(100);

            let mut buffer = Vec::with_capacity(BUFFER_SIZE);
            let mut last_buffer_update = Instant::now();

            while let Ok(result) = receiver.recv_async().await {
                if imp.engine.is_current(&result) {
                    match result {
                        SearchMessage::Result(result) => {
                            buffer.push(result);

                            if buffer.len() >= BUFFER_SIZE
                                || last_buffer_update.elapsed() > BUFFER_DURATION
                            {
                                model.extend_with_results(&buffer);
                                app.set_searched_files(app.searched_files() + buffer.len() as u32);
                                buffer.clear();
                            }

                            last_buffer_update = Instant::now();
                        }
                        SearchMessage::Error(error) => {
                            app.errors().append(&format!(
                                "{}: {}",
                                error.path.display(),
                                error.message
                            ));
                        }
                        SearchMessage::Completed { .. } => {
                            if !buffer.is_empty() {
                                model.extend_with_results(&buffer);
                                app.set_searched_files(app.searched_files() + buffer.len() as u32);
                                buffer.clear();
                            }

                            app.set_search_running(false);
                        }
                    }
                }
            }
        });
    }

    fn start_search(&self) {
        self.results.clear();

        if self.content_search.borrow().is_empty() {
            return;
        }

        let search = SearchParameters {
            base_directory: self.search_path.borrow().clone(),
            pattern: self.content_search.borrow().to_string(),
            flags: SearchFlags {
                case_sensitive: self.case_sensitive.get(),
                fixed_string: self.disable_regex.get(),

                search_names: self.search_names.get(),
                search_pdf: self.search_pdf.get(),
                search_office: self.search_office.get(),

                search_hidden: self.include_hidden.get(),
                search_ignored: self.include_ignored.get(),

                same_filesystem: false,
                follow_links: true,
            },
        };

        self.results.clear();
        self.results
            .set_base_path(self.search_path.borrow().clone());
        self.errors.splice(0, self.errors.n_items(), &[]);
        self.obj().set_searched_files(0);
        self.obj().set_search_running(true);
        self.obj().set_search_progress_visible(true);
        self.split_view.set_show_content(true);

        let progress_banner_button =
            Self::find_child_by_name(self.progress_banner.upcast_ref::<gtk::Widget>(), "button")
                .and_downcast::<gtk::Button>()
                .expect("failed to find banner button");
        progress_banner_button.grab_focus();

        self.engine.search(search);
    }

    fn find_child_by_name(widget: &gtk::Widget, child_name: &str) -> Option<gtk::Widget> {
        if widget.css_name() == child_name {
            return Some(widget.clone());
        }

        let mut child = widget.first_child();
        while let Some(a_child) = child {
            let the_child = Self::find_child_by_name(&a_child, child_name);
            if the_child.is_some() {
                return the_child;
            }
            child = a_child.next_sibling();
        }

        None
    }

    fn stop_search(&self) {
        self.obj().set_search_running(false);
        self.engine.cancel();
    }

    fn update_search_progress(&self) {
        let files = self.searched_files.get();
        let matches = self.number_of_matches.get();
        let message = gettext_f(
            "Searched {files} files and found {matches} matches",
            &[
                ("files", &files.to_string()),
                ("matches", &matches.to_string()),
            ],
        );

        *self.search_progress_notification.borrow_mut() = message;
        self.obj().notify("search_progress_notification");
    }

    fn show_update_banner(&self, version: &str) {
        self.update_banner.set_title(&gettext_f(
            "Updated to {version} 🎉",
            &[("version", version)],
        ));
        self.update_banner.set_visible(true);
    }
}

#[glib::derived_properties]
impl ObjectImpl for SearchWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        if APP_ID.contains("Devel") {
            obj.add_css_class("devel");
        }

        self.config
            .bind_property("window_width", obj.as_ref(), "default_width")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("window_height", obj.as_ref(), "default_height")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("window_maximized", obj.as_ref(), "maximized")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("case-sensitive", obj.as_ref(), "case-sensitive")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("include-hidden", obj.as_ref(), "include-hidden")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("include-ignored", obj.as_ref(), "include-ignored")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("disable-regex", obj.as_ref(), "disable-regex")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("search_names", obj.as_ref(), "search_names")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("search_pdf", obj.as_ref(), "search_pdf")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("search_office", obj.as_ref(), "search_office")
            .bidirectional()
            .sync_create()
            .build();

        self.config
            .bind_property("search_path", obj.as_ref(), "search_path")
            .bidirectional()
            .sync_create()
            .build();

        if self.config.last_version() != env!("APP_VERSION") {
            self.show_update_banner(env!("APP_VERSION"));
            self.config.set_last_version(env!("APP_VERSION"));
        }

        obj.results().connect_items_changed(clone!(
            #[weak]
            obj,
            move |items, _, _, _| {
                obj.imp().number_of_matches.set(items.n_items());
                obj.notify("number_of_matches");
            }
        ));
        obj.errors().connect_items_changed(clone!(
            #[weak]
            obj,
            move |items, _, _, _| {
                obj.imp().number_of_errors.set(items.n_items());
                obj.notify("number_of_errors");

                obj.imp().has_errors.set(items.n_items() > 0);
                obj.notify("has_errors");
            }
        ));
        obj.connect_search_path_notify(|obj| {
            let path = obj.search_path();
            *obj.imp().search_directory.borrow_mut() = match path.file_name() {
                Some(directory) => directory.to_string_lossy().to_string(),
                None => path.to_string_lossy().to_string(),
            };
            obj.notify("search-directory")
        });

        obj.connect_search_running_notify(|obj| {
            let imp = obj.imp();

            if imp.search_running.get() {
                let label = gettext("Cancel Search");
                obj.set_search_progress_action(label);
            } else {
                let label = gettext("Close");
                obj.set_search_progress_action(label);
            }

            if imp.search_running.get() || imp.number_of_matches.get() > 0 {
                imp.results_stack
                    .set_visible_child(&imp.results_page.child());
            } else {
                imp.results_stack
                    .set_visible_child(&imp.no_results_page.child());
            }
        });

        obj.connect_searched_files_notify(|obj| {
            obj.imp().update_search_progress();
        });
        obj.connect_number_of_matches_notify(|obj| {
            obj.imp().update_search_progress();
        });

        obj.connect_number_of_errors_notify(|obj| {
            let errors = obj.number_of_errors();
            *obj.imp().search_errors_notification.borrow_mut() = gettext_f(
                "Encountered {errors} errors during search",
                &[("errors", &errors.to_string())],
            );
            obj.notify("search_errors_notification")
        });

        self.init_manager();
    }
}

impl WidgetImpl for SearchWindow {}

impl WindowImpl for SearchWindow {}

impl ApplicationWindowImpl for SearchWindow {}

impl AdwApplicationWindowImpl for SearchWindow {}
