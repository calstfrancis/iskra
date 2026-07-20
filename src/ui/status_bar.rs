//! Bottom status bar: sermon-level tag chips (scripture "s." tags and theme
//! "t." tags — distinct from the per-idea idea/part tags on each idea row,
//! see Plans/iskra-kickoff-prompt.md §4.6) and the version indicator on the
//! far right, which opens the changelog window.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Image, Label, MenuButton, Orientation, Paned, Popover, ScrolledWindow, Separator};

use crate::commands::{Cmd, SermonTagKind};
use crate::model::{Idea, Movement, Sermon};
use crate::state::AppState;
use crate::ui::editor::ApplyFn;

const MAX_RECENT_DELETIONS: usize = 20;

/// One deleted idea or movement, kept around for the "Recently deleted"
/// tray — a session-scoped safety net alongside undo (never persisted, lost
/// on quit), for the case of noticing a delete several edits later, where
/// plain Ctrl+Z would have to unwind everything since. Carries the same
/// data `Cmd::DeleteIdea`/`Cmd::DeleteMovement` already carry for their own
/// undo inversion (see `commands.rs`), reused here rather than duplicated.
pub enum DeletedEntry {
    Idea { movement: usize, index: usize, idea: Idea },
    Movement { at: usize, movement: Movement },
}

impl DeletedEntry {
    fn label(&self) -> String {
        match self {
            DeletedEntry::Idea { idea, .. } if !idea.text.is_empty() => idea.text.clone(),
            DeletedEntry::Idea { .. } => "(untitled idea)".to_string(),
            DeletedEntry::Movement { movement, .. } if !movement.name.is_empty() => {
                format!("Movement: {}", movement.name)
            }
            DeletedEntry::Movement { .. } => "(untitled movement)".to_string(),
        }
    }
}

pub struct StatusBar {
    pub root: GtkBox,
    pub version_btn: Button,
    pub simple_toggle: Button,
    s_tags_box: GtkBox,
    t_tags_box: GtkBox,
    saved_label: Label,
    recent_btn: MenuButton,
    recent_popover: Popover,
    recent_list: GtkBox,
    recent_deletions: RefCell<VecDeque<DeletedEntry>>,
    apply: RefCell<Option<ApplyFn>>,
    state: RefCell<Option<Rc<RefCell<AppState>>>>,
}

