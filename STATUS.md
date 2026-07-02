# RetroShell Project Status

## Summary

RetroShell is a working native Rust prototype, not a complete desktop environment yet. Current work has a `wgpu` rendered shell window, Classic-inspired desktop surface, interactive menu bar, desktop icons, first-party app foundations, and managed Finder-style shell windows with focus/raise, active-window close, titlebar close/zoom controls, View-menu fullscreen, drag, resize, desktop folder icons opening filesystem-backed shell windows, and child folder windows from managed shell windows.

Finder has sidebar/icon-grid foundations, sorted directory listing, status/path bar, visible Back/Forward/Up controls, folder entry, parent-folder navigation, back/forward history, visible New Folder/Duplicate/Trash controls, and file operation helpers.

TextEdit has its first real document flow: open an optional document path, edit native multiline text, save to disk, New/Save toolbar actions, Cmd-N/Cmd-S, and saved/edited status.

Settings has its first real persistent workflow: Light/Dark/System appearance controls load and save `settings.conf` under `RETROSHELL_CONFIG_DIR` or `~/.config/retroshell`, and the VM smoke verifies Dark mode preference persistence.

Terminal has a real PTY-backed output path with async repaint, resize propagation, scrollback navigation, and baseline Cmd-C/Cmd-V clipboard behavior.

App Store has a first native package-manager backed workflow: it detects host Linux/BSD package managers and runs read-only package searches against the detected backend.

Definition of done remains a full desktop environment: working Finder, TextEdit, Settings, Terminal, package-manager backed App Store, native dark mode, compositor/session path, HDR, VRR, and real application/game validation including Doom video evidence with audio.

## Workspace

| Crate/App | Path | Status |
|-----------|------|--------|
| retro-render | `crates/retro-render/` | Prototype: native `wgpu` rendering works; text, clipping, compositor/display features, HDR, VRR remain incomplete. |
| retro-kit | `crates/retro-kit/` | Prototype: core widget layout exists; polished accessibility, drag/drop, focus visuals, menus, theme coverage remain incomplete. |
| retro-shell | `crates/retro-shell/` | Prototype: rendered desktop, menu bar, shell services, managed shell windows, close, zoom, fullscreen, drag, resize, desktop folder icon launch, and child folder opening exist; focus visuals, minimize, sessions, app lifecycle, compositor integration remain incomplete. |
| retro-bus | `crates/retro-bus/` | Foundation: local transport primitives exist; broader service integration remains incomplete. |
| retro-sdk | `crates/retro-sdk/` | First-party app runtime works for basic windows/menus/widgets and now repaints after async widget updates; app lifecycle, dialogs, platform clipboard, platform services, and polished text rendering remain incomplete. |
| Finder | `apps/finder/` | In progress: navigation history, visible navigation controls, and file operation helpers exist; contextual menus, drag/drop, trash polish, desktop integration, and multi-window workflows remain incomplete. |
| Settings | `apps/settings/` | In progress: persistent Light/Dark/System appearance preference works; live shell theme application, input/display controls, HDR/VRR controls remain incomplete. |
| TextEdit | `apps/textedit/` | In progress: opens optional document path, edits text, saves existing files, tracks dirty state; Save As, file dialogs, selection, clipboard, undo/redo remain incomplete. |
| Terminal | `apps/terminal/` | In progress: PTY launch/output, resize propagation, scrollback navigation, and baseline clipboard shortcuts work; selection UI, persistent sessions, robust shell lifecycle, and polished tabs remain incomplete. |
| App Store | `apps/appstore/` | In progress: detects APT/DNF/Pacman/pkg/apk/zypper/brew, runs read-only package searches, and renders results; install/remove/update flows, privilege prompts, and transaction logs remain incomplete. |

## Recent Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace -q` (61 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `retro-shell` starts 1280x800, accepts pointer interaction, toggles active Finder-style managed shell window into fullscreen through the View menu, opens Home into a managed folder window, opens a child folder inside the initial managed folder window, and captures `docs/screenshots/current-retroshell-desktop.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `finder` starts against a demo home directory, renders visible Back/Forward/Up and file-operation controls, creates New Folder from the toolbar, refreshes path/status display, and captures `docs/screenshots/current-finder.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `textedit` opens a document path, renders document text with saved/path status, and captures `docs/screenshots/current-textedit.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `settings` clicks Dark appearance, verifies `appearance=dark`, renders selected mode/status UI, and captures `docs/screenshots/current-settings.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `terminal` launches a PTY-backed shell script, consumes async output, renders live terminal text, and captures `docs/screenshots/current-terminal.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `appstore` detects APT, searches for `doom`, renders package-manager results, and captures `docs/screenshots/current-appstore.png`.

## Next Milestones

1. Finish shell window management: focus rings, multiple real app surfaces, minimize controls, modal dialogs, persisted placement.
2. Make Finder real: add contextual menus, drag/drop, trash UI polish, desktop integration, polished multi-window workflows.
3. Make TextEdit real: Save As/file dialogs, selection, clipboard, undo/redo, text wrapping/scrolling polish.
4. Make Terminal robust: selection UI, platform clipboard, shell lifecycle handling, tab controls, scrollback UI, shell session persistence.
5. Make Settings persistent beyond appearance: input, display, package-manager settings, future HDR/VRR controls.
6. Add App Store: package manager backend abstraction, search, install, remove, update, privilege prompts, transaction logs.
7. Add native dark mode: end-to-end token coverage, live switching, app compliance, screenshot verification.
8. Build compositor/session path: Wayland session behavior, external app surfaces, multi-monitor, HiDPI, input methods, clipboard, accessibility.
9. Add display targets: HDR metadata/color pipeline and VRR frame pacing.
10. Produce release evidence: video with audio of Doom in windowed, borderless fullscreen, and exclusive fullscreen modes.
