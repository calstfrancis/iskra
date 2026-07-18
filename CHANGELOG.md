# Changelog

All notable changes to Iskra are recorded here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.1.0-dev4] — Library

### Added
- Sermon library: search (title, idea/note text, movement names, tags, secular date, lectionary week), a clickable scripture/theme tag sidebar with occurrence counts, new/open/delete, and a badge marking the currently-open sermon (which can't be deleted from the library).
- Idea/part tag autocomplete: typing in an idea's tag tab now suggests values already used anywhere in the library, not just the open sermon.
- `Ctrl+L` opens the library, alongside the header's Library button (now enabled).

## [0.1.0-dev3] — Header, lectionary & tags

### Added
- Title/date popover in the header: click the sermon title to rename it and pick a planned date from a calendar; picking a date resolves the Revised Common Lectionary reading for that Sunday as one undo step.
- `rcl` module: a from-scratch Rust port of Rubric's lectionary engine (readings table, Easter/Advent/Proper calendar math), so Iskra resolves the same season, colour, and readings as Rubric for any given date.
- Lectionary sidebar panel: season name, colour swatch, week label, and the day's OT/Psalm/Epistle/Gospel readings, with an empty state before a date is planned.
- Status bar: scripture (`s.`) and theme (`t.`) tag chips, addable and removable, for tagging a sermon's scripture references and themes.
- Changelog window, opened from the version button in the status bar, rendering this file live.
- Welcome window on first run, and a "What's New" window after an update.
- Window geometry, sidebar width, and sidebar visibility now persist across launches (debounced, ignoring GTK's initial layout pass).

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
