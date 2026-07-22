# Iskra — Claude Instructions

Iskra is the newest app in the Fond suite. Reference `../CLAUDE.md` (repo root) for suite-wide conventions (dev build / release workflow, versioning, flatpak publishing) — this file only covers what's Iskra-specific.

## Build and release rules

- **Never build or release unless Cal explicitly says to.** Code changes alone do not trigger a build.
- Saying "prep a dev build for iskra" triggers a build. Saying "release iskra" triggers a release. Nothing else does.

## Version Management

Same scheme as the rest of the suite (see root `CLAUDE.md`): dev builds are `X.Y.Z-devN`, releases strip the suffix. Version lives in `Cargo.toml`, and the release name in `RELEASE_NAME` in `src/ui/welcome_window.rs` (matching Zerkalo's pattern).

- `RELEASE_NAME` holds the **last shipped release's** name, since the welcome window renders `Version {VERSION} "{RELEASE_NAME}"` on every launch, including dev builds. Bump it at release time only — never during a dev build, where the version has moved on but the name hasn't been chosen yet. It went stale exactly this way once: the constant sat on v0.2.0's "First Light" straight through the v0.3.0 "Kindled Verse" release.

- Update `CHANGELOG.md` on every dev build (in place for repeated dev builds of the same version, not a new heading each time).
- **Never** add a metainfo `<release>` entry for a dev build — only at actual release time.

## Code Style

- No comments unless the WHY is non-obvious.
- No multi-line docstrings or comment blocks.
- No trailing summaries at the end of responses — the user can read the diff.

## Architecture

- **Framework: raw gtk4-rs, not relm4** — matches Zerkalo exactly (pins: gtk4 0.7/v4_10, libadwaita 0.5/v1_4, glib 0.18). Programmatic widgets only, no `.ui`/Blueprint/gresource.
- **Single door**: every mutation to the open sermon goes through `apply(Cmd)` in `src/ui/app_window.rs`. Never mutate `AppState.sermon` directly from a widget callback — build a `Cmd` and route it through `apply` so undo/redo and autosave stay correct.
- **Full rebuild on structural change**: adding/deleting/moving an idea or movement rebuilds the whole editor widget tree from the model rather than patching individual widgets. Pure text edits (typing in an idea/notes/title field) do not rebuild — see `Cmd::is_structural`.
- **Undo/redo**: `src/commands.rs`. Every command carries its own inverse data. Text edits coalesce into one undo step per typing burst (see `UndoStack::push_applying`) — don't bypass `apply` for text fields or coalescing breaks.
- New UI panels go in `src/ui/` as their own file, registered in `src/ui/mod.rs`, following Zerkalo's one-file-per-panel convention.

## Persistence

- One TOML file per sermon under `work_dir/sermons/` (`~/Documents/Iskra/sermons/` by default). Filenames are generated once at creation and never renamed — the sermon's `id` field is the real identity.
- Every serializable struct needs `#[serde(default)]` on new fields so old sermon files never fail to load after a schema change. Bump `model::SCHEMA_VERSION` only for breaking changes, and keep `storage::load_sermon`'s version gate in sync.
- Config at `~/.config/iskra/config.toml`, same `atomic_write` (temp-file + rename) pattern as Zerkalo's `config.rs`/`error.rs`.

## Git Sync

Lifted from Zerkalo's `git_sync.rs`/`github_auth.rs`/`secret_store.rs` — **git ops shell out to the `git` CLI** (via `flatpak-spawn --host` when sandboxed), not git2, matching what Zerkalo's code actually does (its own CLAUDE.md's claim to the contrary is stale — see Zerkalo's `git_sync.rs`). git2 is only used for local repo discovery/init/identity. Iskra needs its own GitHub OAuth App client ID — do not reuse Zerkalo's or Rubric's.

## Flatpak packaging

- `packaging/cargo-sources.json` vendors all crate dependencies for the offline flatpak build (the `iskra-deps` module in the manifest copies it to `/app/iskra-cargo-vendor`). It must be regenerated whenever `Cargo.lock` changes — a stale one makes `dev-build.sh`/`publish-flatpak.sh` fail at the `cp cargo/vendor` step with "No such file or directory" because the vendor dir it describes no longer matches what's actually in the lockfile.
- Regenerate with [`flatpak-cargo-generator.py`](https://github.com/flatpak/flatpak-builder-tools/blob/master/cargo/flatpak-cargo-generator.py) (needs `aiohttp`, `PyYAML`, `tomlkit` — use a venv, this system's Python is externally managed):
  ```bash
  python3 -m venv /tmp/fcg-venv
  /tmp/fcg-venv/bin/pip install 'aiohttp<4.0.0,>=3.9.5' 'PyYAML<7.0.0,>=6.0.2' 'tomlkit>=0.13.3,<1.0'
  curl -sL -o /tmp/flatpak-cargo-generator.py https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
  /tmp/fcg-venv/bin/python3 /tmp/flatpak-cargo-generator.py Cargo.lock -o packaging/cargo-sources.json
  ```
- Commit the regenerated file whenever a dependency is added, removed, or bumped — don't leave it to Cal to discover via a failed build.

## Error Handling

- Use `thiserror` types in `src/error.rs`; don't `unwrap()`/`expect()` in UI code paths.

## Testing

- Model, storage, and command-stack logic must stay unit-testable without a display (no GTK init required) — see the `#[cfg(test)]` modules in `src/model.rs`, `src/storage.rs`, `src/commands.rs` for the pattern. Run `cargo test` before considering a change to those files done.
