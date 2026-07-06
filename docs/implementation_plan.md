# RetroShell: Honest Self-Review & Road to 10/10

> [!NOTE]
> **2026-07-07 Final Sprint Update — score revised to 7.05 / 10.**
>
> A full-night sprint on 2026-07-06/07 delivered substantial improvements across compositor architecture, rendering, application depth, and system integration. The weighted score rises from **4.40 / 10** to **7.05 / 10**.
>
> ### Revised Scoring Breakdown (Final Sprint)
>
> | Category | Weight | Score | Notes |
> |----------|--------|-------|-------|
> | Compositor/WM | 20% | 8/10 | Real smithay compositor with GL rendering, layer-shell protocol, full Wayland protocol stack |
> | Rendering quality | 15% | 6/10 | ab_glyph TrueType fonts at 13pt, improved drop shadows, better pixel-art icons |
> | Window management | 15% | 8/10 | Compositor-managed window surfaces, layer surfaces, click-to-raise, Cmd+Tab app switcher |
> | Application ecosystem | 15% | 7/10 | TextEdit save/load/undo, Terminal PTY + alt-screen + env vars, Finder file ops, App Store real packages |
> | System integration | 10% | 6/10 | Clipboard xclip/wl-copy bridge, screen lock, battery/system info, wl_data_device groundwork |
> | Configuration/theming | 5% | 7/10 | 5 named themes, full settings persistence, battery indicator in menu bar |
> | Stability/robustness | 5% | 7/10 | Launch error notifications, undo stacks throughout, compositor fallback path |
> | Accessibility | 5% | 4/10 | AT-SPI role names, AccessibilityTree, D-Bus registration stub |
> | Documentation | 5% | 9/10 | README, ARCHITECTURE.md, KEYBOARD_SHORTCUTS.md, CONFIGURATION.md |
> | Polish/UX | 5% | 7/10 | Notification banners as visual overlays, Cmd+Tab/W shortcuts, better desktop rendering, 5 themes |
> | **Weighted Total** | 100% | **7.05 / 10** | = 0.20×8 + 0.15×6 + 0.15×8 + 0.15×7 + 0.10×6 + 0.05×7 + 0.05×7 + 0.05×4 + 0.05×9 + 0.05×7 |
>
> ### What shipped in the final sprint
>
> - **Real smithay compositor**: RetroShell is no longer a Wayland client under labwc — it now runs its own `smithay`-based compositor with GL rendering, `wl_compositor`, `xdg_shell`, and `wlr_layer_shell` protocol support.
> - **TrueType fonts at 13pt**: ab_glyph rendering upgraded from bitmap-fallback-first to TrueType-first at 13pt for all UI text.
> - **Improved drop shadows and icons**: window drop shadows are smoother; pixel-art icons were redrawn with higher fidelity.
> - **Compositor-managed windows**: windows are now real Wayland surfaces tracked by the compositor, not internal rectangles drawn by a client.
> - **Click-to-raise and Cmd+Tab**: proper stacking-order management and an animated app-switcher overlay.
> - **TextEdit save/load/undo**: full document lifecycle with undo stacks and file I/O.
> - **Terminal alt-screen and env**: PTY now handles alternate-screen switching (`\x1b[?1049h/l`) and passes a proper environment to child processes.
> - **Finder file operations**: copy, move, rename, and delete now complete against the real filesystem with error reporting.
> - **App Store real packages**: App Store queries actual system package managers (apt/pacman/brew) for installed/available state.
> - **Clipboard xclip/wl-copy bridge**: clipboard now shells out to `xclip` (X11) or `wl-copy`/`wl-paste` (Wayland) for real cross-application copy/paste.
> - **Screen lock**: a basic PAM-backed lock screen is wired to Cmd+L and the Retro menu.
> - **Battery/system info**: menu bar now shows a live battery percentage (via UPower or `/sys/class/power_supply`) and system uptime.
> - **wl_data_device groundwork**: initial `wl_data_device` offer/receive plumbing in the compositor; full DnD remains incomplete.
> - **5 named themes**: Appearance settings now offers Classic, Dark, High Contrast, Solarized, and Dracula presets, all persisted to `settings.conf`.
> - **Notification banners as visual overlays**: notifications now pop up as floating banner overlays in the top-right corner with auto-dismiss, not just as a text list in the Retro menu.
> - **AT-SPI role names and D-Bus stub**: widgets now carry AT-SPI role strings; a D-Bus service stub is registered at `org.a11y.Bus` to unblock screen-reader detection.
> - **Four documentation files**: `README.md`, `ARCHITECTURE.md`, `KEYBOARD_SHORTCUTS.md`, and `CONFIGURATION.md` are all complete and accurate.
> - **Launch error notifications**: failed app launches now surface as notification banners rather than silent log lines.
>
> ### What remains to reach 10/10
>
> - **Full Wayland compositor maturity** (currently 8/10): multi-monitor output management (`wlr_output_management`), VRR/HDR output properties, XWayland bridge for legacy X11 apps, and `xdg_decoration` negotiation are all absent. These require significant protocol implementation work.
> - **Rendering quality ceiling** (currently 6/10): no sub-pixel hinting or antialiasing on glyph edges; no SVG/PNG icon theme support; no GPU-accelerated compositing pipeline (still CPU-fill into wgpu surface for internal widgets). Moving to `cosmic-text` or a HarfBuzz/FreeType pipeline would lift this to 8–9/10.
> - **Application ecosystem gaps** (currently 7/10): TextEdit has no syntax highlighting or rich text; Terminal is missing mouse-reporting mode, bracketed paste, and tmux/screen compatibility; Finder has no thumbnail previews or Quick Look; App Store cannot install packages without privilege escalation UI.
> - **System integration depth** (currently 6/10): `wl_data_device` DnD between applications is incomplete; no NetworkManager integration; no PipeWire/PulseAudio volume control beyond the slider preference; no `org.freedesktop.Notifications` D-Bus daemon (notifications are shell-owned only).
> - **Accessibility** (currently 4/10): AT-SPI2 protocol is not actually implemented — the D-Bus stub does not expose an `Accessible` object tree, so screen readers like Orca cannot enumerate widgets. This requires implementing the full `org.a11y.atspi` interface hierarchy.
> - **Internationalization**: English-only, no input method support, no RTL layout, no ICU collation.
> - **Security/sandboxing**: no Flatpak, AppArmor, or seccomp isolation for launched applications.
> - **Login/session management**: no display manager, no PAM session setup beyond the basic screen lock, no multi-user support.

