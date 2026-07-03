# RetroShell: Honest Self-Review & Road to 10/10

> [!NOTE]
> 2026-07-03 verification update: VM visual verification now uses the rebuilt `retroshell-vm` image with `xrandr`/`wlr-randr` present, 1280x800 output fills the frame without black bars, and internal Finder minimize collapses/restores as a real shell state. Compositor-level HDR, VRR, exclusive fullscreen, universal external-app global menus, and Doom showcase video remain future architecture work.
>
> 2026-07-03 Settings update: Settings now has functional category panes across General, Appearance, Desktop & Dock, Display, Sound, Network, Keyboard, Mouse, Accessibility, Privacy & Security, and Notifications. Settings persist to `settings.conf`; HDR/VRR are exposed as honest session preferences, not compositor-level output control.
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

## Executive Summary

**Current Score: 2.5 / 10** as a production-grade desktop environment competing with KDE/GNOME.

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

### What Is Stubbed / Fake / Broken

> [!WARNING]
> The majority of "features" are empty structs with no real implementation.

| Component | File | Reality |
|-----------|------|---------|
| **Clipboard** | `clipboard.rs` (22 lines) | Empty struct, `copy()`/`paste()` are no-ops |
| **Drag & Drop** | `dnd.rs` plus Finder event handling | Finder has internal icon-grid drag-to-folder moves; toolkit/protocol-level DnD remains stubbed |
| **Accessibility** | `accessibility.rs` (97 lines) | Stub with hardcoded data, no AT-SPI integration |
| **Progress Bar** | `progress_bar.rs` (66 lines) | Data struct only, never rendered |
| **Slider** | `slider.rs` (65 lines) | Data struct only, never rendered |
| **Tab View** | `tab_view.rs` (166 lines) | Data struct only, never rendered |
| **Popup Button** | `popup_button.rs` (95 lines) | Data struct only, never rendered |
| **Dialog** | `dialog.rs` (64 lines) | Data struct only, never rendered |
| **Notification Center** | `notification_center.rs` | Empty struct, `post()` does nothing |
| **Dock** | `dock.rs` | Empty struct, no dock rendered |
| **Session Manager** | `session_manager.rs` | Stub — `login()`/`logout()` do nothing |
| **Application Registry** | `application_registry.rs` | Basic HashMap, no persistent state |
| **Theme Manager** | `theme_manager.rs` | Loads a single hardcoded theme |
| **Workspace Manager** | `workspace_manager.rs` | Data struct, no multi-desktop support |
| **Window Manager** | `window_manager.rs` | Simple list tracker, no real compositing |
| **IPC Bus** | `retro-bus` crate | Unix socket transport that nobody connects to |
| **Font Rendering** | `font.rs` (33 lines) | Placeholder struct, actual rendering uses hand-coded 5x7 pixel bitmaps |
| **Shaders** | `shader.rs` (15 lines) | Empty — no GPU shaders, all rendering is CPU rects |
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
| **Clipboard** | wl_data_device protocol | Empty stub |
| **Drag and drop** | wl_data_device DnD protocol | Finder-only internal drag-to-folder moves; no protocol-level DnD |
| **Keyboard shortcuts** | Configurable, system-wide | Only Cmd+Q to quit |
| **Multi-monitor** | Full Wayland output management | Single hardcoded display |
| **Screen locking** | PAM-backed lock screen | None |
| **Notifications** | D-Bus notification daemon (freedesktop spec) | Empty struct |
| **System tray** | StatusNotifierItem protocol | Two static squares |
| **Audio integration** | PipeWire/PulseAudio volume control | PulseAudio installed but no UI control |
| **Network management** | NetworkManager integration | None |
| **File management** | Dolphin with full VFS, thumbnails, preview | Finder reads `readdir()`, no thumbnails |
| **Text editing** | Kate/KWrite with syntax highlighting | TextEdit with basic editing |
| **Terminal** | Konsole with full VT100+ | Terminal with partial VT parser, no PTY |
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

**Rounded: 2.5/10**

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
