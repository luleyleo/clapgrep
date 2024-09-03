use adw::prelude::*;

mod search_match;
mod search_model;
mod search_result;
mod window;

fn main() {
    let result = gettextrs::TextDomain::new("de.leopoldluley.Clapgrep")
        .skip_system_data_paths()
        .push("assets")
        .init();
    if let Err(error) = result {
        println!("Failed to setup gettext: {}", error);
    };

    let application = adw::Application::builder()
        .application_id("de.leopoldluley.Clapgrep")
        .build();

    application.connect_activate(|app| {
        let app = app.downcast_ref::<adw::Application>().unwrap();
        let window = window::Window::new(app);
        window.present();
    });

    application.run();
}