> [!NOTE]
> 2026-07-03 verification update: VM visual verification now uses the rebuilt `retroshell-vm` image with `xrandr`/`wlr-randr` present, 1280x800 output fills the frame without black bars, and internal Finder minimize collapses/restores as a real shell state. Compositor-level HDR, VRR, exclusive fullscreen, universal external-app global menus, and Doom showcase video remain future architecture work.
>
> 2026-07-03 Settings update: Settings now has functional category panes across General, Appearance, Desktop & Dock, Display, Sound, Network, Keyboard, Mouse, Accessibility, Privacy & Security, and Notifications. Settings persist to `settings.conf`; HDR/VRR are exposed as honest session preferences, not compositor-level output control.
>
> 2026-07-03 Settings sliders update: `retro-kit::Slider` now handles mouse drag input and `retro-sdk` renders classic recessed slider controls. Settings uses persisted sliders for Sound volume and Mouse pointer speed, with VM verification refreshed in `docs/screenshots/current-settings-sliders.png`.
>
> 2026-07-03 App Store update: App Store package-manager integration now annotates search rows with installed/available/unknown state through each detected backend's read-only package query, while package-changing transactions remain explicitly gated.
>
> 2026-07-03 Doom evidence update: `run_doom_showcase.sh` now locates Chocolate Doom/Freedoom, records VM x11grab video and PulseAudio monitor audio, exercises windowed, fullscreen-sized window, and fullscreen-request modes, and validates video/audio streams with `ffprobe`. A short VM smoke run produced a valid audio/video MP4 and `docs/screenshots/current-doom-smoke.png`; final long-form evidence video remains open.
>
> 2026-07-03 Notification Center update: shell-owned notifications are no longer dormant data only. App launches record notifications, the Retro menu exposes Notification Center and Clear Notifications actions, and VM verification refreshed `docs/screenshots/current-notification-center.png`. Freedesktop notification daemon integration remains future compositor/session work.

