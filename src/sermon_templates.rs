//! Built-in movement/idea skeletons offered when creating a new sermon from
//! the library window, alongside a blank sermon.

use crate::model::{Idea, Movement, Sermon};

pub struct SermonTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    movements: &'static [(&'static str, &'static [&'static str])],
}

pub const TEMPLATES: &[SermonTemplate] = &[
    SermonTemplate {
        id: "three-point",
        name: "Three-Point Expository",
        description: "Introduction, three points, and a conclusion.",
        movements: &[
            ("Introduction", &["Hook", "Context", "Big idea"]),
            ("Point One", &[]),
            ("Point Two", &[]),
            ("Point Three", &[]),
            ("Conclusion", &["Summary", "Call to action"]),
        ],
    },
    SermonTemplate {
        id: "narrative-arc",
        name: "Narrative Arc",
        description: "Setting, rising tension, a turning point, and resolution.",
        movements: &[
            ("Setting the Scene", &[]),
            ("Rising Tension", &[]),
            ("Turning Point", &[]),
            ("Resolution", &[]),
            ("So What?", &["Application"]),
        ],
    },
    SermonTemplate {
        id: "topical",
        name: "Topical",
        description: "The problem, what Scripture says, and the response.",
        movements: &[
            ("The Problem", &[]),
            ("What Scripture Says", &[]),
            ("The Response", &[]),
        ],
    },
];

/// Builds a fresh `Sermon`, seeded from the named template's movement/idea
/// skeleton. `None` (or an unknown id) falls back to `Sermon::new()`'s
/// single blank movement.
pub fn build_sermon(template_id: Option<&str>) -> Sermon {
    let mut sermon = Sermon::new();
    let Some(template) = template_id.and_then(|id| TEMPLATES.iter().find(|t| t.id == id)) else {
        return sermon;
    };
    sermon.movements = template
        .movements
        .iter()
        .enumerate()
        .map(|(i, (name, ideas))| {
            let mut movement = Movement::new(i);
            movement.name = name.to_string();
            movement.ideas = ideas
                .iter()
                .map(|text| {
                    let mut idea = Idea::new();
                    idea.text = text.to_string();
                    idea
                })
                .collect();
            movement
        })
        .collect();
    sermon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_gives_a_blank_sermon() {
        let sermon = build_sermon(None);
        assert_eq!(sermon.movements.len(), 1);
        assert!(sermon.movements[0].ideas.is_empty());
    }

    #[test]
    fn unknown_id_falls_back_to_blank() {
        let sermon = build_sermon(Some("does-not-exist"));
        assert_eq!(sermon.movements.len(), 1);
    }

    #[test]
    fn three_point_template_has_five_movements() {
        let sermon = build_sermon(Some("three-point"));
        assert_eq!(sermon.movements.len(), 5);
        assert_eq!(sermon.movements[0].name, "Introduction");
        assert_eq!(sermon.movements[0].ideas.len(), 3);
        assert_eq!(sermon.movements[0].ideas[0].text, "Hook");
    }

    #[test]
    fn every_template_produces_unique_ids() {
        for template in TEMPLATES {
            let sermon = build_sermon(Some(template.id));
            let mut ids: Vec<&str> = sermon.movements.iter().map(|m| m.id.as_str()).collect();
            ids.extend(
                sermon
                    .movements
                    .iter()
                    .flat_map(|m| m.ideas.iter())
                    .map(|i| i.id.as_str()),
            );
            let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
            assert_eq!(ids.len(), unique.len(), "duplicate id in template {}", template.id);
        }
    }
}
