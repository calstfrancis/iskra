#!/usr/bin/env bash
# capture-screenshots.sh — capture a fresh screenshot of Iskra against demo data
#
# Runs the existing target/release/iskra binary (build one first with
# `cargo build --release` if it doesn't exist or is stale) under a fully
# isolated home — HOME, XDG_CONFIG_HOME, XDG_DATA_HOME, XDG_CACHE_HOME, and
# XDG_STATE_HOME are all redirected to a throwaway directory, so it never
# touches Cal's real config/sermons. Iskra is a hybrid here: `config.rs`
# resolves its config file via `shellexpand::tilde("~/.config/iskra")`,
# which only consults $HOME (like Rubric/Gost/Kopilka/Skrizhal) — but
# `main.rs`'s log dir and the welcome-marker file in `welcome_window.rs`
# both use `glib::user_data_dir()`, which prefers $XDG_DATA_HOME over $HOME
# when set (like Zerkalo). Since $XDG_CONFIG_HOME/$XDG_DATA_HOME are set to
# real paths on this machine, overriding only $HOME would leave those two
# glib-resolved paths pointing at Cal's real ~/.local/share — so, as with
# Zerkalo, all five variables need to be redirected together.
#
# Also runs under its own private D-Bus session (dbus-run-session) —
# GApplication enforces single-instance per app ID over the session bus, so
# without this, a real running Iskra instance (dev build or flatpak) would
# just get relay-activated instead of the throwaway one actually launching.
#
# Also forces GDK_BACKEND=x11 and unsets WAYLAND_DISPLAY: GTK4 otherwise
# prefers the real Wayland session and would render on the actual desktop
# instead of the isolated Xvfb display.
#
# No window manager is started — like Zerkalo/Skrizhal's scripts, this only
# needs the window to render for a screenshot, not to receive real input
# focus, and GTK4 places an undecorated first window at (0,0) by default
# against a bare X server.
#
# The demo sermon (screenshots/demo-sermon.toml) is entirely fictional — a
# generic, public-domain parable outline, not any of Cal's real sermon
# content — seeded directly as a sermon TOML file rather than driven through
# the UI, since the header's date picker would also need a resolved
# Revised Common Lectionary snapshot to render correctly, and reproducing
# that resolution outside the app isn't worth the fragility for a
# screenshot. The demo sermon has no planned date, so the lectionary sidebar
# shows its normal empty state.
#
# Requires: Xvfb, dbus-run-session, ImageMagick (magick), a built and
# current target/release/iskra binary.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BINARY="target/release/iskra"
if [[ ! -x "$BINARY" ]]; then
  echo "ERROR: $BINARY not found. Run 'cargo build --release' first." >&2
  exit 1
fi

DEMO_HOME=$(mktemp -d /tmp/iskra-demo-home.XXXXXX)
OUT="screenshots/iskra-main.png"
OUT_DARK="screenshots/iskra-main-dark.png"
WINDOW_W=1200
WINDOW_H=800

cleanup() {
  [[ -n "${APP_PID:-}" ]] && kill "$APP_PID" 2>/dev/null || true
  [[ -n "${XVFB_PID:-}" ]] && kill "$XVFB_PID" 2>/dev/null || true
  rm -rf "$DEMO_HOME"
}
trap cleanup EXIT

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

export HOME="$DEMO_HOME"
export XDG_CONFIG_HOME="$DEMO_HOME/.config"
export XDG_DATA_HOME="$DEMO_HOME/.local/share"
export XDG_CACHE_HOME="$DEMO_HOME/.cache"
export XDG_STATE_HOME="$DEMO_HOME/.local/state"

echo "==> Seeding demo home in $DEMO_HOME"
WORK_DIR="$DEMO_HOME/Documents/Iskra"
mkdir -p "$WORK_DIR/sermons" "$XDG_CONFIG_HOME/iskra" "$XDG_DATA_HOME/iskra"

cp screenshots/demo-sermon.toml "$WORK_DIR/sermons/demo-sermon.toml"

cat > "$XDG_CONFIG_HOME/iskra/config.toml" <<EOF
work_dir = "$WORK_DIR"
sidebar_visible = true
window_width = $WINDOW_W
window_height = $WINDOW_H
EOF