impl StatusBar {
    pub fn new() -> Rc<Self> {
        let root = GtkBox::new(Orientation::Horizontal, 8);
        root.set_margin_top(4);
        root.set_margin_bottom(4);
        root.set_margin_start(10);
        root.set_margin_end(10);

        // A draggable divider between the two tag sections, not a fixed
        // split — Scripture and Theme tags compete for the same row, and a
        // sermon can lean heavily one way (many citations, few themes, or
        // vice versa). Each side wraps its tag box in a plain `ScrolledWindow`
        // that just fills whatever width the `Paned` gives it (no
        // `propagate_natural_width` — that combination was the previous
        // approach and it visibly broke: every chip rendered as a bare "…"
        // with none of its actual text, because the box's width request
        // collapsed instead of taking its natural content size).
        let s_group = tag_group_label("bookmark-new-symbolic", "Scripture", "Scripture tags");
        let s_tags_box = GtkBox::new(Orientation::Horizontal, 4);
        let s_side = GtkBox::new(Orientation::Horizontal, 6);
        s_side.append(&s_group);
        s_side.append(&scroll_tags_box(&s_tags_box));

        let t_group = tag_group_label("emblem-favorite-symbolic", "Themes", "Theme tags");
        let t_tags_box = GtkBox::new(Orientation::Horizontal, 4);
        let t_side = GtkBox::new(Orientation::Horizontal, 6);
        t_side.append(&t_group);
        t_side.append(&scroll_tags_box(&t_tags_box));

        // `hexpand(true)` on the Paned (not on a separate spacer after it) —
        // an earlier version had the Paned fixed-width with a `hexpand`
        // spacer next to it, which is exactly backwards: the spacer ate all
        // the bottom bar's real width while the tag area stayed pinned to
        // its narrow initial size, "bound by the sidebar" as reported.
        // `resize_end_child(true)` means the Scripture side keeps its
        // dragged width when the window resizes and the Themes side
        // absorbs the extra space — both remain manually resizable via the
        // handle regardless.
        let tags_paned = Paned::new(Orientation::Horizontal);
        tags_paned.set_start_child(Some(&s_side));
        tags_paned.set_end_child(Some(&t_side));
        tags_paned.set_resize_start_child(false);
        tags_paned.set_resize_end_child(true);
        tags_paned.set_shrink_start_child(true);
        tags_paned.set_shrink_end_child(true);
        tags_paned.set_position(320);
        tags_paned.set_hexpand(true);
        root.append(&tags_paned);

        let saved_label = Label::new(Some("Saved"));
        saved_label.add_css_class("dim-label");
        saved_label.add_css_class("caption");
        root.append(&saved_label);

        let recent_btn = MenuButton::new();
        recent_btn.set_icon_name("edit-undo-symbolic");
        recent_btn.add_css_class("flat");
        recent_btn.add_css_class("dim-label");
        recent_btn.set_tooltip_text(Some("Recently deleted"));
        recent_btn.set_visible(false);

        let recent_list = GtkBox::new(Orientation::Vertical, 2);
        recent_list.set_margin_top(6);
        recent_list.set_margin_bottom(6);
        recent_list.set_margin_start(6);
        recent_list.set_margin_end(6);
        let recent_popover = Popover::new();
        recent_popover.set_child(Some(&recent_list));
        recent_btn.set_popover(Some(&recent_popover));
        root.append(&recent_btn);

        // Simple Mode toggle — bold when ON (picker hidden), regular when
        // OFF (picker shown in the lectionary sidebar). Same name-as-label,
        // font-weight-only convention as every other status-bar toggle.
        let simple_toggle = Button::with_label("simple");
        simple_toggle.add_css_class("flat");
        simple_toggle.add_css_class("caption");
        simple_toggle.add_css_class("status-toggle");
        simple_toggle.set_tooltip_text(Some(
            "Simple Mode hides the lectionary/track picker — turn off to switch lectionaries",
        ));
        root.append(&simple_toggle);

        let sep = Separator::new(Orientation::Vertical);
        sep.add_css_class("statusbar-sep");
        root.append(&sep);

        let version_btn = Button::with_label(&format!("v{}", env!("CARGO_PKG_VERSION")));
        version_btn.add_css_class("flat");
        version_btn.add_css_class("dim-label");
        version_btn.add_css_class("caption");
        version_btn.set_tooltip_text(Some("View changelog"));
        root.append(&version_btn);

        Rc::new(Self {
            root,
            version_btn,
            simple_toggle,
            s_tags_box,
            t_tags_box,
            saved_label,
            recent_btn,
            recent_popover,
            recent_list,
            recent_deletions: RefCell::new(VecDeque::new()),
            apply: RefCell::new(None),
            state: RefCell::new(None),
        })
    }

    /// Stores `apply` and `state` so tag chips built by `refresh` can route
    /// add/remove through the single door, and so the "Recently deleted"
    /// tray can clamp a restore's target indices against the sermon's
    /// *current* shape (structural edits since the deletion may have moved
    /// or removed the movement it originally lived in). Call once, after
    /// both exist.
    pub fn init(&self, apply: ApplyFn, state: Rc<RefCell<AppState>>) {
        *self.apply.borrow_mut() = Some(apply);
        *self.state.borrow_mut() = Some(state);
    }

    pub fn refresh(&self, sermon: &Sermon) {
        let apply = self.apply.borrow().clone();
        let Some(apply) = apply else { return };
        rebuild_tag_group(&self.s_tags_box, &sermon.s_tags, SermonTagKind::S, &apply);
        rebuild_tag_group(&self.t_tags_box, &sermon.t_tags, SermonTagKind::T, &apply);
    }

    /// Called the instant a change is made, before the debounced autosave
    /// actually runs — so "Unsaved changes" shows immediately rather than
    /// lagging behind the edit by the autosave debounce window.
    pub fn set_dirty(&self) {
        self.saved_label.set_text("● Unsaved changes");
    }

    /// Called when autosave completes successfully.
    pub fn set_saved(&self) {
        let now = chrono::Local::now().format("%-I:%M %p");
        self.saved_label.set_text(&format!("Saved {now}"));
    }

