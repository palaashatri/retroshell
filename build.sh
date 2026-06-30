#!/usr/bin/env bash
set -euo pipefail

RUN_SHELL=false
if [ "${1:-}" = "run" ] || [ "${1:-}" = "--run" ]; then
    RUN_SHELL=true
fi

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

echo "=== Creating Application Bundles ==="
APPS_DIR="target/release/Applications"
mkdir -p "$APPS_DIR"

for app in finder settings textedit terminal; do
    if [ "$app" = "finder" ]; then APP_NAME="Finder"; fi
    if [ "$app" = "settings" ]; then APP_NAME="Settings"; fi
    if [ "$app" = "textedit" ]; then APP_NAME="TextEdit"; fi
    if [ "$app" = "terminal" ]; then APP_NAME="Terminal"; fi
    
    BUNDLE_DIR="$APPS_DIR/$APP_NAME.app"
    echo "Packaging $APP_NAME.app..."
    
    mkdir -p "$BUNDLE_DIR/Executable"
    mkdir -p "$BUNDLE_DIR/Resources"
    mkdir -p "$BUNDLE_DIR/Assets"
    
    # Copy App.toml
    if [ -f "apps/$app/App.toml" ]; then
        cp "apps/$app/App.toml" "$BUNDLE_DIR/App.toml"
    fi
    
    # Copy Executable
    if [ -f "target/release/$app" ]; then
        cp "target/release/$app" "$BUNDLE_DIR/Executable/$app"
    fi
done

echo "Bundles created in target/release/Applications/"
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

if [ "$RUN_SHELL" = true ]; then
    echo "=== Launching RetroShell ==="
    
    # Check if cage is installed, if not try installing it on Debian/Ubuntu
    if ! command -v cage &>/dev/null; then
        echo "Wayland kiosk compositor 'cage' is not installed."
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            if [[ "$ID" == "ubuntu" || "$ID" == "debian" ]]; then
                echo "Installing 'cage' and software Vulkan drivers..."
                sudo apt-get update
                sudo apt-get install -y cage mesa-vulkan-drivers vulkan-tools seatd
            else
                echo "Please install 'cage' or another Wayland compositor on your system."
                exit 1
            fi
        else
            echo "Please install 'cage' or another Wayland compositor on your system."
            exit 1
        fi
    fi
    
    echo "Starting Wayland session via cage..."
    # Ensure seatd is running or we have appropriate permissions if on a TTY
    if [ "${WAYLAND_DISPLAY:-}" = "" ] && [ "${DISPLAY:-}" = "" ]; then
        echo "Running on a TTY console. Launching cage in direct KMS mode."
        if systemctl is-active seatd &>/dev/null; then
            echo "seatd is active."
        else
            echo "Starting seatd system service..."
            sudo systemctl start seatd || true
        fi
    fi
    
    # Run retro-shell inside cage
    export WGPU_POWER_PREF=low-power
    exec cage ./target/release/retro-shell
fi
