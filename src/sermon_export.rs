//! `.sermon` interchange export — see `Plans/sermon-interchange-spec.md`.
//! TOML container with a `[sermon]` metadata block and a `[content].typst`
//! body rendered in Rubric's existing Typst subset only (`=` headings,
//! `*bold*`, `_italic_`, `-` bullets, plain text), so the body can be
//! dropped directly into a service item's `content_typst` field with no new
//! rendering support on Rubric's side.

use serde::Serialize;

use crate::model::Sermon;

pub const SPEC_VERSION: u32 = 1;

/// Escapes Typst special characters in plain user text, character-for-character
/// matching Rubric's own `rubric_package/utils/typst.py` `_TYPST_ESCAPES`/
/// `typst_escape` — including escaping a literal backslash as the Typst
/// unicode escape `\u{5c}` rather than `\\` (the source of truth is the
/// Python implementation; an earlier draft of the spec doc had `\\` here,
/// which has been corrected to match).
pub fn typst_escape(text: &str) -> String {
    const ESCAPES: &[(char, &str)] = &[
        ('\\', "\\u{5c}"),
        ('#', "\\#"),
        ('@', "\\@"),
        ('*', "\\*"),
        ('_', "\\_"),
        ('~', "\\~"),
        ('$', "\\$"),
        ('`', "\\`"),
        ('<', "\\<"),
        ('>', "\\>"),
    ];
    let mut out = text.to_string();
    for (ch, escaped) in ESCAPES {
        out = out.replace(*ch, escaped);
    }
    out
}

#[derive(Serialize)]
struct ExportDoc {
    spec_version: u32,
    generator: String,
    sermon: SermonMeta,
    content: Content,
}

#[derive(Serialize)]
struct SermonMeta {
    id: String,
    title: String,
    date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lectionary_week: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    season: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    psalm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    epistle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gospel: Option<String>,
    s_tags: Vec<String>,
    t_tags: Vec<String>,
}

#[derive(Serialize)]
struct Content {
    typst: String,
}

/// Renders the Typst body only (movements → headings, ideas → numbered
/// bullets, notes → indented continuation lines, optional idea/part tag
/// suffix). Exposed separately from `export_sermon` so the export dialog can
/// preview it without round-tripping through TOML.
pub fn render_typst_body(sermon: &Sermon, include_tags: bool) -> String {
    let numbering = sermon.numbering();
    let mut numbers = numbering.into_iter();
    let mut out = String::new();

    for (m_idx, movement) in sermon.movements.iter().enumerate() {
        if m_idx > 0 {
            out.push('\n');
        }
        out.push_str("= ");
        out.push_str(&typst_escape(&movement.name));
        out.push('\n');

        for idea in &movement.ideas {
            let (_, _, n) = numbers.next().expect("numbering covers every idea");
            out.push_str(&format!("- *{n}.* {}", typst_escape(&idea.text)));
            if include_tags {
                out.push_str(&tag_suffix(&idea.idea_tag, &idea.part_tag));
            }
            out.push('\n');
            for line in idea.notes.lines().filter(|l| !l.trim().is_empty()) {
                out.push_str("  ");
                out.push_str(&typst_escape(line));
                out.push('\n');
            }
        }
    }

    out.trim_end().to_string()
}

fn tag_suffix(idea_tag: &str, part_tag: &str) -> String {
    match (idea_tag.is_empty(), part_tag.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!(" _[{}]_", typst_escape(idea_tag)),
        (true, false) => format!(" _[{}]_", typst_escape(part_tag)),
        (false, false) => format!(" _[{} · {}]_", typst_escape(idea_tag), typst_escape(part_tag)),
    }
}

/// Renders the full `.sermon` TOML document.
pub fn export_sermon(sermon: &Sermon, include_tags: bool) -> String {
    let link = sermon.lectionary.as_ref();
    let doc = ExportDoc {
        spec_version: SPEC_VERSION,
        generator: format!("iskra {}", env!("CARGO_PKG_VERSION")),
        sermon: SermonMeta {
            id: sermon.id.clone(),
            title: sermon.title.clone(),
            date: sermon.planned_date.map(|d| d.to_string()).unwrap_or_default(),
            lectionary_week: link.map(|l| l.week.clone()),
            season: link.map(|l| l.season.clone()),
            year: link.map(|l| l.year.clone()),
            colour: link.map(|l| l.colour.clone()),
            ot: link.map(|l| l.ot.clone()),
            psalm: link.map(|l| l.psalm.clone()),
            epistle: link.map(|l| l.epistle.clone()),
            gospel: link.map(|l| l.gospel.clone()),
            s_tags: sermon.s_tags.clone(),
            t_tags: sermon.t_tags.clone(),
        },
        content: Content {
            typst: render_typst_body(sermon, include_tags),
        },
    };
    toml::to_string_pretty(&doc).expect("ExportDoc is always serializable")
}

