use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::error::{atomic_write, IskraError, Result};
use crate::model::{slug, Sermon, SCHEMA_VERSION};

pub fn save_sermon(path: &Path, sermon: &Sermon) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = toml::to_string_pretty(sermon)?;
    atomic_write(path, text.as_bytes())?;
    Ok(())
}

pub fn load_sermon(path: &Path) -> Result<Sermon> {
    let text = std::fs::read_to_string(path)?;
    let sermon: Sermon = toml::from_str(&text)?;
    if sermon.schema_version > SCHEMA_VERSION {
        return Err(IskraError::SchemaTooNew {
            found: sermon.schema_version,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(sermon)
}

/// Filename for a new sermon: `{date}-{slug}-{id6}.toml`. Generated once at
/// creation and never auto-renamed afterwards — the id inside the file is the
/// identity; the filename is cosmetic, and leaving it alone keeps git history
/// contiguous.
pub fn filename_for(sermon: &Sermon) -> String {
    let date = sermon
        .planned_date
        .map(|d| d.to_string())
        .unwrap_or_else(|| sermon.created.date_naive().to_string());
    let id6: String = sermon.id.chars().filter(|c| *c != '-').take(6).collect();
    format!("{}-{}-{}.toml", date, slug(&sermon.title), id6)
}

pub fn new_sermon_path(sermons_dir: &Path, sermon: &Sermon) -> PathBuf {
    sermons_dir.join(filename_for(sermon))
}

/// All sermon files in the directory, with parse failures skipped (logged)
/// rather than aborting the scan — one corrupt file must not hide the rest.
pub fn scan_sermons(sermons_dir: &Path) -> Vec<(PathBuf, Sermon)> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(sermons_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "toml").unwrap_or(false) {
            match load_sermon(&path) {
                Ok(sermon) => out.push((path, sermon)),
                Err(e) => tracing::warn!("skipping unreadable sermon {}: {e}", path.display()),
            }
        }
    }
    out.sort_by(|a, b| {
        let key = |s: &Sermon| s.planned_date.unwrap_or_else(|| s.created.date_naive());
        key(&b.1).cmp(&key(&a.1))
    });
    out
}

/// Bumps `modified` and saves. The single save entry point the UI uses.
pub fn save_touched(path: &Path, sermon: &mut Sermon) -> Result<()> {
    sermon.modified = Utc::now();
    save_sermon(path, sermon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Idea, Movement};

    fn sample() -> Sermon {
        let mut s = Sermon::new();
        s.title = "Fruit in Season".into();
        s.planned_date = Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 16).unwrap());
        s.s_tags = vec!["Luke 13".into(), "Isaiah 55".into()];
        s.t_tags = vec!["repentance".into()];
        let mut idea = Idea::new();
        idea.text = "The fig tree is not condemned yet".into();
        idea.notes = "Owner wants it cut down.\nGardener asks for one more year.".into();
        idea.idea_tag = "image".into();
        idea.part_tag = "opening".into();
        s.movements[0].ideas.push(idea);
        let mut second = Movement::new(1);
        second.ideas.push(Idea::new());
        s.movements.push(second);
        s
    }

    #[test]
    fn round_trip_is_lossless() {
        let dir = tempfile::tempdir().unwrap();
        let s = sample();
        let path = new_sermon_path(dir.path(), &s);
        save_sermon(&path, &s).unwrap();
        let loaded = load_sermon(&path).unwrap();
        assert_eq!(s, loaded);
    }

    #[test]
    fn multiline_notes_serialize_as_triple_quoted_strings() {
        let s = sample();
        let text = toml::to_string_pretty(&s).unwrap();
        assert!(
            text.contains("'''") || text.contains("\"\"\""),
            "multiline notes must use TOML multi-line strings for git diffability, got:\n{text}"
        );
        assert!(text.contains("Gardener asks for one more year."));
    }

    #[test]
    fn filename_uses_date_slug_and_id() {
        let s = sample();
        let name = filename_for(&s);
        assert!(name.starts_with("2026-08-16-fruit-in-season-"));
        assert!(name.ends_with(".toml"));
    }

    #[test]
    fn filename_falls_back_to_created_date() {
        let mut s = sample();
        s.planned_date = None;
        assert!(filename_for(&s).starts_with(&s.created.date_naive().to_string()));
    }

    #[test]
    fn schema_too_new_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = sample();
        s.schema_version = SCHEMA_VERSION + 1;
        let path = dir.path().join("future.toml");
        save_sermon(&path, &s).unwrap();
        assert!(matches!(
            load_sermon(&path),
            Err(IskraError::SchemaTooNew { .. })
        ));
    }

    #[test]
    fn minimal_file_gets_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("minimal.toml");
        std::fs::write(
            &path,
            "id = \"x\"\ncreated = \"2026-01-01T00:00:00Z\"\nmodified = \"2026-01-01T00:00:00Z\"\n",
        )
        .unwrap();
        let s = load_sermon(&path).unwrap();
        assert_eq!(s.schema_version, SCHEMA_VERSION);
        assert!(s.movements.is_empty());
        assert!(s.planned_date.is_none());
    }

    #[test]
    fn scan_skips_corrupt_files_and_sorts_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        let mut older = sample();
        older.planned_date = Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 4).unwrap());
        let newer = sample();
        save_sermon(&new_sermon_path(dir.path(), &older), &older).unwrap();
        save_sermon(&new_sermon_path(dir.path(), &newer), &newer).unwrap();
        std::fs::write(dir.path().join("corrupt.toml"), "not = [valid").unwrap();
        let scanned = scan_sermons(dir.path());
        assert_eq!(scanned.len(), 2);
        assert_eq!(scanned[0].1.planned_date, newer.planned_date);
    }
}
