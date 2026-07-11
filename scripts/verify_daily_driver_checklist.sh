#!/usr/bin/env bash
# verify_daily_driver_checklist.sh — honest §12 / scorecard smoke (no live greeter claim).
#
# Exit 0 only if packaging + unit-testable artifacts look install-ready.
# Does NOT prove: greeter login, DRM seat, Orca, PipeWire streams, Plasma week.
#
# Toward §12 criterion 1 (greeter → session): packaging + install-session dry-run
# must pass. Live DM login remains host/hardware evidence, not this script.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail=0
pass() { echo "OK  $*"; }
warn() { echo "WARN $*"; }
die() { echo "FAIL $*"; fail=1; }

echo "==> packaging / greeter install artifacts"
if [[ -x scripts/verify_greeter_session.sh ]]; then
  if scripts/verify_greeter_session.sh; then
    pass "greeter packaging smoke"
  else
    die "greeter packaging smoke"
  fi
else
  die "missing verify_greeter_session.sh"
fi

echo "==> install-session-files.sh present + dry-run"
if [[ -x scripts/install-session-files.sh ]]; then
  if scripts/install-session-files.sh --dry-run --prefix /tmp/retroshell-session-dryrun >/tmp/retroshell-install-dryrun.log 2>&1; then
    if grep -q "DRY-RUN install" /tmp/retroshell-install-dryrun.log \
      && grep -q "wayland-sessions/retroshell.desktop" /tmp/retroshell-install-dryrun.log \
      && grep -q "bin/start-retroshell" /tmp/retroshell-install-dryrun.log; then
      pass "install-session-files dry-run"
    else
      die "install-session-files dry-run missing expected paths (see /tmp/retroshell-install-dryrun.log)"
    fi
  else
    die "install-session-files dry-run failed"
  fi
else
  die "missing or non-executable scripts/install-session-files.sh"
fi

echo "==> desktop keys consistent (DesktopNames, TryExec, Keywords)"
for f in packaging/retroshell.desktop packaging/retroshell-wayland.desktop; do
  for key in DesktopNames TryExec Keywords; do
    if grep -qE "^${key}=" "$f"; then
      pass "$f has $key"
    else
      die "$f missing $key"
    fi
  done
  if ! grep -q "DesktopNames=RetroShell" "$f"; then
    die "$f DesktopNames must be RetroShell"
  fi
  if ! grep -q "TryExec=start-retroshell" "$f"; then
    die "$f TryExec must be start-retroshell"
  fi
  if ! grep -q "Keywords=RetroShell;Wayland;Desktop;" "$f"; then
    die "$f Keywords must match RetroShell;Wayland;Desktop;"
  fi
done

echo "==> start-retroshell documents OUTPUTS_LAYOUT + compositor selection"
if grep -q "RETROSHELL_OUTPUTS_LAYOUT" scripts/start-retroshell; then
  pass "start-retroshell documents RETROSHELL_OUTPUTS_LAYOUT"
else
  die "start-retroshell missing RETROSHELL_OUTPUTS_LAYOUT docs/export"
fi
if grep -q "compositor selection" scripts/start-retroshell; then
  pass "start-retroshell logs compositor selection honestly"
else
  die "start-retroshell missing honest compositor selection logs"
fi

echo "==> pure module presence (warpath integration targets)"
for f in \
  crates/retro-shell/src/session_actions.rs \
  crates/retro-shell/src/display_arrange.rs \
  crates/retro-shell/src/window_rules.rs \
  crates/retro-shell/src/idle_policy.rs \
  crates/retro-shell/src/i18n.rs \
  crates/retro-shell/src/portal_extra.rs \
  crates/retro-shell/src/a11y_actions.rs \
  crates/retro-shell/src/session_packaging.rs \
  crates/retro-compositor/src/lib.rs
do
  if [[ -f "$f" ]]; then pass "exists $f"; else die "missing $f"; fi
done

echo "==> greeter readiness stays honest (no live DM claim in notes path)"
if rg -q "live greeter login still requires DM" crates/retro-shell/src/session_packaging.rs; then
  pass "session_packaging honest greeter note present"
else
  die "session_packaging missing honest greeter note"
fi
if rg -q "install_ready" crates/retro-shell/src/session_packaging.rs \
  && rg -q "Does \*\*not\*\* claim a live display manager" crates/retro-shell/src/session_packaging.rs; then
  pass "install_ready documented as packaging-only"
else
  die "install_ready honesty comment missing"
fi

echo "==> compositor workspace filter is referenced from main (live path)"
if rg -q "workspace_state|is_visible|windows_visible_for_paint" crates/retro-compositor/src/main.rs; then
  pass "compositor main references workspace visibility"
else
  die "compositor main missing workspace visibility wiring"
fi

echo "==> portal Secret/Print/Inhibit on dbus module"
if rg -q "PortalSecretIface|PortalPrintIface|PortalInhibitIface" crates/retro-shell/src/portal_dbus.rs; then
  pass "portal_dbus exports Secret/Print/Inhibit interfaces"
else
  die "portal_dbus missing Secret/Print/Inhibit"
fi

echo "==> i18n used outside catalog module"
if rg -q '\btr\(' crates/retro-shell/src/lib.rs crates/retro-shell/src/menu_server.rs 2>/dev/null; then
  pass "tr() used in shell UI paths"
else
  warn "tr() may still be lock-only — check lib.rs"
fi

echo "==> host unit tests (exclude full compositor binary if needed)"
if command -v cargo >/dev/null; then
  if cargo test -p retro-shell -p retro-kit -p retro-compositor --lib --quiet; then
    pass "cargo test shell+kit+compositor lib"
  else
    die "cargo test failed"
  fi
else
  warn "cargo not on PATH — skipped unit tests"
fi

echo
if [[ "$fail" -ne 0 ]]; then
  echo "daily-driver checklist FAILED (install/unit evidence incomplete)"
  echo "NOTE: failure here is packaging/unit evidence only — not a live greeter result"
  exit 1
fi
echo "daily-driver checklist PASSED (packaging + unit evidence only — not Plasma-100)"
echo "§12 greeter criterion: install artifacts ready; live DM login still NOT RUN"
exit 0
