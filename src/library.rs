//! In-memory index of every sermon in `work_dir/sermons/`. No rusqlite —
//! hundreds of ~4 KB TOMLs parse in tens of ms, files stay the single source
//! of truth (important with git sync pulling changes from elsewhere), and it
//! drops a bundled-C dependency. Rebuilt on library-window open and after
//! every save; the tag census doubles as the idea/part tag autocomplete
//! source (see `ui::tag_popover`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::commands::SermonTagKind;
use crate::model::Sermon;
use crate::storage;

/// A sidebar selection in the library window — either a tag or a series,
/// mutually exclusive with each other and with the free-text search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryFilter {
    Tag(SermonTagKind, String),
    Series(String),
}

pub struct LibraryIndex {
    pub sermons: Vec<(PathBuf, Sermon)>,
}

impl LibraryIndex {
    pub fn scan(sermons_dir: &Path) -> Self {
        Self {
            sermons: storage::scan_sermons(sermons_dir),
        }
    }

    /// Every sermon whose title, idea/note text, movement names, tags,
    /// secular date, or lectionary week contains `query` (case-insensitive).
    /// An empty query matches everything.
    pub fn filter<'a>(&'a self, query: &str, filter: Option<&LibraryFilter>) -> Vec<&'a (PathBuf, Sermon)> {
        let query = query.trim().to_lowercase();
        self.sermons
            .iter()
            .filter(|(_, s)| filter.is_none_or(|f| sermon_matches_filter(s, f)))
            .filter(|(_, s)| query.is_empty() || sermon_matches(s, &query))
            .collect()
    }

    /// Series names in use, with occurrence counts, for the library sidebar.
    pub fn series_census(&self) -> BTreeMap<String, usize> {
        census(self.sermons.iter().filter_map(|(_, s)| s.series.as_ref()))
    }

    /// Scripture (s.) tag census with occurrence counts, for the library
    /// sidebar and as an autocomplete source.
    pub fn s_tag_census(&self) -> BTreeMap<String, usize> {
        census(self.sermons.iter().flat_map(|(_, s)| s.s_tags.iter()))
    }

    pub fn t_tag_census(&self) -> BTreeMap<String, usize> {
        census(self.sermons.iter().flat_map(|(_, s)| s.t_tags.iter()))
    }

    /// Idea-tag values used anywhere in the library, for the idea-tag
    /// autocomplete popover.
    pub fn idea_tag_census(&self) -> BTreeMap<String, usize> {
        census(self.sermons.iter().flat_map(|(_, s)| {
            s.movements
                .iter()
                .flat_map(|m| m.ideas.iter())
                .map(|i| &i.idea_tag)
        }))
    }

    pub fn part_tag_census(&self) -> BTreeMap<String, usize> {
        census(self.sermons.iter().flat_map(|(_, s)| {
            s.movements
                .iter()
                .flat_map(|m| m.ideas.iter())
                .map(|i| &i.part_tag)
        }))
    }
}

fn census<'a>(values: impl Iterator<Item = &'a String>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for v in values {
        if !v.is_empty() {
            *out.entry(v.clone()).or_insert(0) += 1;
        }
    }
    out
}

fn sermon_matches_filter(s: &Sermon, filter: &LibraryFilter) -> bool {
    match filter {
        LibraryFilter::Tag(SermonTagKind::S, tag) => s.s_tags.iter().any(|t| t == tag),
        LibraryFilter::Tag(SermonTagKind::T, tag) => s.t_tags.iter().any(|t| t == tag),
        LibraryFilter::Series(series) => s.series.as_deref() == Some(series.as_str()),
    }
}

