use std::path::PathBuf;
use gtk::{gio::SimpleAction, License};
use adw::prelude::*;
use gtk::glib::{self, clone};

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

        window.present();
    });

    app.run();
}
