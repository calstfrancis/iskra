//! Large-print, chrome-free, read-only display of the open sermon for use
//! at the pulpit — distinct from Rubric's Typst export (`sermon_export.rs`),
//! this never leaves the screen and has no formatting options.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Overlay, ScrolledWindow, ToggleButton};

use crate::state::AppState;
use crate::ui::preaching_print;
use crate::ui::styles;

pub struct PreachingView {
    window: gtk4::Window,
}

impl PreachingView {
    pub fn new(parent: &impl IsA<gtk4::Window>, state: &Rc<RefCell<AppState>>) -> Self {
        let sermon = state.borrow().sermon.clone();
        let window = gtk4::Window::new();
        window.set_transient_for(Some(parent));
        window.set_decorated(false);
        window.add_css_class("preaching-view-window");
        if state.borrow().config.preaching_warm_bg {
            window.add_css_class("warm");
        }

        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_margin_top(56);
        content.set_margin_bottom(56);
        content.set_margin_start(72);
        content.set_margin_end(72);

        // A thin strip in the day's liturgical color, when one is set — ties
        // Preaching View visually to the lectionary sidebar's season dot.
        if let Some(link) = &sermon.lectionary {
            let season_bar = GtkBox::new(Orientation::Horizontal, 0);
            season_bar.add_css_class("preaching-season-bar");
            season_bar.add_css_class(styles::season_dot_class(&link.colour_hex));
            content.append(&season_bar);
        }

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

        // Heading labels in movement order, so the scroll tracker below can
        // measure each one's position within `content` to know which
        // movement the viewport is currently showing.
        let mut movement_headings: Vec<Label> = Vec::new();
        let mut idea_number = 1u32;
        for movement in &sermon.movements {
            let m_lbl = Label::new(Some(&movement.name));
            m_lbl.add_css_class("preaching-movement");
            m_lbl.set_xalign(0.0);
            m_lbl.set_margin_top(28);
            content.append(&m_lbl);
            movement_headings.push(m_lbl);

            for idea in &movement.ideas {
                // Counted (not just displayed) for every idea, matching
                // `Sermon::numbering`'s own counting — a blank idea still
                // occupies a number, it just never renders here.
                let this_number = idea_number;
                idea_number += 1;
                if idea.text.is_empty() && idea.notes.is_empty() {
                    continue;
                }
                let idea_box = GtkBox::new(Orientation::Vertical, 4);
                idea_box.set_margin_top(14);
                idea_box.set_margin_start(28);

                if !idea.text.is_empty() {
                    let idea_row = GtkBox::new(Orientation::Horizontal, 0);
                    idea_row.set_valign(Align::Start);
                    let number_lbl = Label::new(Some(&this_number.to_string()));
                    number_lbl.add_css_class("preaching-idea-number");
                    number_lbl.set_valign(Align::Start);
                    idea_row.append(&number_lbl);
                    let idea_lbl = Label::new(Some(&idea.text));
                    idea_lbl.add_css_class("preaching-idea");
                    idea_lbl.set_xalign(0.0);
                    idea_lbl.set_wrap(true);
                    idea_lbl.set_hexpand(true);
                    idea_row.append(&idea_lbl);
                    idea_box.append(&idea_row);
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
        close_btn.add_css_class("preaching-overlay-btn");
        close_btn.set_valign(Align::Start);
        close_btn.set_halign(Align::End);
        close_btn.set_margin_top(16);
        close_btn.set_margin_end(16);
        close_btn.set_tooltip_text(Some("Close (Esc)"));

        let print_btn = Button::from_icon_name("document-print-symbolic");
        print_btn.add_css_class("flat");
        print_btn.add_css_class("circular");
        print_btn.add_css_class("osd");
        print_btn.add_css_class("preaching-overlay-btn");
        print_btn.set_valign(Align::Start);
        print_btn.set_halign(Align::End);
        print_btn.set_margin_top(16);
        print_btn.set_margin_end(64);
        print_btn.set_tooltip_text(Some("Print (Ctrl+P)"));

        let warm_btn = ToggleButton::new();
        warm_btn.set_icon_name("weather-clear-symbolic");
        warm_btn.add_css_class("flat");
        warm_btn.add_css_class("circular");
        warm_btn.add_css_class("osd");
        warm_btn.add_css_class("preaching-overlay-btn");
        warm_btn.set_valign(Align::Start);
        warm_btn.set_halign(Align::End);
        warm_btn.set_margin_top(16);
        warm_btn.set_margin_end(112);
        warm_btn.set_tooltip_text(Some("Warm background"));
        warm_btn.set_active(state.borrow().config.preaching_warm_bg);

        // One dot per movement — filled/enlarged for whichever movement the
        // viewport currently shows, doubling as a where-am-I-in-the-sermon
        // progress indicator and a scroll-synced "current movement" marker.
        let dots_box = GtkBox::new(Orientation::Horizontal, 0);
        dots_box.set_halign(Align::Center);
        dots_box.set_valign(Align::Start);
        dots_box.set_margin_top(18);
        let mut dots: Vec<GtkBox> = Vec::new();
        for (i, _) in sermon.movements.iter().enumerate() {
            let dot = GtkBox::new(Orientation::Horizontal, 0);
            dot.add_css_class("preaching-progress-dot");
            if i == 0 {
                dot.add_css_class("preaching-progress-dot-current");
            }
            dots_box.append(&dot);
            dots.push(dot);
        }

        let overlay = Overlay::new();
        overlay.set_child(Some(&scroll));
        overlay.add_overlay(&close_btn);
        overlay.add_overlay(&print_btn);
        overlay.add_overlay(&warm_btn);
        if !dots.is_empty() {
            overlay.add_overlay(&dots_box);
        }

        window.set_child(Some(&overlay));

        if !movement_headings.is_empty() {
            let content = content.clone();
            let dots = dots.clone();
            let heads = movement_headings.clone();
            scroll.vadjustment().connect_value_changed(move |adj| {
                let y = adj.value();
                let mut current = 0usize;
                for (i, lbl) in heads.iter().enumerate() {
                    if let Some((_, top)) = lbl.translate_coordinates(&content, 0.0, 0.0) {
                        if top <= y + 40.0 {
                            current = i;
                        }
                    }
                }
                for (i, dot) in dots.iter().enumerate() {
                    if i == current {
                        dot.add_css_class("preaching-progress-dot-current");
                    } else {
                        dot.remove_css_class("preaching-progress-dot-current");
                    }
                }
            });
        }

        {
            let window = window.clone();
            close_btn.connect_clicked(move |_| window.close());
        }
        {
            let window = window.clone();
            let state = state.clone();
            print_btn.connect_clicked(move |_| preaching_print::print_sermon(&window, &state));
        }
        {
            let window = window.clone();
            let state = state.clone();
            warm_btn.connect_toggled(move |btn| {
                let active = btn.is_active();
                if active {
                    window.add_css_class("warm");
                } else {
                    window.remove_css_class("warm");
                }
                let mut st = state.borrow_mut();
                st.config.preaching_warm_bg = active;
                let _ = st.config.save();
            });
        }
        {
            let for_key = window.clone();
            let state = state.clone();
            let key_ctl = gtk4::EventControllerKey::new();
            key_ctl.connect_key_pressed(move |_, key, _, modifiers| {
                if key == gtk4::gdk::Key::Escape {
                    for_key.close();
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::p && modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                    preaching_print::print_sermon(&for_key, &state);
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
