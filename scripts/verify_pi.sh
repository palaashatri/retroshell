#!/usr/bin/env bash
# RetroShell — Raspberry Pi / native Linux verification
# Run on the Pi (or any Linux host with GPU/Wayland deps).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REPORT="/tmp/retroshell-pi-verify-$(date +%Y%m%d-%H%M%S).txt"
exec > >(tee "$REPORT") 2>&1

echo "=== RetroShell Pi/Linux verification ==="
echo "date: $(date -Iseconds)"
echo "host: $(hostname) $(uname -a)"
echo "pwd:  $ROOT"
echo

echo "=== Phase 1: packages (Debian/Ubuntu) ==="
if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential pkg-config curl git \
    libwayland-dev libwayland-egl-backend-dev \
    libvulkan-dev libegl1-mesa-dev libgles2-mesa-dev \
    libxkbcommon-dev libdbus-1-dev libfontconfig-dev libfreetype6-dev \
    libudev-dev libinput-dev libgbm-dev libdrm-dev libseat-dev libsystemd-dev \
    libxcb1-dev libxcb-icccm4-dev libxcb-keysyms1-dev libxcb-randr0-dev \
    libxcb-util0-dev libxcb-xfixes0-dev \
    mesa-utils vulkan-tools \
    xwayland at-spi2-core pulseaudio-utils \
    network-manager || true
else
  echo "apt-get not found; ensure build deps are installed manually"
fi

if ! command -v rustc >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi
echo "rustc: $(rustc --version)"
echo

echo "=== Phase 2: unit tests ==="
cargo test --workspace 2>&1 | tail -60
echo

echo "=== Phase 3: release build ==="
cargo build --release --workspace 2>&1 | tail -30
echo
ls -la target/release/retro-shell target/release/retro-compositor \
  target/release/finder target/release/settings target/release/terminal \
  target/release/textedit target/release/appstore 2>&1 || true
echo

echo "=== Phase 4: capability probes ==="
echo "-- DRI / GPU --"
ls -la /dev/dri 2>&1 || true
command -v glxinfo >/dev/null && glxinfo -B 2>&1 | head -20 || true
command -v vulkaninfo >/dev/null && vulkaninfo --summary 2>&1 | head -40 || true

echo "-- NetworkManager --"
busctl status org.freedesktop.NetworkManager 2>&1 | head -10 || true
nmcli -t -f STATE,CONNECTIVITY g 2>&1 || true

echo "-- Audio --"
pactl info 2>&1 | head -15 || true
wpctl status 2>&1 | head -20 || true

echo "-- UPower / battery --"
busctl status org.freedesktop.UPower 2>&1 | head -8 || true
ls /sys/class/power_supply/ 2>&1 || true

echo "-- AT-SPI --"
busctl --user list 2>&1 | grep -i a11y || true
echo

echo "=== Phase 5: compositor smoke (30s) ==="
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/runtime-$USER}"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

if [ -n "${DISPLAY:-}" ] || [ -n "${WAYLAND_DISPLAY:-}" ]; then
  timeout 15 ./target/release/retro-compositor > /tmp/retro-compositor-pi.log 2>&1 &
  CPID=$!
  sleep 3
  if kill -0 "$CPID" 2>/dev/null; then
    echo "retro-compositor still running after 3s (good)"
    kill "$CPID" 2>/dev/null || true
  else
    echo "retro-compositor exited early; log:"
    tail -40 /tmp/retro-compositor-pi.log || true
  fi
else
  echo "No DISPLAY/WAYLAND_DISPLAY; skip live compositor (start a session first)"
fi

echo
echo "=== Report written to $REPORT ==="
echo "Next: run under a real session:"
echo "  export RETROSHELL_LOCK_PASSWORD=test"
echo "  ./target/release/retro-compositor &"
echo "  sleep 1; ./target/release/retro-shell"
