use crate::search::SearchResult;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, prelude::*, CompositeTemplate};
use std::cell::RefCell;

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/ui/result_view/result_view.blp")]
#[properties(wrapper_type = super::ResultView)]
pub struct ResultView {
    #[property(get, set)]
    pub result: RefCell<Option<SearchResult>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ResultView {
    const NAME: &'static str = "ClapgrepResultView";
    type Type = super::ResultView;
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
impl ResultView {
    fn update(&self) {
        // TODO: Implement this
    }
}

#[glib::derived_properties]
impl ObjectImpl for ResultView {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        obj.connect_result_notify(|obj| {
            obj.imp().update();
        });
    }
}

impl WidgetImpl for ResultView {}
