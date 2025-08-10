use crate::{
    color::{default_accent_color, pango_color_from_rgba, watch_accent_color},
    search::{SearchHeading, SearchMatch, SearchResult},
};
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{glib, pango, prelude::*, CompositeTemplate};
use std::cell::RefCell;

glib::wrapper! {
    pub struct ResultView(ObjectSubclass<ResultViewImp>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ResultView {
    pub fn new(item: &glib::Object) -> Self {
        glib::Object::builder().property("item", item).build()
    }
}

#[derive(CompositeTemplate, glib::Properties)]
#[template(file = "src/ui/result_view.blp")]
#[properties(wrapper_type = ResultView)]
pub struct ResultViewImp {
    #[property(get, set)]
    pub item: RefCell<Option<glib::Object>>,

    #[template_child]
    pub header_view: TemplateChild<gtk::Label>,
    #[template_child]
    pub result_view: TemplateChild<gtk::Box>,
    #[template_child]
    pub result_location: TemplateChild<gtk::Label>,
    #[template_child]
    pub result_content: TemplateChild<gtk::Label>,

    highlight_color: RefCell<pango::Color>,
}

impl Default for ResultViewImp {
    fn default() -> Self {
        Self {
            item: Default::default(),
            header_view: Default::default(),
            result_view: Default::default(),
            result_location: Default::default(),
            result_content: Default::default(),
            highlight_color: RefCell::new(default_accent_color()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ResultViewImp {
    const NAME: &'static str = "ClapgrepResultView";
    type Type = ResultView;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ResultViewImp {
    fn update_content(&self) {
        let item = self.item.borrow();

        if let Some(item) = item.as_ref().and_then(|r| r.downcast_ref::<SearchResult>()) {
            self.header_view.set_visible(false);
            self.result_view.set_visible(true);
            self.update_result(item);
            return;
        }

        if let Some(item) = item
            .as_ref()
            .and_then(|r| r.downcast_ref::<SearchHeading>())
        {
            self.result_view.set_visible(false);
            self.header_view.set_visible(true);
            self.update_heading(item);
            return;
        }

        self.update_parent_role();
    }

    fn update_heading(&self, result: &SearchHeading) {
        let highlight_color = self.highlight_color.borrow();

        self.header_view
            .set_label(&result.file_path().to_string_lossy());

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
            self.header_view.set_attributes(Some(&attributes));
        }
    }

    fn update_result(&self, result: &SearchResult) {
        let highlight_color = self.highlight_color.borrow();

        self.result_content.set_text(&result.content());

        if result.page() == 0 {
            self.result_location
                .set_label(&format!("{}", result.line()));
        } else {
            self.result_location
                .set_label(&format!("{}", result.page()));
        }

        let matches = result.content_matches();
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
            self.result_content.set_attributes(Some(&attributes));
        }
    }

    fn update_parent_role(&self) {
        if let Some(item) = self.item.borrow().as_ref() {
            if let Some(parent) = self.obj().parent() {
                if item.type_() == SearchResult::static_type() {
                    parent.set_accessible_role(gtk::AccessibleRole::ListItem);
                } else {
                    parent.set_accessible_role(gtk::AccessibleRole::RowHeader);
                }
            }
        }
    }
}

#[glib::derived_properties]
impl ObjectImpl for ResultViewImp {
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

        obj.connect_item_notify(|obj| {
            obj.imp().update_content();
        });
    }

    fn dispose(&self) {
        // See https://gitlab.gnome.org/GNOME/gtk/-/issues/7302
        self.header_view.unparent();
        self.result_view.unparent();
    }
}

impl WidgetImpl for ResultViewImp {
    fn realize(&self) {
        self.parent_realize();
        self.update_parent_role();
    }
}
