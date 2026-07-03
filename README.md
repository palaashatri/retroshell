# RetroShell

A native Rust desktop environment project inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is not a Linux desktop theme. The current implementation is a native Rust shell and application suite running as a Wayland client under `labwc`; the long-term goal is a real desktop environment with its own compositor/session stack.

---

## 1. System Architecture

```
Applications (Finder, Settings, TextEdit, Terminal, App Store)
    ↓ (linked statically or dynamically via SDK)
RetroSDK (Application wrapper, preference engine, appearance API)
    ↓
RetroKit (Native widgets, custom Layout grids, Event dispatching)
    ↓
RetroShell (shell client, global menu prototype, internal window workspace routing)
    ↓
RetroRender (wgpu rendering engine, cosmic-text rasterizer, vulkan context)
    ↓
Wayland / X11 Compositor today (labwc / Xvfb) -> Linux Kernel
```

### Core Components
* **`retro-shell`**: The primary shell client. It manages the desktop canvas, shell-owned global menus, internal windows, app launching, and workspace state. It is not yet a Wayland compositor.
* **`retro-kit`**: A custom, lightweight native GUI toolkit written in Rust, defining widgets, scroll containers, list view behaviors, grids, and event handling.
* **`retro-render`**: A graphics pipeline built on `wgpu`. It handles low-level pipeline setup and uses the system font database with `ab_glyph` rasterization for crisp glyphs while keeping a retro fallback path.
* **`retro-sdk`**: The software development kit for RetroShell. Houses the native menu routing, configuration stores, and the common application entrypoint loops.
* **`retro-bus`**: A low-level IPC implementation supporting service discovery, cross-process event propagation, and communication between apps and shell services.

---

## 2. Technology Stack & Specs
* **Language**: Rust (edition 2021)
* **Rendering API**: WebGPU (`wgpu` targeting Vulkan/Wayland/X11 backends)
* **Text Rendering**: `cosmic-text` font database and rasterizer
* **Audio**: PipeWire & PulseAudio
* **Compositor integration**: Wayland client protocol today; planned Smithay compositor/session path for a real DE

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

## 5. Current Status

RetroShell is currently a polished prototype shell, not a production desktop environment. The shell-owned menu bar is functional for internal shell/Finder windows and updates with the active shell window. Standalone SDK applications now publish their menu models into the runtime menu manifest directory, first-party apps launched by RetroShell run with local SDK menu bars suppressed, and the shell menu server can load those manifests; focus tracking and action dispatch are still not universal until RetroShell owns the compositor/session layer.

Recent Phase 1 work:
* VM startup now uses configurable `RETROSHELL_VM_WIDTH`, `RETROSHELL_VM_HEIGHT`, and `RETROSHELL_VM_DEPTH` values, starts labwc through its X11 backend explicitly, and configures the discovered wlroots output with `wlr-randr`.
* Docker images install real font packages, and `retro-render` no longer embeds the invalid HTML file that previously pretended to be `DejaVuSans.ttf`.
* Clipboard now persists through a runtime file so first-party apps can copy/paste across process boundaries. This is a practical bridge, not final Wayland `wl_data_device` integration.
* SDK apps publish JSON menu manifests to `${RETROSHELL_MENU_MANIFEST_DIR}` or `${XDG_RUNTIME_DIR}/retroshell/menus`, and `retro-shell` can load those manifests into the shell menu server.
* RetroShell sets `RETROSHELL_GLOBAL_MENU=1` for first-party SDK apps it launches, suppressing their duplicate local menu bars and switching the shell menu model to the launched app.

Still not done:
* RetroShell is not yet a compositor and does not manage external Wayland app surfaces itself.
* HDR and VRR are not complete. Current renderer code can see present modes/formats, but real HDR needs color management, tone mapping, output metadata, and compositor-level presentation control.
* The global menu is not yet universal for standalone SDK/external app windows because labwc, not RetroShell, still owns real window focus and app surface management.
* Screenshots should be refreshed in this README whenever a major visual/UI change is verified in the VM.

