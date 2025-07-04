use crate::{build, ui};
use adw::prelude::*;
use gtk::{
    gdk,
    gio::{self, SimpleAction},
    glib::{self, clone},
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};

pub fn start(app: &adw::Application, files: &[gio::File]) {
    let app = app.downcast_ref::<adw::Application>().unwrap();

    let style_provider = gtk::CssProvider::new();
    style_provider.load_from_string(include_str!("styles.css"));
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &style_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ui::SearchWindow::new(app);

    if let Some(dir) = files.first() {
        if let Some(path) = dir.path() {
            if path.is_dir() {
                window.set_search_path(path);
            }
        }
    }

    let donate_action = SimpleAction::new("donate", None);
    donate_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            gtk::UriLauncher::new("https://ko-fi.com/luleyleo").launch(
                Some(&window),
                gio::Cancellable::NONE,
                |_| {},
            );
        }
    ));
    app.add_action(&donate_action);

    let about_action = SimpleAction::new("about", None);
    about_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            let app = window.application().unwrap();
            let app_path = app.resource_base_path().unwrap();
            let dialog = adw::AboutDialog::from_appdata(
                &format!("{app_path}/metainfo.xml"),
                Some(build::APP_VERSION),
            );
            dialog.present(Some(&window));
        }
    ));
    app.add_action(&about_action);

    let news_action = SimpleAction::new("news", None);
    news_action.connect_activate(clone!(
        #[weak]
        window,
        move |_, _| {
            let app = window.application().unwrap();
            let app_path = app.resource_base_path().unwrap();
            let dialog = adw::AboutDialog::from_appdata(
                &format!("{app_path}/metainfo.xml"),
                Some(build::APP_VERSION),
            );
            dialog.present(Some(&window));

            let navigation_view = dialog
                .first_child() // adw::BreakpointBin
                .unwrap()
                .first_child() // adw::FloatingSheet
                .unwrap()
                .first_child()
                .unwrap()
                .next_sibling() // adw::Gizmo
                .unwrap()
                .first_child() // adw::BreakpointBin
                .unwrap()
                .first_child() // adw::ToastOverlay
                .unwrap()
                .first_child() // adw::NaviationView
                .unwrap()
                .downcast::<adw::NavigationView>()
                .unwrap();
            navigation_view.push_by_tag("whatsnew");
        }
    ));
    app.add_action(&news_action);

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
    app.set_accels_for_action("win.stop-search", &["<ctrl>c"]);

    window.present();
}
