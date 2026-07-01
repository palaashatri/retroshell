# RetroShell

A modern native Rust desktop environment experiment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is made from:

- `retro-shell`: desktop runtime and shell services
- `retro-kit`: native UI toolkit
- `retro-render`: `wgpu` rendering layer
- `retro-sdk`: first-party app runtime
- `retro-bus`: IPC foundation

See [docs/README.md](docs/README.md) for larger architecture notes.

## Screenshots

Every major UI/UX change should refresh the current screenshots. Screenshots live in [docs/screenshots](docs/screenshots/).

### Current Implementation

Captured from a Linux VM/Xvfb/Mesa smoke run after the native `wgpu` desktop fills a 1280x800 surface, interactive menu bar, original desktop icons, managed Finder-style shell windows, desktop folder icons opening filesystem-backed shell windows, folder icons inside managed shell windows opening child folder windows, focus/raise, active-window close, titlebar close/zoom controls, View-menu fullscreen, drag/resize, and visible grow box passed.

![Current RetroShell desktop](docs/screenshots/current-retroshell-desktop.png)

### Finder

Captured from a Linux VM/Xvfb smoke run against a demo home directory after Finder status bar, path display, directory sorting, folder entry, visible Back/Forward/Up controls, visible New Folder/Duplicate/Trash controls, and navigation-history behavior passed.

![Current Finder app](docs/screenshots/current-finder.png)

### Visual Direction

Current visual direction: Classic Mac-inspired desktop proportions, menu density, icon treatment, window chrome, and calm gray desktop texture. Do not commit or ship Apple-owned marks, logos, icons, or copied bitmap assets.

## Current State

RetroShell currently builds and launches a native rendered desktop surface, menu strip, desktop icons, app bundle labels, first-party apps wired through RetroKit/RetroSDK, a first pass at managed shell windows with functional close, zoom, fullscreen controls, desktop Home/Hard Disk/Trash icons opening folder-backed shell windows, and in-window folder icons opening child folder windows. Finder now has directory listing, folder entry, visible Back/Forward/Up controls, parent navigation, back/forward history, visible New Folder/Duplicate/Trash controls, and VM-smoked path/status display. This implementation is still foundation work, not a polished desktop environment.

Verified locally:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace -q` (50 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- `retro-shell` under Linux/Xvfb/Mesa Vulkan llvmpipe at 1280x800, including View-menu fullscreen for the active managed Finder-style shell window, desktop Home icon opening a managed folder window, and in-window folder double-click opening a child managed folder window
- `finder` under Linux/Xvfb/Mesa Vulkan llvmpipe against a demo home directory, including visible Back/Forward/Up and file-operation controls, New Folder from the toolbar, and refreshed path/status display

## What Is Left

Plenty. The next major work is closing the gap between the current functional shell and the full desktop environment target.

- Window management: focus rings, minimize controls, modal dialogs, persisted placement, external app surfaces.
- Finder desktop: contextual menus, drag/drop, trash UI polish, desktop integration, polished multi-window workflows.
- Dock/application launching: running indicators, focus, lifecycle integration, folders, trash.
- Native dark mode: complete theme-token coverage, live switching from Settings, dark assets/icons, contrast validation.
- Text rendering: proper `cosmic-text` rendering, font metrics, clipping, invalidation, visual regression screenshots.
- App completeness: Finder, Settings, TextEdit, Terminal, and package-manager backed App Store need real workflows.
- Platform integration: Wayland-first shell behavior, input methods, clipboard, accessibility, multi-monitor, HiDPI, packaging, startup sessions.
- Display goals: compositor/session path, HDR metadata/color pipeline, VRR frame pacing.
- Release evidence: video with audio of Doom running on RetroShell in windowed, borderless fullscreen, and exclusive fullscreen modes.