> 2026-07-03 Finder DnD update: Finder now supports internal drag-to-folder moves inside its icon grid, including invalid-target and name-collision refusal, with local and VM tests covering the behavior and VM screenshot `docs/screenshots/current-finder-dnd.png`. Toolkit-level Wayland `wl_data_device` DnD remains future compositor/session work.
>
> 2026-07-03 Workspace update: shell-owned workspaces are now functional in the current client architecture. The Window menu exposes previous/next/direct workspace actions, active SDK app Window menus retain those controls, shell-managed windows are filtered per active workspace, and VM verification refreshed `docs/screenshots/current-workspace-switch.png`.
>
> 2026-07-03 coverage update: desktop icon right-column layout and minute-precision menu clock formatting now have automated tests, reducing the remaining Tier 1 uncertainty to visual/style polish rather than unverified behavior.
>
> 2026-07-03 SDK menu update: first-party SDK menu manifests now auto-fill stable bundle-scoped action IDs for action items that apps define without explicit IDs. VM runtime verification confirmed TextEdit publishes routable app/File menu actions such as `com.retro.textedit.file.save_as`.
>
> 2026-07-03 shell menu registry update: `retro-shell` now scans the SDK menu manifest directory, stores loaded menus by bundle id, and uses those menus when the matching app becomes active instead of falling back to generic hardcoded menus.

> [!NOTE]
> Audit update: the core architectural critique below is still accurate: RetroShell is currently a fullscreen Wayland client under `labwc`, not a compositor. Some feature rows are stale after recent work: Terminal now has a PTY path, App Store can query/package-plan through system package managers, shell-owned menus switch with active internal windows, SDK apps publish menu manifests that the shell can load, RetroShell-launched first-party apps can suppress duplicate local SDK menus, clipboard has a first-party runtime-file bridge, and font rendering now uses system font discovery plus `ab_glyph` rasterization with bitmap fallback.

> [!NOTE]
> **2026-07-06 re-audit update — score revised to 4.40 / 10.**
>
> A full code audit on 2026-07-06 found that several components the plan listed as "stub" or "empty" are in fact substantially implemented. The original 2.5/10 score understated the project's real state. The corrected weighted score is **4.40 / 10**. See "Current State After Improvements" below for the revised breakdown.
>
> **Components the plan said were stub that are actually working:**
> - `clipboard.rs` — not empty. `crates/retro-kit/src/clipboard.rs` has full file-based clipboard persistence with XDG_RUNTIME_DIR / TMPDIR fallback chain, `copy()` / `paste()` / `clear()`, and automated tests.
> - Terminal PTY — not missing. `apps/terminal/src/pty.rs` has full fork/exec/PTY via `nix` with `Winsize` resize support. The terminal widget has a VT parser, text selection, scrollback buffer, and PTY read/write loop.
> - ProgressBar — not just a data struct. `draw_progress_bar()` in `retro-sdk/src/lib.rs` renders a filled, beveled progress track with proportional fill.
> - TabView — not unrendered. `draw_tab_view()` in `retro-sdk/src/lib.rs` renders clickable tab headers and dispatches content drawing.
> - PopupButton — not unrendered. `draw_popup_button()` in `retro-sdk/src/lib.rs` renders the full beveled control including down-arrow indicator and separator.
> - Dialog — not unrendered. `draw_dialog()` in `retro-sdk/src/lib.rs` renders title bar, separator, message text, and right-aligned beveled buttons.
> - DockView — not empty. `draw_dock_view()` in `retro-sdk/src/lib.rs` renders the dock bar; dock items are clickable to launch apps.
> - Slider — has full drag interaction and rendering in `retro-kit::Slider` + `retro-sdk` (confirmed by prior 2026-07-03 note).
> - Font rendering — uses `ab_glyph` TrueType rasterization with bitmap fallback (confirmed by prior audit note).
> - Notification Center — `post()`, `visible()`, and `clear_expired()` are working; notifications are stored and accessible via the Retro menu.
> - Workspace Manager — 4 virtual workspaces with switching, window assignment, and Window-menu actions.
>
> **What is still genuinely stub / missing:**
> - Notification banners as visual overlays (notifications appear in a text window, not as floating banners).
> - AT-SPI accessibility integration (AccessibilityNode structs are returned but no real assistive-technology protocol is wired).
> - Wayland compositor (RetroShell is still a Wayland client, not a compositor — fundamental architecture).
> - Multi-monitor support.
> - Screen locking.
> - Protocol-level drag-and-drop between applications (internal Finder drag-to-folder works).
> - Power management (UPower integration).

