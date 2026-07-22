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

/// "This idea is actually its own section": the idea leaves its movement and
/// becomes a new movement inserted directly below, named after the idea's
/// text. Built entirely from existing commands, so undo/redo needs no new
/// inversion logic.
///
/// An idea carrying notes or tags is kept as the new movement's first idea
/// rather than discarded — its text is then duplicated in the movement name,
/// which is the lesser evil against silently dropping notes. A bare idea
/// (the common case) yields a clean empty movement.
pub fn promote_idea_to_movement(sermon: &Sermon, movement: usize, index: usize) -> Option<Cmd> {
    let idea = sermon.movements.get(movement)?.ideas.get(index)?.clone();
    let carries_content = !idea.notes.trim().is_empty()
        || !idea.idea_tag.trim().is_empty()
        || !idea.part_tag.trim().is_empty();
    let mut new_movement = Movement::new(movement + 1);
    new_movement.name = idea.text.clone();
    if carries_content {
        new_movement.ideas.push(idea.clone());
    }
    Some(Cmd::Composite(vec![
        Cmd::DeleteIdea {
            movement,
            index,
            idea,
        },
        Cmd::InsertMovement {
            at: movement + 1,
            movement: new_movement,
        },
    ]))
}

/// The inverse shape: a movement folds into the one above it, its name
/// becoming an idea and its own ideas following in order. Distinct from
/// "merge with movement above", which discards the name entirely.
///
/// Returns `None` for the first movement — there's nothing above to fold into.
pub fn demote_movement_to_idea(sermon: &Sermon, at: usize) -> Option<Cmd> {
    if at == 0 {
        return None;
    }
    let movement = sermon.movements.get(at)?.clone();
    let target = at - 1;
    let base = sermon.movements.get(target)?.ideas.len();

    // DeleteMovement first: `target` sits above `at`, so removing `at` can't
    // shift it, and every InsertIdea then indexes into the final shape.
    let mut cmds = vec![Cmd::DeleteMovement {
        at,
        movement: movement.clone(),
    }];
    let mut next = base;
    if !movement.name.trim().is_empty() {
        let mut heading = Idea::new();
        heading.text = movement.name.clone();
        cmds.push(Cmd::InsertIdea {
            movement: target,
            index: next,
            idea: heading,
        });
        next += 1;
    }
    for idea in movement.ideas {
        cmds.push(Cmd::InsertIdea {
            movement: target,
            index: next,
            idea,
        });
        next += 1;
    }
    Some(Cmd::Composite(cmds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Movement;

    fn sermon_two_movements() -> Sermon {
        let mut s = Sermon::new();
        s.movements.clear();
        for (name, texts) in [("One", vec!["a", "b"]), ("Two", vec!["c"])] {
            let mut m = Movement::new(0);
            m.name = name.to_string();
            m.ideas.clear();
            for text in texts {
                let mut i = Idea::new();
                i.text = text.to_string();
                m.ideas.push(i);
            }
            s.movements.push(m);
        }
        s
    }

    #[test]
    fn promote_moves_the_idea_out_and_names_the_new_movement_after_it() {
        let mut s = sermon_two_movements();
        let cmd = promote_idea_to_movement(&s, 0, 1).expect("valid position");
        cmd.apply_to(&mut s);
        assert_eq!(s.movements.len(), 3);
        assert_eq!(s.movements[0].ideas.len(), 1);
        assert_eq!(s.movements[0].ideas[0].text, "a");
        assert_eq!(s.movements[1].name, "b");
        assert!(s.movements[1].ideas.is_empty());
        assert_eq!(s.movements[2].name, "Two");
    }

    #[test]
    fn promote_keeps_an_idea_that_carries_notes() {
        let mut s = sermon_two_movements();
        s.movements[0].ideas[1].notes = "worth keeping".to_string();
        let cmd = promote_idea_to_movement(&s, 0, 1).expect("valid position");
        cmd.apply_to(&mut s);
        assert_eq!(s.movements[1].name, "b");
        assert_eq!(s.movements[1].ideas.len(), 1);
        assert_eq!(s.movements[1].ideas[0].notes, "worth keeping");
    }

    #[test]
    fn promote_rejects_an_out_of_range_position() {
        let s = sermon_two_movements();
        assert!(promote_idea_to_movement(&s, 0, 9).is_none());
        assert!(promote_idea_to_movement(&s, 9, 0).is_none());
    }

    #[test]
    fn demote_folds_the_movement_into_the_one_above_name_first() {
        let mut s = sermon_two_movements();
        let cmd = demote_movement_to_idea(&s, 1).expect("not the first movement");
        cmd.apply_to(&mut s);
        assert_eq!(s.movements.len(), 1);
        let texts: Vec<_> = s.movements[0].ideas.iter().map(|i| i.text.as_str()).collect();
        assert_eq!(texts, vec!["a", "b", "Two", "c"]);
    }

    #[test]
    fn demote_refuses_the_first_movement() {
        let s = sermon_two_movements();
        assert!(demote_movement_to_idea(&s, 0).is_none());
    }

    #[test]
    fn promote_then_undo_restores_the_original_shape() {
        let mut s = sermon_two_movements();
        let before = s.clone();
        let mut undo = UndoStack::new();
        let cmd = promote_idea_to_movement(&s, 0, 1).expect("valid position");
        undo.push_applying(&mut s, cmd);
        assert_ne!(s, before);
        assert!(undo.undo(&mut s));
        assert_eq!(s, before);
    }

    #[test]
    fn demote_then_undo_restores_the_original_shape() {
        let mut s = sermon_two_movements();
        let before = s.clone();
        let mut undo = UndoStack::new();
        let cmd = demote_movement_to_idea(&s, 1).expect("not the first movement");
        undo.push_applying(&mut s, cmd);
        assert_ne!(s, before);
        assert!(undo.undo(&mut s));
        assert_eq!(s, before);
    }

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
    fn split_movement_composite_moves_the_tail_into_a_new_movement_in_order() {
        // Same shape `editor.rs`'s right-click "Split movement here" builds:
        // insert an empty movement right after, then move everything from
        // `split_at` onward into it, each `MoveIdea` still reading `split_at`
        // as its `from` index since removing from that index each time
        // shifts the remainder down to fill it.
        let mut s = Sermon::new();
        s.movements.clear();
        s.movements.push(Movement::new(0));
        for _ in 0..4 {
            s.movements[0].ideas.push(Idea::new());
        }
        let ids: Vec<String> = s.movements[0].ideas.iter().map(|i| i.id.clone()).collect();
        let before = s.clone();
        let split_at = 2;
        let ideas_len = s.movements[0].ideas.len();

        let mut cmds = vec![Cmd::InsertMovement {
            at: 1,
            movement: Movement::new(1),
        }];
        for offset in 0..(ideas_len - split_at) {
            cmds.push(Cmd::MoveIdea {
                from: (0, split_at),
                to: (1, offset),
            });
        }
        let cmd = Cmd::Composite(cmds);
        cmd.apply_to(&mut s);

        assert_eq!(s.movements.len(), 2);
        assert_eq!(
            s.movements[0].ideas.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
            ids[..2]
        );
        assert_eq!(
            s.movements[1].ideas.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
            ids[2..]
        );

        cmd.inverted().apply_to(&mut s);
        assert_eq!(s, before);
    }

    #[test]
    fn bulk_delete_composite_removes_highest_index_first() {
        // Same shape `editor.rs::bulk_delete_cmd` builds: descending by
        // (movement, index) so within a movement the highest index is
        // always removed first, leaving lower queued indices valid.
        let mut s = Sermon::new();
        s.movements.clear();
        s.movements.push(Movement::new(0));
        for _ in 0..5 {
            s.movements[0].ideas.push(Idea::new());
        }
        let ids: Vec<String> = s.movements[0].ideas.iter().map(|i| i.id.clone()).collect();
        let before = s.clone();

        let mut positions = vec![(0usize, 1usize), (0, 4), (0, 3)];
        positions.sort_by(|a, b| b.cmp(a));
        let cmd = Cmd::Composite(
            positions
                .into_iter()
                .map(|(m, i)| Cmd::DeleteIdea {
                    movement: m,
                    index: i,
                    idea: s.movements[m].ideas[i].clone(),
                })
                .collect(),
        );
        cmd.apply_to(&mut s);

        assert_eq!(
            s.movements[0].ideas.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
            vec![ids[0].clone(), ids[2].clone()]
        );

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