fn sermon_matches(s: &Sermon, query_lower: &str) -> bool {
    if s.title.to_lowercase().contains(query_lower) {
        return true;
    }
    if let Some(d) = s.planned_date {
        if d.to_string().contains(query_lower) {
            return true;
        }
    }
    if let Some(link) = &s.lectionary {
        if link.week.to_lowercase().contains(query_lower) {
            return true;
        }
    }
    if s.s_tags.iter().any(|t| t.to_lowercase().contains(query_lower)) {
        return true;
    }
    if s.t_tags.iter().any(|t| t.to_lowercase().contains(query_lower)) {
        return true;
    }
    for m in &s.movements {
        if m.name.to_lowercase().contains(query_lower) {
            return true;
        }
        for idea in &m.ideas {
            if idea.text.to_lowercase().contains(query_lower)
                || idea.notes.to_lowercase().contains(query_lower)
                || idea.idea_tag.to_lowercase().contains(query_lower)
                || idea.part_tag.to_lowercase().contains(query_lower)
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Idea, Movement};

    fn seeded(dir: &Path) {
        let mut a = Sermon::new();
        a.title = "Fruit in Season".into();
        a.s_tags = vec!["Luke 13".into()];
        a.t_tags = vec!["repentance".into()];
        a.series = Some("Parables".into());
        let mut idea = Idea::new();
        idea.text = "The fig tree is not condemned yet".into();
        idea.idea_tag = "image".into();
        a.movements[0].ideas.push(idea);
        storage::save_sermon(&storage::new_sermon_path(dir, &a), &a).unwrap();

        let mut b = Sermon::new();
        b.title = "Bread for the Journey".into();
        b.t_tags = vec!["provision".into(), "repentance".into()];
        let mut m2 = Movement::new(1);
        let mut idea2 = Idea::new();
        idea2.text = "manna in the wilderness".into();
        idea2.idea_tag = "image".into();
        m2.ideas.push(idea2);
        b.movements.push(m2);
        storage::save_sermon(&storage::new_sermon_path(dir, &b), &b).unwrap();
    }

    #[test]
    fn filter_matches_title() {
        let dir = tempfile::tempdir().unwrap();
        seeded(dir.path());
        let idx = LibraryIndex::scan(dir.path());
        assert_eq!(idx.filter("fruit", None).len(), 1);
        assert_eq!(idx.filter("bread", None).len(), 1);
        assert_eq!(idx.filter("", None).len(), 2);
    }

    #[test]
    fn filter_matches_idea_text_and_tags() {
        let dir = tempfile::tempdir().unwrap();
        seeded(dir.path());
        let idx = LibraryIndex::scan(dir.path());
        assert_eq!(idx.filter("manna", None).len(), 1);
        assert_eq!(idx.filter("luke 13", None).len(), 1);
    }

    #[test]
    fn filter_by_sermon_tag() {
        let dir = tempfile::tempdir().unwrap();
        seeded(dir.path());
        let idx = LibraryIndex::scan(dir.path());
        assert_eq!(
            idx.filter("", Some(&LibraryFilter::Tag(SermonTagKind::T, "repentance".into()))).len(),
            2
        );
        assert_eq!(
            idx.filter("", Some(&LibraryFilter::Tag(SermonTagKind::S, "Luke 13".into()))).len(),
            1
        );
    }

    #[test]
    fn series_census_and_filter() {
        let dir = tempfile::tempdir().unwrap();
        seeded(dir.path());
        let idx = LibraryIndex::scan(dir.path());
        assert_eq!(idx.series_census().get("Parables"), Some(&1));
        assert_eq!(
            idx.filter("", Some(&LibraryFilter::Series("Parables".into()))).len(),
            1
        );
        assert_eq!(
            idx.filter("", Some(&LibraryFilter::Series("Nonexistent".into()))).len(),
            0
        );
    }

    #[test]
    fn tag_census_counts_occurrences() {
        let dir = tempfile::tempdir().unwrap();
        seeded(dir.path());
        let idx = LibraryIndex::scan(dir.path());
        assert_eq!(idx.t_tag_census().get("repentance"), Some(&2));
        assert_eq!(idx.idea_tag_census().get("image"), Some(&2));
    }
}
