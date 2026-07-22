//! The movements-and-ideas editor. Every mutation triggers a full rebuild
//! from the model rather than a patched-in-place update: numbering is global
//! and undo can restore arbitrary structure, so one rebuild path is simpler
//! than shadowing it with fine-grained widget updates.
//!
//! Drag-and-drop uses a single `DropTarget` on the movements column (see
//! `ui::dnd` and `examples/dnd_proto.rs` for why one target, not one per
//! movement), set up once in [`Editor::init_dnd`] rather than rebuilt every
//! time — the column's controllers survive `rebuild()` clearing its
//! children, since controllers attach to the widget instance, not its
//! child list.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, DropTarget, Label, Orientation, ScrolledWindow};
use libadwaita as adw;

use crate::bible;
use crate::commands::{Cmd, SermonTagKind, TagKind};
use crate::model::{Idea, Movement, Sermon};
use crate::state::AppState;
use crate::ui::dnd::{self, DropZone};
use crate::ui::idea_row::build_idea_row;
use crate::ui::movement_card::build_movement_card;

pub struct Editor {
    scroller: ScrolledWindow,
    column: GtkBox,
    indicator: GtkBox,
    drag_active: Rc<Cell<bool>>,
    selected: Rc<RefCell<HashSet<String>>>,
    last_selected: Rc<RefCell<Option<String>>>,
    active_tag_filter: Rc<RefCell<Option<String>>>,
    /// Last idea entry to take focus. The command palette takes focus itself
    /// when it opens, so a palette command acting on "the current idea" has
    /// to consult this rather than asking GTK what's focused right now.
    last_focused_idea: Rc<RefCell<Option<String>>>,
    on_copy_movement: RefCell<Option<Box<dyn Fn(Movement)>>>,
}

pub type ApplyFn = Rc<dyn Fn(Cmd)>;

impl Editor {
    pub fn new() -> Rc<Self> {
        let column = GtkBox::new(Orientation::Vertical, 14);
        column.set_margin_top(12);
        column.set_margin_bottom(12);
        column.set_margin_start(12);
        column.set_margin_end(12);

        let scroller = ScrolledWindow::new();
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);
        scroller.set_child(Some(&column));

