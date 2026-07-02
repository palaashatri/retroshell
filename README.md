# RetroShell

A native Rust desktop environment experiment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is NOT a Linux desktop theme. It is a complete, self-contained desktop platform with its own window manager, native widget toolkit, GPU-accelerated rendering pipeline, first-party SDK, and application suite.

---

## 1. System Architecture

```
Applications (Finder, Settings, TextEdit, Terminal, App Store)
    ↓ (linked statically or dynamically via SDK)
RetroSDK (Application wrapper, preference engine, appearance API)
    ↓
RetroKit (Native widgets, custom Layout grids, Event dispatching)
    ↓
RetroShell (Session manager, global menu bar, window workspace routing, Dock)
    ↓
RetroRender (wgpu rendering engine, cosmic-text rasterizer, vulkan context)
    ↓
Wayland / X11 Compositor (e.g., labwc / Xvfb) -> Linux Kernel
```

### Core Components
* **`retro-shell`**: The primary desktop compositor runtime, managing global menus, dock layouts, active workspaces, notification dispatch, and desktop/window management.
* **`retro-kit`**: A custom, lightweight native GUI toolkit written in Rust, defining widgets, scroll containers, list view behaviors, grids, and event handling.
* **`retro-render`**: A fast graphics pipeline built on `wgpu`. Handles low-level pipeline setup, vertex shaders, and texture upload. Integrated with `cosmic-text` for glyph layout and rasterization.
* **`retro-sdk`**: The software development kit for RetroShell. Houses the native menu routing, configuration stores, and the common application entrypoint loops.
* **`retro-bus`**: A low-level IPC implementation supporting service discovery, cross-process event propagation, and communication between apps and shell services.

---

## 2. Technology Stack & Specs
* **Language**: Rust (edition 2021)
* **Rendering API**: WebGPU (`wgpu` targeting Vulkan/Wayland/X11 backends)
* **Text Rendering**: `cosmic-text` font database and rasterizer
* **Audio**: PipeWire & PulseAudio
* **Compositor integration**: Wayland client protocol + wlr-protocols (running nested or direct)

---

## 3. Visual & Coding Standards

### Design Guidelines (Style Guide)
* **Metaphor Consistency**: Retain a compact, highly dense, classic desktop visual metaphor (Mac OS System 7 to OS 9 style).
* **Color Palette**: Calm, low-saturation gray tones (Platinum) by default. Use token-based colors to avoid ad-hoc values.
* **No Web Noise**: Avoid rounded borders exceeding 4px, heavy drop shadows, or high-contrast modern web visual noise.
* **Native Dark Mode**: Support dark mode natively using system theme preference stores. Color selection dynamically switches based on appearance keys.

### Coding Standards
* **Rust Idioms**: Avoid unnecessary `unsafe` blocks. Prefer clear ownership boundaries and leverage type systems for state machines.
* **Widget Structure**: All widgets must inherit from base layouts and pass rendering tasks back to the active `Canvas` using NDC coordinate translation.
* **Portability**: Keep the core crates decoupled from OS-specific backends where possible to facilitate future portability to custom kernels.

---

## 4. First-Party Application Suite
* **Finder**: The file and spatial navigation shell. Displays directory items in grids, manages file operations (New Folder, Duplicate, Delete), and shows file metadata (`INFO`).
* **Settings**: Manages system state. Allows toggling appearance (Light/Dark) and writing `settings.conf`.
* **TextEdit**: Lightweight text editor supporting multi-line text fields, dirty-state tracking, and disk I/O.
* **Terminal**: PTY-backed terminal emulator supporting interactive shells, selection copy/paste, and custom scrollback.
* **App Store**: Package manager front-end querying system package indices (e.g. `APT`) and running package searches.

---

## 5. Development & Verification VM

RetroShell comes with a Docker-based visual environment (`retroshell-vm`) for visual development and automated testing in a sandboxed Wayland/X11 compositor.

### Building and Running the VM
1. **Build the container**:
   ```bash
   docker build -f Dockerfile.vm -t retroshell-vm .
   ```
2. **Start the environment**:
   ```bash
   docker run -d -p 6080:6080 -v "$(pwd):/app" --name retroshell-running retroshell-vm
   ```
3. **Connect to the Desktop**:
   Open a browser and navigate to `http://localhost:6080/vnc.html` to access the desktop session.

### Running Applications inside the VM
Compile the workspace and launch the desktop shell:
```bash
docker exec -t retroshell-running cargo build --release
docker exec -d retroshell-running env -u DISPLAY WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/tmp/runtime-root /app/target-docker/release/retro-shell
```

---

## 6. Ubuntu Server Installation Guide

To configure a bare Ubuntu Server machine to boot directly into RetroShell, follow these installation steps:

### Prerequisites
1. **Graphics Card**: A GPU compatible with Vulkan 1.2+ (Intel, AMD, or NVIDIA).
2. **Base Packages**: Ensure your system is updated:
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

