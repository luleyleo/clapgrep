use adw::prelude::*;
use gtk::gio::ApplicationFlags;

mod app;
mod config;
mod i18n;
mod search;
mod ui;

const APP_ID: &str = env!("APP_ID");

fn main() {
    i18n::setup_gettext();

    let application = adw::Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();

    application.connect_open(|a, files, _| app::start(a, files));
    application.connect_activate(|a| app::start(a, &[]));

    application.run();
}
