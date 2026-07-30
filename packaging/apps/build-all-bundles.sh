#!/usr/bin/env bash
# Package the five first-party RetroShell apps as .app bundles.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCRIPT="$ROOT/packaging/apps/build-app-bundle.sh"
OUTDIR="${OUTDIR:-/tmp/Applications}"
VERSION="0.1.0"

mkdir -p "$OUTDIR"

build() {
  local crate="$1" name="$2" id="$3" icon="$4"
  if [ -n "$icon" ] && [ -f "$ROOT/$icon" ]; then
    bash "$SCRIPT" "$crate" "$name" "$id" "$VERSION" "$OUTDIR" "$ROOT/$icon"
  else
    bash "$SCRIPT" "$crate" "$name" "$id" "$VERSION" "$OUTDIR"
  fi
}

build finder "Finder" com.retro.finder "themes/platinum/icons/finder.png"
build settings "Settings" com.retro.settings "themes/platinum/icons/settings.png"
build textedit "TextEdit" com.retro.textedit "themes/platinum/icons/textedit.png"
build terminal "Terminal" com.retro.terminal "themes/platinum/icons/terminal.png"
if [ -f "$ROOT/themes/platinum/icons/appstore.png" ]; then
  build appstore "App Store" com.retro.appstore "themes/platinum/icons/appstore.png"
else
  build appstore "App Store" com.retro.appstore ""
fi

echo "Bundles in $OUTDIR:"
ls -d "$OUTDIR"/*.app