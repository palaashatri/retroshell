# RetroShell Project Status

## Summary

RetroShell is a working native Rust prototype, not a complete desktop environment yet. Current work has a `wgpu` rendered shell window, original Classic-inspired desktop surface, interactive menu bar, draggable/resizable Finder-style desktop window, visible grow box, and first-party app foundations. Finder now has a sidebar, icon grid, sorted directory listing, status/path bar, parent-folder navigation, and folder-entry helpers.

The definition of done is a full desktop environment: working Finder, TextEdit, Settings, Terminal, package-manager backed App Store, native dark mode, compositor/session path, HDR, VRR, and real application/game validation.

## Workspace

| Crate | Path | Status |
|-------|------|--------|
| retro-render | `crates/retro-render/` | Prototype: native `wgpu` rendering works; text, clipping, compositor/display features, HDR, and VRR remain incomplete. |
| retro-kit | `crates/retro-kit/` | Prototype: core widgets and layout exist; polished accessibility, drag/drop, focus, menus, and theme coverage remain incomplete. |
| retro-shell | `crates/retro-shell/` | Prototype: shell services and rendered desktop exist; full window management, sessions, app lifecycle, and compositor integration remain incomplete. |
| retro-bus | `crates/retro-bus/` | Foundation: message and local transport primitives exist; broader service integration remains incomplete. |
| retro-sdk | `crates/retro-sdk/` | Prototype: app runtime and immediate renderer exist; command routing and mature app integration remain incomplete. |

## Applications

| App | Path | Status |
|-----|------|--------|
| Finder | `apps/finder/` | Prototype: menus, sorted file listing, status/path bar, folder entry, parent navigation, and file operation helpers exist; multi-window workflows, desktop integration, trash UI, context menus, and richer status/details remain incomplete. |
| Settings | `apps/settings/` | Prototype: category shell exists; persistent settings, appearance switching, display/HDR/VRR controls, input settings, and package-manager settings remain incomplete. |
| TextEdit | `apps/textedit/` | Prototype: editor surface exists; open/save, document lifecycle, undo/redo, formatting, and file dialogs remain incomplete. |
| Terminal | `apps/terminal/` | Foundation: PTY and terminal grid exist; robust terminal behavior, resizing, scrollback UI, selection, clipboard, and session integration remain incomplete. |
| App Store | `apps/app-store/` | Not started: package-manager adapter, search, install/remove/update, permissions, and distro/BSD backend support remain incomplete. |

## Recent Verification

- `cargo check --workspace --all-targets`
- `cargo test --workspace -q`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Linux VM/Xvfb/Mesa smoke: `retro-shell` starts, accepts pointer interaction, opens the menu bar, and captures `docs/screenshots/current-retroshell-desktop.png`.
- Linux VM/Xvfb/Mesa smoke: `finder` starts against a demo home directory and captures `docs/screenshots/current-finder.png`.

## Next Milestones

1. Finish shell window management: z-order, focus, multiple windows, close/zoom/minimize, drag/resize for all managed windows.
2. Make Finder real: browse directories, open folders in shell windows, wire file operations to UI, add context menus and trash behavior.
3. Make TextEdit real: open/save documents, dirty state, undo/redo, selection, clipboard, and file dialogs.
4. Make Terminal robust: PTY lifecycle, resize propagation, scrollback UI, selection, copy/paste, and shell session persistence.
5. Make Settings persistent: appearance, input, display, package-manager, and future HDR/VRR controls.
6. Add App Store: backend abstraction for Linux/BSD package managers, search/install/remove/update flows, and safe privilege handling.
7. Build compositor/display path: Wayland/session integration, multi-monitor, HiDPI, HDR metadata/color pipeline, and VRR frame pacing.
8. Capture release evidence: video with audio of Doom running on RetroShell in windowed, borderless fullscreen, and exclusive fullscreen modes.
