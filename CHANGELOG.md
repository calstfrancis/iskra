# Changelog

All notable changes to Iskra are recorded here.  
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [0.2.1-dev2] — Movement tools, multi-select, and workflow shortcuts

### Added
- Right-click blank space in a movement to split it: everything from that point down moves into a new movement inserted right below.
- Multi-select ideas within a movement via rubber-band drag, or Ctrl/Shift-click on an idea's number. Selected ideas can be dragged together, and bulk-deleted with Delete/BackSpace or a right-click "Delete N ideas" menu item.
- "Merge with movement above" button on every movement (disabled on the first one) — the inverse of split, folding a movement's ideas onto the previous one and removing it.
- Idea/part tag quick-filter: Ctrl+click a tag chip to dim every idea that doesn't carry that tag.
- "Rename everywhere" in the idea/part tag popover — retypes that tag's value on every idea in the sermon that currently shares it, not just the one being edited.
- Collapse All / Expand All Movements commands in the command palette (Ctrl+K).
- Alt+Shift+Up/Down jumps the focused idea or movement straight to the top/bottom of its list, alongside the existing single-step Alt+Up/Down.
- "Recently deleted" tray in the status bar — a session-scoped safety net listing the last 20 deleted ideas/movements with one-click restore, alongside undo.
- "Copy movement to another sermon…" button on every movement header, opening a sermon picker and appending a duplicate of the movement (fresh ids) to the chosen sermon's file.
- Right-click a lectionary reading in the sidebar to add it as a scripture tag on the sermon.

### Changed
- Idea/part tags moved from a separate row hanging below each idea bar into small inline chips within the bar itself, right after the idea text — every idea is now exactly one row tall regardless of whether it's tagged, instead of always reserving a second row (even when untagged, as a ghosted placeholder). An untagged chip collapses to a bare "+" icon rather than a full-width ghost pill, so untagged ideas cost only a small icon's worth of extra width, not a whole placeholder tag.
- The idea row's drag handle moved to the left of the number, ahead of the idea text.
- Status bar's sermon-tag group labels spelled out as "Scripture"/"Themes" instead of "s."/"t.".

### Fixed
- Clicking "+ Add movement" (button or command palette) left nothing focused after the rebuild, so GTK's default focus-chain silently jumped to the first movement's name entry instead — new movements now focus their own name entry, matching "Duplicate movement".

## [0.2.0] "First Light" — Workflow improvements: templates, preaching view, history

### Added
- Keyboard reordering: Alt+Up/Alt+Down moves the focused idea or movement, as an alternative to dragging.
- Movement cards now have a delete button (previously only removable via the underlying command, with no UI affordance).
- Duplicate buttons on both ideas and movements (duplicating a movement also duplicates all its ideas, each with a fresh id).
- "New Sermon" in the library is now a menu offering built-in templates (Three-Point Expository, Narrative Arc, Topical) alongside a blank sermon.
- Preaching View (Ctrl+Shift+P, or hamburger menu): a large-print, chrome-free fullscreen display of the sermon's movements and ideas for pulpit use.
- A "Saved"/"Unsaved changes" indicator in the status bar, with the last-saved time.
- A toast notification when adding a scripture tag that was also used in a past sermon, naming that sermon and its date.
- Quick-pick tag chips: the idea/part tag popovers now show the most-frequently-used values as one-click chips when the entry is empty.
- History… (Ctrl+Shift+H, or hamburger menu): browse past committed versions of the open sermon (via its git backup history) and restore any of them.
- Sermons can now belong to a named series, set from the title/date popover; the library sidebar groups and filters by series.

### Fixed
- Undoing or redoing a change never marked the sermon dirty, so autosave could silently skip an undone/redone edit.
- The command palette's "jump to movement" outline entries never actually moved focus, because a movement card's root container isn't itself focusable.
- Git sync's dev8 fix for the mid-rebase false alarm checked for capitalized "No rebase in progress," but git 2.55 prints it lowercase — so on that git version, every non-conflict pull failure (auth, network, an empty remote) still fell through to the scary "repository may be in mid-rebase state" message instead of the real cause. The check is now case-insensitive.
- The very first sync to a brand-new, completely empty backup remote (freshly created on GitHub, nothing pushed yet) always failed: `pull --rebase` errors with "couldn't find remote ref" since there's nothing there, and that was being treated like any other pull failure — skipping the push entirely. That specific failure now falls through straight to the push instead.
- Dragging an idea onto an *existing* movement's ideas list always silently rejected the drop: the ideas box lives inside a `Revealer` (added for collapse/expand), and the lookup that finds it by CSS class only checked a movement card's direct children, never descending into the revealer — so it always came back empty and the drop handler treated that as "nothing to drop onto." Now finds it as a descendant, wherever it's nested.
- Dragging an idea anywhere on the movements column could send the view scrolling rapidly downward and not stop: autoscroll compared the pointer's position in the (very tall, mostly off-screen) movements column against the *scroller's* viewport height, two unrelated coordinate spaces — so almost any drag past the very top of the document read as "near the bottom edge." The pointer position is now translated into the scroller's own coordinate space before that check.
- Any structural change (drag-drop reorder, deleting a movement or idea, toggling collapse, ...) rebuilds the whole editor from the model, and if the widget that had keyboard focus was one of the ones just torn down, GTK reassigned focus on its own — in practice always landing on and selecting the first movement's title. Focus is now explicitly cleared first whenever it's about to be destroyed, so a rebuild no longer jumps the view anywhere.

