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
    use crate::{color::watch_accent_color, search::SearchResult};
    use adw::subclass::prelude::*;
    use anyhow::Context;
    use glib::subclass::InitializingObject;
    use gtk::{gdk, gio, glib, prelude::*, CompositeTemplate, DrawingArea};
    use poppler::Document;
    use std::cell::{Cell, RefCell};

    #[derive(CompositeTemplate, glib::Properties)]
    #[template(file = "src/ui/preview/pdf_preview.blp")]
    #[properties(wrapper_type = super::PdfPreview)]
    pub struct PdfPreview {
        #[property(get, set)]
        pub result: RefCell<Option<SearchResult>>,

        #[template_child]
        pub pdf_view: TemplateChild<DrawingArea>,

        highlight_color: Cell<gdk::RGBA>,
    }

    impl Default for PdfPreview {
        fn default() -> Self {
            Self {
                result: Default::default(),
                pdf_view: Default::default(),
                highlight_color: Cell::new(gdk::RGBA::BLACK),
            }
        }
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
            if let Some(result) = result.as_ref() {
                let file = result.absolute_path();
                let page_index = result.page() - 1;

                let doc =
                    Document::from_gfile(&gio::File::for_path(&file), None, gio::Cancellable::NONE)
                        .context("Failed to open document")?;
                let page = doc.page(page_index as i32).expect("out of range");

                let matched_strings = result.matched_strings();
                let highlights = matched_strings
                    .iter()
                    .flat_map(|m| page.find_text(m))
                    .collect::<Vec<_>>();
                let hc = self.highlight_color.get();

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

                        // draw highlights
                        context.set_source_rgba(
                            hc.red().into(),
                            hc.green().into(),
                            hc.blue().into(),
                            0.3,
                        );
                        for h in highlights.iter() {
                            context.rectangle(
                                h.x1(),
                                ph - h.y2(),
                                h.x2() - h.x1(),
                                h.y2() - h.y1(),
                            );
                            _ = context.fill();
                        }
                    });
            } else {
                self.pdf_view
                    .set_draw_func(|_pdf_view, _context, _cw, _ch| {});
            }

            Ok(())
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PdfPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            watch_accent_color(glib::clone!(
                #[weak]
                obj,
                move |accent_color| {
                    obj.imp().highlight_color.replace(accent_color);
                    if let Err(err) = obj.imp().update_preview() {
                        log::error!("Failed to udpate PDF preview after color change: {}", err);
                    }
                }
            ));

            obj.connect_result_notify(|obj| {
                if let Err(err) = obj.imp().update_preview() {
                    log::error!("Failed to udpate PDF preview: {}", err);
                }
            });
        }
    }

    impl WidgetImpl for PdfPreview {}
}
