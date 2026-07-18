//! Large-print, chrome-free, read-only display of the open sermon for use
//! at the pulpit — distinct from Rubric's Typst export (`sermon_export.rs`),
//! this never leaves the screen and has no formatting options.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Overlay, ScrolledWindow};

use crate::model::Sermon;

pub struct PreachingView {
    window: gtk4::Window,
}

impl PreachingView {
    pub fn new(parent: &impl IsA<gtk4::Window>, sermon: &Sermon) -> Self {
        let window = gtk4::Window::new();
        window.set_transient_for(Some(parent));
        window.set_decorated(false);
        window.add_css_class("preaching-view-window");

        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_margin_top(56);
        content.set_margin_bottom(56);
        content.set_margin_start(72);
        content.set_margin_end(72);

        let title = Label::new(Some(sermon.display_title()));
        title.add_css_class("preaching-title");
        title.set_xalign(0.0);
        title.set_wrap(true);
        content.append(&title);

        if let Some(date) = sermon.planned_date {
            let date_lbl = Label::new(Some(&date.format("%B %-d, %Y").to_string()));
            date_lbl.add_css_class("preaching-date");
            date_lbl.set_xalign(0.0);
            content.append(&date_lbl);
        }

        for movement in &sermon.movements {
            let m_lbl = Label::new(Some(&movement.name));
            m_lbl.add_css_class("preaching-movement");
            m_lbl.set_xalign(0.0);
            m_lbl.set_margin_top(28);
            content.append(&m_lbl);

            for idea in &movement.ideas {
                if idea.text.is_empty() && idea.notes.is_empty() {
                    continue;
                }
                let idea_box = GtkBox::new(Orientation::Vertical, 4);
                idea_box.set_margin_top(14);
                idea_box.set_margin_start(28);

                if !idea.text.is_empty() {
                    let idea_lbl = Label::new(Some(&idea.text));
                    idea_lbl.add_css_class("preaching-idea");
                    idea_lbl.set_xalign(0.0);
                    idea_lbl.set_wrap(true);
                    idea_box.append(&idea_lbl);
                }
                if !idea.notes.is_empty() {
                    let notes_lbl = Label::new(Some(&idea.notes));
                    notes_lbl.add_css_class("preaching-notes");
                    notes_lbl.set_xalign(0.0);
                    notes_lbl.set_wrap(true);
                    idea_box.append(&notes_lbl);
                }
                content.append(&idea_box);
            }
        }

        let scroll = ScrolledWindow::new();
        scroll.set_child(Some(&content));
        scroll.set_vexpand(true);
        scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);

        let close_btn = Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        close_btn.add_css_class("circular");
        close_btn.add_css_class("osd");
        close_btn.set_valign(Align::Start);
        close_btn.set_halign(Align::End);
        close_btn.set_margin_top(16);
        close_btn.set_margin_end(16);
        close_btn.set_tooltip_text(Some("Close (Esc)"));

        let overlay = Overlay::new();
        overlay.set_child(Some(&scroll));
        overlay.add_overlay(&close_btn);

        window.set_child(Some(&overlay));

        {
            let window = window.clone();
            close_btn.connect_clicked(move |_| window.close());
        }
        {
            let for_key = window.clone();
            let key_ctl = gtk4::EventControllerKey::new();
            key_ctl.connect_key_pressed(move |_, key, _, _| {
                if key == gtk4::gdk::Key::Escape {
                    for_key.close();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            window.add_controller(key_ctl);
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
        self.window.fullscreen();
    }
}
