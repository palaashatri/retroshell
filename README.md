# RetroShell

A native Rust desktop environment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is not a Linux desktop theme. It is a custom GUI toolkit, application
framework, and shell client written entirely in Rust. The shell renders its own windows
using a wgpu graphics pipeline, ships a suite of first-party applications, and runs as
a Wayland client. RetroShell ships a separate Smithay-based nested-X11 compositor
(`retro-compositor`) that the entrypoint prefers; if unavailable it falls back to labwc.
Note: the compositor requires DRI3 X11 extension and fails in nested Docker environments
without GPU acceleration.

---

## Features

### What works

- [x] Desktop with spatial icon grid (Hard Disk, Home, Applications, Trash)
- [x] Shell-managed windows — move, resize, close, minimize, zoom, fullscreen
- [x] Window stacking / click-to-raise / z-order
- [x] System menu bar with keyboard shortcuts
- [x] Global menu bar — first-party apps publish menus; shell shows them system-wide
- [x] Four virtual workspaces with per-workspace window filtering
- [x] Dock bar at bottom with clickable app-launch items
- [x] Notification Center — post, query, clear notifications
- [x] Lock screen
- [x] Force Quit dialog with live window list
- [x] About window, workspace switcher overlay
- [x] Finder — filesystem browser, New Folder, Move to Trash, Get Info, Rename,
      internal drag-to-folder moves
- [x] TextEdit — multi-line text editor with dirty-state tracking and disk I/O
- [x] Terminal — PTY-backed emulator with VT100/VT220, 256-color SGR, true-color
      SGR, erase-in-line, scroll margins, tab management, scrollback, selection copy/paste
- [x] Settings — 11 preference panes (General, Appearance, Desktop & Dock, Display,
      Sound, Network, Keyboard, Mouse, Accessibility, Privacy & Security, Notifications)
      with persistent `settings.conf` writes
- [x] App Store — reads system package indices (APT), shows install state per package,
      search with package-change gate
- [x] Eight color themes: Classic, Dark, Grape, Blueberry, Strawberry, Solarized, Dracula, HighContrast
- [x] Dark mode with per-token palette switching
- [x] TrueType font rendering via ab_glyph with system font discovery and bitmap fallback
- [x] File-based clipboard persistence across process boundaries
- [x] Drop shadows, pixel art icons, custom window chrome
- [x] Docker VM with noVNC browser access for visual development

### In progress / planned

- [x] retro-compositor — Smithay-based nested-X11 compositor (real GL rendering, protocol stubs)
- [ ] Universal global menu for external apps (requires compositor session ownership)
- [ ] Wayland wl_data_device protocol drag-and-drop between apps
- [ ] Notification banners as floating visual overlays
- [ ] HiDPI / display scale (UI uses logical pixels; no scale-factor tree yet)
- [ ] HDR / VRR output control (preferences stored; compositor work required)
- [ ] AT-SPI accessibility protocol integration
- [ ] Multi-monitor support
- [ ] Power management (UPower)
- [ ] Screen recording / screenshot service

---

## Architecture

```
  ┌─────────────────────────────────────────────────────┐
  │  First-Party Applications                           │
  │  Finder  TextEdit  Terminal  Settings  App Store    │
  └──────────────────┬──────────────────────────────────┘
                     │ links
  ┌──────────────────▼──────────────────────────────────┐
  │  retro-sdk  (Application framework, menu manifests, │
  │             preference engine, draw helpers)        │
  └────┬────────────────────────────┬───────────────────┘
       │ links                      │ links
  ┌────▼────────┐            ┌──────▼──────────────────┐
  │  retro-kit  │            │  retro-bus              │
  │  (Widgets,  │            │  (IPC, service registry,│
  │   toolkit,  │            │   D-Bus transport)      │
  │   themes)   │            └─────────────────────────┘
  └────┬────────┘
       │ links
  ┌────▼──────────────────────────────────────────────────┐
  │  retro-render  (wgpu pipeline, text rasterization,    │
  │                Canvas, NDC translation)               │
  └────┬──────────────────────────────────────────────────┘
       │
  ┌────▼───────────────────────────────────────────┐
  │  wgpu  →  Vulkan / Wayland / X11 backend       │
  └────┬───────────────────────────────────────────┘
       │
  ┌────▼───────────────────────────────────────────┐
  │  labwc (Wayland compositor, today)             │
  │  retro-compositor  (Smithay, future)           │
  └────┬───────────────────────────────────────────┘
       │
  Linux kernel  DRM / KMS
```

**retro-shell** (the shell process) also links retro-sdk, retro-kit, retro-render, and
retro-bus. It is the root process that manages internal windows, the dock, workspaces,
the menu server, and app launch.

---

## Quick Start (Docker)

The fastest way to see RetroShell running is the Docker VM with browser VNC access.