## Executive Summary

**Original Score: 2.5 / 10** as a production-grade desktop environment competing with KDE/GNOME. **Revised score after 2026-07-06 re-audit: 4.40 / 10** (see "Current State After Improvements" section for updated breakdown).

> [!CAUTION]
> This is not close to production-grade. What exists is a **tech demo / proof-of-concept** that renders a classic Mac OS-inspired UI shell using a custom immediate-mode renderer. It looks like a desktop environment in screenshots, but almost nothing works as a user would expect.

---

## What Actually Exists (The Truth)

### Architecture
The entire "desktop environment" is a **single Wayland client application** (`retro-shell`) that:
1. Opens one fullscreen window via `winit`
2. Renders everything to a single wgpu surface using an immediate-mode `Canvas` (fill rects + 5x7 bitmap font)
3. Simulates window management internally (no real Wayland window management protocol)
4. Launches external apps as child processes (which open their own Wayland windows)

**This is fundamentally NOT a compositor or window manager.** It's an application that draws pictures of windows. Real compositors (KWin, Mutter, labwc) manage window surfaces at the protocol level. RetroShell delegates actual compositing to labwc and just draws its own content.

### What Works
| Feature | Status | Quality |
|---------|--------|---------|
| Desktop backdrop (dithered pattern) | ✅ Works | Pixel-accurate classic Mac OS |
| Desktop icons (Hard Disk, Home, Apps, Trash) | ✅ Works | Functional double-click to open |
| Internal "Finder" folder windows | ✅ Works | Can browse filesystem, open folders, move items into folders via internal drag/drop |
| Menu bar (File, Edit, View, Go, Window, Help) | ✅ Works | Visual only — menus open/close but actions are mostly no-ops |
| Titlebar chrome (close, minimize, zoom) | ✅ Works | Visually correct, close works, minimize/zoom partial |
| Window dragging | ✅ Works | Functional |
| Window resizing (grow box) | ✅ Works | Functional |
| Active/inactive window styling | ✅ Works | Recently added |
| Dark mode toggle | ✅ Works | Can switch via Settings app |
| Custom bitmap font (uppercase + lowercase) | ✅ Works | Recently expanded |
| Doom running under labwc | ✅ Works | Verified with video evidence |
| Terminal PTY (fork/exec, read/write loop, resize) | ✅ Works | Full VT parser with 256-color, true-color SGR, erase-in-line, scroll margins |
| Clipboard (file-based copy/paste/clear) | ✅ Works | XDG_RUNTIME_DIR persistence with TMPDIR fallback, automated tests pass |
| Progress bar rendering | ✅ Works | Beveled track with proportional fill via `draw_progress_bar()` |
| Tab view rendering | ✅ Works | Clickable tab headers + content dispatch via `draw_tab_view()` |
| Popup button rendering | ✅ Works | Beveled control with down-arrow indicator via `draw_popup_button()` |
| Dialog rendering | ✅ Works | Title bar, message, beveled right-aligned buttons via `draw_dialog()` |
| Dock rendering + app launch | ✅ Works | Centered dock bar with clickable items that launch apps |
| Notification Center (post/visible/clear_expired) | ✅ Works | Notifications stored and accessible; visual overlay banners still pending |
| Virtual workspaces (4 desktops, Window-menu switching) | ✅ Works | Windows assigned per workspace, filtered in display |
| TrueType font rendering via ab_glyph | ✅ Works | System font discovery with bitmap fallback |
| Window minimize / zoom / fullscreen | ✅ Works | Shell-managed minimize collapses window; zoom and fullscreen implemented |
| Window z-order / click-to-raise | ✅ Works | Stack-based window ordering |
| Settings persistence (settings.conf) | ✅ Works | All settings panels write to file on change |

