use crate::{
    search::SearchResult,
    ui::preview::{pdf_preview::PdfPreview, text_preview::TextPreview},
};
use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::subclass::InitializingObject;
use gtk::{glib, prelude::*, CompositeTemplate};
use std::cell::RefCell;

mod pdf_preview;
mod text_preview;

glib::wrapper! {
    pub struct Preview(ObjectSubclass<PreviewImp>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Preview {
    pub fn new(result: &SearchResult) -> Self {
        glib::Object::builder().property("result", result).build()
    }
}

#[derive(CompositeTemplate, glib::Properties, Default)]
#[template(file = "src/ui/preview.blp")]
#[properties(wrapper_type = Preview)]
pub struct PreviewImp {
    #[property(get, set)]
    pub result: RefCell<SearchResult>,

    #[template_child]
    pub title: TemplateChild<adw::WindowTitle>,

    #[template_child]
    pub views: TemplateChild<gtk::Stack>,
    #[template_child]
    pub no_selection: TemplateChild<gtk::StackPage>,
    #[template_child]
    pub no_preview: TemplateChild<gtk::StackPage>,
    #[template_child]
    pub some_text_preview: TemplateChild<gtk::StackPage>,
    #[template_child]
    pub some_pdf_preview: TemplateChild<gtk::StackPage>,

    #[template_child]
    pub text_preview: TemplateChild<TextPreview>,
    #[template_child]
    pub pdf_preview: TemplateChild<PdfPreview>,
}

#[glib::object_subclass]
impl ObjectSubclass for PreviewImp {
    const NAME: &'static str = "ClapgrepPreview";
    type Type = Preview;
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
impl PreviewImp {
    fn update_preview(&self) {
        let file = self.result.borrow().absolute_path();

        if !file.exists() {
            return;
        }

        // Set title to file name.
        let file_name = file.file_name().unwrap().to_string_lossy();
        self.title.set_title(file_name.as_ref());

        if let Some(ext) = file.extension().and_then(|ext| ext.to_str()) {
            // Try PDF
            if clapgrep_core::extra::pdf::EXTENSIONS.contains(&ext) {
                self.pdf_preview.set_result(self.obj().result());
                self.views.set_visible_child(&self.some_pdf_preview.child());
                return;
            }

            // Try Office
            if clapgrep_core::extra::office::EXTENSIONS.contains(&ext) {
                self.title.set_title(&gettext("Content Preview"));
                self.views.set_visible_child(&self.no_preview.child());
                return;
            }
        }

        // Fall back to text
        self.text_preview.set_result(self.obj().result());
        self.views
            .set_visible_child(&self.some_text_preview.child());
    }
}

#[glib::derived_properties]
impl ObjectImpl for PreviewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        obj.connect_result_notify(|obj| {
            obj.imp().update_preview();
        });
    }
}

impl WidgetImpl for PreviewImp {}
