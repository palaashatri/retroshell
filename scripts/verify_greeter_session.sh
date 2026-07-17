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

echo "==> DesktopNames=RetroShell on both session desktops"
grep -q "DesktopNames=RetroShell" "$ROOT/packaging/retroshell.desktop"
grep -q "DesktopNames=RetroShell" "$ROOT/packaging/retroshell-wayland.desktop"

echo "==> TryExec present on both session desktops (greeter can probe binary)"
grep -q "TryExec=start-retroshell" "$ROOT/packaging/retroshell.desktop"
grep -q "TryExec=start-retroshell" "$ROOT/packaging/retroshell-wayland.desktop"

echo "==> Keywords consistent on both session desktops"
grep -q "Keywords=RetroShell;Wayland;Desktop;" "$ROOT/packaging/retroshell.desktop"
grep -q "Keywords=RetroShell;Wayland;Desktop;" "$ROOT/packaging/retroshell-wayland.desktop"

echo "==> install-session-files.sh dry-run"
test -x "$ROOT/scripts/install-session-files.sh"
DRY_LOG="$(mktemp)"
"$ROOT/scripts/install-session-files.sh" --dry-run --prefix /tmp/retroshell-greeter-dryrun >"$DRY_LOG"
grep -q "wayland-sessions/retroshell.desktop" "$DRY_LOG"
grep -q "bin/start-retroshell" "$DRY_LOG"
rm -f "$DRY_LOG"

echo "==> session_entry_smoke_report source is honest (no live greeter claim)"
# Structural evidence only — never assert live DM. The Rust report hard-codes
# live_greeter_verified: false; this script only checks packaging + that honesty.
grep -q "session_entry_smoke_report" "$ROOT/crates/retro-shell/src/session_packaging.rs"
grep -q "live_greeter_verified: false" "$ROOT/crates/retro-shell/src/session_packaging.rs"
grep -q "live_greeter_verified" "$ROOT/crates/retro-shell/src/session_packaging.rs"

echo
echo "greeter session packaging smoke PASSED (no live DM required)"
echo "NOTE: live_greeter_verified remains false — packaging evidence only"