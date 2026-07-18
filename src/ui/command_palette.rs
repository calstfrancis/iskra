//! Ctrl+K command palette: a modal search window listing commands plus, for
//! Iskra, every movement/idea in the open sermon as jump targets — doubling
//! as a document outline. Adapted from Zerkalo's `command_palette.rs`;
//! dispatch is plain string command IDs rather than an enum, matching that
//! module's "simpler to extend" rationale.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Entry, EventControllerKey, Label, ListBox, Orientation, PolicyType,
    PropagationPhase, ScrolledWindow, SelectionMode,
};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::model::Sermon;

#[derive(Clone)]
pub struct PaletteItem {
    pub id: String,
    pub label: String,
    pub subtitle: String,
}

#[derive(Clone)]
pub struct CommandPalette {
    window: adw::Window,
    entry: Entry,
    list: ListBox,
    items: Rc<RefCell<Vec<PaletteItem>>>,
    on_activate: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl CommandPalette {
    pub fn new(parent: &impl IsA<gtk4::Window>) -> Self {
        let window = adw::Window::builder()
            .transient_for(parent)
            .modal(true)
            .default_width(520)
            .default_height(420)
            .title("Command Palette")
            .resizable(false)
            .build();
        window.set_hide_on_close(true);

        let header = adw::HeaderBar::new();
        header.set_show_end_title_buttons(false);
        header.set_show_start_title_buttons(false);

        let entry = Entry::new();
        entry.set_placeholder_text(Some("Search commands, movements, ideas…"));
        entry.set_hexpand(true);
        header.set_title_widget(Some(&entry));

        let list = ListBox::new();
        list.set_selection_mode(SelectionMode::Browse);
        list.add_css_class("navigation-sidebar");

        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_child(Some(&list));
        scroll.set_vexpand(true);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&scroll));
        window.set_content(Some(&toolbar));

        let items: Rc<RefCell<Vec<PaletteItem>>> = Rc::new(RefCell::new(Vec::new()));
        let on_activate: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        {
            let list_c = list.clone();
            let items_c = items.clone();
            entry.connect_changed(move |e| {
                let query = e.text().to_lowercase();
                rebuild_list(&list_c, &items_c.borrow(), &query);
            });
        }

        {
            let win_c = window.clone();
            let list_c = list.clone();
            let on_act = on_activate.clone();
            entry.connect_activate(move |_| {
                activate_selected(&list_c, &on_act, &win_c);
            });
        }

        {
            let win_c = window.clone();
            let on_act = on_activate.clone();
            list.connect_row_activated(move |_, row| {
                let id = row.widget_name().to_string();
                if !id.is_empty() {
                    if let Some(f) = on_act.borrow().as_ref() {
                        f(&id);
                    }
                    win_c.close();
                }
            });
        }

        {
            let list_c = list.clone();
            let kc = EventControllerKey::new();
            kc.set_propagation_phase(PropagationPhase::Capture);
            kc.connect_key_pressed(move |_, key, _, _| {
                use gtk4::gdk::Key;
                match key {
                    Key::Down => {
                        move_selection(&list_c, 1);
                        glib::Propagation::Stop
                    }
                    Key::Up => {
                        move_selection(&list_c, -1);
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            });
            entry.add_controller(kc);
        }

        {
            let win_c = window.clone();
            let kc2 = EventControllerKey::new();
            kc2.connect_key_pressed(move |_, key, _, _| {
                if key == gtk4::gdk::Key::Escape {
                    win_c.close();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            });
            window.add_controller(kc2);
        }

        Self {
            window,
            entry,
            list,
            items,
            on_activate,
        }
    }

    pub fn set_on_activate(&self, f: impl Fn(&str) + 'static) {
        *self.on_activate.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_items(&self, items: Vec<PaletteItem>) {
        *self.items.borrow_mut() = items;
    }

    pub fn show(&self) {
        self.entry.set_text("");
        let query = String::new();
        rebuild_list(&self.list, &self.items.borrow(), &query);
        self.window.present();
        self.entry.grab_focus();
        if let Some(row) = self.list.row_at_index(0) {
            self.list.select_row(Some(&row));
        }
    }
}

fn rebuild_list(list: &ListBox, items: &[PaletteItem], query: &str) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let mut first = true;
    for item in items {
        if !query.is_empty()
            && !item.label.to_lowercase().contains(query)
            && !item.subtitle.to_lowercase().contains(query)
        {
            continue;
        }

        let row = make_row(item, query);
        list.append(&row);

        if first {
            list.select_row(Some(&row));
            first = false;
        }
    }

    if first {
        let row = gtk4::ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(false);
        let lbl = Label::new(Some("No matching commands"));
        lbl.add_css_class("dim-label");
        lbl.set_margin_top(12);
        lbl.set_margin_bottom(12);
        row.set_child(Some(&lbl));
        list.append(&row);
    }
}

fn highlight_match(text: &str, query: &str) -> String {
    if query.is_empty() {
        return glib::markup_escape_text(text).to_string();
    }
    let lower_text = text.to_lowercase();
    if let Some(start) = lower_text.find(query) {
        let end = start + query.len();

        // `start`/`end` are byte offsets into `lower_text`, not `text` — case
        // folding can change a character's byte length, so they aren't
        // guaranteed to land on char boundaries in the original. Clamp into
        // range, then snap outward to the nearest valid boundary — slicing
        // at a non-boundary offset panics.
        let mut start = start.min(text.len());
        let mut end = end.min(text.len());
        while start > 0 && !text.is_char_boundary(start) {
            start -= 1;
        }
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        if end < start {
            end = start;
        }

        let prefix = glib::markup_escape_text(&text[..start]);
        let matched = glib::markup_escape_text(&text[start..end]);
        let suffix = glib::markup_escape_text(&text[end..]);
        format!("{prefix}<b>{matched}</b>{suffix}")
    } else {
        glib::markup_escape_text(text).to_string()
    }
}

fn make_row(item: &PaletteItem, query: &str) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();
    row.set_widget_name(&item.id);

    let hbox = GtkBox::new(Orientation::Horizontal, 8);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);

