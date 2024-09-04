use adw::subclass::prelude::*;
use clapgrep_core::{
    extended::ExtendedType, manager::{Manager, SearchResult}, options::{Options, Sort}, search::Search
};
use glib::subclass::InitializingObject;
use gtk::{glib::{self, clone}, prelude::*, CompositeTemplate, StringList};
use std::{
    cell::{Cell, RefCell}, path::PathBuf
};

use crate::{error_window::ErrorWindow, search_model::SearchModel};

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/search_window/search_window.blp")]
#[properties(wrapper_type = super::SearchWindow)]
pub struct SearchWindow {
    #[property(get, set)]
    pub file_search: RefCell<String>,
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
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl SearchWindow {
    #[template_callback]
    fn on_search(&self, _: &adw::ActionRow) {
        if self.manager.borrow().is_none() {
            self.init_manager();
        }

        self.results.clear();

        if let Some(manager) = self.manager.borrow().as_ref() {
            let search = Search {
                directory: PathBuf::from("."),
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

    #[template_callback]
    fn on_cancel_search(&self, _: &adw::ActionRow) {
        self.obj().set_search_running(false);
        if let Some(manager) = self.manager.borrow().as_ref() {
            manager.stop();
        }
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
    }
}

impl WidgetImpl for SearchWindow {}

impl WindowImpl for SearchWindow {}

impl ApplicationWindowImpl for SearchWindow {}

impl AdwApplicationWindowImpl for SearchWindow {}
