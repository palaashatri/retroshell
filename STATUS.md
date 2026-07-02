# RetroShell Project Status

## Summary

RetroShell is a working native Rust prototype, not a complete desktop environment yet. Current work has a `wgpu` rendered shell window, Classic-inspired desktop surface, interactive menu bar, desktop icons, first-party app foundations, managed Finder-style shell windows, focus/raise behavior, active-window close, titlebar close/zoom controls, View-menu fullscreen, drag, resize, desktop folder icons opening filesystem-backed shell windows, and child folder windows managed by the shell.

Finder has sidebar/icon-grid foundations, sorted directory listing, status/path bar, visible Back/Forward/Up controls, folder entry, parent-folder navigation, back/forward history, visible New Folder/Duplicate/Trash controls, file operation helpers, and a working Get Info path through Cmd-I and the `INFO` toolbar button. The VM-smoked UI shows selected-file metadata in the status bar.

TextEdit has real document-flow foundations: open an optional document path, edit native multiline text, save to disk, New/Save toolbar actions, Cmd-N/Cmd-S, saved/edited status, undo/redo history, and baseline whole-document copy/cut/paste/select-all.

Settings has the first persistent workflow: Light/Dark/System appearance controls load and save `settings.conf` under `RETROSHELL_CONFIG_DIR` or `~/.config/retroshell`, and VM smoke verifies Dark mode preference persistence.

Native dark mode has its first runtime path: RetroSDK reads the same `settings.conf` preference and renders shared window chrome, menus, labels, buttons, text fields, lists, sidebars, split panels, toolbars, and status bars with dark-aware colors.

Terminal has a real PTY-backed output path, async repaint, resize propagation, scrollback navigation, and baseline Cmd-C/Cmd-V clipboard behavior.

App Store has the first native package-manager backed workflow: it detects host Linux/BSD package managers and runs read-only package searches against the detected backend.

Definition of done remains a full desktop environment: working Finder, TextEdit, Settings, Terminal, package-manager backed App Store, native dark mode, compositor/session path, HDR, VRR, and Doom video evidence with audio in windowed, borderless fullscreen, and exclusive fullscreen modes. That is not complete yet.

## Component Status

| Component | Path | Status |
| --- | --- | --- |
| retro-shell | `crates/retro-shell/` | Prototype: rendered desktop, menu bar, shell services, managed shell windows, close, zoom, fullscreen, drag, resize, desktop folder icon launch, and child folder opening exist; focus visuals, minimize, sessions, app lifecycle, compositor integration remain incomplete. |
| retro-bus | `crates/retro-bus/` | Foundation: local transport primitives exist; broader service integration remains incomplete. |
| retro-sdk | `crates/retro-sdk/` | First-party app runtime works for basic windows/menus/widgets, repaints async widget updates, consumes `appearance=dark` shared native rendering; app lifecycle, dialogs, platform clipboard, platform services, complete theme tokens, and polished text rendering remain incomplete. |
| Finder | `apps/finder/` | In progress: navigation history, visible navigation controls, file operation helpers, and Get Info status metadata exist; contextual menus, drag/drop, trash polish, desktop integration, and multi-window workflows remain incomplete. |
| Settings | `apps/settings/` | In progress: persistent Light/Dark/System appearance preference works and SDK consumes it for native dark rendering; broader settings panes, input/display controls, HDR/VRR controls remain incomplete. |
| TextEdit | `apps/textedit/` | In progress: opens optional document path, edits text, saves existing files, tracks dirty state, supports undo/redo and baseline whole-document clipboard commands; Save As, file dialogs, selection UI, platform clipboard, and text wrapping/scrolling polish remain incomplete. |
| Terminal | `apps/terminal/` | In progress: PTY launch/output, resize propagation, scrollback navigation, and baseline clipboard shortcuts work; selection UI, persistent sessions, robust shell lifecycle, and polished tabs remain incomplete. |
| App Store | `apps/appstore/` | In progress: detects APT/DNF/Pacman/pkg/apk/zypper/brew, runs read-only package searches, and renders results; install/remove/update flows, privilege prompts, and transaction logs remain incomplete. |

## Recent Verification

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace -q` (68 tests)
- `cargo clippy --workspace --all-targets -- -D warnings`
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `retro-shell` renders the desktop, handles menu interaction/window controls/drag/resize/fullscreen, opens a managed folder window, and captures `docs/screenshots/current-retroshell-desktop.png`.
- Linux VM/Xvfb smoke: `finder` starts against a demo home directory, selects `note.txt`, triggers the `INFO` toolbar action, renders `INFO - FILE - NOTE.TXT - 5 BYTES` in the status bar, and captures `docs/screenshots/current-finder.png`.
- Linux VM/Xvfb smoke: `textedit` opens a document path, renders edit controls, and captures `docs/screenshots/current-textedit.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `settings` clicks Dark appearance, verifies `appearance=dark`, renders selected mode/status UI, and captures `docs/screenshots/current-settings.png`.
- Linux VM/Xvfb/Mesa Vulkan llvmpipe smoke: `settings` launches with `appearance=dark`, renders dark native chrome/controls, and captures `docs/screenshots/current-dark-mode-settings.png`.
- Linux VM/Xvfb smoke: `terminal` launches a PTY-backed shell script, consumes async output, renders live terminal text, and captures `docs/screenshots/current-terminal.png`.
- Linux VM/Xvfb smoke: `appstore` detects APT, searches for `doom`, renders package-manager results, and captures `docs/screenshots/current-appstore.png`.

## Next Milestones

1. Finish shell window management: focus rings, multiple real app surfaces, minimize controls, modal dialogs, persisted placement.
2. Make Finder real: contextual menus, drag/drop, trash UI polish, desktop integration, polished multi-window workflows.
3. Make TextEdit real: Save As/file dialogs, selection UI, platform clipboard, text wrapping/scrolling polish.
4. Make Terminal robust: selection UI, platform clipboard, shell lifecycle handling, tab controls, scrollback UI, shell session persistence.
5. Make Settings persistent beyond appearance: input, display, package-manager settings, future HDR/VRR controls.
6. Finish App Store package manager flows: install/remove/update, privilege prompts, transaction logs, backend-specific error handling.
7. Finish native dark mode: end-to-end token coverage, live switching polish, app-specific asset/icon compliance, contrast validation.
8. Build compositor/session path: Wayland session behavior, external app surfaces, multi-monitor, HiDPI, startup integration.
9. Build display pipeline goals: HDR metadata/color pipeline, VRR frame pacing, fullscreen mode policy.
10. Produce release evidence: Doom running with audio/video in windowed, borderless fullscreen, and exclusive fullscreen modes.
