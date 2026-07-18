# Changelog

All notable changes to Iskra are recorded here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.1.0-dev2] — Movements & drag-and-drop

### Added
- Movement cards: editable name, collapse triangle (ideas hidden behind a `Revealer`, still accept idea drops when collapsed), and a drag grabber to reorder whole movements.
- Full drag-and-drop: reorder ideas within a movement, move ideas between movements, reorder movements themselves, and drag an idea into blank space above/below/between movements to spin up a new movement holding it — all as single undo steps.
- "+ Add movement" affordance alongside each movement's own "+ Add idea".
- Numbering stays continuous and correct across all of the above, recomputed on every change.

### Fixed
- Ctrl+Z/Ctrl+Shift+Z now work while a text entry has focus — the shortcut was previously swallowed by whichever entry currently had focus, since the window's key controller listened on the default bubble phase; it now listens on the capture phase instead.

## [0.1.0-dev1] — Skeleton & ideas

### Added
- Window shell in Zerkalo's design language: header bar (Library button, sermon title + planned date, undo/redo, hamburger), a collapsible sidebar reserved for the lectionary panel (dev3), and a status bar with tag placeholders and the version indicator.
- Idea bars: add, delete, click-to-edit text, expansion triangle with a notes area, and idea/part tag tabs.
- Continuous auto-numbering of ideas, recomputed on every change.
- Single-sermon TOML persistence (one human-readable, git-diffable file per sermon) with atomic writes and autosave.
- Undo/redo command stack (`Cmd`/`UndoStack`) covering every structural and text edit, with typing coalesced into single undo steps.
