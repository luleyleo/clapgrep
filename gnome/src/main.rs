use std::path::PathBuf;
use gtk::{gio::SimpleAction, License};
use adw::prelude::*;
use gtk::glib::{self, clone};
use gtk_blueprint::include_blp;

mod search_match;
mod search_model;
mod search_result;
mod search_window;
mod error_window;

fn setup_gettext() {
    let mut text_domain = gettextrs::TextDomain::new("de.leopoldluley.Clapgrep");

    if cfg!(debug_assertions) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("assets");
        text_domain = text_domain.push(assets_dir);
    }

    if let Err(error) = text_domain.init() {
        println!("Failed to setup gettext: {}", error);
    };
}

fn main() {
    setup_gettext();

    let app = adw::Application::builder()
        .application_id("de.leopoldluley.Clapgrep")
        .build();

    app.connect_activate(|app| {
        let app = app.downcast_ref::<adw::Application>().unwrap();
        let window = search_window::SearchWindow::new(app);

        let about_action = SimpleAction::new("about", None);
        about_action.connect_activate(clone!(#[weak] window, move |_, _| {
            adw::AboutDialog::builder()
                .application_name("Clapgrep")
                .version("0.1")
                .application_icon("de.leopoldluley.Clapgrep")
                .developer_name("Leopold Luley")
                .website("https://github.com/luleyleo/clapgrep")
                .issue_url("https://github.com/luleyleo/clapgrep/issues")
                .license_type(License::Gpl30)
                .copyright("© 2024 Leopold Luley")
                .build()
                .present(Some(&window));
        }));
        app.add_action(&about_action);

        let shortcuts_action = SimpleAction::new("shortcuts", None);
        shortcuts_action.connect_activate(clone!(#[weak] window, move |_, _| {
            let builder = gtk::Builder::from_string(include_blp!("gnome/src/shortcuts.blp"));
            let help_overlay = builder.object::<gtk::ShortcutsWindow>("help-overlay").unwrap();
            help_overlay.set_transient_for(Some(&window));
            help_overlay.set_application(window.application().as_ref());
            help_overlay.present();
        }));
        app.add_action(&shortcuts_action);

        let quit_action = SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(#[weak] window, move |_, _| {
            window.close();
        }));
        app.add_action(&quit_action);

        app.set_accels_for_action("app.quit", &["<ctrl>q"]);
        app.set_accels_for_action("app.shortcuts", &["<ctrl>h"]);
        app.set_accels_for_action("win.start-search", &["<ctrl>Return"]);
        app.set_accels_for_action("app.stop-search", &["<ctrl>s"]);

        window.present();
    });

    app.run();
}
