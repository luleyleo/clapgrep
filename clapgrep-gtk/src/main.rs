use adw::prelude::*;

mod window;

fn main() {
    let application = adw::Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();

    application.connect_activate(|app| {
        let app = app.downcast_ref::<adw::Application>().unwrap();
        let window = window::Window::new(app);
        window.present();
    });

    application.run();
}
