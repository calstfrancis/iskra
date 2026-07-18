# Changelog

All notable changes to Iskra are recorded here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.1.0-dev1] — Skeleton & ideas

### Added
- Window shell in Zerkalo's design language: header bar (Library button, sermon title + planned date, undo/redo, hamburger), a collapsible sidebar reserved for the lectionary panel (dev3), and a status bar with tag placeholders and the version indicator.
- Idea bars: add, delete, click-to-edit text, expansion triangle with a notes area, and idea/part tag tabs.
- Continuous auto-numbering of ideas, recomputed on every change.
- Single-sermon TOML persistence (one human-readable, git-diffable file per sermon) with atomic writes and autosave.
- Undo/redo command stack (`Cmd`/`UndoStack`) covering every structural and text edit, with typing coalesced into single undo steps.