        Rc::new(Self {
            scroller,
            column,
            indicator: dnd::new_drop_indicator(),
            drag_active: Rc::new(Cell::new(false)),
            selected: Rc::new(RefCell::new(HashSet::new())),
            last_selected: Rc::new(RefCell::new(None)),
            active_tag_filter: Rc::new(RefCell::new(None)),
            last_focused_idea: Rc::new(RefCell::new(None)),
            on_copy_movement: RefCell::new(None),
        })
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.scroller
    }

    /// The idea the user was last editing, if it still exists.
    pub fn focused_idea_id(&self) -> Option<String> {
        self.last_focused_idea.borrow().clone()
    }

    /// Focuses the movement/idea row tagged with `widget_name() == name` (see
    /// the `set_widget_name` calls in `rebuild`), for the command palette's
    /// outline-jump items. `GtkScrolledWindow` auto-scrolls a focused
    /// descendant into view, so grabbing focus is enough. Idea rows use the
    /// `idea-entry:{id}` name on the text entry itself (not the row) so
    /// focusing it also opens it for editing.
    pub fn focus_by_name(&self, name: &str) -> bool {
        let target = if let Some(id) = name.strip_prefix("idea:") {
            format!("idea-entry:{id}")
        } else if let Some(id) = name.strip_prefix("movement:") {
            // The card's own root (tagged `movement:{id}`) is a plain
            // GtkBox and isn't focusable — redirect to its name entry, same
            // as the idea case above.
            format!("movement-entry:{id}")
        } else {
            name.to_string()
        };
        find_by_name(self.column.upcast_ref(), &target)
            .map(|w| w.grab_focus())
            .unwrap_or(false)
    }

    /// Ctrl/Shift-click on an idea's number (see `idea_row.rs`). Shift
    /// extends a range from the last-clicked idea, but only within the same
    /// movement — selection never spans movements, so a cross-movement
    /// shift-click just falls through to a plain single-select instead.
    fn toggle_select(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, id: &str, ctrl: bool, shift: bool) {
        let movement_ids = {
            let st = state.borrow();
            let Some((m_idx, _)) = st.sermon.find_idea(id) else {
                return;
            };
            st.sermon.movements[m_idx]
                .ideas
                .iter()
                .map(|i| i.id.clone())
                .collect::<Vec<_>>()
        };
        let anchor = self.last_selected.borrow().clone();
        let anchor_same_movement = anchor.as_ref().is_some_and(|a| movement_ids.contains(a));

        let mut selected = self.selected.borrow_mut();
        if shift && anchor_same_movement {
            let anchor = anchor.unwrap();
            let a_idx = movement_ids.iter().position(|x| x == &anchor).unwrap();
            let b_idx = movement_ids.iter().position(|x| x == id).unwrap();
            let (lo, hi) = if a_idx <= b_idx { (a_idx, b_idx) } else { (b_idx, a_idx) };
            if !ctrl {
                selected.clear();
            }
            selected.extend(movement_ids[lo..=hi].iter().cloned());
        } else if ctrl {
            if !selected.remove(id) {
                selected.insert(id.to_string());
            }
            *self.last_selected.borrow_mut() = Some(id.to_string());
        } else {
            selected.clear();
            selected.insert(id.to_string());
            *self.last_selected.borrow_mut() = Some(id.to_string());
        }
        drop(selected);
        self.refresh_selection_classes();
    }

    /// Stores the "Copy movement to another sermon…" handler (see
    /// `app_window.rs`, which owns the sermons directory, the current
    /// sermon's path, and the toast overlay this needs — none of which
    /// `Editor` itself holds). Call once, mirroring `init_dnd`/`init_keys`.
    pub fn set_on_copy_movement(&self, f: impl Fn(Movement) + 'static) {
        *self.on_copy_movement.borrow_mut() = Some(Box::new(f));
    }

    /// Ctrl+click on an idea/part tag chip (see `idea_row.rs`). Dims every
    /// idea not carrying that exact tag value, toggling off on a second
    /// click of the same tag. Implemented as a plain `rebuild()` rather than
    /// a live widget-tree walk: unlike `.idea-row-selected`, which walks the
    /// live tree because it fires on every marquee-drag frame, this fires at
    /// most once per click, so the simplicity of reusing `rebuild()`'s
    /// existing dimming pass (see the idea-row-creation loop) outweighs the
    /// cost of a full rebuild.
    fn toggle_tag_filter(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: &ApplyFn, tag: String) {
        {
            let mut filter = self.active_tag_filter.borrow_mut();
            if filter.as_deref() == Some(tag.as_str()) {
                *filter = None;
            } else {
                *filter = Some(tag);
            }
        }
        self.rebuild(state, apply.clone());
    }

    /// Result of a rubber-band drag in one movement's ideas box (see
    /// `movement_card.rs`). `additive` (Ctrl held at drag-end) merges into
    /// the existing selection instead of replacing it — an empty, non-
    /// additive `ids` (a drag or click that landed on nothing) clears the
    /// selection, matching the file-manager convention of blank-space click
    /// deselecting everything.
    fn apply_marquee_selection(self: &Rc<Self>, ids: Vec<String>, additive: bool) {
        let mut selected = self.selected.borrow_mut();
        if !additive {
            selected.clear();
        }
        selected.extend(ids.iter().cloned());
        drop(selected);
        *self.last_selected.borrow_mut() = ids.last().cloned();
        self.refresh_selection_classes();
    }

    /// Drops the whole selection. Shared by Escape and the blank-space
    /// click on the movements column.
    fn clear_selection(self: &Rc<Self>) {
        self.selected.borrow_mut().clear();
        *self.last_selected.borrow_mut() = None;
        self.refresh_selection_classes();
    }

    /// Re-applies `.idea-row-selected` to the live widget tree from
    /// `self.selected` without a full `rebuild()` — selection is ephemeral
    /// UI state, not a `Cmd`, so no undo entry or model change is involved.
    fn refresh_selection_classes(self: &Rc<Self>) {
        let selected = self.selected.borrow();
        fn walk(w: &gtk4::Widget, selected: &HashSet<String>) {
            if w.css_classes().iter().any(|c| c == "idea-row") {
                let is_selected = w
                    .widget_name()
                    .strip_prefix("idea:")
                    .map(|id| selected.contains(id))
                    .unwrap_or(false);
                if is_selected {
                    w.add_css_class("idea-row-selected");
                } else {
                    w.remove_css_class("idea-row-selected");
                }
            }
            let mut child = w.first_child();
            while let Some(c) = child {
                walk(&c, selected);
                child = c.next_sibling();
            }
        }
        walk(self.column.upcast_ref(), &selected);
    }

    /// Wires the movements column's single `DropTarget`. Call once, after
    /// construction — not from `rebuild()` — since re-adding a controller on
    /// every rebuild would accumulate duplicates while `rebuild()` only
    /// clears *children*, not controllers attached to `column` itself.
    pub fn init_dnd(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: ApplyFn) {
        let target = DropTarget::new(dnd::DragPayload::static_type(), gtk4::gdk::DragAction::MOVE);
        target.set_preload(true);
        {
            let editor = self.clone();
            target.connect_motion(move |t, _x, y| {
                let Some(payload) = t.value_as::<dnd::DragPayload>() else {
                    return gtk4::gdk::DragAction::empty();
                };
                editor.on_motion(&payload.0, y);
                gtk4::gdk::DragAction::MOVE
            });
        }
        {
            let editor = self.clone();
            target.connect_leave(move |_| {
                dnd::clear_indicator(&editor.indicator);
            });
        }
        {
            let editor = self.clone();
            let state = state.clone();
            target.connect_drop(move |_, value, _x, y| {
                dnd::clear_indicator(&editor.indicator);
                let Ok(payload) = value.get::<dnd::DragPayload>() else {
                    return false;
                };
                editor.on_drop(&state, &apply, &payload.0, y)
            });
        }
        self.column.add_controller(target);
    }

    /// Wires Delete/BackSpace to bulk-delete the current selection. Call
    /// once, same reasoning as `init_dnd`. Deliberately *not* capture-phase:
    /// if a focused `Entry`/`TextView` is what actually has focus, this
    /// explicitly steps aside so normal in-field text deletion is
    /// untouched — the guard checks focus directly rather than racing
    /// against it.
    pub fn init_keys(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: ApplyFn) {
        let editor = self.clone();
        let state = state.clone();
        let key_ctl = gtk4::EventControllerKey::new();
        key_ctl.connect_key_pressed(move |_, key, _, _| {
            // Escape clears the selection from anywhere in the editor —
            // previously the only way to deselect was to click blank space
            // *inside the same ideas box*, which is undiscoverable and
            // impossible for a movement whose rows fill it edge to edge.
            if key == gtk4::gdk::Key::Escape {
                if editor.selected.borrow().is_empty() {
                    return glib::Propagation::Proceed;
                }
                editor.clear_selection();
                return glib::Propagation::Stop;
            }
            if !matches!(key, gtk4::gdk::Key::Delete | gtk4::gdk::Key::BackSpace) {
                return glib::Propagation::Proceed;
            }
            if editor.selected.borrow().is_empty() {
                return glib::Propagation::Proceed;
            }
            if let Some(root) = editor.column.root() {
                if let Some(focus) = gtk4::prelude::RootExt::focus(&root) {
                    if focus.downcast_ref::<gtk4::Entry>().is_some()
                        || focus.downcast_ref::<gtk4::TextView>().is_some()
                    {
                        return glib::Propagation::Proceed;
                    }
                }
            }
            editor.delete_selected(&state, &apply);
            glib::Propagation::Stop
        });
        self.scroller.add_controller(key_ctl);

        // Clicking the empty column around/below the movement cards clears
        // the selection, the file-manager convention. Guarded by `pick` so
        // it only fires on genuinely blank column space and never swallows
        // a click that landed on a card.
        let editor = self.clone();
        let click = gtk4::GestureClick::new();
        click.set_button(gtk4::gdk::BUTTON_PRIMARY);
        let column = self.column.clone();
        click.connect_pressed(move |_, _, x, y| {
            if editor.selected.borrow().is_empty() {
                return;
            }
            let hit_blank = match column.pick(x, y, gtk4::PickFlags::DEFAULT) {
                Some(w) => w == column.clone().upcast::<gtk4::Widget>(),
                None => true,
            };
            if hit_blank {
                editor.clear_selection();
            }
        });
        self.column.add_controller(click);
    }

    /// Bulk-deletes the current selection and clears it (the ids no longer
    /// resolve to anything after this). Shared by the Delete/BackSpace key
    /// handler and the right-click "Delete N ideas" menu item.
    fn delete_selected(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: &ApplyFn) {
        let cmd = {
            let st = state.borrow();
            bulk_delete_cmd(&st.sermon, &self.selected.borrow())
        };
        if let Some(cmd) = cmd {
            apply(cmd);
        }
        self.selected.borrow_mut().clear();
        *self.last_selected.borrow_mut() = None;
    }

    fn on_motion(self: &Rc<Self>, payload: &str, y: f64) {
        // `y` arrives in `self.column`'s own coordinate space (the widget
        // the `DropTarget` is attached to), which grows far taller than the
        // visible viewport as movements/ideas accumulate — comparing it
        // directly against the scroller's viewport height made autoscroll
        // trigger (and stay triggered) almost anywhere below the very top of
        // the document, not just near the actual bottom edge, producing a
        // runaway scroll-down on nearly every drag. Translate into the
        // scroller's own coordinate space first so the edge check reflects
        // where the pointer actually is relative to the visible viewport.
        let scroller_y = self
            .column
            .translate_coordinates(&self.scroller, 0.0, y)
            .map(|(_, ty)| ty)
            .unwrap_or(y);
        dnd::autoscroll_if_near_edge(&self.scroller, scroller_y);
        if payload.starts_with(dnd::IDEA_PAYLOAD_PREFIX) || payload.starts_with(dnd::IDEAS_PAYLOAD_PREFIX) {
            match dnd::locate_drop_zone(&self.column, y) {
                DropZone::InMovementIdeas { movement_index, local_y } => {
                    dnd::place_idea_indicator(&self.indicator, &self.column, movement_index, local_y);
                }
                DropZone::BlankSpace { .. } => dnd::clear_indicator(&self.indicator),
            }
        } else if payload.starts_with(dnd::MOVEMENT_PAYLOAD_PREFIX) {
            dnd::clear_indicator(&self.indicator);
        }
    }

    fn on_drop(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: &ApplyFn, payload: &str, y: f64) -> bool {
        if let Some(ids_csv) = payload.strip_prefix(dnd::IDEAS_PAYLOAD_PREFIX) {
            // Selection is always scoped to one movement (see
            // `toggle_select`/`apply_marquee_selection`), so every id here
            // shares the same source movement — `positions` below is never
            // a mix of movements. Sorted ascending by original index so the
            // `orig_i - k` trick (same one `MoveIdea`'s single-item drop
            // uses via `adjusted`/`insertion_index - 1` above) generalizes:
            // removing the k-th-lowest idea from a movement shifts every
            // idea after it down by one, so the k-th removal's *live*
            // source index is its original index minus the k removals
            // already done ahead of it.
            let mut positions: Vec<(usize, usize)> = {
                let st = state.borrow();
                ids_csv
                    .split(',')
                    .filter_map(|id| st.sermon.find_idea(id))
                    .collect()
            };
            if positions.is_empty() {
                return false;
            }
            positions.sort();
            let src_m = positions[0].0;
            let idxs: Vec<usize> = positions.iter().map(|(_, i)| *i).collect();

            match dnd::locate_drop_zone(&self.column, y) {
                DropZone::InMovementIdeas { movement_index, local_y } => {
                    let Some(card) = dnd::nth_movement_card(&self.column, movement_index) else {
                        return false;
                    };
                    let Some(ideas_box) = dnd::movement_ideas_box(&card) else {
                        return false;
                    };
                    let insertion_index = dnd::idea_insertion_index(&ideas_box, local_y);
                    let base = if movement_index == src_m {
                        let removed_before = idxs.iter().filter(|&&i| i < insertion_index).count();
                        insertion_index.saturating_sub(removed_before)
                    } else {
                        insertion_index
                    };
                    let cmds = idxs
                        .iter()
                        .enumerate()
                        .map(|(k, orig_i)| Cmd::MoveIdea {
                            from: (src_m, orig_i - k),
                            to: (movement_index, base + k),
                        })
                        .collect();
                    apply(Cmd::Composite(cmds));
                }
                DropZone::BlankSpace { insert_index } => {
                    let new_movement = Movement::new(insert_index);
                    let adjusted_src_m = if src_m >= insert_index { src_m + 1 } else { src_m };
                    let mut cmds = vec![Cmd::InsertMovement {
                        at: insert_index,
                        movement: new_movement,
                    }];
                    cmds.extend(idxs.iter().enumerate().map(|(k, orig_i)| Cmd::MoveIdea {
                        from: (adjusted_src_m, orig_i - k),
                        to: (insert_index, k),
                    }));
                    apply(Cmd::Composite(cmds));
                }
            }
            true
        } else if let Some(idea_id) = payload.strip_prefix(dnd::IDEA_PAYLOAD_PREFIX) {
            let Some((from_m, from_i)) = state.borrow().sermon.find_idea(idea_id) else {
                return false;
            };
            match dnd::locate_drop_zone(&self.column, y) {
                DropZone::InMovementIdeas { movement_index, local_y } => {
                    let Some(card) = dnd::nth_movement_card(&self.column, movement_index) else {
                        return false;
                    };
                    let Some(ideas_box) = dnd::movement_ideas_box(&card) else {
                        return false;
                    };
                    let insertion_index = dnd::idea_insertion_index(&ideas_box, local_y);
                    let adjusted = if from_m == movement_index && from_i < insertion_index {
                        insertion_index - 1
                    } else {
                        insertion_index
                    };
                    let target_len = state.borrow().sermon.movements[movement_index].ideas.len();
                    apply(Cmd::MoveIdea {
                        from: (from_m, from_i),
                        to: (movement_index, adjusted.min(target_len)),
                    });
                }
                DropZone::BlankSpace { insert_index } => {
                    let new_movement = Movement::new(insert_index);
                    let adjusted_from = if from_m >= insert_index {
                        (from_m + 1, from_i)
                    } else {
                        (from_m, from_i)
                    };
                    apply(Cmd::Composite(vec![
                        Cmd::InsertMovement {
                            at: insert_index,
                            movement: new_movement,
                        },
                        Cmd::MoveIdea {
                            from: adjusted_from,
                            to: (insert_index, 0),
                        },
                    ]));
                }
            }
            true
        } else if let Some(movement_id) = payload.strip_prefix(dnd::MOVEMENT_PAYLOAD_PREFIX) {
            let Some(from) = state.borrow().sermon.find_movement(movement_id) else {
                return false;
            };
            let insertion = dnd::movement_insertion_index(&self.column, y);
            let adjusted = if insertion > from { insertion - 1 } else { insertion };
            apply(Cmd::MoveMovement { from, to: adjusted });
            true
        } else {
            false
        }
    }

    /// Tears down and repopulates the movements column from `state.sermon`.
    /// Called after every structural command and once at startup.
    pub fn rebuild(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: ApplyFn) {
        // Every structural `Cmd` (drag-drop reorder, delete movement/idea,
        // toggle collapse, ...) tears down and rebuilds the whole widget
        // tree below. If the widget that currently has keyboard focus is
        // about to be destroyed, GTK doesn't just drop focus — it
        // reassigns it to *some* focusable widget still in the window, in
        // practice the first movement's name entry in tab order, which then
        // reads as the view "jumping" to select that entry's text. Clearing
        // focus first (only when it's actually about to be destroyed, i.e.
        // it's a descendant of this editor) makes that reassignment a no-op:
        // after rebuild, nothing is focused unless a caller explicitly
        // requests it via `focus_by_name` (e.g. after adding/duplicating an
        // idea), which still runs after this and wins.
        if let Some(root) = self.column.root() {
            if let Some(focus) = gtk4::prelude::RootExt::focus(&root) {
                if focus.is_ancestor(&self.column) || focus == self.column.clone().upcast::<gtk4::Widget>() {
                    gtk4::prelude::RootExt::set_focus(&root, None::<&gtk4::Widget>);
                }
            }
        }

        while let Some(child) = self.column.first_child() {
            self.column.remove(&child);
        }

        {
            // Ideas deleted by other structural commands (or by this
            // rebuild's own caller) shouldn't linger as ghost selection.
            let sermon = &state.borrow().sermon;
            self.selected.borrow_mut().retain(|id| sermon.idea(id).is_some());
        }

        let numbering = state.borrow().sermon.numbering();
        let total_ideas: usize = numbering.len();
        let movements: Vec<Movement> = state.borrow().sermon.movements.clone();
        let idea_tag_census: Vec<(String, usize)> = state.borrow().library.idea_tag_census().into_iter().collect();
        let part_tag_census: Vec<(String, usize)> = state.borrow().library.part_tag_census().into_iter().collect();

        if total_ideas == 0 {
            let status = adw::StatusPage::new();
            status.set_title("No ideas yet");
            status.set_description(Some("Add your first idea below."));
            status.set_icon_name(Some("document-edit-symbolic"));
            self.column.append(&status);
        }

        let mut flat_idx = 0;
        for (m_idx, movement) in movements.iter().enumerate() {
            let card = build_movement_card(
                movement,
                m_idx == 0,
                &self.drag_active,
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    move |name| {
                        let old = state
                            .borrow()
                            .sermon
                            .find_movement(&id)
                            .map(|idx| state.borrow().sermon.movements[idx].name.clone())
                            .unwrap_or_default();
                        apply(Cmd::RenameMovement {
                            id: id.clone(),
                            old,
                            new: name,
                        });
                    }
                },
                {
                    let state = state.clone();
                    move || state.borrow_mut().undo.break_coalescing()
                },
                {
                    let id = movement.id.clone();
                    let apply = apply.clone();
                    move || apply(Cmd::ToggleMovementCollapsed { id: id.clone() })
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    move || {
                        let Some(idx) = state.borrow().sermon.find_movement(&id) else {
                            return;
                        };
                        let movement = state.borrow().sermon.movements[idx].clone();
                        apply(Cmd::DeleteMovement { at: idx, movement });
                    }
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    let editor = self.clone();
                    move || {
                        let Some(idx) = state.borrow().sermon.find_movement(&id) else {
                            return;
                        };
                        let dup = state.borrow().sermon.movements[idx].duplicate();
                        let new_id = dup.id.clone();
                        apply(Cmd::InsertMovement { at: idx + 1, movement: dup });
                        editor.focus_by_name(&format!("movement:{new_id}"));
                    }
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    move || {
                        let Some(idx) = state.borrow().sermon.find_movement(&id) else {
                            return;
                        };
                        if idx == 0 {
                            return;
                        }
                        let this_movement = state.borrow().sermon.movements[idx].clone();
                        let prev_len = state.borrow().sermon.movements[idx - 1].ideas.len();
                        let mut cmds: Vec<Cmd> = (0..this_movement.ideas.len())
                            .map(|offset| Cmd::MoveIdea {
                                from: (idx, 0),
                                to: (idx - 1, prev_len + offset),
                            })
                            .collect();
                        cmds.push(Cmd::DeleteMovement {
                            at: idx,
                            movement: Movement {
                                ideas: Vec::new(),
                                ..this_movement
                            },
                        });
                        apply(Cmd::Composite(cmds));
                    }
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    let editor = self.clone();
                    move |dir: i32| {
                        let Some(from) = state.borrow().sermon.find_movement(&id) else {
                            return;
                        };
                        let len = state.borrow().sermon.movements.len();
                        let to = match dir {
                            i32::MIN => 0,
                            i32::MAX => len - 1,
                            dir => {
                                let t = from as i32 + dir;
                                if t < 0 || t as usize >= len {
                                    return;
                                }
                                t as usize
                            }
                        };
                        apply(Cmd::MoveMovement { from, to });
                        editor.focus_by_name(&format!("movement:{id}"));
                    }
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    move |split_at: usize| {
                        let Some(m_idx) = state.borrow().sermon.find_movement(&id) else {
                            return;
                        };
                        let ideas_len = state.borrow().sermon.movements[m_idx].ideas.len();
                        if split_at >= ideas_len {
                            return;
                        }
                        let new_movement = Movement::new(m_idx + 1);
                        let mut cmds = vec![Cmd::InsertMovement {
                            at: m_idx + 1,
                            movement: new_movement,
                        }];
                        for offset in 0..(ideas_len - split_at) {
                            cmds.push(Cmd::MoveIdea {
                                from: (m_idx, split_at),
                                to: (m_idx + 1, offset),
                            });
                        }
                        apply(Cmd::Composite(cmds));
                    }
                },
                {
                    let editor = self.clone();
                    move |ids: Vec<String>, additive: bool| {
                        editor.apply_marquee_selection(ids, additive);
                    }
                },
                self.selected.clone(),
                {
                    let editor = self.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    move || {
                        editor.delete_selected(&state, &apply);
                    }
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let editor = self.clone();
                    move || {
                        let Some(idx) = state.borrow().sermon.find_movement(&id) else {
                            return;
                        };
                        let movement = state.borrow().sermon.movements[idx].clone();
                        if let Some(f) = editor.on_copy_movement.borrow().as_ref() {
                            f(movement);
                        }
                    }
                },
                {
                    let id = movement.id.clone();
                    let state = state.clone();
                    let apply = apply.clone();
                    move || {
                        let cmd = {
                            let st = state.borrow();
                            st.sermon.find_movement(&id).and_then(|at| {
                                crate::commands::demote_movement_to_idea(&st.sermon, at)
                            })
                        };
                        if let Some(cmd) = cmd {
                            apply(cmd);
                        }
                    }
                },
            );
            card.root.set_widget_name(&format!("movement:{}", movement.id));
            card.name_entry.set_widget_name(&format!("movement-entry:{}", movement.id));

            for idea in &movement.ideas {
                let (_, _, number) = numbering[flat_idx];
                flat_idx += 1;
                let id = idea.id.clone();

                let row = build_idea_row(
                    idea,
                    number,
                    &idea_tag_census,
                    &part_tag_census,
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        move |text: String| {
                            let old = state
                                .borrow()
                                .sermon
                                .idea(&id)
                                .map(|i| i.text.clone())
                                .unwrap_or_default();
                            let edit = Cmd::EditIdeaText {
                                id: id.clone(),
                                old,
                                new: text.clone(),
                            };
                            // "@john3:16" auto-tags the sermon with "John
                            // 3:16" as soon as the citation is complete.
                            // Only wrapped in a Composite on the (rare)
                            // keystroke that actually completes a new
                            // citation — `Cmd::Composite` has no coalesce
                            // key (see `commands.rs::Cmd::coalesce_key`),
                            // so wrapping *every* keystroke would silently
                            // break the usual same-field text-edit
                            // coalescing, turning ordinary typing into one
                            // undo step per character. Never removes a tag
                            // on its own: if you delete the citation, the
                            // tag just stays, removable by hand from the
                            // status bar like any other.
                            let existing_s_tags = state.borrow().sermon.s_tags.clone();
                            let mut new_s_tags = existing_s_tags.clone();
                            for found in bible::find_citations(&text) {
                                let display = found.citation.display();
                                if !new_s_tags.contains(&display) {
                                    new_s_tags.push(display);
                                }
                            }
                            if new_s_tags == existing_s_tags {
                                apply(edit);
                            } else {
                                apply(Cmd::Composite(vec![
                                    edit,
                                    Cmd::SetSermonTags {
                                        kind: SermonTagKind::S,
                                        old: existing_s_tags,
                                        new: new_s_tags,
                                    },
                                ]));
                            }
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        move |notes| {
                            let old = state
                                .borrow()
                                .sermon
                                .idea(&id)
                                .map(|i| i.notes.clone())
                                .unwrap_or_default();
                            apply(Cmd::EditIdeaNotes {
                                id: id.clone(),
                                old,
                                new: notes,
                            });
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        move |tag| {
                            let old = state
                                .borrow()
                                .sermon
                                .idea(&id)
                                .map(|i| i.idea_tag.clone())
                                .unwrap_or_default();
                            apply(Cmd::SetIdeaTag {
                                id: id.clone(),
                                kind: TagKind::Idea,
                                old,
                                new: tag,
                            });
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        move |tag| {
                            let old = state
                                .borrow()
                                .sermon
                                .idea(&id)
                                .map(|i| i.part_tag.clone())
                                .unwrap_or_default();
                            apply(Cmd::SetIdeaTag {
                                id: id.clone(),
                                kind: TagKind::Part,
                                old,
                                new: tag,
                            });
                        }
                    },
                    {
                        let state = state.clone();
                        move || state.borrow_mut().undo.break_coalescing()
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        move || {
                            let found = state.borrow().sermon.find_idea(&id);
                            if let Some((m, i)) = found {
                                let idea = state.borrow().sermon.movements[m].ideas[i].clone();
                                apply(Cmd::DeleteIdea {
                                    movement: m,
                                    index: i,
                                    idea,
                                });
                            }
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        let editor = self.clone();
                        move || {
                            let Some((m, i)) = state.borrow().sermon.find_idea(&id) else {
                                return;
                            };
                            let new_idea = Idea::new();
                            let new_id = new_idea.id.clone();
                            apply(Cmd::InsertIdea {
                                movement: m,
                                index: i + 1,
                                idea: new_idea,
                            });
                            editor.focus_by_name(&format!("idea:{new_id}"));
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        let editor = self.clone();
                        move || {
                            let Some((m, i)) = state.borrow().sermon.find_idea(&id) else {
                                return;
                            };
                            let dup = state.borrow().sermon.movements[m].ideas[i].duplicate();
                            let new_id = dup.id.clone();
                            apply(Cmd::InsertIdea {
                                movement: m,
                                index: i + 1,
                                idea: dup,
                            });
                            editor.focus_by_name(&format!("idea:{new_id}"));
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        let editor = self.clone();
                        move |dir: i32| {
                            let Some((m, i)) = state.borrow().sermon.find_idea(&id) else {
                                return;
                            };
                            let len = state.borrow().sermon.movements[m].ideas.len();
                            let to = match dir {
                                i32::MIN => 0,
                                i32::MAX => len - 1,
                                dir => {
                                    let t = i as i32 + dir;
                                    if t < 0 || t as usize >= len {
                                        return;
                                    }
                                    t as usize
                                }
                            };
                            apply(Cmd::MoveIdea { from: (m, i), to: (m, to) });
                            editor.focus_by_name(&format!("idea:{id}"));
                        }
                    },
                    {
                        let id = id.clone();
                        let state = state.clone();
                        let editor = self.clone();
                        move |ctrl: bool, shift: bool| {
                            editor.toggle_select(&state, &id, ctrl, shift);
                        }
                    },
                    {
                        let state = state.clone();
                        let apply = apply.clone();
                        let editor = self.clone();
                        move |tag: String| {
                            editor.toggle_tag_filter(&state, &apply, tag);
                        }
                    },
                    {
                        let state = state.clone();
                        let apply = apply.clone();
                        move |old: String, new: String| {
                            rename_tag_everywhere(&state, &apply, TagKind::Idea, old, new);
                        }
                    },
                    {
                        let state = state.clone();
                        let apply = apply.clone();
                        move |old: String, new: String| {
                            rename_tag_everywhere(&state, &apply, TagKind::Part, old, new);
                        }
                    },
                    {
                        // Resolved from the id at click time, not from the
                        // (m_idx, i) baked in at build time — a rebuild can
                        // land between the two.
                        let id = id.clone();
                        let state = state.clone();
                        let apply = apply.clone();
                        move || {
                            let cmd = {
                                let st = state.borrow();
                                st.sermon.find_idea(&id).and_then(|(m, i)| {
                                    crate::commands::promote_idea_to_movement(&st.sermon, m, i)
                                })
                            };
                            if let Some(cmd) = cmd {
                                apply(cmd);
                            }
                        }
                    },
                );
                dnd::setup_drag_source(
                    &row.grabber,
                    &row.root,
                    {
                        // Dragging a row that's part of a live multi-selection
                        // (>1 member) carries the whole selection; dragging an
                        // unselected row (even while others are selected)
                        // moves just that one, unaffected. Checked fresh at
                        // drag-start, not baked in at row-build time, so this
                        // stays correct across selection changes that don't
                        // trigger a rebuild (ctrl/shift-click, marquee).
                        let id = id.clone();
                        let selected = self.selected.clone();
                        move || {
                            let sel = selected.borrow();
                            if sel.len() > 1 && sel.contains(&id) {
                                format!("{}{}", dnd::IDEAS_PAYLOAD_PREFIX, sel.iter().cloned().collect::<Vec<_>>().join(","))
                            } else {
                                format!("{}{}", dnd::IDEA_PAYLOAD_PREFIX, id)
                            }
                        }
                    },
                    &self.drag_active,
                );
                row.root.set_widget_name(&format!("idea:{id}"));
                row.entry.set_widget_name(&format!("idea-entry:{id}"));
                {
                    let id = id.clone();
                    let last_focused = self.last_focused_idea.clone();
                    let focus_ctl = gtk4::EventControllerFocus::new();
                    focus_ctl.connect_enter(move |_| {
                        *last_focused.borrow_mut() = Some(id.clone());
                    });
                    row.entry.add_controller(focus_ctl);
                }
                if self.selected.borrow().contains(&id) {
                    row.root.add_css_class("idea-row-selected");
                }
                if let Some(filter) = &*self.active_tag_filter.borrow() {
                    if idea.idea_tag != *filter && idea.part_tag != *filter {
                        row.root.add_css_class("idea-row-tag-dimmed");
                    }
                }
                card.ideas_box.append(&row.root);
            }

            if movement.ideas.is_empty() {
                card.ideas_box.append(&build_empty_movement_placeholder());
            }

            let add_idea_btn = Button::with_label("+ Add idea");
            add_idea_btn.add_css_class("flat");
            add_idea_btn.add_css_class("ghost-add-btn");
            add_idea_btn.set_halign(Align::Start);
            add_idea_btn.set_margin_top(4);
            {
                let state = state.clone();
                let apply = apply.clone();
                let editor = self.clone();
                add_idea_btn.connect_clicked(move |_| {
                    let index = state
                        .borrow()
                        .sermon
                        .movements
                        .get(m_idx)
                        .map(|m| m.ideas.len())
                        .unwrap_or(0);
                    let idea = Idea::new();
                    let new_id = idea.id.clone();
                    apply(Cmd::InsertIdea {
                        movement: m_idx,
                        index,
                        idea,
                    });
                    editor.focus_by_name(&format!("idea:{new_id}"));
                });
            }
            card.ideas_box.append(&add_idea_btn);

            self.column.append(&card.root);
        }

        let add_movement_btn = Button::with_label("+ Add movement");
        add_movement_btn.add_css_class("flat");
        add_movement_btn.add_css_class("ghost-add-btn");
        add_movement_btn.set_halign(Align::Start);
        {
            let state = state.clone();
            let apply = apply.clone();
            let editor = self.clone();
            add_movement_btn.connect_clicked(move |_| {
                let at = state.borrow().sermon.movements.len();
                let movement = Movement::new(at);
                let new_id = movement.id.clone();
                apply(Cmd::InsertMovement { at, movement });
                editor.focus_by_name(&format!("movement:{new_id}"));
            });
        }
        self.column.append(&add_movement_btn);
    }
}

/// A compact inline empty state for one movement's ideas box — distinct from
/// the full `adw::StatusPage` used above for the whole-sermon case, which
/// would be oversized nested inside a movement card.
fn build_empty_movement_placeholder() -> GtkBox {
    let placeholder = GtkBox::new(Orientation::Horizontal, 0);
    placeholder.add_css_class("empty-movement-placeholder");
    let label = Label::new(Some("No ideas yet — click + Add idea below"));
    label.add_css_class("dim-label");
    label.add_css_class("caption");
    placeholder.append(&label);
    placeholder
}

fn find_by_name(root: &gtk4::Widget, name: &str) -> Option<gtk4::Widget> {
    if root.widget_name() == name {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(c) = child {
        if let Some(found) = find_by_name(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

/// "Rename everywhere" from a tag popover (see `tag_popover.rs`): every
/// idea in the sermon whose tag of this `kind` currently equals `old` gets
/// set to `new` in one undo step.
fn rename_tag_everywhere(state: &Rc<RefCell<AppState>>, apply: &ApplyFn, kind: TagKind, old: String, new: String) {
    if old.is_empty() || old == new {
        return;
    }
    let cmds: Vec<Cmd> = {
        let st = state.borrow();
        st.sermon
            .movements
            .iter()
            .flat_map(|m| m.ideas.iter())
            .filter(|idea| match kind {
                TagKind::Idea => idea.idea_tag == old,
                TagKind::Part => idea.part_tag == old,
            })
            .map(|idea| Cmd::SetIdeaTag {
                id: idea.id.clone(),
                kind,
                old: old.clone(),
                new: new.clone(),
            })
            .collect()
    };
    if !cmds.is_empty() {
        apply(Cmd::Composite(cmds));
    }
}

/// Builds one `Cmd::Composite(DeleteIdea...)` for every id in `ids` that
/// still resolves to a real idea. Sorted descending by `(movement, index)`
/// so within any one movement the highest index is removed first — each
/// removal only ever shifts *later* indices in that movement, never the
/// ones still queued, so no index-adjustment bookkeeping is needed the way
/// `on_drop`'s multi-idea move requires.
fn bulk_delete_cmd(sermon: &Sermon, ids: &HashSet<String>) -> Option<Cmd> {
    let mut positions: Vec<(usize, usize)> = ids.iter().filter_map(|id| sermon.find_idea(id)).collect();
    if positions.is_empty() {
        return None;
    }
    positions.sort_by(|a, b| b.cmp(a));
    Some(Cmd::Composite(
        positions
            .into_iter()
            .map(|(m, i)| Cmd::DeleteIdea {
                movement: m,
                index: i,
                idea: sermon.movements[m].ideas[i].clone(),
            })
            .collect(),
    ))
}
