#!/usr/bin/env bash
# Reproduction for the cheatsheet window placement bugs (requires X11).
#
# Spec (specs/cheatsheet-specs.md):
#   - the window starts in the top right corner, flush to the screen edges;
#   - the arrow keys move it between the four corners;
#   - with no shortcuts to show, the window is hidden.
#
# Observed on KWin/X11: the window opens at the WM's default placement
# (~centered), arrow keys persist the new corner to settings.json but the
# window never moves, and it stays visible with an empty table.
#
# Usage: tests/integration/window-position.sh [path-to-binary]
# Exits 0 if all checks pass, 1 on the first failure, 77 when skipped.

set -u

BIN="${1:-$(dirname "$0")/../../src-tauri/target/debug/shortcut-cheatsheet}"

if [[ -z "${DISPLAY:-}" ]] || ! command -v xdotool >/dev/null; then
  echo "SKIP: needs DISPLAY and xdotool" >&2
  exit 77
fi
if [[ ! -x "$BIN" ]]; then
  echo "SKIP: app binary not built ($BIN)" >&2
  exit 77
fi

fail() { echo "FAIL: $*" >&2; FAILURES=$((FAILURES + 1)); }
FAILURES=0

"$BIN" & APP_PID=$!
trap 'kill $APP_PID 2>/dev/null' EXIT
sleep 3

WIN=$(xdotool search --name "Shortcut Cheatsheet" | head -1)
[[ -n "$WIN" ]] || { echo "FAIL: window did not open" >&2; exit 1; }

# The window should be hidden until a tracked application is focused: the
# shortcut database only seeds Chrome entries and nothing has been focused.
# (skipTaskbar hides it from wmctrl, so query via xdotool.)
map_state=$(xprop -id "$WIN" WM_STATE 2>/dev/null || true)
if [[ -z "$map_state" ]]; then
  echo "ok: window hidden while no app has shortcuts (empty state)"
else
  fail "window is mapped with no shortcuts to show — renders an empty blob"
fi

# Top-right corner of the primary work area.
read -r WORK_X WORK_Y WORK_W WORK_H < <(
  xprop -root -notype _NET_WORKAREA | tr '=,' ' ' | awk '{print $2, $3, $4, $5}'
)
read -r WIN_X WIN_Y WIN_W WIN_H < <(
  xdotool getwindowgeometry "$WIN" |
    awk '/Position:/{split($2,p,","); x=p[1]; y=p[2]}
         /Geometry:/{split($2,g,"x"); print x, y, g[1], g[2]}'
)

[[ -n "$WORK_W" && -n "$WIN_W" ]] || { echo "FAIL: could not read work area or window geometry" >&2; exit 1; }

right=$((WORK_X + WORK_W - WIN_W))
top=$WORK_Y
if [[ "$WIN_X" -eq "$right" && "$WIN_Y" -eq "$top" ]]; then
  echo "ok: window starts flush in the top-right corner"
else
  fail "window opened at ${WIN_X}x${WIN_Y}, expected ${right}x${top} (top-right of work area)"
fi

# ArrowDown must move it to the bottom-right corner.
xdotool windowactivate "$WIN" && sleep 0.3
xdotool key Down
sleep 0.5
NEW_Y=$(xdotool getwindowgeometry "$WIN" | sed -n 's/.*Position: [0-9]*,\([0-9]*\).*/\1/p')
want=$((WORK_Y + WORK_H - WIN_H))
if [[ "$NEW_Y" -eq "$want" ]]; then
  echo "ok: ArrowDown moved the window to the bottom edge"
else
  fail "ArrowDown left the window at y=$NEW_Y, expected y=$want"
fi

if [[ "$FAILURES" -gt 0 ]]; then
  echo "$FAILURES check(s) failed" >&2
  exit 1
fi
echo "all window placement checks passed"
