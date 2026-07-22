//! A single idea bar: grabber · number · click-to-edit text · idea/part tag
//! chips · expansion triangle (→ notes) — every idea is exactly one row
//! tall regardless of whether it's tagged. An untagged chip collapses to a
//! bare "+" icon rather than a ghosted placeholder pill, so untagged ideas
//! don't cost any more width than tagged ones beyond the icon itself. The
//! grabber only gets its `DragSource` wired by the caller (`editor.rs`, via
//! `ui::dnd::setup_drag_source`) — building it here would need the idea's
//! id and the shared `drag_active` flag, both of which `editor.rs` already
//! has in scope, so there's nothing this module would add by taking them
//! too.

use std::rc::Rc;

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
    pub idea_tag_popover: Rc<TagPopover>,
    pub part_tag_popover: Rc<TagPopover>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_idea_row(
    idea: &Idea,
    number: u32,
    idea_tag_census: &[(String, usize)],
    part_tag_census: &[(String, usize)],
    on_text_changed: impl Fn(String) + 'static,
    on_notes_changed: impl Fn(String) + 'static,
    on_idea_tag_changed: impl Fn(String) + 'static,
    on_part_tag_changed: impl Fn(String) + 'static,
    on_field_focus_out: impl Fn() + 'static,
    on_delete: impl Fn() + 'static,
    on_enter: impl Fn() + 'static,
    on_duplicate: impl Fn() + 'static,
    on_move: impl Fn(i32) + 'static,
    on_select: impl Fn(bool, bool) + 'static,
    on_toggle_tag_filter: impl Fn(String) + 'static,
    on_rename_idea_tag_everywhere: impl Fn(String, String) + 'static,
    on_rename_part_tag_everywhere: impl Fn(String, String) + 'static,
    on_promote: impl Fn() + 'static,
) -> IdeaRowWidgets {
    let on_toggle_tag_filter: Rc<dyn Fn(String)> = Rc::new(on_toggle_tag_filter);
    let root = GtkBox::new(Orientation::Vertical, 2);
    root.add_css_class("idea-row");

    let bar = GtkBox::new(Orientation::Horizontal, 8);
    bar.add_css_class("idea-bar");

    let grabber = dnd::drag_grabber("Drag to reorder");
    bar.append(&grabber);

    let number_label = Label::new(Some(&number.to_string()));
    number_label.add_css_class("idea-number");
    number_label.set_width_chars(2);
    number_label.set_justify(gtk4::Justification::Center);
    number_label.set_halign(Align::Center);
    number_label.set_valign(Align::Center);
    number_label.set_can_target(true);
    bar.append(&number_label);
    {
        // Ctrl/Shift-click on the number toggles/extends multi-selection.
        // A plain click is left alone (does nothing) so clicking the number
        // by habit doesn't surprise anyone — the number is otherwise inert.
        let click = gtk4::GestureClick::new();
        click.set_button(gtk4::gdk::BUTTON_PRIMARY);
        click.connect_pressed(move |gesture, _n_press, _x, _y| {
            let mods = gesture.current_event_state();
            let ctrl = mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
            let shift = mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK);
            if ctrl || shift {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                on_select(ctrl, shift);
            }
        });
        number_label.add_controller(click);
    }

    let entry = gtk4::Entry::new();
    entry.set_has_frame(false);
    entry.add_css_class("idea-entry");
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Idea… (type @ for a scripture citation)"));
    entry.set_text(&idea.text);
    bar.append(&entry);

    wire_citation_autocomplete(&entry);

    let idea_tag_popover = TagPopover::new("Idea tag…");
    idea_tag_popover.set_text(&idea.idea_tag);
    idea_tag_popover.set_census(idea_tag_census.to_vec());
    let idea_tag_btn = MenuButton::new();
    idea_tag_btn.add_css_class("idea-tag-chip");
    idea_tag_btn.add_css_class("idea-tag-chip-idea");
    idea_tag_btn.add_css_class("flat");
    refresh_tag_button(&idea_tag_btn, &idea.idea_tag, "Add idea tag");
    idea_tag_btn.set_popover(Some(idea_tag_popover.popover()));
    wire_tag_filter_toggle(&idea_tag_btn, idea_tag_popover.entry(), &on_toggle_tag_filter);
    idea_tag_popover.set_on_rename_everywhere(on_rename_idea_tag_everywhere);
    bar.append(&idea_tag_btn);

    let part_tag_popover = TagPopover::new("Part tag…");
    part_tag_popover.set_text(&idea.part_tag);
    part_tag_popover.set_census(part_tag_census.to_vec());
    let part_tag_btn = MenuButton::new();
    part_tag_btn.add_css_class("idea-tag-chip");
    part_tag_btn.add_css_class("idea-tag-chip-part");
    part_tag_btn.add_css_class("flat");
    refresh_tag_button(&part_tag_btn, &idea.part_tag, "Add part tag");
    part_tag_btn.set_popover(Some(part_tag_popover.popover()));
    wire_tag_filter_toggle(&part_tag_btn, part_tag_popover.entry(), &on_toggle_tag_filter);
    part_tag_popover.set_on_rename_everywhere(on_rename_part_tag_everywhere);
    bar.append(&part_tag_btn);

    let expander = ToggleButton::new();
    expander.set_icon_name(if idea.expanded {
        "pan-down-symbolic"
    } else {
        "pan-end-symbolic"
    });
    expander.add_css_class("flat");
    expander.add_css_class("idea-expander");
    // An idea with notes keeps its triangle visible as the only signal that
    // there's hidden content; an empty one fades with the row's other
    // occasional actions rather than implying there's something to expand.
    if idea.notes.trim().is_empty() {
        expander.add_css_class("idea-expander-empty");
    }
    expander.set_active(idea.expanded);
    expander.set_tooltip_text(Some("Expand notes"));
    bar.append(&expander);

    let duplicate_btn = Button::from_icon_name("edit-copy-symbolic");
    duplicate_btn.add_css_class("flat");
    duplicate_btn.add_css_class("idea-delete");
    duplicate_btn.set_tooltip_text(Some("Duplicate idea"));
    bar.append(&duplicate_btn);

    let delete_btn = Button::from_icon_name("user-trash-symbolic");
    delete_btn.add_css_class("flat");
    delete_btn.add_css_class("idea-delete");
    delete_btn.set_tooltip_text(Some("Delete idea"));
    bar.append(&delete_btn);

    root.append(&bar);

    // Notes revealer
    let notes_view = TextView::new();
    notes_view.add_css_class("idea-notes");
    notes_view.set_wrap_mode(WrapMode::WordChar);
    notes_view.set_top_margin(8);
    notes_view.set_bottom_margin(8);
    notes_view.set_left_margin(10);
    notes_view.set_right_margin(10);
    notes_view.set_margin_start(40);
    notes_view.set_margin_end(16);
    notes_view.set_margin_top(4);
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

    // Wiring
    entry.connect_changed(move |e| on_text_changed(e.text().to_string()));
    entry.connect_activate(move |_| on_enter());
    {
        let focus_ctl = gtk4::EventControllerFocus::new();
        focus_ctl.connect_leave(move |_| on_field_focus_out());
        entry.add_controller(focus_ctl);
    }
    {
        // Alt+Up/Alt+Down reorders this idea within its movement — a
        // keyboard alternative to dragging the grabber. Alt+Shift+Up/Down
        // jumps straight to the top/bottom instead of one step at a time;
        // `on_move` takes `i32::MIN`/`i32::MAX` as "to the very end" sentinels
        // rather than a literal delta (see `editor.rs`'s interpretation).
        // Alt+Left promotes the idea into its own movement, borrowing the
        // outliner convention where left/right change an item's level.
        let key_ctl = gtk4::EventControllerKey::new();
        key_ctl.connect_key_pressed(move |_, key, _, modifiers| {
            if !modifiers.contains(gtk4::gdk::ModifierType::ALT_MASK) {
                return glib::Propagation::Proceed;
            }
            let shift = modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK);
            match key {
                gtk4::gdk::Key::Up => {
                    on_move(if shift { i32::MIN } else { -1 });
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Down => {
                    on_move(if shift { i32::MAX } else { 1 });
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Left => {
                    on_promote();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        entry.add_controller(key_ctl);
    }
    {
        // Notes edits are non-structural (no rebuild), so the "has notes"
        // styling has to be maintained live rather than set once at build.
        let expander_for_notes = expander.clone();
        notes_view.buffer().connect_changed(move |buf| {
            let (start, end) = buf.bounds();
            let text = buf.text(&start, &end, false).to_string();
            if text.trim().is_empty() {
                expander_for_notes.add_css_class("idea-expander-empty");
            } else {
                expander_for_notes.remove_css_class("idea-expander-empty");
            }
            on_notes_changed(text);
        });
    }
    {
        let popover = idea_tag_popover.popover().clone();
        let btn = idea_tag_btn.clone();
        let entry = idea_tag_popover.entry().clone();
        entry.connect_activate(move |e| {
            refresh_tag_button(&btn, &e.text(), "Add idea tag");
            popover.popdown();
        });
        let btn2 = idea_tag_btn.clone();
        idea_tag_popover.entry().connect_changed(move |e| {
            let text = e.text();
            on_idea_tag_changed(text.to_string());
            refresh_tag_button(&btn2, &text, "Add idea tag");
        });
    }
    {
        let popover = part_tag_popover.popover().clone();
        let btn = part_tag_btn.clone();
        let entry = part_tag_popover.entry().clone();
        entry.connect_activate(move |e| {
            refresh_tag_button(&btn, &e.text(), "Add part tag");
            popover.popdown();
        });
        let btn2 = part_tag_btn.clone();
        part_tag_popover.entry().connect_changed(move |e| {
            let text = e.text();
            on_part_tag_changed(text.to_string());
            refresh_tag_button(&btn2, &text, "Add part tag");
        });
    }
    delete_btn.connect_clicked(move |_| on_delete());
    duplicate_btn.connect_clicked(move |_| on_duplicate());

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

/// Ctrl+click on a tag chip toggles the sermon-wide quick-filter for that
/// tag's current value instead of opening the chip's edit popover — claimed
/// only when Ctrl is held and the tag isn't empty, so a plain click still
/// opens the `MenuButton`'s popover exactly as before. Reads the tag text
/// live from `entry` at click time (not a value captured at row-build time)
/// since a tag edit doesn't trigger a rebuild (`SetIdeaTag` isn't
/// structural — see `commands.rs::Cmd::is_structural`), so a baked-in
/// closure would go stale the moment the tag was retyped.
fn wire_tag_filter_toggle(btn: &MenuButton, entry: &gtk4::Entry, on_toggle_tag_filter: &Rc<dyn Fn(String)>) {
    let click = gtk4::GestureClick::new();
    click.set_button(gtk4::gdk::BUTTON_PRIMARY);
    {
        let entry = entry.clone();
        let on_toggle_tag_filter = on_toggle_tag_filter.clone();
        click.connect_pressed(move |gesture, _n_press, _x, _y| {
            if !gesture
                .current_event_state()
                .contains(gtk4::gdk::ModifierType::CONTROL_MASK)
            {
                return;
            }
            let text = entry.text().to_string();
            if text.is_empty() {
                return;
            }
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            on_toggle_tag_filter(text);
        });
    }
    btn.add_controller(click);
}

/// Renders a tag chip as its colored text pill when set, or collapses it to
/// a bare "+" icon when empty — an untagged idea costs no more width than
/// the icon itself, instead of a ghosted placeholder pill the size of a real
/// tag. Uses `set_child` with a plain `Label`/`Image` rather than
/// `MenuButton`'s `set_label`/`set_icon_name` convenience methods — those
/// auto-append a dropdown-arrow indicator, which reads as visual clutter at
/// this chip's small size (a whole row of "tag ⌄" pills); a custom child
/// opts out of that indicator entirely.
/// Live book-name autocomplete for the `@book chapter:verse` citation
/// syntax (see `bible.rs`) — typing `@` then a partial book name pops a
/// filtered list; picking one completes `@BookName ` in place so the user
/// continues straight into typing chapter:verse. Independent of the
/// idea-text-changed wiring in `editor.rs`: this only manipulates the
/// entry's own text/cursor locally and never touches the model, so it
/// needs no callback out of this module.
fn wire_citation_autocomplete(entry: &gtk4::Entry) {
    let popover = gtk4::Popover::new();
    popover.set_parent(entry);
    popover.set_has_arrow(false);
    popover.set_autohide(false);
    let list = gtk4::ListBox::new();
    list.add_css_class("boxed-list");
    popover.set_child(Some(&list));

    {
        let entry = entry.clone();
        let popover = popover.clone();
        let list = list.clone();
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            let cursor_byte = char_pos_to_byte(&text, e.position());
            let Some((_start, _end, query)) = crate::bible::citation_query_at_cursor(&text, cursor_byte) else {
                popover.popdown();
                return;
            };
            let matches = crate::bible::search_books(&query);
            if matches.is_empty() {
                popover.popdown();
                return;
            }
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            for book in &matches {
                let row = gtk4::ListBoxRow::new();
                let label = Label::new(Some(book));
                label.set_xalign(0.0);
                label.set_margin_top(4);
                label.set_margin_bottom(4);
                label.set_margin_start(8);
                label.set_margin_end(8);
                row.set_child(Some(&label));
                list.append(&row);
            }
            popover.popup();
        });
    }
    {
        let entry = entry.clone();
        let popover = popover.clone();
        list.connect_row_activated(move |_, row| {
            let Some(label) = row.child().and_downcast::<Label>() else {
                return;
            };
            let book = label.text().to_string();
            let text = entry.text().to_string();
            let cursor_byte = char_pos_to_byte(&text, entry.position());
            if let Some((start, end, _query)) = crate::bible::citation_query_at_cursor(&text, cursor_byte) {
                let mut new_text = text.clone();
                new_text.replace_range(start..end, &format!("@{book} "));
                entry.set_text(&new_text);
                let new_cursor_byte = start + 1 + book.len() + 1;
                entry.set_position(byte_pos_to_char(&new_text, new_cursor_byte));
            }
            popover.popdown();
        });
    }
    {
        let key_ctl = gtk4::EventControllerKey::new();
        let popover = popover.clone();
        key_ctl.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape && popover.is_visible() {
                popover.popdown();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        entry.add_controller(key_ctl);
    }
}

fn char_pos_to_byte(text: &str, char_pos: i32) -> usize {
    if char_pos < 0 {
        return text.len();
    }
    text.char_indices()
        .nth(char_pos as usize)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

fn byte_pos_to_char(text: &str, byte_pos: usize) -> i32 {
    text[..byte_pos.min(text.len())].chars().count() as i32
}

fn refresh_tag_button(btn: &MenuButton, text: &str, tooltip_when_empty: &str) {
    if text.is_empty() {
        let icon = Image::from_icon_name("list-add-symbolic");
        icon.set_pixel_size(11);
        btn.set_child(Some(&icon));
        btn.set_tooltip_text(Some(tooltip_when_empty));
        btn.add_css_class("idea-tag-chip-empty");
    } else {
        let label = Label::new(Some(text));
        btn.set_child(Some(&label));
        btn.set_tooltip_text(Some(text));
        btn.remove_css_class("idea-tag-chip-empty");
    }
}
