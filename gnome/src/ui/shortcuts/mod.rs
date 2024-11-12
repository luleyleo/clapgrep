use gtk::prelude::*;
use gtk_blueprint::include_blp;

use crate::ui::SearchWindow;

pub fn show_shortcuts(window: &SearchWindow) {
    let blueprint = include_blp!("gnome/src/ui/shortcuts/shortcuts.blp");
    let builder = gtk::Builder::from_string(blueprint);
    let help_overlay = builder
        .object::<gtk::ShortcutsWindow>("help-overlay")
        .unwrap();
    help_overlay.set_transient_for(Some(window));
    help_overlay.set_application(window.application().as_ref());
    help_overlay.present();
}
