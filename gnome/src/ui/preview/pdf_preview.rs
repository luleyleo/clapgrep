use crate::search::SearchResult;
use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct PdfPreview(ObjectSubclass<imp::PdfPreview>)
        @extends gtk::Widget;
}

impl PdfPreview {
    pub fn new(result: &SearchResult) -> Self {
        Object::builder().property("result", result).build()
    }
}

mod imp {
    use crate::search::SearchResult;
    use adw::subclass::prelude::*;
    use anyhow::Context;
    use glib::subclass::InitializingObject;
    use gtk::{gio, glib, prelude::*, CompositeTemplate, DrawingArea};
    use poppler::Document;
    use std::cell::RefCell;

    #[derive(CompositeTemplate, glib::Properties, Default)]
    #[template(file = "src/ui/preview/pdf_preview.blp")]
    #[properties(wrapper_type = super::PdfPreview)]
    pub struct PdfPreview {
        #[property(get, set)]
        pub result: RefCell<SearchResult>,

        #[template_child]
        pub pdf_view: TemplateChild<DrawingArea>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PdfPreview {
        const NAME: &'static str = "ClapgrepPdfPreview";
        type Type = super::PdfPreview;
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
    impl PdfPreview {
        fn update_preview(&self) -> anyhow::Result<()> {
            let result = self.result.borrow();
            let file = result.absolute_path();
            let page_index = result.line() - 1;

            let doc =
                Document::from_gfile(&gio::File::for_path(&file), None, gio::Cancellable::NONE)
                    .context("Failed to open document")?;
            let page = doc.page(page_index as i32).expect("out of range");

            self.pdf_view
                .set_draw_func(move |_pdf_view, context, cw, ch| {
                    let (cw, ch) = (cw as f64, ch as f64);
                    let (pw, ph) = page.size();
                    let scale = f64::min(cw / pw, ch / ph);

                    // center horizontally when fit to height
                    if (cw / pw) > (ch / ph) {
                        let tx = (cw - pw * scale) / 2.0;
                        context.translate(tx, 0.0);
                    }

                    // fit page to drawing area
                    context.scale(scale, scale);

                    // draw background
                    context.set_source_rgb(1.0, 1.0, 1.0);
                    context.rectangle(0.0, 0.0, pw, ph);
                    let _ = context.fill();

                    // draw pdf
                    page.render(context);
                });

            Ok(())
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PdfPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.connect_result_notify(|obj| {
                if let Err(err) = obj.imp().update_preview() {
                    log::error!("Failed to udpate PDF preview: {}", err);
                }
            });
        }
    }

    impl WidgetImpl for PdfPreview {}
}