# Welcome/What's New dialog checks this marker against the current version.
echo -n "$VERSION" > "$XDG_DATA_HOME/iskra/.welcome_version"

# Isolated Xvfb display, well clear of any real display number in use.
DISPLAY_NUM=229
while [[ -e "/tmp/.X${DISPLAY_NUM}-lock" ]]; do
  DISPLAY_NUM=$((DISPLAY_NUM + 1))
done

echo "==> Starting isolated Xvfb on :$DISPLAY_NUM"
Xvfb ":$DISPLAY_NUM" -screen 0 "${WINDOW_W}x${WINDOW_H}x24" &
XVFB_PID=$!
sleep 2

export BINARY DISPLAY_NUM WINDOW_W WINDOW_H

# Capture the app once per colour scheme. libadwaita normally resolves
# light/dark from the desktop's settings portal, which on this machine is
# answered by a backend that ignores our isolated config and always reports
# light. ADW_DISABLE_PORTAL=1 makes libadwaita read the GSettings
# color-scheme key instead, and GSETTINGS_BACKEND=keyfile feeds it a value we
# write into the throwaway config — so we can force either scheme
# deterministically without touching the real desktop. (Iskra's own config
# defaults to Theme::System, which follows this.)
#
# The whole launch+capture runs *inside* dbus-run-session so the private bus
# (and the dbus-daemon it spawns) is torn down when the inner shell exits —
# backgrounding dbus-run-session and killing it instead leaves an orphaned
# daemon holding the script's stdout open.
capture_scheme() {
  local scheme="$1"
  export OUTFILE="$2"
  mkdir -p "$XDG_CONFIG_HOME/glib-2.0/settings"
  cat > "$XDG_CONFIG_HOME/glib-2.0/settings/keyfile" <<KEYFILE
[org/gnome/desktop/interface]
color-scheme='$scheme'
KEYFILE

  echo "==> Capturing Iskra ($scheme) -> $OUTFILE"
  dbus-run-session -- bash -c '
    env -u WAYLAND_DISPLAY GDK_BACKEND=x11 ADW_DISABLE_PORTAL=1 GSETTINGS_BACKEND=keyfile \
      DISPLAY=":$DISPLAY_NUM" "./$BINARY" &
    app=$!
    sleep 5
    DISPLAY=":$DISPLAY_NUM" magick x:root -crop "${WINDOW_W}x${WINDOW_H}+0+0" +repage "$OUTFILE"
    kill "$app" 2>/dev/null || true
    wait "$app" 2>/dev/null || true
  '
}

capture_scheme default     "$OUT"
capture_scheme prefer-dark "$OUT_DARK"

echo "Done. Wrote $OUT and $OUT_DARK"

# Publish web-ready copies into the personal website repo, one PNG + WebP per
# scheme, named as the site expects (<slug>.png/.webp + <slug>-dark.png/.webp).
# The capture crop already matches the site's image dimensions, so this is a
# straight convert+copy — no resize. Override the destination with
# WEBSITE_DIR=/path ./capture-screenshots.sh; if it doesn't exist the export is
# skipped with a note rather than failing. The website is a separate repo —
# commit and push it there yourself after reviewing the refreshed images.
SLUG="iskra"
WEBSITE_DIR="${WEBSITE_DIR:-$(dirname "$SCRIPT_DIR")/calstfrancis.github.io}"
if [[ -d "$WEBSITE_DIR" ]]; then
  echo "==> Publishing web images to $WEBSITE_DIR"
  cp "$OUT"      "$WEBSITE_DIR/$SLUG.png"
  cp "$OUT_DARK" "$WEBSITE_DIR/$SLUG-dark.png"
  magick "$OUT"      -quality 80 "$WEBSITE_DIR/$SLUG.webp"
  magick "$OUT_DARK" -quality 80 "$WEBSITE_DIR/$SLUG-dark.webp"
  echo "    wrote $SLUG.{png,webp} and $SLUG-dark.{png,webp}"
else
  echo "NOTE: website dir not found ($WEBSITE_DIR) — skipping web export."
fi
