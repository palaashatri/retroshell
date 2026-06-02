#!/usr/bin/env bash
set -euo pipefail

echo "=== RetroShell Build Script ==="
echo ""

# Check for Rust
if ! command -v rustc &>/dev/null; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "Rust version: $(rustc --version)"
echo ""

# Install system dependencies (Ubuntu/Debian)
if [ -f /etc/os-release ]; then
    . /etc/os-release
    if [[ "$ID" == "ubuntu" || "$ID" == "debian" ]]; then
        echo "=== Installing system dependencies ==="
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            pkg-config \
            libwayland-dev \
            libwayland-egl-backend-dev \
            libvulkan-dev \
            libegl1-mesa-dev \
            libgles2-mesa-dev \
            libxkbcommon-dev \
            libdbus-1-dev \
            libfontconfig-dev \
            libfreetype6-dev \
            mesa-utils \
            vulkan-validationlayers \
            vulkan-tools \
            cmake \
            libsystemd-dev \
            pipewire \
            libpipewire-0.3-dev \
            libspa-0.2-dev
    fi
fi

echo "=== Building RetroShell ==="
echo ""

# Build all workspace crates
cargo build --release 2>&1

echo ""
echo "=== Build complete ==="
echo ""
echo "Binaries:"
echo "  target/release/retro-shell   - Desktop environment"
echo "  target/release/finder        - File manager"
echo "  target/release/settings      - System settings"
echo "  target/release/textedit      - Text editor"
echo ""

# Check if running under Wayland
if [ "${WAYLAND_DISPLAY:-}" != "" ]; then
    echo "Wayland display detected: $WAYLAND_DISPLAY"
    echo ""
    echo "To start RetroShell:"
    echo "  cargo run -p retro-shell"
    echo ""
    echo "Or run directly from a Wayland compositor (e.g. labwc, river, sway):"
    echo "  exec ./target/release/retro-shell"
else
    echo "No Wayland display detected."
    echo "RetroShell requires a Wayland compositor to run."
    echo ""
    echo "Quick start with a nested compositor:"
    echo "  sudo apt install labwc"
    echo "  labwc &"
    echo "  cargo run -p retro-shell"
    echo ""
    echo "Or via SSH/tmux build-only:"
    echo "  ./build.sh              # builds everything"
    echo "  scp -r target/release/* user@server:~  # copy binaries"
fi