    let vbox = GtkBox::new(Orientation::Vertical, 2);
    vbox.set_hexpand(true);

    let lbl = Label::new(None);
    lbl.set_use_markup(true);
    lbl.set_markup(&highlight_match(&item.label, query));
    lbl.set_xalign(0.0);
    vbox.append(&lbl);

    if !item.subtitle.is_empty() {
        let sub = Label::new(None);
        sub.set_use_markup(true);
        sub.set_markup(&highlight_match(&item.subtitle, query));
        sub.set_xalign(0.0);
        sub.add_css_class("dim-label");
        sub.add_css_class("caption");
        vbox.append(&sub);
    }

    hbox.append(&vbox);
    row.set_child(Some(&hbox));
    row
}

fn activate_selected(
    list: &ListBox,
    on_act: &Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    win: &adw::Window,
) {
    if let Some(row) = list.selected_row() {
        let id = row.widget_name().to_string();
        if !id.is_empty() {
            if let Some(f) = on_act.borrow().as_ref() {
                f(&id);
            }
        }
        win.close();
    }
}

fn move_selection(list: &ListBox, delta: i32) {
    let current = list.selected_row().map(|r| r.index()).unwrap_or(0);
    let next = (current + delta).max(0);
    if let Some(row) = list.row_at_index(next) {
        list.select_row(Some(&row));
        row.grab_focus();
    }
}

// ── Iskra command items ─────────────────────────────────────────────────

pub fn default_commands() -> Vec<PaletteItem> {
    vec![
        PaletteItem {
            id: "new_sermon".into(),
            label: "New Sermon".into(),
            subtitle: "Start a new sermon".into(),
        },
        PaletteItem {
            id: "open_library".into(),
            label: "Open Library…".into(),
            subtitle: "Search and open a sermon (Ctrl+L)".into(),
        },
        PaletteItem {
            id: "export".into(),
            label: "Export…".into(),
            subtitle: "Export as a .sermon file for Rubric (Ctrl+E)".into(),
        },
        PaletteItem {
            id: "preaching_view".into(),
            label: "Preaching View".into(),
            subtitle: "Large-print, chrome-free pulpit display (Ctrl+Shift+P)".into(),
        },
        PaletteItem {
            id: "history".into(),
            label: "History…".into(),
            subtitle: "Browse and restore past committed versions (Ctrl+Shift+H)".into(),
        },
        PaletteItem {
            id: "undo".into(),
            label: "Undo".into(),
            subtitle: "Undo the last change (Ctrl+Z)".into(),
        },
        PaletteItem {
            id: "redo".into(),
            label: "Redo".into(),
            subtitle: "Redo the last undone change (Ctrl+Shift+Z)".into(),
        },
        PaletteItem {
            id: "toggle_sidebar".into(),
            label: "Toggle Sidebar".into(),
            subtitle: "Show or hide the lectionary sidebar".into(),
        },
        PaletteItem {
            id: "add_movement".into(),
            label: "Add Movement".into(),
            subtitle: "Append a new movement to the sermon".into(),
        },
        PaletteItem {
            id: "changelog".into(),
            label: "Changelog".into(),
            subtitle: "View what's changed in this version".into(),
        },
    ]
}

/// One item per movement/idea in `sermon`, doubling as a document outline —
/// jumping focuses that movement's name entry or idea's text entry.
pub fn outline_items(sermon: &Sermon) -> Vec<PaletteItem> {
    let mut out = Vec::new();
    for movement in &sermon.movements {
        out.push(PaletteItem {
            id: format!("movement:{}", movement.id),
            label: format!("≡ {}", movement.name),
            subtitle: format!("{} idea(s)", movement.ideas.len()),
        });
        for idea in &movement.ideas {
            let label = if idea.text.is_empty() {
                "(untitled idea)".to_string()
            } else {
                idea.text.clone()
            };
            out.push(PaletteItem {
                id: format!("idea:{}", idea.id),
                label: format!("  · {label}"),
                subtitle: movement.name.clone(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_match_wraps_the_match_in_bold() {
        let out = highlight_match("hello world", "world");
        assert_eq!(out, "hello <b>world</b>");
    }

    #[test]
    fn highlight_match_does_not_panic_when_lowercasing_shifts_byte_offsets_mid_character() {
        let text = "stanİ日";
        let out = highlight_match(text, "stan");
        assert!(out.contains("<b>"), "should still produce a highlighted result: {out}");
    }

    #[test]
    fn highlight_match_returns_escaped_text_unchanged_when_no_match() {
        let out = highlight_match("<tag> hello", "zzz");
        assert_eq!(out, "&lt;tag&gt; hello");
    }

    #[test]
    fn outline_items_lists_movements_and_ideas_in_order() {
        use crate::model::{Idea, Movement};
        let mut s = Sermon::new();
        s.movements.clear();
        let mut m = Movement::new(0);
        m.name = "Movement One".into();
        let mut idea = Idea::new();
        idea.text = "First idea".into();
        m.ideas.push(idea);
        s.movements.push(m);

        let items = outline_items(&s);
        assert_eq!(items.len(), 2);
        assert!(items[0].label.contains("Movement One"));
        assert!(items[1].label.contains("First idea"));
    }
}
