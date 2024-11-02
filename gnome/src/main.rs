use adw::prelude::*;
use gtk::gio::SimpleAction;
use gtk::gio::{self, ApplicationFlags};
use gtk::glib::{self, clone};
use gtk::{gdk, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk_blueprint::include_blp;
use std::path::PathBuf;

mod about;
mod config;
mod error_window;
mod search_match;
mod search_model;
mod search_result;
mod search_window;

const APP_ID: &str = env!("APP_ID");

fn setup_gettext() {
    let mut text_domain = gettextrs::TextDomain::new(APP_ID);

    if cfg!(debug_assertions) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets");
        text_domain = text_domain.push(assets_dir);
    }

    if let Err(error) = text_domain.init() {
        println!("Failed to setup gettext: {}", error);
    };
}

fn main() {
    setup_gettext();

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_open(|app, files, _| start(app, files));
    app.connect_activate(|app| start(app, &[]));

    app.run();
}

fn start(app: &adw::Application, files: &[gio::File]) {
    let app = app.downcast_ref::<adw::Application>().unwrap();

    let style_provider = gtk::CssProvider::new();
    style_provider.load_from_string(include_str!("styles.css"));
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &style_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = search_window::SearchWindow::new(app);

    if let Some(dir) = files.first() {
        if let Some(path) = dir.path() {
            if path.is_dir() {
                window.set_search_path(path);
            }
        }
    }

    let about_action = SimpleAction::new("about", None);
    about_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| about::dialog().present(Some(&window))
    ));
    app.add_action(&about_action);

    let shortcuts_action = SimpleAction::new("shortcuts", None);
    shortcuts_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            let blueprint = include_blp!("gnome/src/shortcuts.blp");
            let builder = gtk::Builder::from_string(blueprint);
            let help_overlay = builder
                .object::<gtk::ShortcutsWindow>("help-overlay")
                .unwrap();
            help_overlay.set_transient_for(Some(&window));
            help_overlay.set_application(window.application().as_ref());
            help_overlay.present();
        }
    ));
    app.add_action(&shortcuts_action);

    let quit_action = SimpleAction::new("quit", None);
    quit_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            window.close();
        }
    ));
    app.add_action(&quit_action);

    app.set_accels_for_action("app.quit", &["<ctrl>q"]);
    app.set_accels_for_action("app.shortcuts", &["<ctrl>h"]);
    app.set_accels_for_action("win.start-search", &["<ctrl>Return"]);
    app.set_accels_for_action("app.stop-search", &["<ctrl>s"]);

    window.present();
}
