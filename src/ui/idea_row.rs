//! A single idea bar: number · click-to-edit text · expansion triangle
//! (→ notes) · grabber, with idea/part tag tabs hanging below. The grabber
//! only gets its `DragSource` wired by the caller (`editor.rs`, via
//! `ui::dnd::setup_drag_source`) — building it here would need the idea's
//! id and the shared `drag_active` flag, both of which `editor.rs` already
//! has in scope, so there's nothing this module would add by taking them
//! too.

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Image, Label, MenuButton, Orientation, Revealer,
    RevealerTransitionType, TextView, ToggleButton, WrapMode,
};

use crate::model::Idea;
use crate::ui::dnd;
use crate::ui::tag_popover::TagPopover;

pub struct IdeaRowWidgets {
    pub root: GtkBox,
    pub entry: gtk4::Entry,
    pub notes_view: TextView,
    pub expander: ToggleButton,
    pub grabber: Image,
    pub idea_tag_popover: TagPopover,
    pub part_tag_popover: TagPopover,
}

#[allow(clippy::too_many_arguments)]
pub fn build_idea_row(
    idea: &Idea,
    number: u32,
    on_text_changed: impl Fn(String) + 'static,
    on_notes_changed: impl Fn(String) + 'static,
    on_idea_tag_changed: impl Fn(String) + 'static,
    on_part_tag_changed: impl Fn(String) + 'static,
    on_field_focus_out: impl Fn() + 'static,
    on_delete: impl Fn() + 'static,
) -> IdeaRowWidgets {
    let root = GtkBox::new(Orientation::Vertical, 2);
    root.add_css_class("idea-row");

    let bar = GtkBox::new(Orientation::Horizontal, 6);
    bar.set_margin_top(2);
    bar.set_margin_bottom(2);

    let number_label = Label::new(Some(&format!("{number}.")));
    number_label.add_css_class("idea-number");
    number_label.set_xalign(1.0);
    bar.append(&number_label);

    let entry = gtk4::Entry::new();
    entry.set_has_frame(false);
    entry.add_css_class("idea-entry");
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Idea…"));
    entry.set_text(&idea.text);
    bar.append(&entry);

    let expander = ToggleButton::new();
    expander.set_icon_name(if idea.expanded {
        "pan-down-symbolic"
    } else {
        "pan-end-symbolic"
    });
    expander.add_css_class("flat");
    expander.set_active(idea.expanded);
    expander.set_tooltip_text(Some("Expand notes"));
    bar.append(&expander);

    let delete_btn = Button::from_icon_name("user-trash-symbolic");
    delete_btn.add_css_class("flat");
    delete_btn.set_tooltip_text(Some("Delete idea"));
    bar.append(&delete_btn);

    let grabber = dnd::drag_grabber("Drag to reorder");
    bar.append(&grabber);

    root.append(&bar);

    // Notes revealer
    let notes_view = TextView::new();
    notes_view.set_wrap_mode(WrapMode::WordChar);
    notes_view.set_top_margin(4);
    notes_view.set_bottom_margin(4);
    notes_view.set_left_margin(36);
    notes_view.set_right_margin(8);
    notes_view.buffer().set_text(&idea.notes);

    let notes_revealer = Revealer::new();
    notes_revealer.set_transition_type(RevealerTransitionType::SlideDown);
    notes_revealer.set_child(Some(&notes_view));
    notes_revealer.set_reveal_child(idea.expanded);
    root.append(&notes_revealer);

    {
        let notes_revealer = notes_revealer.clone();
        expander.connect_toggled(move |btn| {
            let active = btn.is_active();
            btn.set_icon_name(if active {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            });
            notes_revealer.set_reveal_child(active);
        });
    }

    // Tag tabs
    let tags_row = GtkBox::new(Orientation::Horizontal, 4);
    tags_row.set_margin_start(36);
    tags_row.set_halign(Align::Start);

    let idea_tag_popover = TagPopover::new("Idea tag…");
    idea_tag_popover.set_text(&idea.idea_tag);
    let idea_tag_btn = MenuButton::new();
    idea_tag_btn.add_css_class("tag-tab");
    idea_tag_btn.add_css_class("flat");
    idea_tag_btn.set_label(if idea.idea_tag.is_empty() {
        "idea"
    } else {
        &idea.idea_tag
    });
    idea_tag_btn.set_popover(Some(idea_tag_popover.popover()));
    tags_row.append(&idea_tag_btn);

    let part_tag_popover = TagPopover::new("Part tag…");
    part_tag_popover.set_text(&idea.part_tag);
    let part_tag_btn = MenuButton::new();
    part_tag_btn.add_css_class("tag-tab");
    part_tag_btn.add_css_class("flat");
    part_tag_btn.set_label(if idea.part_tag.is_empty() {
        "part"
    } else {
        &idea.part_tag
    });
    part_tag_btn.set_popover(Some(part_tag_popover.popover()));
    tags_row.append(&part_tag_btn);

    root.append(&tags_row);

    // Wiring
    entry.connect_changed(move |e| on_text_changed(e.text().to_string()));
    {
        let focus_ctl = gtk4::EventControllerFocus::new();
        focus_ctl.connect_leave(move |_| on_field_focus_out());
        entry.add_controller(focus_ctl);
    }
    notes_view.buffer().connect_changed(move |buf| {
        let (start, end) = buf.bounds();
        on_notes_changed(buf.text(&start, &end, false).to_string());
    });
    {
        let popover = idea_tag_popover.popover().clone();
        let btn = idea_tag_btn.clone();
        let entry = idea_tag_popover.entry().clone();
        entry.connect_activate(move |e| {
            let text = e.text();
            btn.set_label(if text.is_empty() { "idea" } else { &text });
            popover.popdown();
        });
        let btn2 = idea_tag_btn.clone();
        idea_tag_popover.entry().connect_changed(move |e| {
            let text = e.text();
            on_idea_tag_changed(text.to_string());
            btn2.set_label(if text.is_empty() { "idea" } else { &text });
        });
    }
    {
        let popover = part_tag_popover.popover().clone();
        let btn = part_tag_btn.clone();
        let entry = part_tag_popover.entry().clone();
        entry.connect_activate(move |e| {
            let text = e.text();
            btn.set_label(if text.is_empty() { "part" } else { &text });
            popover.popdown();
        });
        let btn2 = part_tag_btn.clone();
        part_tag_popover.entry().connect_changed(move |e| {
            let text = e.text();
            on_part_tag_changed(text.to_string());
            btn2.set_label(if text.is_empty() { "part" } else { &text });
        });
    }
    delete_btn.connect_clicked(move |_| on_delete());

    IdeaRowWidgets {
        root,
        entry,
        notes_view,
        expander,
        grabber,
        idea_tag_popover,
        part_tag_popover,
    }
}
