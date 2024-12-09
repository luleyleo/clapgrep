use gtk::glib::{self, Object};

use crate::search::SearchResult;

glib::wrapper! {
    pub struct PlainPreview(ObjectSubclass<imp::PlainPreview>)
        @extends gtk::Widget;
}

impl PlainPreview {
    pub fn new(result: &SearchResult) -> Self {
        Object::builder().property("result", result).build()
    }
}

mod imp {
    use crate::search::SearchResult;
    use adw::subclass::prelude::*;
    use gettextrs::gettext;
    use glib::subclass::InitializingObject;
    use gtk::{glib, prelude::*, CompositeTemplate};
    use sourceview5::prelude::*;
    use std::{cell::RefCell, fs, time::Duration};

    #[derive(CompositeTemplate, glib::Properties, Default)]
    #[template(file = "src/ui/preview/plain_preview.blp")]
    #[properties(wrapper_type = super::PlainPreview)]
    pub struct PlainPreview {
        #[property(get, set)]
        pub result: RefCell<SearchResult>,

        #[template_child]
        pub title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub text_view: TemplateChild<sourceview5::View>,

        #[template_child]
        pub views: TemplateChild<gtk::Stack>,
        #[template_child]
        pub no_preview: TemplateChild<gtk::StackPage>,
        #[template_child]
        pub some_preview: TemplateChild<gtk::StackPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlainPreview {
        const NAME: &'static str = "ClapgrepPlainPreview";
        type Type = super::PlainPreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl PlainPreview {
        fn buffer(&self) -> sourceview5::Buffer {
            self.text_view
                .buffer()
                .downcast::<sourceview5::Buffer>()
                .unwrap()
        }

        fn update_preview(&self) {
            let result = self.result.borrow();
            let file = result.absolute_path();

            if !file.exists() {
                return;
            }

            if let Ok(full_text) = fs::read_to_string(&file) {
                let buffer = self.buffer();
                buffer.set_text(&full_text);

                // Setup syntax highlighting
                let lm = sourceview5::LanguageManager::default();
                let language = lm.guess_language(Some(&file), None);
                buffer.set_language(language.as_ref());
                self.text_view.set_monospace(language.is_some());

                // Place cursor on result line.
                let mut cursor_position = buffer.start_iter();
                cursor_position.forward_lines((result.line() - 1) as i32);
                buffer.place_cursor(&cursor_position);

                // Set title to file name.
                let file_name = file.file_name().unwrap().to_string_lossy();
                self.title.set_title(file_name.as_ref());

                // Scroll to result line after 100ms.
                //
                // The delay is needed because scroll_to_iter only works
                // once the line hights have been calculated in an idle handler.
                let text_view = self.text_view.clone();
                glib::timeout_add_local_once(Duration::from_millis(100), move || {
                    text_view.scroll_to_iter(&mut cursor_position, 0.0, true, 0.0, 0.3);
                });

                self.views.set_visible_child(&self.some_preview.child());
            } else {
                self.title.set_title(&gettext("Content Preview"));
                self.views.set_visible_child(&self.no_preview.child());
            }
        }

        fn setup_style(&self) {
            let text_view_buffer = self.buffer();

            let asm = adw::StyleManager::default();
            let sm = sourceview5::StyleSchemeManager::default();

            let light_style = sm.scheme("Adwaita").unwrap();
            let dark_style = sm.scheme("Adwaita-dark").unwrap();

            let setter = move |asm: &adw::StyleManager| {
                let current_style = if asm.is_dark() {
                    &dark_style
                } else {
                    &light_style
                };

                text_view_buffer.set_style_scheme(Some(current_style));
            };

            setter(&asm);
            asm.connect_dark_notify(setter);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PlainPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.setup_style();

            obj.connect_result_notify(|obj| {
                obj.imp().update_preview();
            });
        }
    }

    impl WidgetImpl for PlainPreview {}
}
