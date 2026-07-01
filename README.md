# RetroShell

A modern desktop environment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is a native Rust desktop environment experiment made from:

- `retro-shell`: desktop runtime and shell services
- `retro-kit`: native UI toolkit
- `retro-render`: `wgpu` rendering layer
- `retro-sdk`: first-party app runtime
- `retro-bus`: IPC foundation

See [docs/README.md](docs/README.md) for full documentation.

## Screenshots

Every major UI/UX change must update this section with a fresh screenshot and,
when useful, keep a target/reference image nearby for comparison. Screenshots
live in [docs/screenshots](docs/screenshots/).

### Current Implementation

Captured from a Linux VM/Xvfb smoke run after the native `wgpu` desktop,
interactive menu bar, original desktop icons, draggable/resizable Finder-style
desktop window, and visible grow box pass.

![Current RetroShell desktop](docs/screenshots/current-retroshell-desktop.png)

### Finder

Captured from a Linux VM/Xvfb smoke run with a demo home directory after the
Finder status bar, path display, directory sorting, and folder navigation pass.

![Current Finder app](docs/screenshots/current-finder.png)

### Visual Direction

The current visual direction is Classic Mac-inspired desktop proportions, menu
density, icon treatment, window chrome, and calm gray desktop texture. Do not
commit or ship Apple-owned marks, logos, icons, or copied bitmap assets.

## Current State

RetroShell currently builds and launches a native window with a basic rendered
desktop surface, menu strip, desktop icons, app bundle labels, and first-party
apps wired through RetroKit/RetroSDK. This is an implementation foundation, not
yet a polished desktop environment.

Verified locally and in a Linux VM:

- `cargo check --workspace --all-targets`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- VM UI smoke: `retro-shell` starts, renders, and stays alive under Xvfb/Mesa

## What Is Left

Plenty. The next major work is to close the gap between the current functional
shell and the Classic Mac-inspired desktop shown above.

- Visual fidelity: real Platinum-style chrome, patterned desktop background,
  crisp borders, shadow rules, better spacing, icon art, and accurate typography.
- Global menu bar: one persistent shell-owned menu bar with app focus switching,
  command routing, shortcuts, status items, and menus that open/close correctly.
- Window system polish: movable/resizable windows, z-order, focus rings,
  minimize/zoom/close behavior, modal dialogs, and persisted window placement.
- Finder desktop: selectable desktop icons, disk/trash behavior, folder windows,
  file operations surfaced in UI, drag/drop, contextual menus, and status bars.
- Dock/application launching: launch, focus, running indicators, trash, folders,
  hover/selection states, and app lifecycle integration.
- Native dark mode: complete theme-token coverage, live theme switching from
  Settings, dark assets/icons, contrast validation, and app-wide propagation.
- Text and rendering: replace placeholder bitmap glyph drawing with proper
  `cosmic-text` rendering, font metrics, clipping, invalidation, and visual
  regression screenshots.
- App completeness: Finder, Settings, TextEdit, and Terminal need real workflows
  rather than mostly static/control-surface demos.
- Platform integration: Wayland-first shell behavior, input methods, clipboard,
  accessibility, multi-monitor handling, HiDPI, packaging, and startup sessions.
- Release evidence: video with audio of Doom running on RetroShell in windowed,
  borderless fullscreen, and exclusive fullscreen modes.
