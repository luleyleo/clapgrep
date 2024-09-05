use adw::subclass::prelude::*;
use clapgrep_core::{
    extended::ExtendedType, manager::{Manager, SearchResult}, options::{Options, Sort}, search::Search
};
use glib::subclass::InitializingObject;
use gtk::{gio::{self, Cancellable}, glib::{self, clone}, prelude::*, CompositeTemplate, FileDialog, StringList};
use std::{
    cell::{Cell, RefCell}, path::{Path, PathBuf}
};

use crate::{error_window::ErrorWindow, search_model::SearchModel};

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/search_window/search_window.blp")]
#[properties(wrapper_type = super::SearchWindow)]
pub struct SearchWindow {
    #[property(get, set)]
    pub search_path: RefCell<String>,
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
    pub search_pdf: Cell<bool>,
    #[property(get, set)]
    pub search_office: Cell<bool>,

    #[property(get, set)]
    pub search_running: Cell<bool>,
    #[property(get, set)]
    pub searched_files: Cell<u32>,
    #[property(get)]
    pub number_of_matches: Cell<u32>,

    #[property(get)]
    pub errors: StringList,
    #[property(get)]
    pub number_of_errors: Cell<u32>,
    #[property(get)]
    pub has_errors: Cell<bool>,

    pub manager: RefCell<Option<Manager>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchWindow {
    const NAME: &'static str = "ClapgrepSearchWindow";
    type Type = super::SearchWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
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
    fn on_cancel_search(&self, _: &adw::ActionRow) {
        self.stop_search();
    }

    #[template_callback]
    fn on_cd(&self, _: &gtk::Button) {
        let obj = self.obj();
        let initial_folder = gio::File::for_path(Path::new(self.search_path.borrow().as_str()));

        FileDialog::builder()
            .title("Choose Search Path")
            .initial_folder(&initial_folder)
            .modal(true)
            .build()
            .select_folder(Some(self.obj().as_ref()), Cancellable::NONE, clone!(#[weak] obj, move |result| {
                if let Ok(result) = result {
                    let path = result.path().unwrap().to_string_lossy().to_string();
                    obj.set_search_path(path);
                }
            }));
    }

    #[template_callback]
    fn on_show_errors(&self, _: &adw::ActionRow) {
        let error_window = ErrorWindow::new(&self.obj());
        error_window.present();
    }
}

impl SearchWindow {
    fn init_manager(&self) {
        assert!(self.manager.borrow().is_none());

        let (sender, receiver) = flume::unbounded();
        let manager = Manager::new(sender);
        manager.set_sort(Sort::Path);
        *self.manager.borrow_mut() = Some(manager);

        let app = self.obj().clone();
        let model = self.results.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(result) = receiver.recv_async().await {
                match result {
                    SearchResult::FinalResults(results) => {
                        model.clear();
                        model.extend_with_results(&results.data);
                        app.set_search_running(false);
                    }
                    SearchResult::InterimResult(file_info) => {
                        model.append_file_info(&file_info);
                    }
                    SearchResult::SearchErrors(errors) => {
                        app.errors().extend(errors);
                    }
                    SearchResult::SearchCount(count) => {
                        app.set_searched_files(count as u32);
                    }
                }
            }
        });
    }

    fn start_search(&self) {
        if self.manager.borrow().is_none() {
            self.init_manager();
        }

        self.results.clear();

        if let Some(manager) = self.manager.borrow().as_ref() {
            let search = Search {
                directory: PathBuf::from(self.search_path.borrow().as_str()),
                pattern: self.content_search.borrow().to_string(),
            };
            let options = Options {
                sort: Sort::Path,
                case_sensitive: self.case_sensitive.get(),
                ignore_dot: !self.include_hidden.get(),
                use_gitignore: !self.include_ignored.get(),
                fixed_string: self.disable_regex.get(),
                extended: self.get_extended_types(),
                ..Options::default()
            };

            self.results.clear();
            self.errors.splice(0, self.errors.n_items(), &[]);
            self.obj().set_searched_files(0);
            self.obj().set_search_running(true);

            manager.set_options(options);
            manager.search(&search);
        }
    }

    fn stop_search(&self) {
        self.obj().set_search_running(false);
        if let Some(manager) = self.manager.borrow().as_ref() {
            manager.stop();
        }
    }

    fn get_extended_types(&self) -> Vec<ExtendedType> {
        let mut types = Vec::new();

        if self.search_pdf.get() {
            types.push(ExtendedType::Pdf);
        }

        if self.search_office.get() {
            types.push(ExtendedType::Office);
        }

        types
    }
}

#[glib::derived_properties]
impl ObjectImpl for SearchWindow {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.results().connect_items_changed(clone!(#[weak] obj, move |items, _, _, _| {
            obj.imp().number_of_matches.set(items.n_items());
            obj.notify("number_of_matches");
        }));
        obj.errors().connect_items_changed(clone!(#[weak] obj, move |items, _, _, _| {
            obj.imp().number_of_errors.set(items.n_items());
            obj.notify("number_of_errors");

            obj.imp().has_errors.set(items.n_items() > 0);
            obj.notify("has_errors");
        }));
        obj.connect_search_path_notify(|obj| {
            let path = obj.search_path();
            let final_component = path.split("/").last().map(String::from);
            obj.imp().search_directory.set(match final_component {
                Some(directory) => directory,
                None => path,
            });
            obj.notify("search-directory")
        });

        if self.search_path.borrow().is_empty() {
            if let Ok(absolute) = Path::new(".").canonicalize() {
                self.obj().set_search_path(absolute.to_string_lossy().to_string());
            }
        }
    }
}

impl WidgetImpl for SearchWindow {}

impl WindowImpl for SearchWindow {}

impl ApplicationWindowImpl for SearchWindow {}

impl AdwApplicationWindowImpl for SearchWindow {}
