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
    use glib::subclass::InitializingObject;
    use gtk::{glib, prelude::*, CompositeTemplate};
    use std::{cell::RefCell, fs};

    #[derive(CompositeTemplate, glib::Properties, Default)]
    #[template(file = "src/ui/preview/plain_preview.blp")]
    #[properties(wrapper_type = super::PlainPreview)]
    pub struct PlainPreview {
        #[property(get, set)]
        pub result: RefCell<SearchResult>,

        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
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
        fn update_preview(&self) {
            let result = self.result.borrow();

            if !result.file().exists() {
                return;
            }

            let full_text = fs::read_to_string(result.file())
                .expect("This can file but I don't care right now");

            let buffer = self.text_view.buffer();
            buffer.set_text(&full_text);
            let mut cursor_position = buffer.start_iter();
            cursor_position.forward_lines(result.line() as i32);
            buffer.place_cursor(&cursor_position);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PlainPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.connect_result_notify(|obj| {
                obj.imp().update_preview();
            });
        }
    }

    impl WidgetImpl for PlainPreview {}
}
