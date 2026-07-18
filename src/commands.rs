use std::time::{Duration, Instant};

use crate::model::{Idea, Movement, Sermon};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagKind {
    Idea,
    Part,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SermonTagKind {
    S,
    T,
}

/// Every structural or textual mutation to a `Sermon` goes through one of
/// these. Each variant carries enough of the "before" state to invert itself
/// mechanically — no separate widget-level undo system to keep in sync.
#[derive(Clone, Debug)]
pub enum Cmd {
    InsertIdea {
        movement: usize,
        index: usize,
        idea: Idea,
    },
    DeleteIdea {
        movement: usize,
        index: usize,
        idea: Idea,
    },
    EditIdeaText {
        id: String,
        old: String,
        new: String,
    },
    EditIdeaNotes {
        id: String,
        old: String,
        new: String,
    },
    SetIdeaTag {
        id: String,
        kind: TagKind,
        old: String,
        new: String,
    },
    MoveIdea {
        from: (usize, usize),
        to: (usize, usize),
    },
    InsertMovement {
        at: usize,
        movement: Movement,
    },
    DeleteMovement {
        at: usize,
        movement: Movement,
    },
    RenameMovement {
        id: String,
        old: String,
        new: String,
    },
    MoveMovement {
        from: usize,
        to: usize,
    },
    ToggleMovementCollapsed {
        id: String,
    },
    SetTitle {
        old: String,
        new: String,
    },
    SetSeries {
        old: Option<String>,
        new: Option<String>,
    },
    SetPlannedDate {
        old: (Option<chrono::NaiveDate>, Option<crate::model::LectionaryLink>),
        new: (Option<chrono::NaiveDate>, Option<crate::model::LectionaryLink>),
    },
    SetSermonTags {
        kind: SermonTagKind,
        old: Vec<String>,
        new: Vec<String>,
    },
    Composite(Vec<Cmd>),
}

/// A key that identifies "the same editable field" across consecutive
/// commands, used to decide whether two commands should coalesce into one
/// undo step. Distinct from `Cmd` equality — two `EditIdeaText` commands on
/// the same idea coalesce even though their `old`/`new` text differs.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CoalesceKey {
    IdeaText(String),
    IdeaNotes(String),
    MovementName(String),
    Title,
    Series,
}

impl Cmd {
    pub fn apply_to(&self, s: &mut Sermon) {
        match self {
            Cmd::InsertIdea { movement, index, idea } => {
                s.movements[*movement].ideas.insert(*index, idea.clone());
            }
            Cmd::DeleteIdea { movement, index, .. } => {
                s.movements[*movement].ideas.remove(*index);
            }
            Cmd::EditIdeaText { id, new, .. } => {
                if let Some(idea) = s.idea_mut(id) {
                    idea.text = new.clone();
                }
            }
            Cmd::EditIdeaNotes { id, new, .. } => {
                if let Some(idea) = s.idea_mut(id) {
                    idea.notes = new.clone();
                }
            }
            Cmd::SetIdeaTag { id, kind, new, .. } => {
                if let Some(idea) = s.idea_mut(id) {
                    match kind {
                        TagKind::Idea => idea.idea_tag = new.clone(),
                        TagKind::Part => idea.part_tag = new.clone(),
                    }
                }
            }
            Cmd::MoveIdea { from, to } => {
                let idea = s.movements[from.0].ideas.remove(from.1);
                s.movements[to.0].ideas.insert(to.1, idea);
            }
            Cmd::InsertMovement { at, movement } => {
                s.movements.insert(*at, movement.clone());
            }
            Cmd::DeleteMovement { at, .. } => {
                s.movements.remove(*at);
            }
            Cmd::RenameMovement { id, new, .. } => {
                if let Some(idx) = s.find_movement(id) {
                    s.movements[idx].name = new.clone();
                }
            }
            Cmd::MoveMovement { from, to } => {
                let m = s.movements.remove(*from);
                s.movements.insert(*to, m);
            }
            Cmd::ToggleMovementCollapsed { id } => {
                if let Some(idx) = s.find_movement(id) {
                    s.movements[idx].collapsed = !s.movements[idx].collapsed;
                }
            }
            Cmd::SetTitle { new, .. } => {
                s.title = new.clone();
            }
            Cmd::SetSeries { new, .. } => {
                s.series = new.clone();
            }
            Cmd::SetPlannedDate { new, .. } => {
                s.planned_date = new.0;
                s.lectionary = new.1.clone();
            }
            Cmd::SetSermonTags { kind, new, .. } => match kind {
                SermonTagKind::S => s.s_tags = new.clone(),
                SermonTagKind::T => s.t_tags = new.clone(),
            },
            Cmd::Composite(cmds) => {
                for c in cmds {
                    c.apply_to(s);
                }
            }
        }
    }