/// Filename for a `.sermon` export: same slug/date convention as the
/// internal storage filename, different extension.
pub fn export_filename(sermon: &Sermon) -> String {
    let date = sermon
        .planned_date
        .map(|d| d.to_string())
        .unwrap_or_else(|| sermon.created.date_naive().to_string());
    format!("{}-{}.sermon", date, crate::model::slug(&sermon.title))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Idea, Movement};
    use chrono::NaiveDate;

    fn sample_sermon() -> Sermon {
        let mut s = Sermon::new();
        s.title = "Fruit in Season".into();
        s.planned_date = Some(NaiveDate::from_ymd_opt(2026, 8, 16).unwrap());
        s.s_tags = vec!["Luke 13".into()];
        s.t_tags = vec!["repentance".into()];
        s.movements.clear();

        let mut m = Movement::new(0);
        m.name = "Movement One".into();
        let mut idea1 = Idea::new();
        idea1.text = "The fig tree is not yet fruitless".into();
        idea1.notes = "Owner wants it cut down.\nGardener asks for one more year.".into();
        let mut idea2 = Idea::new();
        idea2.text = "Patience is not passivity".into();
        idea2.idea_tag = "image".into();
        idea2.part_tag = "closing".into();
        m.ideas.push(idea1);
        m.ideas.push(idea2);
        s.movements.push(m);
        s
    }

    #[test]
    fn escape_table_matches_rubric_order_and_backslash_form() {
        assert_eq!(typst_escape("a\\b"), "a\\u{5c}b");
        assert_eq!(typst_escape("#hashtag"), "\\#hashtag");
        assert_eq!(typst_escape("*bold* _italic_"), "\\*bold\\* \\_italic\\_");
        assert_eq!(typst_escape("$math$ `code`"), "\\$math\\$ \\`code\\`");
        assert_eq!(typst_escape("a<b>c"), "a\\<b\\>c");
        assert_eq!(typst_escape("plain text"), "plain text");
    }

    #[test]
    fn body_renders_heading_numbered_bullets_and_notes() {
        let body = render_typst_body(&sample_sermon(), false);
        assert_eq!(
            body,
            "= Movement One\n\
             - *1.* The fig tree is not yet fruitless\n\
             \x20\x20Owner wants it cut down.\n\
             \x20\x20Gardener asks for one more year.\n\
             - *2.* Patience is not passivity"
        );
    }

    #[test]
    fn numbering_is_continuous_across_movements() {
        let mut s = sample_sermon();
        let mut m2 = Movement::new(1);
        m2.name = "Movement Two".into();
        let mut idea3 = Idea::new();
        idea3.text = "Third idea".into();
        m2.ideas.push(idea3);
        s.movements.push(m2);

        let body = render_typst_body(&s, false);
        assert!(body.contains("- *3.* Third idea"));
        assert!(body.contains("= Movement Two"));
    }

    #[test]
    fn tag_suffix_included_only_when_toggled_and_present() {
        let s = sample_sermon();
        let without = render_typst_body(&s, false);
        assert!(!without.contains("_[image"));

        let with = render_typst_body(&s, true);
        assert!(with.contains("- *2.* Patience is not passivity _[image · closing]_"));
        // idea 1 has no tags set, so no suffix even with the toggle on.
        assert!(with.contains("- *1.* The fig tree is not yet fruitless\n"));
    }

    #[test]
    fn user_text_is_escaped_in_body() {
        let mut s = sample_sermon();
        s.movements[0].ideas[0].text = "Cost is $5 and *urgent*".into();
        let body = render_typst_body(&s, false);
        assert!(body.contains("Cost is \\$5 and \\*urgent\\*"));
    }

    #[test]
    fn export_sermon_round_trips_through_toml() {
        let s = sample_sermon();
        let text = export_sermon(&s, true);
        let parsed: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(parsed["spec_version"].as_integer(), Some(1));
        assert_eq!(parsed["sermon"]["title"].as_str(), Some("Fruit in Season"));
        assert_eq!(parsed["sermon"]["date"].as_str(), Some("2026-08-16"));
        assert!(parsed["content"]["typst"].as_str().unwrap().contains("= Movement One"));
    }

    #[test]
    fn export_omits_lectionary_fields_when_no_date_planned() {
        let mut s = sample_sermon();
        s.planned_date = None;
        s.lectionary = None;
        let text = export_sermon(&s, false);
        let parsed: toml::Value = toml::from_str(&text).unwrap();
        assert!(parsed["sermon"].get("season").is_none());
        assert!(parsed["sermon"].get("lectionary_week").is_none());
    }

    #[test]
    fn export_filename_uses_date_and_slug() {
        let s = sample_sermon();
        assert_eq!(export_filename(&s), "2026-08-16-fruit-in-season.sermon");
    }
}