```bash
# Build the VM image (first time, ~5 min)
docker build -f Dockerfile.vm -t retroshell-vm .

# Start the VM
docker run -d -p 6080:6080 -v "$(pwd):/app" --name retroshell-running retroshell-vm

# Build and launch the shell inside the VM
docker exec -t retroshell-running cargo build --release
docker exec -d retroshell-running \
  env -u DISPLAY WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/tmp/runtime-root \
  /app/target-docker/release/retro-shell

# Open http://localhost:6080/vnc.html in your browser
```

Override the display resolution with environment variables:

```bash
docker run -d -p 6080:6080 \
  -e RETROSHELL_VM_WIDTH=1920 \
  -e RETROSHELL_VM_HEIGHT=1080 \
  -v "$(pwd):/app" --name retroshell-running retroshell-vm
```

---

## Development Setup

### Prerequisites

- Rust toolchain (stable, edition 2021): install via [rustup.rs](https://rustup.rs)
- Vulkan-capable GPU drivers
- System libraries: `libwayland-dev`, `libxkbcommon-dev`, `libdbus-1-dev`,
  `libfontconfig-dev`, `libfreetype6-dev`

On Ubuntu/Debian:

```bash
sudo apt install -y \
  libwayland-dev libxkbcommon-dev libdbus-1-dev \
  libfontconfig-dev libfreetype6-dev \
  fonts-dejavu-core build-essential pkg-config
```

### Build

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run the shell (requires a running Wayland compositor such as labwc)
env -u DISPLAY WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/run/user/$(id -u) \
  ./target/release/retro-shell

# Run a first-party app standalone
./target/release/finder
./target/release/terminal
./target/release/settings
```

### Tests

```bash
cargo test
```

Tests cover: clipboard persistence, icon layout, menu clock formatting, bus message
serialization, VT parser escape sequences, Finder file operations, and compositor stubs.

### Docker builds

```bash
# VM image (visual development, noVNC)
docker build -f Dockerfile.vm -t retroshell-vm .

# QA image (automated headless testing)
docker build -f Dockerfile.qa -t retroshell-qa .
```

---

## Keyboard Shortcuts

### Shell (global)

| Shortcut          | Action                         |
|-------------------|--------------------------------|
| Cmd+N             | New Finder window              |
| Cmd+W             | Close front window             |
| Cmd+Tab           | Cycle windows (same workspace) |
| Cmd+F             | Toggle fullscreen              |
| Cmd+Q             | Quit RetroShell                |
| Cmd+Shift+Q       | Log Out                        |
| Ctrl+Cmd+L        | Lock Screen                    |
| Cmd+Alt+Escape    | Force Quit dialog              |

### Terminal

| Shortcut          | Action                         |
|-------------------|--------------------------------|
| Cmd+T             | New tab                        |
| Cmd+Shift+W       | Close tab                      |
| Cmd+W             | Close window                   |
| Cmd+C             | Copy selection                 |
| Cmd+V             | Paste                          |

Full keyboard reference: [docs/KEYBOARD_SHORTCUTS.md](docs/KEYBOARD_SHORTCUTS.md)

---

## Themes

Set the `theme` key in `~/.config/retroshell/settings.conf` or use Settings > Appearance.

| Theme       | Mode  | Character                                     |
|-------------|-------|-----------------------------------------------|
| `classic`   | Light | Mac OS 7–9 Platinum, blue accent. Default.    |
| `dark`      | Dark  | Dark Platinum with blue accent                |
| `grape`     | Dark  | Purple-tinted dark theme                      |
| `blueberry` | Dark  | Deep navy dark theme                          |
| `strawberry`| Light | Warm red-orange accent on light gray          |
| `solarized` | Dark  | Solarized dark theme with blue accent         |
| `dracula`   | Dark  | Dracula dark theme with purple accent         |
| `highcontrast` | Light | Pure black/white with yellow accent        |

---

## Configuration

Configuration file: `~/.config/retroshell/settings.conf`

| Key                 | Values                                                                       | Default   |
|---------------------|---------------------------------------------------------------------|-----------|
| `theme`             | `classic` `dark` `grape` `blueberry` `strawberry` `solarized` `dracula` `highcontrast` | `classic` |
| `appearance`        | `light` `dark`                                    | `light`   |
| `sound_volume`      | `0`–`100`                                         | `50`      |
| `mouse_speed`       | `0`–`100`                                         | `50`      |
| `hdr_request`       | `true` `false`                                    | `false`   |
| `vrr_adaptive`      | `true` `false`                                    | `false`   |
| `do_not_disturb`    | `true` `false`                                    | `false`   |

Full configuration reference: [docs/CONFIGURATION.md](docs/CONFIGURATION.md)

---

## Ubuntu Server Installation

To configure a bare Ubuntu Server to boot into RetroShell:

### 1. Install system dependencies

```bash
sudo apt install -y --no-install-recommends \
  xserver-xorg-core xinit labwc dbus-x11 \
  pipewire pipewire-audio-client-libraries pulseaudio-utils \
  libwayland-dev libxkbcommon-dev libdbus-1-dev \
  libfontconfig-dev libfreetype6-dev fontconfig fonts-dejavu-core \
  build-essential pkg-config git curl
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 3. Clone and build

```bash
git clone https://github.com/palaashatri/retroshell.git
cd retroshell
cargo build --release
```

### 4. Configure labwc autostart

```bash
mkdir -p ~/.config/labwc
cat << 'EOF' > ~/.config/labwc/rc.xml
<?xml version="1.0" encoding="utf-8"?>
<labwc_config>
  <theme><decoration>none</decoration></theme>
  <windowRules>
    <windowRule identifier="com.retro.shell">
      <action name="Maximize"/>
    </windowRule>
  </windowRules>
</labwc_config>
EOF

cat << 'EOF' > ~/.config/labwc/autostart
pipewire &
env -u DISPLAY WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/run/user/$(id -u) \
  ~/retroshell/target/release/retro-shell &
EOF
chmod +x ~/.config/labwc/autostart
```

### 5. Start

```bash
xinit /usr/bin/labwc
```

---

## Progress

| Milestone        | Score | Notes                                                  |
|------------------|-------|--------------------------------------------------------|
| Initial prototype| 2.5   | Single wgpu canvas, bitmap font, no real widgets       |
| Phase 1 complete | 4.40  | PTY terminal, real font rendering, Settings, workspaces, SDK menus, Finder DnD, clipboard |
| Current          | 5.9   | Drop shadows, pixel art icons, dock, tab switching, VT parser expansion, workspace grid view, polished window chrome |
| Target           | 10.0  | Full Smithay compositor, HiDPI, universal global menu, AT-SPI, protocol DnD |

The gap between the current score and 10 is primarily architectural: while RetroShell
ships a Smithay-based nested-X11 compositor, `retro-shell` itself remains a single fullscreen
Wayland client rendering all internal windows into one surface. A true per-app Wayland session
compositor with multi-window protocol support is tracked as long-term work.

---

## Architecture Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — crate graph, rendering pipeline,
  Wayland protocol stack, how to add a new app
- [docs/CONFIGURATION.md](docs/CONFIGURATION.md) — all settings.conf keys,
  environment variables, theme system
- [docs/KEYBOARD_SHORTCUTS.md](docs/KEYBOARD_SHORTCUTS.md) — full shortcut reference

---

## Contributing

1. Fork and clone the repository.
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Follow the coding standards below.
4. Run `cargo test` and ensure all tests pass.
5. Open a pull request with a clear description of the change.

### Coding standards

- **Rust idioms** — avoid `unsafe` blocks. Prefer clear ownership and type-state
  machines over raw flags.
- **Widget structure** — all widgets implement the `Widget` trait from `retro-kit`.
  Pass rendering tasks back to the active `Canvas` using NDC coordinate translation.
  Never hard-code colors; use `ThemeToken` values.
- **No design noise** — border radius must not exceed 4 px. Avoid heavy drop shadows
  or high-contrast modern gradients. Retain the compact Platinum metaphor.
- **Portability** — keep core crates decoupled from OS-specific backends. The
  `retro-compositor` crate uses `cfg(target_os = "linux")` guards; other crates
  must not.
- **Settings persistence** — new user-facing preferences go through `settings.conf`
  via the Settings app. Do not write ad-hoc config files in other locations.
- **Tests** — new behavior should have at least one unit or integration test.

### Screenshots

After a visual change is verified in the Docker VM, update the relevant screenshot
in `docs/screenshots/` and reference it in this README under "Latest VM Screenshots".

### Repository layout

```
Cargo.toml              — workspace root
Dockerfile.vm           — visual development Docker image (noVNC)
Dockerfile.qa           — headless QA Docker image
crates/
  retro-render/         — wgpu rendering pipeline
  retro-kit/            — widget toolkit
  retro-bus/            — IPC layer
  retro-sdk/            — application framework
  retro-shell/          — shell process
  retro-compositor/     — future Smithay compositor
apps/
  finder/               — file manager
  settings/             — system preferences
  textedit/             — text editor
  terminal/             — PTY terminal emulator
  appstore/             — package manager front-end
docs/
  ARCHITECTURE.md
  CONFIGURATION.md
  KEYBOARD_SHORTCUTS.md
  implementation_plan.md
  screenshots/
```

---

## License

See [LICENSE](LICENSE).

---

## Latest VM Screenshots

![RetroShell desktop (Docker QA)](docs/screenshots/current-compositor-qa.png)

*Docker QA screenshot: retro-compositor running, full desktop with Finder window, menu bar, dock, and right-column desktop icons.*

![RetroShell desktop](docs/screenshots/current-retroshell-desktop.png)

![Finder drag-to-folder](docs/screenshots/current-finder-dnd.png)

![Settings Display pane](docs/screenshots/current-settings.png)

![Settings Sound slider](docs/screenshots/current-settings-sliders.png)

![App Store package search](docs/screenshots/current-appstore.png)

![Notification Center](docs/screenshots/current-notification-center.png)

![Workspace switcher](docs/screenshots/current-workspace-switch.png)

![Minimized Finder window](docs/screenshots/current-minimized-window.png)

![About window](docs/screenshots/current-about-window.png)

![Force Quit window](docs/screenshots/current-force-quit-window.png)
