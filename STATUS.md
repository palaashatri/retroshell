# RetroShell Project Status

## Summary

RetroShell is a working native Rust prototype, not a complete desktop environment yet. Current work has a `wgpu` rendered shell window, original Classic-inspired desktop surface, interactive menu bar, desktop icons, first-party app foundations, and managed Finder-style shell windows with menu-driven creation, focus/raise, active-window close, titlebar close/zoom controls, View-menu fullscreen, drag, and resize.

Finder has sidebar, icon grid, sorted directory listing, status/path bar, parent-folder navigation, and folder-entry helpers.

Definition of done remains the full desktop environment: working Finder, TextEdit, Settings, Terminal, package-manager backed App Store, native dark mode, compositor/session path, HDR, VRR, and real application/game validation.

## Workspace

| Crate | Path | Status |
|-------|------|--------|
| retro-render | `crates/retro-render/` | Prototype: native `wgpu` rendering works; text, clipping, compositor/display features, HDR, and VRR remain incomplete. |
| retro-kit | `crates/retro-kit/` | Prototype: core widgets and layout exist; polished accessibility, drag/drop, focus visuals, menus, and theme coverage remain incomplete. |
| retro-shell | `crates/retro-shell/` | Prototype: rendered desktop, menu bar, shell services, managed shell windows, close, zoom, fullscreen, drag, and resize exist; focus visuals, minimize, sessions, app lifecycle, and compositor integration remain incomplete. |
| retro-bus | `crates/retro-bus/` | Foundation: local transport primitives exist; broader service integration remains incomplete. |
| retro-sdk | `crates/retro-sdk/` | Prototype: app runtime and immediate renderer exist; command routing and mature app integration remain incomplete. |

## Applications

| App | Path | Status |
|-----|------|--------|
| Finder | `apps/finder/` | Prototype: menus, sorted file listing, status/path bar, folder entry, parent navigation, and file operation helpers exist; real desktop integration, trash UI, contextual menus, and polished multi-window workflows remain incomplete. |
| Settings | `apps/settings/` | Stub/prototype; persistence and real controls remain incomplete. |
| TextEdit | `apps/textedit/` | Stub/prototype; open/save, editing model, selection, clipboard, and undo/redo remain incomplete. |
| Terminal | `apps/terminal/` | Prototype terminal surface and PTY foundation; lifecycle, resize, scrollback, copy/paste, and persistence remain incomplete. |
| App Store | Not started | Needs Linux/BSD package-manager backend abstraction, search/install/remove/update flows, and privilege handling. |

## Recent Verification

- `cargo check --workspace --all-targets`
- `cargo test --workspace -q` (39 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `retro-shell` starts, accepts pointer interaction, toggles the active Finder-style managed shell window into fullscreen from the View menu, and captures `docs/screenshots/current-retroshell-desktop.png`.
- Prior Linux VM/Xvfb/Mesa smoke: `finder` starts against a demo home directory and captures `docs/screenshots/current-finder.png`.

## Next Milestones

1. Finish shell window management: focus rings, multiple real app surfaces, minimize controls, modal dialogs, persisted placement.
2. Make Finder real: browse directories from shell windows, open folders, wire file operations to UI, add context menus and trash behavior.
3. Make TextEdit real: open/save documents, dirty state, undo/redo, selection, clipboard, and file dialogs.
4. Make Terminal robust: PTY lifecycle, resize propagation, scrollback UI, selection, copy/paste, and shell session persistence.
5. Make Settings persistent: appearance, input, display, package-manager settings, and future HDR/VRR controls.
6. Add App Store: backend abstraction for Linux/BSD package managers, search/install/remove/update flows, and safe privilege handling.
7. Build compositor/display path: Wayland/session integration, multi-monitor, HiDPI, HDR metadata/color pipeline, and VRR frame pacing.
8. Capture release evidence: video with audio of Doom running on RetroShell in windowed, borderless fullscreen, and exclusive fullscreen modes.
