# Iskra (Искра)

Sermon planning for preachers who work from structured notes rather than a full manuscript. Jot single-line ideas, expand any of them with longer notes, group them into movements, drag everything into order, tag it, and export a clean outline for [Rubric](https://github.com/calstfrancis/rubric) to drop into its sermon field.

Part of the Fond suite (alongside Zerkalo, Rubric, Kopilka, Gost, Skrizhal, Chered) — same monastic-scribal naming and visual identity, and the same GTK4/libadwaita design language pioneered in Zerkalo.

## Status

Early development (v0.1.0-dev). See `Plans/plan.md` for the full design and milestone plan, and `CHANGELOG.md` for what's landed so far.

## Building

```bash
cargo build --release
```

Requires GTK 4.10+ and libadwaita 1.4+ development packages. Targets openSUSE Tumbleweed / KDE Plasma / Wayland, but should run on any modern Linux desktop.

## Data

Sermons are stored as one TOML file per sermon under `~/Documents/Iskra/sermons/` (configurable via `~/.config/iskra/config.toml`). The format is designed to produce clean, readable git diffs — the folder is also the git-backup repository once sync is set up (v0.1.0).

## License

MIT — see `LICENSE`.