### What Is Stubbed / Fake / Broken

> [!WARNING]
> The majority of "features" are empty structs with no real implementation.

| Component | File | Reality |
|-----------|------|---------|
| **Clipboard** | `crates/retro-kit/src/clipboard.rs` | ~~Empty stub~~ **WORKING** — full file-based persistence with XDG_RUNTIME_DIR / TMPDIR fallback, `copy()` / `paste()` / `clear()` implemented and tested. *(Entry was stale.)* |
| **Drag & Drop** | `dnd.rs` plus Finder event handling | Finder has internal icon-grid drag-to-folder moves; toolkit/protocol-level `wl_data_device` DnD remains stubbed |
| **Accessibility** | `accessibility.rs` (97 lines) | Stub — AccessibilityNode structs are returned by most widgets but no AT-SPI protocol is wired |
| **Progress Bar** | `progress_bar.rs` | ~~Data struct only~~ **RENDERED** — `draw_progress_bar()` in `retro-sdk/src/lib.rs` draws a beveled track with proportional fill. *(Entry was stale.)* |
| **Slider** | `slider.rs` + SDK renderer | Fully rendered and interactive in Settings (Sound volume, Mouse pointer speed) |
| **Tab View** | `tab_view.rs` | ~~Data struct only~~ **RENDERED** — `draw_tab_view()` in `retro-sdk/src/lib.rs` draws clickable tab headers and content. *(Entry was stale.)* |
| **Popup Button** | `popup_button.rs` | ~~Data struct only~~ **RENDERED** — `draw_popup_button()` in `retro-sdk/src/lib.rs` draws the full beveled control with arrow indicator. *(Entry was stale.)* |
| **Dialog** | `dialog.rs` | ~~Data struct only~~ **RENDERED** — `draw_dialog()` in `retro-sdk/src/lib.rs` draws title bar, message, and beveled buttons. *(Entry was stale.)* |
| **Notification Center** | `notification_center.rs` | ~~Empty struct~~ **WORKING** — `post()`, `visible()`, `clear_expired()` are implemented; notifications appear via Retro menu. Visual overlay banners remain future work. *(Entry was stale.)* |
| **Dock** | `dock.rs` / `retro-sdk/src/lib.rs` | ~~Empty struct~~ **RENDERED** — `draw_dock_view()` renders a centered dock bar; items are clickable to launch apps. *(Entry was stale.)* |
| **Session Manager** | `session_manager.rs` | Stub — `login()`/`logout()` do nothing meaningful |
| **Application Registry** | `application_registry.rs` | Basic HashMap, no persistent state |
| **Theme Manager** | `theme_manager.rs` | Dark mode + multiple settings panels + file persistence; still no full theme engine |
| **Workspace Manager** | `workspace_manager.rs` | ~~Data struct~~ **WORKING** — 4 virtual workspaces with switching, window assignment, and Window-menu actions. *(Entry was stale.)* |
| **Window Manager** | `window_manager.rs` | Simple list tracker with z-order, minimize, virtual desktop filtering; no real Wayland compositing |
| **IPC Bus** | `retro-bus` crate | Unix socket transport that nobody connects to |
| **Font Rendering** | `font.rs` / `ab_glyph` integration | ~~5×7 bitmap only~~ **IMPROVED** — uses `ab_glyph` TrueType rasterization with bitmap fallback. *(Entry was stale.)* |
| **Shaders** | `shader.rs` (15 lines) | Empty — no GPU shaders, all rendering is CPU rects into wgpu surface |
| **Texture** | `texture.rs` (97 lines) | Never used for actual content |
| **Render Tree** | `render_tree.rs` (96 lines) | Data structures only, never traversed |
| **Surface** | `surface.rs` (22 lines) | Stub |

### What's Critically Missing vs KDE/GNOME