### Latest VM Screenshots

![RetroShell current VM desktop](docs/screenshots/current-retroshell-desktop.png)

![RetroShell minimized Finder window](docs/screenshots/current-minimized-window.png)

![RetroShell About window](docs/screenshots/current-about-window.png)

![RetroShell Force Quit window](docs/screenshots/current-force-quit-window.png)

### Verified UI Update

The latest VM verification uses the rebuilt `retroshell-vm` image at 1280x800. The display fills the full VM frame without black bars, and internal Finder minimize is no longer a placeholder: clicking the titlebar minimize control collapses the window into a bottom titlebar tab, and clicking the same control restores it.

The Retro menu's About action now opens a real shell message window instead of a fake folder view, and Finder Get Info opens an internal metadata window for the active shell/Finder window.

The remaining default shell menu items now have routable action IDs. Items that still depend on future subsystems open explicit shell status windows instead of silently doing nothing.

Remaining compositor-level goals are still open: RetroShell is not yet a Wayland compositor, and HDR/VRR/exclusive fullscreen require compositor/session ownership plus color-management and presentation-control work rather than only shell-client UI changes.

---

## 6. Development & Verification VM

RetroShell comes with a Docker-based visual environment (`retroshell-vm`) for visual development and automated testing in a sandboxed Wayland/X11 compositor. The VM defaults to `1280x800x24`; override with `RETROSHELL_VM_WIDTH`, `RETROSHELL_VM_HEIGHT`, and `RETROSHELL_VM_DEPTH`.

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

## 7. Ubuntu Server Installation Guide

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
    fontconfig \
    fonts-dejavu-core \
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

## 8. Production-Grade Assessment & Self-Review

### Production Readiness Score: **3.5 / 10 (Prototype / Concept Stage)**

While RetroShell is a highly optimized, responsive, and visually appealing simulation of classic visual metaphors, it is not yet a production-grade desktop environment.

### Architectural Gaps (Why it is not Production Grade)

1. **Nested Composite Window Drawing (Lack of true Wayland Compositing)**:
   - *Current Implementation*: RetroShell runs as a single Wayland client, and draws "windows" inside its own `wgpu` canvas layout.
   - *Production Requirement*: A true desktop shell compositor (like Mutter or Sway) acts as the display compositor itself, exposing Wayland surface sockets to independent client processes. In RetroShell, external applications cannot natively display their window surfaces unless they are fully rewritten as child sub-widgets in RetroShell's monolithic layout canvas.

2. **Font Engine & Text Limitations**:
   - *Current Implementation*: Uses system font discovery plus per-glyph `ab_glyph` rasterization, with a bitmap fallback. It is visibly better than the original bitmap-only path but still lacks full shaping, fallback font runs, kerning, and subpixel policy.
   - *Production Requirement*: A production environment requires a full text stack (HarfBuzz-style shaping, font fallback, international input, ligatures, and dynamic antialiasing policy).

3. **Missing HiDPI / Display Scale Adaptability**:
   - *Current Implementation*: layouts (such as the 90x90px Finder grid) utilize hardcoded pixel offsets.
   - *Production Requirement*: Modern systems must seamlessly dynamically scale their entire UI tree according to display scaling factors (e.g. 1.25x, 2.0x HiDPI Retina) without visual clipping or alignment breakage.

4. **Weak Process Separation & Security Sandboxing**:
   - *Current Implementation*: Spawns applications with the same UID/permissions as the root process.
   - *Production Requirement*: Desktop sessions must enforce strict privilege boundaries (e.g. sandboxed Flatpak-style apps) communicating over access-restricted D-Bus interfaces.

5. **Color/Presentation Tone Gaps**:
   - *Current Implementation*: Dynamically selects `Rgba16Float` or `Rgb10a2Unorm` color spaces if the surface reports support (HDR/VRR), but doesn't perform proper SDR-to-HDR color grading/tonemapping. SDR application textures can appear washed out or oversaturated depending on physical monitor calibration.