## [0.1.0-dev8] — Fast entry, drag fixes, sync robustness

### Added
- Clicking "+ Add idea" (or pressing Enter at the end of an idea) now focuses the new idea's text field immediately, instead of leaving focus on the movement name — lets you add a string of ideas without touching the mouse.
- "Show Sermons Folder" in the hamburger menu opens `work_dir/sermons/` in the system file manager, so it's no longer a mystery where sermons are saved.

### Fixed
- Dragging an idea onto a movement's header/name (not just its ideas list) used to create a brand-new movement instead of adding the idea to the one you dropped it on — the drop-zone math only treated the ideas list as "this movement," so drops on the header fell through to blank-space handling.
- Dragging an idea over an Entry (idea text, movement name, tag popovers) inserted the drag's raw payload text ("idea:&lt;uuid&gt;") into the field instead of reordering — `Entry` has its own built-in text-drop handling that intercepted our payload because it shared `GType` with plain text. The drag payload is now a distinct type so it never matches.
- Idea/part tag tabs moved from hanging below-left of the idea bar to hanging from its right edge.
- Git sync mishandled failures that happened *before* a rebase started (auth, network, no such branch): it unconditionally ran `rebase --abort`, and when that itself failed with "No rebase in progress," the user saw a scary "repository may be in mid-rebase state" message for a repo that was already clean, with the real cause (e.g. an expired GitHub token) buried underneath.
- A real merge conflict during sync could be misreported as an unrelated failure (or vice versa): the internal helper that reads git's command output picked *either* stdout *or* stderr, and git splits a rebase conflict's diagnostics across both — the actual `CONFLICT (content): ...` marker was silently dropped whenever stderr was non-empty. Sync conflict detection is covered by a new test that reproduces a real conflict against a throwaway local git repo.

## [0.1.0-dev7] — Visual polish fixes

### Fixed
- Dragging an idea onto an empty movement could draw the drop-indicator line in the wrong place (below the "No ideas yet" placeholder) even though the idea always actually lands at the top; the placeholder is now correctly ignored by the drag insertion-point math.
- The movement name field lost its focus highlight in the dev6 redesign; restored (via the movement header, not the entry itself).
- The expanded-notes box and the idea/part tag tabs beneath it no longer lined up after the dev6 redesign; realigned.
- The movement collapse triangle's tooltip stayed "Collapse movement" even when already collapsed.
- Idea numbers 10 and above stretched the round number badge into an oval; now a consistent width regardless of digit count.

## [0.1.0-dev6] — Visual polish

### Changed
- Idea rows are now rounded bars (matching the original mockup) instead of flat, unboxed rows, with a narrower inset rounded box for expanded notes and two coloured tabs (idea tag / part tag) hanging below.
- Empty idea/part tags now render ghosted/muted instead of in the same bold colour as a genuinely-set tag, so it's clear at a glance which ideas are actually tagged.
- Movement cards get a visibly tinted background, border, and soft shadow so idea bars read as clearly nested within them.
- Idea numbers are now small accent-coloured badges instead of plain "1." text.
- The drag grabber and delete-idea icons are dimmed by default and brighten on hover, so they recede until needed; the movement collapse triangle is similarly dimmed relative to the (now bold) movement name.
- A movement with no ideas yet shows a dashed placeholder instead of an empty gap.
- Scripture (s.) and theme (t.) tag chips in the status bar are now colour-coded to match the idea/part tag language used on idea rows.

## [0.1.0-dev5] — Export, command palette, git backup

### Added
- `.sermon` export: renders the open sermon into Rubric's Typst subset (movements → headings, ideas → continuously-numbered bullets, notes → indented continuation lines, optional idea/part tag suffix) with a live preview dialog (Ctrl+E / hamburger menu → Export…). See `Plans/sermon-interchange-spec.md`.
- Command palette (Ctrl+K): search commands plus every movement/idea in the open sermon as jump targets, doubling as a document outline.
- Error toasts for autosave failure, opening a corrupt sermon, creating a new sermon, and deleting a sermon — previously these failures were silent.
- "No matching commands" empty state in the command palette.
- Git backup: sign in with GitHub (device flow), create or link a repository, and sync (commit + push) from the header's sync button or Ctrl+Shift+G. Sign-in token stored in the system keyring, scoped to `https://github.com/` only. Repo is `~/Documents/Iskra/`.

### Fixed
- `Plans/sermon-interchange-spec.md`'s escape table had a stale entry for backslash (`\\`) that didn't match Rubric's actual `rubric_package/utils/typst.py` source (`\u{5c}`) — corrected, and Iskra's exporter was written against the source directly.

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