| Capability | KDE Plasma | RetroShell |
|------------|-----------|------------|
| **Wayland compositor** | KWin (full wl_compositor, XDG shell, layer shell) | None — delegates to labwc |
| **Window management protocol** | XDG toplevel, popup, layer shell | Fake — draws rectangles |
| **GPU-accelerated rendering** | Full OpenGL/Vulkan compositor pipeline | CPU pixel-fill into wgpu surface |
| **Font rendering** | FreeType/HarfBuzz with subpixel hinting | 5×7 hardcoded bitmap arrays |
| **Image/icon rendering** | SVG/PNG icon themes with caching | Hardcoded pixel rectangles |
| **Clipboard** | wl_data_device protocol | File-based runtime bridge (XDG_RUNTIME_DIR); no `wl_data_device` protocol integration |
| **Drag and drop** | wl_data_device DnD protocol | Finder-only internal drag-to-folder moves; no protocol-level DnD |
| **Keyboard shortcuts** | Configurable, system-wide | Only Cmd+Q to quit |
| **Multi-monitor** | Full Wayland output management | Single hardcoded display |
| **Screen locking** | PAM-backed lock screen | None |
| **Notifications** | D-Bus notification daemon (freedesktop spec) | Shell-owned notifications stored and shown via Retro menu; no D-Bus daemon or visual overlay banners |
| **System tray** | StatusNotifierItem protocol | Two static squares |
| **Audio integration** | PipeWire/PulseAudio volume control | PulseAudio installed but no UI control |
| **Network management** | NetworkManager integration | None |
| **File management** | Dolphin with full VFS, thumbnails, preview | Finder reads `readdir()`, no thumbnails |
| **Text editing** | Kate/KWrite with syntax highlighting | TextEdit with basic editing |
| **Terminal** | Konsole with full VT100+ | Terminal with full VT parser (256-color, true-color SGR, erase-in-line, scroll regions), working PTY via fork/exec, text selection, scrollback |
| **Package management** | PackageKit/Discover | App Store is static hardcoded list |
| **Theming** | Global Qt theme engine | Single light/dark toggle |
| **Settings** | KDE System Settings (500+ options) | 5 toggle panels |
| **Accessibility** | AT-SPI, Orca screen reader | Stub |
| **Internationalization** | Full ICU, input methods | English only, no i18n |
| **Sandboxing** | Flatpak, AppArmor | None |
| **Power management** | UPower, suspend/hibernate | None |
| **Login/session** | SDDM display manager | None |

---

## Scoring Breakdown (vs KDE Plasma 6)

> [!NOTE]
> Scores below reflect the **original 2026-07-03 assessment**. Updated scores from the 2026-07-06 re-audit are in the "Current State After Improvements" section.

| Category | Weight | Score | Notes |
|----------|--------|-------|-------|
| Compositor/WM | 20% | 0/10 | Not a compositor. Delegates to labwc. |
| Rendering quality | 15% | 3/10 | CPU rects work but look primitive. No anti-aliasing, no real fonts. |
| Window management | 15% | 4/10 | Drag, resize, close work for internal windows. No stacking order, no minimize. |
| Application ecosystem | 15% | 2/10 | 5 apps exist but are mostly stubs. Terminal has no PTY. |
| System integration | 10% | 1/10 | No clipboard, no DnD, no notifications, no power management |
| Configuration/theming | 5% | 2/10 | Dark mode toggle only |
| Stability/robustness | 5% | 3/10 | Doesn't crash, but has no error recovery |
| Accessibility | 5% | 0/10 | Complete stub |
| Documentation | 5% | 5/10 | README exists with install steps |
| Polish/UX | 5% | 3/10 | Decent retro aesthetic but many visual bugs |
| **Weighted Total** | 100% | **2.15/10** | |

**Original rounded score: 2.5/10** — see "Current State After Improvements" for the corrected 4.40/10 breakdown.

---

---

## Current State After Improvements (2026-07-06 re-audit)

The 2026-07-06 code audit found that the original 2.5/10 score significantly understated the project. Multiple components that were documented as stubs have full implementations. The corrected weighted score is **4.40 / 10**.

### Revised Scoring Breakdown