    /// Bold when Simple Mode is on (picker hidden), regular when off.
    pub fn set_simple_mode(&self, on: bool) {
        if on {
            self.simple_toggle.add_css_class("status-toggle-on");
        } else {
            self.simple_toggle.remove_css_class("status-toggle-on");
        }
    }

    /// Called from `app_window.rs::make_apply` for every applied `Cmd` that
    /// contained at least one `DeleteIdea`/`DeleteMovement` (see
    /// `collect_deletions`). Newest first, capped at `MAX_RECENT_DELETIONS`.
    pub fn record_deletions(self: &Rc<Self>, entries: Vec<DeletedEntry>) {
        if entries.is_empty() {
            return;
        }
        {
            let mut list = self.recent_deletions.borrow_mut();
            for entry in entries {
                list.push_front(entry);
            }
            while list.len() > MAX_RECENT_DELETIONS {
                list.pop_back();
            }
        }
        self.refresh_recent_list();
    }

    fn refresh_recent_list(self: &Rc<Self>) {
        while let Some(child) = self.recent_list.first_child() {
            self.recent_list.remove(&child);
        }
        let count = self.recent_deletions.borrow().len();
        self.recent_btn.set_visible(count > 0);
        for idx in 0..count {
            let label_text = self.recent_deletions.borrow()[idx].label();
            let row = GtkBox::new(Orientation::Horizontal, 6);
            let label = Label::new(Some(&label_text));
            label.set_xalign(0.0);
            label.set_hexpand(true);
            label.set_max_width_chars(28);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            row.append(&label);
            let restore_btn = Button::with_label("Restore");
            restore_btn.add_css_class("flat");
            {
                let this = self.clone();
                restore_btn.connect_clicked(move |_| this.restore_deletion(idx));
            }
            row.append(&restore_btn);
            self.recent_list.append(&row);
        }
    }

    /// Re-inserts the entry at `idx`, clamping its target movement/index
    /// against the sermon's current shape — structural edits since the
    /// deletion (movements added/removed/reordered) may have made the
    /// original position no longer valid, and `Cmd::InsertIdea`/
    /// `InsertMovement` index straight into a `Vec` with no bounds check of
    /// their own (see `commands.rs::Cmd::apply_to`), so an un-clamped
    /// restore of a stale entry could panic instead of just landing
    /// somewhere slightly different.
    fn restore_deletion(self: &Rc<Self>, idx: usize) {
        let (Some(apply), Some(state)) = (self.apply.borrow().clone(), self.state.borrow().clone()) else {
            return;
        };
        let entry = {
            let mut list = self.recent_deletions.borrow_mut();
            if idx >= list.len() {
                return;
            }
            list.remove(idx)
        };
        if let Some(entry) = entry {
            let movements_len = state.borrow().sermon.movements.len();
            let cmd = match entry {
                DeletedEntry::Idea { .. } if movements_len == 0 => {
                    // Nowhere left to put it back — every movement was
                    // deleted since. Drop the restore rather than crash.
                    self.refresh_recent_list();
                    self.recent_popover.popdown();
                    return;
                }
                DeletedEntry::Idea { movement, index, idea } => {
                    let movement = movement.min(movements_len - 1);
                    let idea_count = state
                        .borrow()
                        .sermon
                        .movements
                        .get(movement)
                        .map(|m| m.ideas.len())
                        .unwrap_or(0);
                    Cmd::InsertIdea {
                        movement,
                        index: index.min(idea_count),
                        idea,
                    }
                }
                DeletedEntry::Movement { at, movement } => Cmd::InsertMovement {
                    at: at.min(movements_len),
                    movement,
                },
            };
            apply(cmd);
        }
        self.refresh_recent_list();
        self.recent_popover.popdown();
    }
}

