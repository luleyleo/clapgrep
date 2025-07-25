use crate::build::APP_PATH;
use crate::config::Config;
use adw::subclass::prelude::*;
use gtk::gio::{self, Cancellable, FileCopyFlags};
use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
use sourceview5::prelude::{FileExt, ObjectExt};
use std::fs;

#[derive(CompositeTemplate, Default)]
#[template(file = "src/ui/preferences/preferences.blp")]
pub struct PreferencesDialog {
    #[template_child]
    max_results_spinner: TemplateChild<adw::SpinRow>,
    #[template_child]
    nautilus_integration_toggle: TemplateChild<adw::SwitchRow>,

    config: Config,
}

#[glib::object_subclass]
impl ObjectSubclass for PreferencesDialog {
    const NAME: &'static str = "ClapgrepPreferencesDialog";
    type Type = super::PreferencesDialog;
    type ParentType = adw::PreferencesDialog;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl PreferencesDialog {}

impl PreferencesDialog {
    fn connect_nautilus_integration_toggle(&self) {
        // TODO: This should use `glib::user_data_dir()`, but that doesn't work inside of Flatpak.
        let nautilus_extension_path = glib::home_dir()
            .join(".local/share")
            .join("nautilus-python/extensions/clapgrep.py");

        let is_nautilus_integration_installed = nautilus_extension_path.is_file();

        self.nautilus_integration_toggle
            .set_active(is_nautilus_integration_installed);

        self.nautilus_integration_toggle
            .connect_active_notify(move |toggle| {
                if toggle.is_active() {
                    let extension_file = gio::File::for_path(&nautilus_extension_path);
                    let resource_file = gio::File::for_uri(&format!(
                        "resource://{APP_PATH}/integrations/nautilus/clapgrep.py"
                    ));
                    let _ = resource_file.copy(
                        &extension_file,
                        FileCopyFlags::NONE,
                        Cancellable::NONE,
                        None,
                    );
                } else {
                    let _ = fs::remove_file(&nautilus_extension_path);
                }
            });
    }
}

impl ObjectImpl for PreferencesDialog {
    fn constructed(&self) {
        self.parent_constructed();

        self.config
            .bind_property("max-search-results", &*self.max_results_spinner, "value")
            .bidirectional()
            .sync_create()
            .build();

        self.connect_nautilus_integration_toggle();
    }
}

impl WidgetImpl for PreferencesDialog {}

impl AdwDialogImpl for PreferencesDialog {}

impl PreferencesDialogImpl for PreferencesDialog {}
