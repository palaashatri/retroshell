# RetroShell Project Status

## Summary

RetroShell is a working native Rust prototype, not a complete desktop environment yet. Current work has a `wgpu` rendered shell window, original Classic-inspired desktop surface filling the 1280x800 VM smoke surface, interactive menu bar, desktop icons, first-party app foundations, managed Finder-style shell windows with menu-driven creation, focus/raise, active-window close, titlebar close/zoom controls, View-menu fullscreen, drag, resize, desktop folder icons opening filesystem-backed shell windows, and folder icons inside managed shell windows opening child folder windows.

Finder has sidebar, icon grid, sorted directory listing, status/path bar, visible Back/Forward/Up controls, folder entry, parent-folder navigation, back/forward history, and file operation helpers. Definition done remains a full desktop environment: working Finder, TextEdit, Settings, Terminal, package-manager backed App Store, native dark mode, compositor/session path, HDR, VRR, and real application/game validation.

## Workspace

| Crate | Path | Status |
|-------|------|--------|
| retro-render | `crates/retro-render/` | Prototype: native `wgpu` rendering works; text, clipping, compositor/display features, HDR, VRR remain incomplete. |
| retro-kit | `crates/retro-kit/` | Prototype: core widget layout exists; polished accessibility, drag/drop, focus visuals, menus, theme coverage remain incomplete. |
| retro-shell | `crates/retro-shell/` | Prototype: rendered desktop, menu bar, shell services, managed shell windows, close, zoom, fullscreen, drag, resize, desktop folder icon launch, and child folder opening exist; focus visuals, minimize, sessions, app lifecycle, compositor integration remain incomplete. |
| retro-bus | `crates/retro-bus/` | Foundation: local transport primitives exist; broader service integration remains incomplete. |
| retro-sdk | `crates/retro-sdk/` | Prototype: app runtime and immediate renderer exist; command routing and mature app integration remain incomplete. |

## Applications

| App | Path | Status |
|-----|------|--------|
| Finder | `apps/finder/` | Prototype: menus, sorted file listing, status/path bar, visible Back/Forward/Up controls, folder entry, parent/back/forward navigation, and file operation helpers exist; real desktop integration, trash UI, contextual menus, and polished multi-window workflows remain incomplete. |
| Settings | `apps/settings/` | Stub/prototype; persistence and real controls remain incomplete. |
| TextEdit | `apps/textedit/` | Stub/prototype; open/save, editing model, selection, clipboard, and undo/redo remain incomplete. |
| Terminal | `apps/terminal/` | Prototype terminal surface and PTY foundation; lifecycle, resize, scrollback, copy/paste, persistence remain incomplete. |
| App Store | Not started | Needs Linux/BSD package-manager backend abstraction, search/install/remove/update flows, privilege handling. |

## Recent Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace -q` (48 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `retro-shell` starts at 1280x800, accepts pointer interaction, toggles the active Finder-style managed shell window into fullscreen from the View menu, opens Home from the desktop into a managed folder window, opens a child folder from inside the initial managed folder window, and captures `docs/screenshots/current-retroshell-desktop.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `finder` starts against a demo home directory, renders visible Back/Forward/Up controls, enters a folder, refreshes path/status display, and captures `docs/screenshots/current-finder.png`.

## Next Milestones

1. Finish shell window management: focus rings, multiple real app surfaces, minimize controls, modal dialogs, persisted placement.
2. Make Finder real: wire file operations to UI, add context menus and trash behavior, polish multi-window workflows.
3. Make TextEdit real: open/save documents, dirty state, undo/redo, selection, clipboard, file dialogs.
4. Make Terminal robust: PTY lifecycle, resize propagation, scrollback UI, selection, copy/paste, shell session persistence.
5. Make Settings persistent: appearance, input, display, package-manager settings, future HDR/VRR controls.
6. Add App Store: backend abstraction for Linux/BSD package managers, search/install/remove/update flows, safe privilege handling.
7. Build compositor/display path: Wayland/session integration, multi-monitor, HiDPI, HDR metadata/color pipeline, VRR frame pacing.
8. Capture release evidence: video with audio of Doom running on RetroShell in windowed, borderless fullscreen, exclusive fullscreen modes.