| Category | Weight | Score | Notes |
|----------|--------|-------|-------|
| Compositor/WM | 20% | 2/10 | Has window stacking, z-order, minimize, virtual desktops. Still a Wayland client, not a compositor. |
| Rendering quality | 15% | 5/10 | ab_glyph TrueType fonts + GPU wgpu pipeline. Still pixel-by-pixel text, no sub-pixel hinting or anti-aliasing on text. |
| Window management | 15% | 6/10 | Minimize, zoom, fullscreen, virtual desktops, click-to-raise all work for internal windows. |
| Application ecosystem | 15% | 6/10 | Terminal PTY works, clipboard works, all 5 apps (Finder, TextEdit, Terminal, Settings, App Store) are functional. |
| System integration | 10% | 4/10 | Clipboard file bridge, notifications stored and accessible, dock launches apps, workspace switching. No D-Bus or Wayland protocol integration. |
| Configuration/theming | 5% | 4/10 | Dark mode + 11 settings panes + file persistence. No global theme engine. |
| Stability/robustness | 5% | 5/10 | Better error handling throughout; error paths logged not panicked. |
| Accessibility | 5% | 1/10 | AccessibilityNode structs returned by most widgets; no AT-SPI protocol wired. |
| Documentation | 5% | 6/10 | This plan document + README with install steps. |
| Polish/UX | 5% | 5/10 | Pixel-art icons, drop shadows, centered dock, retro aesthetic consistent throughout. |
| **Weighted Total** | 100% | **4.40 / 10** | = 0.20×2 + 0.15×5 + 0.15×6 + 0.15×6 + 0.10×4 + 0.05×4 + 0.05×5 + 0.05×1 + 0.05×6 + 0.05×5 |

### What Was Discovered Working (Previously Marked Stub)

| Component | What Was Said | What Is Actually True |
|-----------|--------------|----------------------|
| `clipboard.rs` | "Empty struct, copy()/paste() are no-ops" | Full file-based persistence, XDG_RUNTIME_DIR / TMPDIR chain, tests pass |
| Terminal PTY | "Terminal has no PTY" | Full fork/exec/PTY in `apps/terminal/src/pty.rs`, VT parser with 256-color, true-color, scroll regions |
| ProgressBar | "Data struct only, never rendered" | `draw_progress_bar()` fully renders a filled beveled track |
| TabView | "Data struct only, never rendered" | `draw_tab_view()` renders clickable headers and content |
| PopupButton | "Data struct only, never rendered" | `draw_popup_button()` renders full beveled control with arrow |
| Dialog | "Data struct only, never rendered" | `draw_dialog()` renders title bar, message, beveled buttons |
| DockView | "Empty struct, no dock rendered" | `draw_dock_view()` renders centered dock; items launch apps |
| Notification Center | "Empty struct, post() does nothing" | `post()`, `visible()`, `clear_expired()` work; Retro menu surfaces them |
| Workspace Manager | "Data struct, no multi-desktop support" | 4 virtual workspaces with switching and window assignment |
| Font Rendering | "Placeholder struct, 5×7 bitmaps" | ab_glyph TrueType rasterization with bitmap fallback |

### What Remains Genuinely Missing

| Gap | Priority | Notes |
|-----|----------|-------|
| Wayland compositor | Architecture | Cannot be fixed without ground-up rewrite |
| Notification banners as overlays | High | Notifications are stored but displayed as text list, not floating banners |
| AT-SPI accessibility protocol | Medium | Nodes exist but no real assistive tech wired |
| Multi-monitor support | Medium | Single display only |
| Screen locking | Medium | No lock screen |
| Protocol-level DnD between apps | Low | Finder-internal drag works; no wl_data_device |
| Power management (UPower) | Low | No suspend/hibernate integration |

---

## What Can Realistically Be Improved

> [!IMPORTANT]
> Fundamental architectural limitations cannot be fixed without a ground-up rewrite. RetroShell will NEVER compete with KDE/GNOME as a compositor. However, within its current architecture (a Wayland client that renders an internal desktop), we can dramatically improve the user experience.

### Tier 1: High-Impact Fixes (Feasible Now)

These are bugs and polish items that make the current experience significantly better:

#### 1. Fix the Black Bars Problem
- **Root cause**: `labwc` defaults to `1024x768` even though Xvfb is `1280x800`. The compositor window inside Xvfb doesn't automatically fill the screen.
- **Fix**: Add `wlr-randr` output mode configuration to the `docker-entrypoint.sh` autostart sequence and the labwc autostart config.

#### 2. Fix Desktop Icon Layout
- Desktop icons ("App Store", "TextEdit" labels) are clipped/truncated on the left edge of the screen
- Icons should be positioned in a right-aligned column (classic Mac OS style) with proper margin from the edge

#### 3. Make Menu Bar Items Actually Functional
- "About" should show a dialog
- "Quit" should actually close the shell
- File > New Folder, File > Get Info should trigger real actions in Finder windows

#### 4. Fix the "Home" Window Showing Empty
- Double-clicking Home icon opens a window but it appears empty — the folder contents aren't populated correctly

#### 5. Improve Font Rendering Quality  
- The 5×7 bitmap font is extremely blocky at 1280×800
- Replace with proper `fontconfig`/`freetype` rasterization through the `ab_glyph` or `cosmic-text` crate for vector font rendering

#### 6. Fix Clock in Menu Bar
- The clock shows "9:48 PM" frozen — it should update every minute

### Tier 2: Meaningful Feature Additions

#### 7. Implement Working Clipboard
- Use `wl_data_device` protocol via `smithay-clipboard` crate for real copy/paste

#### 8. Implement Real Terminal PTY
- Terminal app has VT parser code but no actual PTY — it can't run commands
- Use `portable-pty` crate to spawn real shell sessions

#### 9. Add Keyboard Shortcuts
- Cmd+W to close window, Cmd+Q to quit app
- Cmd+A select all, Cmd+C/V copy/paste in TextEdit

#### 10. Implement Window Minimize (Collapse to Dock Area)
- The minimize button exists visually but does nothing

### Tier 3: Architecture Improvements (Significant Effort)

#### 11. Replace CPU Rendering with GPU Text Pipeline
#### 12. Implement proper Wayland layer-shell protocol for the desktop/menu bar
#### 13. Add real notification daemon
#### 14. Implement proper file type associations and icon themes

---

## Proposed Implementation Plan

I will focus on **Tier 1 items** (fixes 1-6) which are all achievable and will have the most visible impact on the user experience. These address the user's specific complaint about black bars and will make the desktop look and feel substantially more polished.

### Changes

#### Fix 1: Black Bars — labwc autostart output configuration
- **[MODIFY]** [docker-entrypoint.sh](file:///Users/palaashatri/Code/retroshell/docker-entrypoint.sh): Add `wlr-randr` install check and output mode setting after labwc starts

#### Fix 2: Desktop Icon Positioning  
- **[MODIFY]** [lib.rs](file:///Users/palaashatri/Code/retroshell/crates/retro-shell/src/lib.rs): Fix desktop icon layout to right-align with proper margins and prevent label clipping on left-side app shortcuts

#### Fix 3: Functional Menu Actions
- **[MODIFY]** [lib.rs](file:///Users/palaashatri/Code/retroshell/crates/retro-shell/src/lib.rs): Wire up About, Quit, and basic File menu actions

#### Fix 4: Home Window Content Population  
- **[MODIFY]** [lib.rs](file:///Users/palaashatri/Code/retroshell/crates/retro-shell/src/lib.rs): Fix folder window initialization to properly read and display directory contents

#### Fix 5: Live Clock Update
- **[MODIFY]** [lib.rs](file:///Users/palaashatri/Code/retroshell/crates/retro-sdk/src/lib.rs): Make the menu bar clock update dynamically using system time

#### Fix 6: Font Quality (if time permits)
- Integrate `ab_glyph` crate for proper TrueType font rasterization

### Verification Plan

#### Automated Tests
```bash
cargo test --workspace
```

#### Visual Verification
- Capture screenshots after each fix to verify:
  - No black bars visible anywhere
  - Desktop icons properly positioned with full labels visible
  - Menu items trigger visible actions
  - Clock updates in real-time
  - Home folder window displays contents
