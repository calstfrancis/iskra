use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

/// Snapshot of the resolved lectionary data for the planned date,
/// denormalized into the sermon file so it survives without re-running the
/// resolver (and so the file is self-describing in git history). Refreshed
/// whenever the planned date OR the selected lectionary/track changes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LectionaryLink {
    #[serde(default)]
    pub source: crate::lectionary::LectionaryKind,
    pub week: String,
    pub season: String,
    pub year: String,
    pub colour: String,
    pub colour_hex: String,
    /// (label, citation) pairs in display order — e.g. `[("OT", "Isa 2:1–5"),
    /// ("Psalm", "Ps 122"), ...]`. A flat list rather than fixed OT/Psalm/
    /// Epistle/Gospel fields because the Narrative Lectionary doesn't share
    /// RCL/Catholic's four-slot shape.
    #[serde(default)]
    pub readings: Vec<(String, String)>,
    /// Read-compat only for sermon files saved before Iskra supported more
    /// than the RCL (pre-0.x multi-lectionary schema) — never written by
    /// this version. See `Self::readings_or_legacy`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) ot: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) psalm: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) epistle: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) gospel: String,
}

impl LectionaryLink {
    /// `readings` if present, else synthesized from the legacy OT/Psalm/
    /// Epistle/Gospel fields for a sermon file saved before this schema.
    pub fn readings_or_legacy(&self) -> Vec<(String, String)> {
        if !self.readings.is_empty() {
            return self.readings.clone();
        }
        [("OT", &self.ot), ("Psalm", &self.psalm), ("Epistle", &self.epistle), ("Gospel", &self.gospel)]
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(label, v)| (label.to_string(), v.clone()))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Idea {
    pub id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub idea_tag: String,
    #[serde(default)]
    pub part_tag: String,
    #[serde(default)]
    pub expanded: bool,
}

impl Idea {
    pub fn new() -> Self {
        Self {
            id: new_id(),
            text: String::new(),
            notes: String::new(),
            idea_tag: String::new(),
            part_tag: String::new(),
            expanded: false,
        }
    }

    /// A copy with a fresh id, for the "Duplicate idea" action — ids must
    /// stay unique within a sermon.
    pub fn duplicate(&self) -> Self {
        Self {
            id: new_id(),
            ..self.clone()
        }
    }
}

// Field order matters for TOML serialization: scalar/array values must come
// before the `ideas` array-of-tables or the serializer rejects the struct.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Movement {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default)]
    pub ideas: Vec<Idea>,
}

impl Movement {
    pub fn new(index: usize) -> Self {
        Self {
            id: new_id(),
            name: default_movement_name(index),
            collapsed: false,
            ideas: Vec::new(),
        }
    }

    /// A copy with a fresh id for the movement *and* every idea it
    /// contains, for the "Duplicate movement" action — ids must stay
    /// unique within a sermon.
    pub fn duplicate(&self) -> Self {
        Self {
            id: new_id(),
            ideas: self.ideas.iter().map(Idea::duplicate).collect(),
            ..self.clone()
        }
    }
}

// `lectionary` (table) and `movements` (array of tables) must stay the last
// two fields — see the note on `Movement`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Sermon {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub series: Option<String>,
    #[serde(default)]
    pub planned_date: Option<NaiveDate>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    #[serde(default)]
    pub s_tags: Vec<String>,
    #[serde(default)]
    pub t_tags: Vec<String>,
    #[serde(default)]
    pub lectionary: Option<LectionaryLink>,
    #[serde(default)]
    pub movements: Vec<Movement>,
}

fn default_schema_version() -> u32 {
    1
}

impl Sermon {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            schema_version: SCHEMA_VERSION,
            id: new_id(),
            title: String::new(),
            series: None,
            planned_date: None,
            created: now,
            modified: now,
            s_tags: Vec::new(),
            t_tags: Vec::new(),
            lectionary: None,
            movements: vec![Movement::new(0)],
        }
    }

    pub fn display_title(&self) -> &str {
        if self.title.is_empty() {
            "Untitled Sermon"
        } else {
            &self.title
        }
    }

    /// Continuous 1-based numbering in reading order: movement order, then
    /// idea order within each movement. Returns (movement_idx, idea_idx, n).
    pub fn numbering(&self) -> Vec<(usize, usize, u32)> {
        let mut out = Vec::new();
        let mut n = 1u32;
        for (m, movement) in self.movements.iter().enumerate() {
            for (i, _) in movement.ideas.iter().enumerate() {
                out.push((m, i, n));
                n += 1;
            }
        }
        out
    }

    pub fn find_idea(&self, id: &str) -> Option<(usize, usize)> {
        for (m, movement) in self.movements.iter().enumerate() {
            for (i, idea) in movement.ideas.iter().enumerate() {
                if idea.id == id {
                    return Some((m, i));
                }
            }
        }
        None
    }

    pub fn find_movement(&self, id: &str) -> Option<usize> {
        self.movements.iter().position(|m| m.id == id)
    }

    pub fn idea_mut(&mut self, id: &str) -> Option<&mut Idea> {
        self.movements
            .iter_mut()
            .flat_map(|m| m.ideas.iter_mut())
            .find(|i| i.id == id)
    }

    pub fn idea(&self, id: &str) -> Option<&Idea> {
        self.movements
            .iter()
            .flat_map(|m| m.ideas.iter())
            .find(|i| i.id == id)
    }
}

