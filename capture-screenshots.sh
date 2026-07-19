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

echo "==> Launching Iskra against demo data inside the isolated display"
dbus-run-session -- env -u WAYLAND_DISPLAY GDK_BACKEND=x11 DISPLAY=":$DISPLAY_NUM" "./$BINARY" &
APP_PID=$!

echo "==> Waiting for window to render"
sleep 5

echo "==> Capturing screenshot"
DISPLAY=":$DISPLAY_NUM" magick x:root -crop "${WINDOW_W}x${WINDOW_H}+0+0" +repage "$OUT"

echo "Done. Wrote $OUT"