    /// Produces the command that undoes this one. Insert/Delete pairs swap
    /// roles, Move swaps from/to, edits swap old/new, and Composite inverts
    /// and reverses its children so a multi-step action unwinds in the
    /// correct order.
    pub fn inverted(&self) -> Cmd {
        match self {
            Cmd::InsertIdea { movement, index, idea } => Cmd::DeleteIdea {
                movement: *movement,
                index: *index,
                idea: idea.clone(),
            },
            Cmd::DeleteIdea { movement, index, idea } => Cmd::InsertIdea {
                movement: *movement,
                index: *index,
                idea: idea.clone(),
            },
            Cmd::EditIdeaText { id, old, new } => Cmd::EditIdeaText {
                id: id.clone(),
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::EditIdeaNotes { id, old, new } => Cmd::EditIdeaNotes {
                id: id.clone(),
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::SetIdeaTag { id, kind, old, new } => Cmd::SetIdeaTag {
                id: id.clone(),
                kind: *kind,
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::MoveIdea { from, to } => Cmd::MoveIdea {
                from: *to,
                to: *from,
            },
            Cmd::InsertMovement { at, movement } => Cmd::DeleteMovement {
                at: *at,
                movement: movement.clone(),
            },
            Cmd::DeleteMovement { at, movement } => Cmd::InsertMovement {
                at: *at,
                movement: movement.clone(),
            },
            Cmd::RenameMovement { id, old, new } => Cmd::RenameMovement {
                id: id.clone(),
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::MoveMovement { from, to } => Cmd::MoveMovement {
                from: *to,
                to: *from,
            },
            Cmd::ToggleMovementCollapsed { id } => Cmd::ToggleMovementCollapsed { id: id.clone() },
            Cmd::SetTitle { old, new } => Cmd::SetTitle {
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::SetSeries { old, new } => Cmd::SetSeries {
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::SetPlannedDate { old, new } => Cmd::SetPlannedDate {
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::SetSermonTags { kind, old, new } => Cmd::SetSermonTags {
                kind: *kind,
                old: new.clone(),
                new: old.clone(),
            },
            Cmd::Composite(cmds) => {
                Cmd::Composite(cmds.iter().rev().map(|c| c.inverted()).collect())
            }
        }
    }

    /// Structural commands (anything that adds/removes/reorders a row) force
    /// a full rebuild of the editor widget tree; pure text edits update the
    /// already-live widget in place with no rebuild.
    pub fn is_structural(&self) -> bool {
        match self {
            Cmd::EditIdeaText { .. }
            | Cmd::EditIdeaNotes { .. }
            | Cmd::SetIdeaTag { .. }
            | Cmd::SetTitle { .. }
            | Cmd::SetSeries { .. }
            | Cmd::SetSermonTags { .. } => false,
            Cmd::RenameMovement { .. } => false,
            Cmd::Composite(cmds) => cmds.iter().any(Cmd::is_structural),
            _ => true,
        }
    }

    fn coalesce_key(&self) -> Option<CoalesceKey> {
        match self {
            Cmd::EditIdeaText { id, .. } => Some(CoalesceKey::IdeaText(id.clone())),
            Cmd::EditIdeaNotes { id, .. } => Some(CoalesceKey::IdeaNotes(id.clone())),
            Cmd::RenameMovement { id, .. } => Some(CoalesceKey::MovementName(id.clone())),
            Cmd::SetTitle { .. } => Some(CoalesceKey::Title),
            Cmd::SetSeries { .. } => Some(CoalesceKey::Series),
            _ => None,
        }
    }
}

const COALESCE_WINDOW: Duration = Duration::from_secs(2);
const MAX_STACK: usize = 200;

pub struct UndoStack {
    undo: Vec<Cmd>,
    redo: Vec<Cmd>,
    last_edit: Option<Instant>,
    last_key: Option<CoalesceKey>,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            last_key: None,
        }
    }

    /// Applies `cmd` to `sermon`, clears the redo stack, and pushes the
    /// command onto the undo stack — merging into the previous entry if it's
    /// a same-field text edit within the coalescing window.
    pub fn push_applying(&mut self, sermon: &mut Sermon, cmd: Cmd) {
        cmd.apply_to(sermon);
        self.redo.clear();

        let key = cmd.coalesce_key();
        let now = Instant::now();
        let can_coalesce = key.is_some()
            && key == self.last_key
            && self
                .last_edit
                .map(|t| now.duration_since(t) < COALESCE_WINDOW)
                .unwrap_or(false);

        if can_coalesce {
            if let Some(top) = self.undo.last_mut() {
                merge_into(top, &cmd);
                self.last_edit = Some(now);
                return;
            }
        }

        self.undo.push(cmd);
        if self.undo.len() > MAX_STACK {
            self.undo.remove(0);
        }
        self.last_key = key;
        self.last_edit = Some(now);
    }

    /// Breaks the coalescing chain (e.g. on focus-out) without pushing a
    /// command — the next edit to any field starts a fresh undo step.
    pub fn break_coalescing(&mut self) {
        self.last_edit = None;
        self.last_key = None;
    }

    pub fn undo(&mut self, sermon: &mut Sermon) -> bool {
        self.break_coalescing();
        match self.undo.pop() {
            Some(cmd) => {
                let inverse = cmd.inverted();
                inverse.apply_to(sermon);
                self.redo.push(cmd);
                true
            }
            None => false,
        }
    }

    pub fn redo(&mut self, sermon: &mut Sermon) -> bool {
        self.break_coalescing();
        match self.redo.pop() {
            Some(cmd) => {
                cmd.apply_to(sermon);
                self.undo.push(cmd);
                true
            }
            None => false,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

/// Merges `next` into the top-of-stack `top`, keeping `top`'s original `old`
/// and adopting `next`'s `new` — the pair collapses to "one step from the
/// value before this burst to the value after it."
fn merge_into(top: &mut Cmd, next: &Cmd) {
    match (top, next) {
        (Cmd::EditIdeaText { new, .. }, Cmd::EditIdeaText { new: next_new, .. }) => {
            *new = next_new.clone();
        }
        (Cmd::EditIdeaNotes { new, .. }, Cmd::EditIdeaNotes { new: next_new, .. }) => {
            *new = next_new.clone();
        }
        (Cmd::RenameMovement { new, .. }, Cmd::RenameMovement { new: next_new, .. }) => {
            *new = next_new.clone();
        }
        (Cmd::SetTitle { new, .. }, Cmd::SetTitle { new: next_new, .. }) => {
            *new = next_new.clone();
        }
        (Cmd::SetSeries { new, .. }, Cmd::SetSeries { new: next_new, .. }) => {
            *new = next_new.clone();
        }
        _ => unreachable!("merge_into called on incompatible/non-coalescable command pair"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Movement;

    fn sermon_with_one_idea() -> (Sermon, String) {
        let mut s = Sermon::new();
        s.movements.clear();
        s.movements.push(Movement::new(0));
        let idea = Idea::new();
        let id = idea.id.clone();
        s.movements[0].ideas.push(idea);
        (s, id)
    }

    #[test]
    fn insert_delete_idea_are_inverses() {
        let (mut s, _) = sermon_with_one_idea();
        let before = s.clone();
        let idea = Idea::new();
        let cmd = Cmd::InsertIdea {
            movement: 0,
            index: 1,
            idea: idea.clone(),
        };
        cmd.apply_to(&mut s);
        assert_eq!(s.movements[0].ideas.len(), 2);
        cmd.inverted().apply_to(&mut s);
        assert_eq!(s, before);
    }

    #[test]
    fn move_idea_inverse_restores_position() {
        let mut s = Sermon::new();
        s.movements.clear();
        s.movements.push(Movement::new(0));
        s.movements.push(Movement::new(1));
        s.movements[0].ideas.push(Idea::new());
        s.movements[0].ideas.push(Idea::new());
        let before = s.clone();
        let cmd = Cmd::MoveIdea {
            from: (0, 0),
            to: (1, 0),
        };
        cmd.apply_to(&mut s);
        assert_eq!(s.movements[0].ideas.len(), 1);
        assert_eq!(s.movements[1].ideas.len(), 1);
        cmd.inverted().apply_to(&mut s);
        assert_eq!(s, before);
    }

    #[test]
    fn composite_inverse_reverses_and_inverts_children() {
        let mut s = Sermon::new();
        s.movements.clear();
        s.movements.push(Movement::new(0));
        s.movements[0].ideas.push(Idea::new());
        let idea_id = s.movements[0].ideas[0].id.clone();
        let before = s.clone();

        // Simulates drag-to-blank-space: insert a new movement, then move
        // the idea into it.
        let new_movement = Movement::new(1);
        let new_movement_id = new_movement.id.clone();
        let cmd = Cmd::Composite(vec![
            Cmd::InsertMovement {
                at: 1,
                movement: new_movement,
            },
            Cmd::MoveIdea {
                from: (0, 0),
                to: (1, 0),
            },
        ]);
        cmd.apply_to(&mut s);
        assert_eq!(s.movements.len(), 2);
        assert_eq!(s.movements[1].ideas[0].id, idea_id);
        assert_eq!(s.movements[1].id, new_movement_id);

        cmd.inverted().apply_to(&mut s);
        assert_eq!(s, before);
    }

    #[test]
    fn text_edits_coalesce_within_window() {
        let (mut s, id) = sermon_with_one_idea();
        let mut stack = UndoStack::new();

        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id.clone(),
                old: "".into(),
                new: "T".into(),
            },
        );
        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id.clone(),
                old: "T".into(),
                new: "Th".into(),
            },
        );
        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id.clone(),
                old: "Th".into(),
                new: "The".into(),
            },
        );

        assert_eq!(stack.undo.len(), 1, "consecutive same-field edits should merge");
        stack.undo(&mut s);
        assert_eq!(s.idea(&id).unwrap().text, "");
    }

    #[test]
    fn coalescing_breaks_on_different_target() {
        let mut s = Sermon::new();
        s.movements.clear();
        s.movements.push(Movement::new(0));
        s.movements[0].ideas.push(Idea::new());
        s.movements[0].ideas.push(Idea::new());
        let id_a = s.movements[0].ideas[0].id.clone();
        let id_b = s.movements[0].ideas[1].id.clone();
        let mut stack = UndoStack::new();

        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id_a.clone(),
                old: "".into(),
                new: "A".into(),
            },
        );
        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id_b.clone(),
                old: "".into(),
                new: "B".into(),
            },
        );

        assert_eq!(stack.undo.len(), 2, "edits to different ideas must not merge");
    }