pub fn new_id() -> String {
    glib::uuid_string_random().to_string()
}

const NUMBER_WORDS: [&str; 12] = [
    "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven",
    "Twelve",
];

pub fn default_movement_name(index: usize) -> String {
    match NUMBER_WORDS.get(index) {
        Some(word) => format!("Movement {word}"),
        None => format!("Movement {}", index + 1),
    }
}

/// Lowercased, dash-separated form of a title for use in filenames.
pub fn slug(title: &str) -> String {
    let s: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let cleaned = s
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if cleaned.is_empty() {
        "untitled".to_string()
    } else {
        cleaned
    }
}

/// A deleted idea or movement held in the "Recently deleted" tray. Scoped to
/// the sermon it came from (`sermon_id`) so restoring into a *different*
/// open sermon can never happen — the tray is global (it lives in the config,
/// not the sermon file) precisely so a deletion is still recoverable after
/// switching sermons and back.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum DeletedRecord {
    Idea {
        sermon_id: String,
        movement: usize,
        index: usize,
        idea: Idea,
    },
    Movement {
        sermon_id: String,
        at: usize,
        movement: Movement,
    },
}

impl DeletedRecord {
    pub fn sermon_id(&self) -> &str {
        match self {
            DeletedRecord::Idea { sermon_id, .. } | DeletedRecord::Movement { sermon_id, .. } => {
                sermon_id
            }
        }
    }

    pub fn label(&self) -> String {
        match self {
            DeletedRecord::Idea { idea, .. } if !idea.text.trim().is_empty() => idea.text.clone(),
            DeletedRecord::Idea { .. } => "(untitled idea)".to_string(),
            DeletedRecord::Movement { movement, .. } if !movement.name.trim().is_empty() => {
                format!("Movement: {}", movement.name)
            }
            DeletedRecord::Movement { .. } => "(untitled movement)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sermon_with(counts: &[usize]) -> Sermon {
        let mut s = Sermon::new();
        s.movements.clear();
        for (m, &count) in counts.iter().enumerate() {
            let mut movement = Movement::new(m);
            for _ in 0..count {
                movement.ideas.push(Idea::new());
            }
            s.movements.push(movement);
        }
        s
    }

    #[test]
    fn numbering_is_continuous_across_movements() {
        let s = sermon_with(&[2, 2]);
        let nums = s.numbering();
        assert_eq!(
            nums.iter().map(|&(m, i, n)| (m, i, n)).collect::<Vec<_>>(),
            vec![(0, 0, 1), (0, 1, 2), (1, 0, 3), (1, 1, 4)]
        );
    }

    #[test]
    fn numbering_skips_empty_movements() {
        let s = sermon_with(&[1, 0, 2]);
        let nums = s.numbering();
        assert_eq!(nums.last().unwrap().2, 3);
        assert_eq!(nums[1], (2, 0, 2));
    }

    #[test]
    fn find_idea_locates_position() {
        let s = sermon_with(&[1, 3]);
        let id = s.movements[1].ideas[2].id.clone();
        assert_eq!(s.find_idea(&id), Some((1, 2)));
        assert_eq!(s.find_idea("nonexistent"), None);
    }

    #[test]
    fn idea_duplicate_gets_a_fresh_id_but_same_content() {
        let mut idea = Idea::new();
        idea.text = "Original".into();
        idea.notes = "Some notes".into();
        let dup = idea.duplicate();
        assert_ne!(dup.id, idea.id);
        assert_eq!(dup.text, idea.text);
        assert_eq!(dup.notes, idea.notes);
    }

    #[test]
    fn movement_duplicate_gets_fresh_ids_for_itself_and_every_idea() {
        let mut m = Movement::new(0);
        m.ideas.push(Idea::new());
        m.ideas.push(Idea::new());
        let dup = m.duplicate();
        assert_ne!(dup.id, m.id);
        assert_eq!(dup.ideas.len(), m.ideas.len());
        for (orig, cloned) in m.ideas.iter().zip(dup.ideas.iter()) {
            assert_ne!(orig.id, cloned.id);
        }
    }

    #[test]
    fn movement_names() {
        assert_eq!(default_movement_name(0), "Movement One");
        assert_eq!(default_movement_name(11), "Movement Twelve");
        assert_eq!(default_movement_name(12), "Movement 13");
    }

    #[test]
    fn slug_normalizes() {
        assert_eq!(slug("Fruit in Season"), "fruit-in-season");
        assert_eq!(slug("  What's Next?! "), "what-s-next");
        assert_eq!(slug(""), "untitled");
        assert_eq!(slug("###"), "untitled");
    }

    #[test]
    fn ids_are_unique() {
        assert_ne!(new_id(), new_id());
    }
}
