//! "Preached before": every past sermon sharing a scripture reference with
//! the open one, grouped by reference and newest first. The browsable form of
//! the transient toast in `app_window::note_past_sermon_reuse` — that fires
//! only at the moment a tag is added and is easy to miss, which left the
//! sermon archive doing nothing for the preacher who tagged it.

use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::library::LibraryIndex;
use crate::model::Sermon;

pub struct PreachedBeforeWindow {
    window: adw::Window,
}

impl PreachedBeforeWindow {
    pub fn new(
        parent: &impl IsA<gtk4::Window>,
        index: &LibraryIndex,
        current: &Sermon,
        on_open: impl Fn(PathBuf) + 'static,
    ) -> Self {
        let window = adw::Window::builder()
            .title("Preached Before")
            .transient_for(parent)
            .default_width(520)
            .default_height(580)
            .build();

        let header = adw::HeaderBar::new();
        let groups = index.preached_before(current);
        let on_open: Rc<dyn Fn(PathBuf)> = Rc::new(on_open);

        let content: gtk4::Widget = if groups.is_empty() {
            let status = adw::StatusPage::new();
            status.set_icon_name(Some("bookmark-new-symbolic"));
            status.set_title("Nothing preached before");
            status.set_description(Some(if current.s_tags.is_empty() {
                "This sermon has no Scripture tags yet. Add one in the status bar, or type @ in an idea to cite a passage."
            } else {
                "None of this sermon's Scripture references appear in your other sermons."
            }));
            status.upcast()
        } else {
            let body = GtkBox::new(Orientation::Vertical, 6);
            body.set_margin_start(18);
            body.set_margin_end(18);
            body.set_margin_top(12);
            body.set_margin_bottom(18);

            for (tag, sermons) in groups {
                let heading = Label::new(Some(&tag));
                heading.add_css_class("heading");
                heading.set_xalign(0.0);
                heading.set_margin_top(10);
                body.append(&heading);

                for (path, sermon) in sermons {
                    body.append(&sermon_row(sermon, path.clone(), &window, &on_open));
                }
            }

            let scroll = ScrolledWindow::new();
            scroll.set_vexpand(true);
            scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
            let clamp = adw::Clamp::new();
            clamp.set_maximum_size(500);
            clamp.set_child(Some(&body));
            scroll.set_child(Some(&clamp));
            scroll.upcast()
        };

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&content));
        window.set_content(Some(&toolbar));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn sermon_row(
    sermon: &Sermon,
    path: PathBuf,
    window: &adw::Window,
    on_open: &Rc<dyn Fn(PathBuf)>,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    row.add_css_class("preached-before-row");

    let text_col = GtkBox::new(Orientation::Vertical, 1);
    text_col.set_hexpand(true);

    let title = Label::new(Some(sermon.display_title()));
    title.set_xalign(0.0);
    title.set_wrap(true);
    title.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    text_col.append(&title);

    let mut meta: Vec<String> = Vec::new();
    if let Some(date) = sermon.planned_date {
        meta.push(date.format("%B %-d, %Y").to_string());
    } else {
        meta.push("No date planned".to_string());
    }
    if let Some(series) = &sermon.series {
        meta.push(series.clone());
    }
    let subtitle = Label::new(Some(&meta.join(" · ")));
    subtitle.add_css_class("dim-label");
    subtitle.add_css_class("caption");
    subtitle.set_xalign(0.0);
    text_col.append(&subtitle);
    row.append(&text_col);

    let open_btn = Button::with_label("Open");
    open_btn.add_css_class("flat");
    open_btn.set_valign(Align::Center);
    {
        let on_open = on_open.clone();
        let window = window.clone();
        open_btn.connect_clicked(move |_| {
            on_open(path.clone());
            window.close();
        });
    }
    row.append(&open_btn);
    row
}
