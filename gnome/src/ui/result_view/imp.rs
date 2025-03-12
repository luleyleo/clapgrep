use crate::{
    color::{default_accent_color, pango_color_from_rgba, watch_accent_color},
    search::{SearchMatch, SearchResult},
};
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, pango, prelude::*, CompositeTemplate};
use std::cell::RefCell;

#[derive(CompositeTemplate, glib::Properties)]
#[template(file = "src/ui/result_view/result_view.blp")]
#[properties(wrapper_type = super::ResultView)]
pub struct ResultView {
    #[property(get, set)]
    pub result: RefCell<Option<SearchResult>>,

    #[template_child]
    pub container: TemplateChild<gtk::Box>,
    #[template_child]
    pub content: TemplateChild<gtk::Label>,

    highlight_color: RefCell<pango::Color>,
}

impl Default for ResultView {
    fn default() -> Self {
        Self {
            result: Default::default(),
            container: Default::default(),
            content: Default::default(),
            highlight_color: RefCell::new(default_accent_color()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ResultView {
    const NAME: &'static str = "ClapgrepResultView";
    type Type = super::ResultView;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ResultView {
    fn update_content(&self) {
        let highlight_color = self.highlight_color.borrow();
        let result = self.result.borrow();
        if let Some(result) = result.as_ref() {
            let content = result.content();
            self.content.set_text(&content);

            let matches = result.matches();
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
                self.content.set_attributes(Some(&attributes));
            }
        }
    }
}

#[glib::derived_properties]
impl ObjectImpl for ResultView {
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
        // See https://gitlab.gnome.org/GNOME/gtk/-/issues/7302
        self.container.unparent();
    }
}

impl WidgetImpl for ResultView {}
