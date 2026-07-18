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
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, DropTarget, Orientation, ScrolledWindow};
use libadwaita as adw;

use crate::commands::{Cmd, TagKind};
use crate::model::{Idea, Movement};
use crate::state::AppState;
use crate::ui::dnd::{self, DropZone};
use crate::ui::idea_row::build_idea_row;
use crate::ui::movement_card::build_movement_card;

pub struct Editor {
    scroller: ScrolledWindow,
    column: GtkBox,
    indicator: GtkBox,
    drag_active: Rc<Cell<bool>>,
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
        })
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.scroller
    }

    /// Wires the movements column's single `DropTarget`. Call once, after
    /// construction — not from `rebuild()` — since re-adding a controller on
    /// every rebuild would accumulate duplicates while `rebuild()` only
    /// clears *children*, not controllers attached to `column` itself.
    pub fn init_dnd(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: ApplyFn) {
        let target = DropTarget::new(String::static_type(), gtk4::gdk::DragAction::MOVE);
        target.set_preload(true);
        {
            let editor = self.clone();
            target.connect_motion(move |t, _x, y| {
                let Some(payload) = t.value_as::<String>() else {
                    return gtk4::gdk::DragAction::empty();
                };
                editor.on_motion(&payload, y);
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
                let Ok(payload) = value.get::<String>() else {
                    return false;
                };
                editor.on_drop(&state, &apply, &payload, y)
            });
        }
        self.column.add_controller(target);
    }

    fn on_motion(self: &Rc<Self>, payload: &str, y: f64) {
        dnd::autoscroll_if_near_edge(&self.scroller, y);
        if payload.starts_with(dnd::IDEA_PAYLOAD_PREFIX) {
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
        if let Some(idea_id) = payload.strip_prefix(dnd::IDEA_PAYLOAD_PREFIX) {
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
        while let Some(child) = self.column.first_child() {
            self.column.remove(&child);
        }

        let numbering = state.borrow().sermon.numbering();
        let total_ideas: usize = numbering.len();
        let movements: Vec<Movement> = state.borrow().sermon.movements.clone();
        let idea_tag_census: Vec<String> = state.borrow().library.idea_tag_census().into_keys().collect();
        let part_tag_census: Vec<String> = state.borrow().library.part_tag_census().into_keys().collect();

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
            );

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
                        move |text| {
                            let old = state
                                .borrow()
                                .sermon
                                .idea(&id)
                                .map(|i| i.text.clone())
                                .unwrap_or_default();
                            apply(Cmd::EditIdeaText {
                                id: id.clone(),
                                old,
                                new: text,
                            });
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
                );
                dnd::setup_drag_source(
                    &row.grabber,
                    &row.root,
                    format!("{}{}", dnd::IDEA_PAYLOAD_PREFIX, id),
                    &self.drag_active,
                );
                card.ideas_box.append(&row.root);
            }

            let add_idea_btn = Button::with_label("+ Add idea");
            add_idea_btn.add_css_class("flat");
            add_idea_btn.set_halign(Align::Start);
            {
                let state = state.clone();
                let apply = apply.clone();
                add_idea_btn.connect_clicked(move |_| {
                    let index = state
                        .borrow()
                        .sermon
                        .movements
                        .get(m_idx)
                        .map(|m| m.ideas.len())
                        .unwrap_or(0);
                    apply(Cmd::InsertIdea {
                        movement: m_idx,
                        index,
                        idea: Idea::new(),
                    });
                });
            }
            card.ideas_box.append(&add_idea_btn);

            self.column.append(&card.root);
        }

        let add_movement_btn = Button::with_label("+ Add movement");
        add_movement_btn.add_css_class("flat");
        add_movement_btn.set_halign(Align::Start);
        {
            let state = state.clone();
            let apply = apply.clone();
            add_movement_btn.connect_clicked(move |_| {
                let at = state.borrow().sermon.movements.len();
                apply(Cmd::InsertMovement {
                    at,
                    movement: Movement::new(at),
                });
            });
        }
        self.column.append(&add_movement_btn);
    }
}