    #[test]
    fn coalescing_breaks_after_explicit_break() {
        let (mut s, id) = sermon_with_one_idea();
        let mut stack = UndoStack::new();
        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id.clone(),
                old: "".into(),
                new: "A".into(),
            },
        );
        stack.break_coalescing();
        stack.push_applying(
            &mut s,
            Cmd::EditIdeaText {
                id: id.clone(),
                old: "A".into(),
                new: "AB".into(),
            },
        );
        assert_eq!(stack.undo.len(), 2, "break_coalescing should force a new step");
    }

    #[test]
    fn undo_redo_round_trip() {
        let (mut s, _) = sermon_with_one_idea();
        let before = s.clone();
        let mut stack = UndoStack::new();
        let new_movement = Movement::new(1);
        stack.push_applying(
            &mut s,
            Cmd::InsertMovement {
                at: 1,
                movement: new_movement,
            },
        );
        assert_eq!(s.movements.len(), 2);
        assert!(stack.undo(&mut s));
        assert_eq!(s, before);
        assert!(!stack.can_undo());
        assert!(stack.can_redo());
        assert!(stack.redo(&mut s));
        assert_eq!(s.movements.len(), 2);
    }

    #[test]
    fn is_structural_classification() {
        assert!(!Cmd::EditIdeaText {
            id: "x".into(),
            old: "".into(),
            new: "a".into()
        }
        .is_structural());
        assert!(Cmd::InsertIdea {
            movement: 0,
            index: 0,
            idea: Idea::new()
        }
        .is_structural());
        assert!(Cmd::Composite(vec![Cmd::InsertMovement {
            at: 0,
            movement: Movement::new(0)
        }])
        .is_structural());
    }

    #[test]
    fn stack_cap_evicts_oldest() {
        let (mut s, id) = sermon_with_one_idea();
        let mut stack = UndoStack::new();
        for i in 0..(MAX_STACK + 10) {
            stack.break_coalescing();
            stack.push_applying(
                &mut s,
                Cmd::EditIdeaText {
                    id: id.clone(),
                    old: format!("{}", i),
                    new: format!("{}", i + 1),
                },
            );
        }
        assert_eq!(stack.undo.len(), MAX_STACK);
    }
}
