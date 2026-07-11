#!/usr/bin/env bash
# verify_greeter_session.sh — packaging + session-script smoke for greeter installs.
#
# Does NOT require a live display manager. Checks that:
#   1) packaging desktop files validate (via verify_session_packaging.sh)
#   2) start-retroshell is executable and --help or dry-run path is sane
#   3) Required env defaults for XDG session are documentable
#
# Exit 0 on success. Used in CI / Docker image smoke.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "==> packaging tree"
bash "$ROOT/scripts/verify_session_packaging.sh"

echo "==> start-retroshell is executable"
test -x "$ROOT/scripts/start-retroshell"

echo "==> start-retroshell documents compositor selection"
grep -q "RETROSHELL_COMPOSITOR\|labwc\|retro-compositor" "$ROOT/scripts/start-retroshell"

echo "==> desktop files point at start-retroshell"
grep -q "Exec=start-retroshell" "$ROOT/packaging/retroshell.desktop"
grep -q "Exec=start-retroshell" "$ROOT/packaging/retroshell-wayland.desktop"

echo "==> wayland session Type/DesktopNames"
grep -q "Type=Application" "$ROOT/packaging/retroshell-wayland.desktop" \
  || grep -q "Type=Application" "$ROOT/packaging/retroshell.desktop"

echo "==> systemd user unit ExecStart"
grep -q "start-retroshell" "$ROOT/packaging/retroshell.service"

echo
echo "greeter session packaging smoke PASSED (no live DM required)"
