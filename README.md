# RetroShell

A modern native Rust desktop environment experiment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is made from:

- `retro-shell`: desktop runtime and shell services
- `retro-kit`: native UI toolkit
- `retro-render`: `wgpu` rendering layer
- `retro-sdk`: first-party app runtime
- `retro-bus`: IPC foundation

See [docs/README.md](docs/README.md) for the larger architecture notes.

## Screenshots

Every major UI/UX change should refresh this section with a current screenshot. Screenshots live in [docs/screenshots](docs/screenshots/).

### Current Implementation

Captured from a Linux VM/Xvfb/Mesa smoke run after the native `wgpu` desktop, interactive menu bar, original desktop icons, managed Finder-style shell windows, focus/raise, active-window close, titlebar close/zoom controls, drag/resize, and visible grow box pass.

![Current RetroShell desktop](docs/screenshots/current-retroshell-desktop.png)

### Finder

Captured from a Linux VM/Xvfb smoke run against a demo home directory after Finder status bar, path display, directory sorting, and folder navigation pass.

![Current Finder app](docs/screenshots/current-finder.png)

### Visual Direction

The current visual direction is Classic Mac-inspired desktop proportions, menu density, icon treatment, window chrome, and a calm gray desktop texture. Do not commit or ship Apple-owned marks, logos, icons, or copied bitmap assets.

## Current State

RetroShell currently builds and launches a native rendered desktop surface, menu strip, desktop icons, app bundle labels, first-party apps wired through RetroKit/RetroSDK, and a first pass at managed shell windows with functional close and zoom controls. This implementation is still a foundation, not a polished desktop environment.

Verified locally:

- `cargo check --workspace --all-targets`
- `cargo test --workspace -q`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `retro-shell` under Linux/Xvfb/Mesa Vulkan llvmpipe, including menu-driven creation of a second managed Finder-style shell window and titlebar zoom to desktop content bounds

## What Is Left

Plenty. The next major work is closing the gap between the current functional shell and the full desktop environment target.

- Window management: focus rings, minimize/fullscreen controls, modal dialogs, persisted placement, external app surfaces.
- Finder desktop: real folder windows, desktop integration, trash UI, surfaced file operations, drag/drop, contextual menus.
- Dock/application launching: running indicators, focus, lifecycle integration, folders, trash.
- Native dark mode: complete theme-token coverage, live switching from Settings, dark assets/icons, contrast validation.
- Text rendering: proper `cosmic-text` rendering, font metrics, clipping, invalidation, visual regression screenshots.
- App completeness: Finder, Settings, TextEdit, Terminal, and package-manager backed App Store need real workflows.
- Platform integration: Wayland-first shell behavior, input methods, clipboard, accessibility, multi-monitor, HiDPI, packaging, startup sessions.
- Display goals: compositor/session path, HDR metadata/color pipeline, VRR frame pacing.
- Release evidence: video with audio of Doom running on RetroShell in windowed, borderless fullscreen, and exclusive fullscreen modes.
