use adw::prelude::*;
use gtk::gio::ApplicationFlags;
use std::path::PathBuf;

mod app;
mod config;
mod search;
mod ui;

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

    let application = adw::Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();

    application.connect_open(|a, files, _| app::start(a, files));
    application.connect_activate(|a| app::start(a, &[]));

    application.run();
}
