use adw::prelude::*;

fn main() {
    let application = adw::Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();

    application.connect_activate(|app| {
        // ActionRows are only available in Adwaita
        let row = adw::ActionRow::builder()
            .activatable(true)
            .title("Click me")
            .build();
        row.connect_activated(|_| {
            eprintln!("Clicked!");
        });

        let list = gtk::ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(gtk::SelectionMode::None)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();
        list.append(&row);

        // Combine the content in a box
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        // Adwaitas' ApplicationWindow does not include a HeaderBar
        content.append(&adw::HeaderBar::new());
        content.append(&list);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("First App")
            .default_width(350)
            // add content to window
            .content(&content)
            .build();
        window.present();
    });

    application.run();
}
