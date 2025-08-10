use crate::{
    color::{default_accent_color, pango_color_from_rgba, watch_accent_color},
    search::{SearchMatch, SearchResult},
};
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, pango, prelude::*, CompositeTemplate};
use std::cell::RefCell;

glib::wrapper! {
    pub struct ResultHeaderView(ObjectSubclass<ResultHeaderViewImp>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ResultHeaderView {
    pub fn new(result: &SearchResult) -> Self {
        glib::Object::builder().property("result", result).build()
    }
}

#[derive(CompositeTemplate, glib::Properties)]
#[template(file = "src/ui/result_header_view.blp")]
#[properties(wrapper_type = ResultHeaderView)]
pub struct ResultHeaderViewImp {
    #[property(get, set)]
    pub result: RefCell<Option<SearchResult>>,

    #[template_child]
    pub content: TemplateChild<gtk::LinkButton>,

    highlight_color: RefCell<pango::Color>,
}

impl Default for ResultHeaderViewImp {
    fn default() -> Self {
        Self {
            result: Default::default(),
            content: Default::default(),
            highlight_color: RefCell::new(default_accent_color()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ResultHeaderViewImp {
    const NAME: &'static str = "ClapgrepResultHeaderView";
    type Type = ResultHeaderView;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ResultHeaderViewImp {
    fn update_content(&self) {
        let highlight_color = self.highlight_color.borrow();
        let result = self.result.borrow();
        if let Some(result) = result.as_ref() {
            self.content.set_uri(&if cfg!(target_os = "windows") {
                format!("{}", result.absolute_path().to_string_lossy())
            } else {
                format!("file://{}", result.absolute_path().to_string_lossy())
            });

            self.content
                .set_label(&result.file_path().to_string_lossy());

            let button_label = self
                .content
                .first_child()
                .expect("gtk::LinkButton should have a child widget")
                .downcast::<gtk::Label>()
                .expect("gtk::LinkButton should have a Label as first child");

            let matches = result.file_name_matches();
            if let Some(matches) = matches {
                let attributes = pango::AttrList::new();
                for m in matches.iter::<SearchMatch>() {
                    let m = m.expect("expected SearchMatch");
                    let mut highlight = pango::AttrColor::new_foreground(
                        highlight_color.red(),
                        highlight_color.green(),
                        highlight_color.blue(),
                    );
                    highlight.set_start_index(m.start());
                    highlight.set_end_index(m.end());
                    attributes.insert(highlight);
                }
                button_label.set_attributes(Some(&attributes));
            }
        }
    }
}

#[glib::derived_properties]
impl ObjectImpl for ResultHeaderViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        watch_accent_color(glib::clone!(
            #[weak]
            obj,
            move |accent_color| {
                let accent_color = pango_color_from_rgba(&accent_color);
                obj.imp().highlight_color.replace(accent_color);
                obj.imp().update_content();
            }
        ));

        obj.connect_result_notify(|obj| {
            obj.imp().update_content();
        });
    }

    fn dispose(&self) {
        self.content.unparent();
    }
}

impl WidgetImpl for ResultHeaderViewImp {}