/// Wraps a tag-chip row so a long list of tags scrolls horizontally within
/// whatever width the enclosing `Paned` side gives it, rather than forcing
/// the status bar (and the whole window) ever wider — nothing capped this
/// at all before, and scripture citations (`editor.rs` adds one tag
/// automatically per completed `@citation`, no manual "is this too many"
/// pause) made it easy to accumulate enough tags to matter. Deliberately
/// *not* `set_propagate_natural_width` — that mode asks the `ScrolledWindow`
/// to size itself to its content's natural width (capped), which for an
/// hexpanding child inside a horizontal `Box` collapsed the whole row's
/// width request instead, and every chip rendered as a bare ellipsis with
/// none of its actual text. A plain `ScrolledWindow` just fills the space
/// its parent (the `Paned`'s start/end slot) allocates it, which is the
/// well-tested default mode.
fn scroll_tags_box(tags_box: &GtkBox) -> ScrolledWindow {
    let scroller = ScrolledWindow::new();
    scroller.set_child(Some(tags_box));
    scroller.set_vscrollbar_policy(gtk4::PolicyType::Never);
    scroller.set_hscrollbar_policy(gtk4::PolicyType::Automatic);
    scroller.set_hexpand(true);
    scroller.set_valign(gtk4::Align::Center);
    scroller
}

/// A small icon + dim caption label pair, for the "Scripture"/"Themes"
/// group headers — the words alone read as easy-to-skim-past text at
/// caption size, an icon gives the eye something to anchor on first.
fn tag_group_label(icon_name: &str, text: &str, tooltip: &str) -> GtkBox {
    let group = GtkBox::new(Orientation::Horizontal, 4);
    group.set_tooltip_text(Some(tooltip));
    let icon = Image::from_icon_name(icon_name);
    icon.set_pixel_size(12);
    icon.add_css_class("dim-label");
    group.append(&icon);
    let label = Label::new(Some(text));
    label.add_css_class("dim-label");
    label.add_css_class("caption");
    group.append(&label);
    group
}

fn rebuild_tag_group(container: &GtkBox, tags: &[String], kind: SermonTagKind, apply: &ApplyFn) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    for tag in tags {
        let chip = GtkBox::new(Orientation::Horizontal, 4);
        chip.add_css_class("tag-chip");
        chip.add_css_class(match kind {
            SermonTagKind::S => "tag-chip-s",
            SermonTagKind::T => "tag-chip-t",
        });

        // No ellipsize/max-width here — a GTK `Label` with ellipsize enabled
        // reports its *minimum* size as just the ellipsis glyph, not the
        // capped-but-natural text width, and inside a non-expanding `Box`
        // that collapsed the whole chip to a bare "…" with none of the
        // actual tag text ever showing, regardless of how much room the
        // surrounding `ScrolledWindow`/`Paned` actually had. Showing the
        // full tag and letting `scroll_tags_box`'s horizontal scrollbar
        // handle overflow is the tradeoff that's actually legible.
        let label = Label::new(Some(tag));
        chip.append(&label);

        let remove_icon = Image::from_icon_name("window-close-symbolic");
        remove_icon.set_pixel_size(10);
        let remove_btn = Button::new();
        remove_btn.set_child(Some(&remove_icon));
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("tag-chip-remove");
        remove_btn.set_valign(gtk4::Align::Center);
        remove_btn.set_tooltip_text(Some("Remove tag"));
        {
            let old: Vec<String> = tags.to_vec();
            let tag = tag.clone();
            let apply = apply.clone();
            remove_btn.connect_clicked(move |_| {
                let mut new = old.clone();
                new.retain(|t| t != &tag);
                apply(Cmd::SetSermonTags {
                    kind,
                    old: old.clone(),
                    new,
                });
            });
        }
        chip.append(&remove_btn);

        container.append(&chip);
    }

    let add_btn = MenuButton::new();
    add_btn.set_icon_name("list-add-symbolic");
    add_btn.add_css_class("flat");
    add_btn.set_tooltip_text(Some("Add tag"));

    let entry = Entry::new();
    entry.set_placeholder_text(Some("New tag…"));
    entry.set_width_chars(14);
    let popover = Popover::new();
    popover.set_child(Some(&entry));
    add_btn.set_popover(Some(&popover));

    {
        let old: Vec<String> = tags.to_vec();
        let apply = apply.clone();
        let popover = popover.clone();
        entry.connect_activate(move |e| {
            let text = e.text().trim().to_string();
            if !text.is_empty() && !old.contains(&text) {
                let mut new = old.clone();
                new.push(text);
                apply(Cmd::SetSermonTags {
                    kind,
                    old: old.clone(),
                    new,
                });
            }
            e.set_text("");
            popover.popdown();
        });
    }

    container.append(&add_btn);
}