### Step 1: Install Display Server, Compositor, and Audio Services
Since Ubuntu Server lacks a graphical display server, install Xorg, a lightweight Wayland compositor (`labwc`), seat management, DBus, and audio servers:
```bash
sudo apt install -y --no-install-recommends \
    xserver-xorg-core \
    xinit \
    labwc \
    dbus-x11 \
    pipewire \
    pipewire-audio-client-libraries \
    pulseaudio-utils \
    libwayland-dev \
    libxkbcommon-dev \
    libdbus-1-dev \
    libfontconfig-dev \
    libfreetype6-dev \
    build-essential \
    pkg-config \
    git \
    curl
```

### Step 2: Install Rust Toolchain
Install the standard Rust compiler toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Step 3: Clone and Compile RetroShell
Clone the repository, compile the release binaries, and build the application bundles:
```bash
git clone https://github.com/palaashatri/retroshell.git
cd retroshell
cargo build --release

# Create the applications bundle directories
mkdir -p target/release/Applications
for app in finder settings textedit terminal appstore; do
    if [ "$app" = "finder" ]; then APP_NAME="Finder"; fi
    if [ "$app" = "settings" ]; then APP_NAME="Settings"; fi
    if [ "$app" = "textedit" ]; then APP_NAME="TextEdit"; fi
    if [ "$app" = "terminal" ]; then APP_NAME="Terminal"; fi
    if [ "$app" = "appstore" ]; then APP_NAME="App Store"; fi
    
    BUNDLE_DIR="target/release/Applications/$APP_NAME.app"
    mkdir -p "$BUNDLE_DIR/Executable" "$BUNDLE_DIR/Resources" "$BUNDLE_DIR/Assets"
    [ -f "apps/$app/App.toml" ] && cp "apps/$app/App.toml" "$BUNDLE_DIR/App.toml"
    [ -f "target/release/$app" ] && cp "target/release/$app" "$BUNDLE_DIR/Executable/$app"
done
```

### Step 4: Configure Autostart for the Compositor
Set up `labwc` to run without decorations and immediately start RetroShell on login:
```bash
mkdir -p ~/.config/labwc
cat << 'EOF' > ~/.config/labwc/rc.xml
<?xml version="1.0" encoding="utf-8"?>
<labwc_config>
  <theme>
    <decoration>none</decoration>
  </theme>
  <windowRules>
    <windowRule identifier="com.retro.shell">
      <action name="Maximize"/>
    </windowRule>
  </windowRules>
</labwc_config>
EOF

cat << 'EOF' > ~/.config/labwc/autostart
# Start Pipewire Audio
pipewire &
# Launch RetroShell Client
env -u DISPLAY WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/run/user/$(id -u) ~/retroshell/target/release/retro-shell &
EOF
chmod +x ~/.config/labwc/autostart
```

### Step 5: Start the Desktop Environment
Start the X11 server running the labwc compositor, which will load RetroShell:
```bash
xinit /usr/bin/labwc
```

---

## 7. Production-Grade Assessment & Self-Review

### Production Readiness Score: **3.5 / 10 (Prototype / Concept Stage)**

While RetroShell is a highly optimized, responsive, and visually appealing simulation of classic visual metaphors, it is not yet a production-grade desktop environment.

### Architectural Gaps (Why it is not Production Grade)

1. **Nested Composite Window Drawing (Lack of true Wayland Compositing)**:
   - *Current Implementation*: RetroShell runs as a single Wayland client, and draws "windows" inside its own `wgpu` canvas layout.
   - *Production Requirement*: A true desktop shell compositor (like Mutter or Sway) acts as the display compositor itself, exposing Wayland surface sockets to independent client processes. In RetroShell, external applications cannot natively display their window surfaces unless they are fully rewritten as child sub-widgets in RetroShell's monolithic layout canvas.

2. **Font Engine & Text Limitations**:
   - *Current Implementation*: Uses a custom, bitmap-styled pixel glyph index (`glyph_pattern`) mapping all characters to their uppercase representations.
   - *Production Requirement*: A production environment requires vectorized font engines (TTF/OTF via HarfBuzz/FreeType) that fully support international languages, font kerning, ligatures, and dynamic subpixel antialiasing.

3. **Missing HiDPI / Display Scale Adaptability**:
   - *Current Implementation*: layouts (such as the 90x90px Finder grid) utilize hardcoded pixel offsets.
   - *Production Requirement*: Modern systems must seamlessly dynamically scale their entire UI tree according to display scaling factors (e.g. 1.25x, 2.0x HiDPI Retina) without visual clipping or alignment breakage.

4. **Weak Process Separation & Security Sandboxing**:
   - *Current Implementation*: Spawns applications with the same UID/permissions as the root process.
   - *Production Requirement*: Desktop sessions must enforce strict privilege boundaries (e.g. sandboxed Flatpak-style apps) communicating over access-restricted D-Bus interfaces.

5. **Color/Presentation Tone Gaps**:
   - *Current Implementation*: Dynamically selects `Rgba16Float` or `Rgb10a2Unorm` color spaces if the surface reports support (HDR/VRR), but doesn't perform proper SDR-to-HDR color grading/tonemapping. SDR application textures can appear washed out or oversaturated depending on physical monitor calibration.

