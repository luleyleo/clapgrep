use std::path::PathBuf;

use adw::prelude::*;

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

    let application = adw::Application::builder()
        .application_id("de.leopoldluley.Clapgrep")
        .build();

    application.connect_activate(|app| {
        let app = app.downcast_ref::<adw::Application>().unwrap();
        let window = search_window::SearchWindow::new(app);
        window.present();
    });

    application.run();
}
