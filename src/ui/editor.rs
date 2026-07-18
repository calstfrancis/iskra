//! The movements-and-ideas editor. For dev1 there is a single implicit
//! movement (movement chrome, drag-and-drop, and multi-movement rendering
//! land in dev2 — see Plans/plan.md). Every mutation triggers a full rebuild
//! from the model rather than a patched-in-place update: numbering is global
//! and undo can restore arbitrary structure, so one rebuild path is simpler
//! than shadowing it with fine-grained widget updates.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Orientation, ScrolledWindow};
use libadwaita as adw;

use crate::commands::{Cmd, TagKind};
use crate::model::Idea;
use crate::state::AppState;
use crate::ui::idea_row::build_idea_row;

pub struct Editor {
    scroller: ScrolledWindow,
    column: GtkBox,
}

pub type ApplyFn = Rc<dyn Fn(Cmd)>;

impl Editor {
    pub fn new() -> Rc<Self> {
        let column = GtkBox::new(Orientation::Vertical, 10);
        column.set_margin_top(12);
        column.set_margin_bottom(12);
        column.set_margin_start(12);
        column.set_margin_end(12);

        let scroller = ScrolledWindow::new();
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);
        scroller.set_child(Some(&column));

        Rc::new(Self { scroller, column })
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.scroller
    }

    /// Tears down and repopulates the idea list from `state.sermon`. Called
    /// after every structural command and once at startup.
    pub fn rebuild(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: ApplyFn) {
        while let Some(child) = self.column.first_child() {
            self.column.remove(&child);
        }

        let numbering = state.borrow().sermon.numbering();
        let ideas: Vec<Idea> = state
            .borrow()
            .sermon
            .movements
            .iter()
            .flat_map(|m| m.ideas.iter().cloned())
            .collect();

        if ideas.is_empty() {
            let status = adw::StatusPage::new();
            status.set_title("No ideas yet");
            status.set_description(Some("Add your first idea below."));
            status.set_icon_name(Some("document-edit-symbolic"));
            self.column.append(&status);
        }

        for (idx, idea) in ideas.iter().enumerate() {
            let (_, _, number) = numbering[idx];
            let id = idea.id.clone();

            let row = build_idea_row(
                idea,
                number,
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
            self.column.append(&row.root);
        }

        let add_btn = Button::with_label("+ Add idea");
        add_btn.add_css_class("flat");
        add_btn.set_halign(Align::Start);
        {
            let state = state.clone();
            let apply = apply.clone();
            add_btn.connect_clicked(move |_| {
                let movement = state.borrow().sermon.movements.len().saturating_sub(1);
                let index = state
                    .borrow()
                    .sermon
                    .movements
                    .get(movement)
                    .map(|m| m.ideas.len())
                    .unwrap_or(0);
                apply(Cmd::InsertIdea {
                    movement,
                    index,
                    idea: Idea::new(),
                });
            });
        }
        self.column.append(&add_btn);
    }
}
