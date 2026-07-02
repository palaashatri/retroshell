# RetroShell

A native Rust desktop environment experiment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is made from:

- `retro-shell`: desktop runtime shell services
- `retro-kit`: native UI toolkit
- `retro-render`: `wgpu` rendering layer
- `retro-sdk`: first-party app runtime
- `retro-bus`: IPC foundation

See [docs/README.md](docs/README.md) for larger architecture notes.

## Screenshots

Every major UI/UX change should refresh current screenshots. Screenshots live in [docs/screenshots](docs/screenshots/).

### Current Implementation

Captured from a Linux VM/Xvfb/Mesa smoke run after the native `wgpu` desktop filled a 1280x800 surface, accepted menu pointer interaction, rendered desktop icons, opened managed Finder-style shell windows, opened folder-backed shell windows, raised/focused windows, closed active windows, used titlebar close/zoom controls, toggled fullscreen through the View menu, dragged/resized windows, and rendered the grow box.

![Current RetroShell desktop](docs/screenshots/current-retroshell-desktop.png)

### Finder

Captured from a Linux VM/Xvfb smoke run against a demo home directory. Finder rendered sidebar/icon-grid browsing, sorted directory contents, status/path display, navigation controls, file-operation toolbar controls, and the `INFO` toolbar action showing selected-file metadata.

![Current Finder app](docs/screenshots/current-finder.png)

### TextEdit

Captured from a Linux VM/Xvfb smoke run after TextEdit opened a real document path, rendered editable document text, exposed New/Save/Undo/Redo/Copy/Paste actions, and showed saved/path status.

![Current TextEdit app](docs/screenshots/current-textedit.png)

### Settings

Captured from a Linux VM/Xvfb smoke run after Settings clicked the Dark appearance control, persisted `appearance=dark`, and rendered the selected mode/status UI.

![Current Settings app](docs/screenshots/current-settings.png)

### Native Dark Mode

Captured from a Linux VM/Xvfb/Mesa smoke run after launching Settings with `appearance=dark`; RetroSDK rendered shared native chrome and controls with dark-aware colors.

![Current dark mode Settings app](docs/screenshots/current-dark-mode-settings.png)

### Terminal

Captured from a Linux VM/Xvfb smoke run after Terminal launched a real PTY-backed shell script, consumed asynchronous output, repainted the native terminal surface, and rendered live output.

![Current Terminal app](docs/screenshots/current-terminal.png)

### App Store

Captured from a Linux VM/Xvfb smoke run after App Store detected the host `APT` backend, ran a real read-only package-manager search for `doom`, and rendered package results.

![Current App Store app](docs/screenshots/current-appstore.png)

## Current State

RetroShell currently builds and launches a native rendered desktop surface with a menu strip, desktop icons, app bundle labels, first-party apps wired through RetroKit/RetroSDK, and first-pass managed shell window close, zoom, fullscreen, drag, and resize behavior. Desktop Home/Hard Disk/Trash icons open folder-backed shell windows, and folder icons inside managed shell windows open child folder windows.

Finder has sidebar/icon-grid browsing, sorted directory listing, folder entry, parent navigation, back/forward history, file-operation toolbar controls, New Folder/Duplicate/Trash helpers, VM-smoked path/status display, and a working `INFO` action that reports selected file/folder metadata in the status bar.

TextEdit opens an optional document path passed on the command line, edits through the native multiline text field, saves back to disk, supports Cmd-N/Cmd-S, supports Cmd-Z/Shift-Cmd-Z undo/redo, exposes baseline whole-document copy/cut/paste/select-all shortcuts, and shows toolbar actions for New/Save/Undo/Redo/Copy/Paste.

Settings loads and saves `settings.conf` under `RETROSHELL_CONFIG_DIR` or `~/.config/retroshell`, exposes Light/Dark/System controls, persists changes immediately, and reports the active mode. RetroSDK consumes the same preference and renders shared native chrome/controls in dark appearance when `appearance=dark`.

Terminal launches a real PTY, propagates layout resize to the terminal grid and PTY, consumes async PTY output through runtime repaint, supports scrollback navigation, and wires Cmd-C/Cmd-V to the in-process clipboard baseline.

App Store launches as a first-party app, detects Linux/BSD package managers, and runs read-only package searches through the detected backend. Install/remove/update transactions and privilege handling remain future work.

This is still foundation work, not a polished full desktop environment.

## Recent Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace -q` (68 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Linux VM/Xvfb/Mesa smoke: `retro-shell` renders the desktop, manages shell windows, handles menu interaction/window controls/drag/resize/fullscreen, and captures `docs/screenshots/current-retroshell-desktop.png`.
- Linux VM/Xvfb smoke: `finder` starts against a demo home directory, selects a real file, triggers the `INFO` toolbar action, renders selected-file metadata in the status bar, and captures `docs/screenshots/current-finder.png`.
- Linux VM/Xvfb smoke: `textedit` opens a document path, renders edit controls, and captures `docs/screenshots/current-textedit.png`.
- Linux VM/Xvfb/Mesa smoke: `settings` clicks Dark appearance, verifies `appearance=dark`, renders selected mode/status UI, and captures `docs/screenshots/current-settings.png`.
- Linux VM/Xvfb/Mesa smoke: `settings` launches with `appearance=dark`, renders dark native chrome/controls, and captures `docs/screenshots/current-dark-mode-settings.png`.
- Linux VM/Xvfb smoke: `terminal` launches a PTY-backed shell script, renders live output, and captures `docs/screenshots/current-terminal.png`.
- Linux VM/Xvfb smoke: `appstore` detects APT, searches `doom`, renders package-manager results, and captures `docs/screenshots/current-appstore.png`.

## Visual Direction

Current visual direction: Classic Mac-inspired desktop proportions, menu density, icon treatment, window chrome, and calm gray desktop texture. Do not commit or ship Apple-owned marks, logos, icons, or copied bitmap assets.

## What Is Left

Plenty. The next major work is closing the gap between the current functional prototype and the full desktop environment target.

- Window management: focus rings, minimize controls, modal dialogs, persisted placement, external app surfaces.
- Finder desktop: contextual menus, drag/drop, trash UI polish, desktop integration, polished multi-window workflows.
- Dock/application launching: running indicators, focus, lifecycle integration, folders, trash.
- Native dark mode: complete theme-token coverage, live switching from Settings, dark assets/icons, contrast validation.
- Text rendering: proper `cosmic-text` rendering, font metrics, clipping, invalidation, visual regression screenshots.
- App completeness: Finder, Settings, TextEdit, Terminal, and package-manager backed App Store need complete workflows.
- Platform integration: Wayland-first shell behavior, input methods, platform clipboard, accessibility, multi-monitor, HiDPI, packaging, startup sessions.
- Display goals: compositor/session path, HDR metadata/color pipeline, VRR frame pacing.
- Release evidence: video with audio of Doom running on RetroShell in windowed, borderless fullscreen, and exclusive fullscreen modes.
