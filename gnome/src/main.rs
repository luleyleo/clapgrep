use adw::prelude::*;
use gtk::{
    gio::{self, ApplicationFlags},
    glib,
};

mod app;
mod config;
mod i18n;
mod search;
mod ui;

const APP_ID: &str = env!("APP_ID");

fn main() {
    i18n::setup_gettext();

    let _ = gtk::init();
    let _ = adw::init();
    sourceview5::init();

    let resource_bytes = include_bytes!(env!("GRESOURCES_BUNDLE"));
    gio::resources_register(
        &gio::Resource::from_data(&glib::Bytes::from_static(resource_bytes)).unwrap(),
    );

    let application = adw::Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::HANDLES_OPEN)
        .build();

    application.connect_open(|a, files, _| app::start(a, files));
    application.connect_activate(|a| app::start(a, &[]));

    application.run();
}
