# /goal: Complete SLOPOS-I as a sovereign, classic-Mac-inspired, modern daily-driver desktop environment

You are working inside the current SLOPOS-I repository after two independent implementation passes:

1. a compositor/session correction pass that introduced `slopos-session`, private compositor socket routing, safer session startup, compositor-owned window interaction paths, cursor work, and initial rendering fixes; and
2. an ongoing OpenCode implementation pass for SLOPOS Vision that may contain uncommitted or partially completed work.

Treat the current working tree as the source input. Do not assume documentation is accurate. Do not reset, discard, overwrite, or reimplement unknown changes until you inspect them.

This goal is the canonical implementation brief for taking SLOPOS-I from an experimental desktop prototype to a feature-rich, coherent, daily-driver desktop environment with its own compositor, shell, SDK, system applications, settings, window-management policy, Spaces implementation, font platform, and local Vision service.

The product must remain recognizably inspired by the interaction principles of classic Macintosh System 7–Mac OS 9, while using modern architecture and supporting the capabilities users expect from a serious KDE Plasma or GNOME-class desktop.

Do not copy Apple source code, proprietary assets, trademarks, application layouts, or protected visual assets. Preserve SLOPOS-I’s own identity.

---

## 0. Non-negotiable project identity

SLOPOS-I is not a theme running on another desktop environment.

Production architecture:

```text
Linux kernel and hardware services
└── slopos-session
    ├── slopos-compositor
    │   ├── DRM/KMS output ownership
    │   ├── rendering and presentation
    │   ├── cursor and input routing
    │   ├── window management
    │   ├── SLOPOS Spaces
    │   ├── clipboard and drag-and-drop
    │   ├── XWayland compatibility
    │   └── shell/application surfaces
    ├── slopos-shell
    ├── SLOPOS system services
    └── SLOPOS and third-party applications
```

Nested development architecture:

```text
Host compositor
└── one slopos-compositor nested output window
    ├── slopos-shell
    ├── Finder
    ├── Settings
    ├── native applications
    └── third-party Wayland/XWayland applications
```

The host compositor must not manage individual SLOPOS application windows.

Do not introduce labwc, Sway, KWin, Mutter, wlroots, GTK, Qt, Electron, Slint, or another compositor/toolkit as the production owner of the SLOPOS desktop.

Smithay may remain the compositor framework. Linux, Mesa, GPU drivers, libinput, PipeWire, NetworkManager, BlueZ, systemd or equivalent system services remain third-party substrate where needed.

All newly authored first-party SLOPOS code must be:

- Rust or assembly;
- MIT-licensed;
- attributed to Palaash Atri;
- designed so that third-party dependency licenses remain separately documented.

Do not claim that all distributed material is MIT-licensed unless dependencies, fonts, icons, sounds, codecs, and model weights actually use MIT. The correct project statement is:

> First-party SLOPOS-I code is MIT-licensed and owned by Palaash Atri. Third-party libraries, fonts, assets, codecs, and model weights retain their respective licenses and notices.

---

## 1. Execution protocol: inspect first, preserve work, then implement

Before changing code:

1. Run and record:
   - `git status --short`
   - `git diff --stat`
   - `git diff`
   - `git log -n 10 --oneline --decorate`
2. Create a local safety checkpoint containing every uncommitted file.
   - Prefer a temporary Git branch and checkpoint commit.
   - If committing is inappropriate, create a timestamped patch and untracked-file archive.
3. Identify all OpenCode-authored SLOPOS Vision files and changes.
4. Identify all files changed by the prior SLOPOS patch.
5. Do not delete or replace either body of work merely because it is incomplete.
6. Build the current tree before broad refactoring.

Run, in this order:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Where the Linux VM supports it, also run the nested compositor and existing session QA.

If the current tree does not compile, fixing compilation is the first implementation task. Do not hide build failures by excluding crates from the workspace.

Use small checkpoints after each coherent phase. Do not perform an uncontrolled repository-wide rewrite.

Do not stop after writing plans, matrices, audits, or walkthroughs. Documentation must describe code that exists.

---

## 2. Current known baseline to verify against the actual tree

The most recent reviewed snapshot contained the following. Verify every item because the current tree may now differ:

- a Rust `crates/slopos-session` supervisor;
- `scripts/start-slopos-i` delegating to that supervisor;
- explicit compositor backends for DRM, nested/X11, and headless modes;
- compositor-owned private Wayland socket publication;
- shell and first-party applications routed to that private socket;
- initial compositor-owned move, resize, focus, minimize, maximize, fullscreen, close, cursor, clipboard, and workspace code;
- a fixed eight-workspace `WorkspaceState` in `slopos-compositor`;
- additional workspace/window state inside `slopos-shell`, creating a likely duplicate-authority risk;
- first-party SDK title-bar hit testing that directly calls `set_maximized()` for the zoom control and title-bar double-click;
- `cosmic-text`, `fontdb`, and `ab_glyph` dependencies, but a renderer that is still substantially character-by-character and not yet a production shaping/glyph-atlas pipeline;
- theme files that refer to `SF Pro Text` and `SF Mono` even though those fonts cannot be assumed to exist or be redistributed by SLOPOS;
- Settings panes for themes and some desktop/display/input choices, but no complete font manager, configurable window-control policy, or full Spaces control panel;
- incomplete App Store, portal, screencast, application-bundle, and system-service paths;
- audit documents that may overstate runtime verification;
- SLOPOS Vision work that may now be present but has not yet been independently audited.

The actual source and executed runtime behavior override this list.

---

## 3. Product design doctrine

SLOPOS-I should combine two layers deliberately.

### Classic Macintosh interaction principles

Preserve and refine:

- a global application menu;
- compact, legible window chrome;
- clear active/inactive window state;
- direct manipulation;
- spatial file browsing where practical;
- predictable keyboard behavior;
- low visual noise;
- visible state instead of hidden gestures only;
- consistent controls across first-party applications;
- strong icon and typography hierarchy;
- fast feedback and restrained animation;
- user ownership of files and local data.

### Modern desktop expectations

Implement:

- Unicode shaping and font fallback;
- user-installable fonts;
- light, dark, classic, modern, high-contrast, and custom appearance profiles;
- accessibility semantics and keyboard navigation;
- fractional scaling and multi-monitor support;
- HDR, VRR, color-management, and presentation timing on supported hardware;
- virtual desktops with an overview and gestures;
- tiling, snapping, fullscreen, and configurable window controls;
- notifications and quick settings;
- audio, network, Bluetooth, power, display, input, and privacy settings;
- secure portals and permission prompts;
- native Wayland and XWayland application support;
- clipboard, drag-and-drop, file associations, screenshots, and screen recording;
- session lock, logout, restart, suspend, and shutdown;
- robust crash recovery and truthful diagnostics;
- local-first Vision and AI features;
- packaging and update mechanisms that do not corrupt installations.

SLOPOS-I must not look like elementary OS, GNOME, Windows 11, or modern macOS with a retro texture applied. Modern capabilities should fit SLOPOS-I’s own classic visual grammar.

---

# IMPLEMENTATION PROGRAM

Complete the following phases in order. Later phases may be developed in parallel only when they do not mask a broken foundation.

---

## Phase A — Canonical truth, build baseline, and evidence hygiene

### A1. Establish one active source of truth

Use:

```text
docs/SLOPOS-I.md
```

as the canonical living product, architecture, maturity, and roadmap document.

`AGENTS.md` should be a concise contributor/agent contract that points to `docs/SLOPOS-I.md`; it must not duplicate the entire program.

Keep these active supporting files only when they serve a distinct purpose:

```text
README.md
AGENTS.md
PATCH_STATUS.md
LICENSE
COPYRIGHT
THIRD_PARTY_LICENSES.md
MODEL_LICENSES.md              # when model weights are distributed
deny.toml
docs/SLOPOS-I.md               # canonical truth source
docs/QA.md                     # reproducible manual/runtime QA procedure
docs/PROTOCOLS.md              # versioned internal protocol registry, if needed
docs/FILE_FORMATS.md           # app bundle/theme/model formats, if needed
```

Before removing old Markdown files:

1. ingest any unique, still-correct information;
2. move obsolete plans, matrices, and audits into a timestamped `docs/archive/` directory;
3. retain raw runtime evidence under `docs/qa/evidence/` only when the underlying artifacts exist;
4. delete duplicate generated matrices only after archival and reconciliation;
5. never delete license notices, model licenses, or evidence required to explain prior claims.

Do not generate new “VERIFIED” matrices from source inspection or unit tests.

### A2. Evidence levels

Use only these labels:

- `DESIGNED`: specification exists;
- `IMPLEMENTED`: code path exists and compiles;
- `UNIT-TESTED`: deterministic unit tests pass;
- `INTEGRATION-TESTED`: automated components communicate successfully;
- `RUNTIME-VERIFIED`: exercised in a real nested or DRM session with captured evidence;
- `HARDWARE-VERIFIED`: exercised on suitable physical hardware.

Never promote a feature automatically because a symbol or test exists.

### A3. Replace fake harness behavior

The QA harness must:

- launch real processes;
- retain PIDs and logs;
- inspect actual sockets;
- distinguish installed, launched, mapped, interactive, and exited states;
- leave fields `UNTESTED` unless a real driver or human action produced evidence;
- never hardcode PASS results;
- never fabricate process trees, surface IDs, geometry deltas, screenshots, or CPU samples.

---

## Phase B — Compositor and session correctness

This phase blocks all feature claims.

### B1. Session supervisor

Finish `slopos-session` as the sole production supervisor.

Required behavior:

- launch the compositor first;
- wait for a securely published private client socket;
- launch the shell and session services as sibling children of the supervisor;
- launch applications through a structured application-launch service, not through shell-owned ad hoc process spawning;
- propagate environment variables explicitly;
- ensure only the nested backend receives the host display variables;
- terminate the complete session when the compositor exits;
- restart noncritical services according to policy;
- perform graceful shutdown followed by bounded forced shutdown;
- clean only sockets/readiness files proven to be SLOPOS-owned;
- never glob-delete `$XDG_RUNTIME_DIR/wayland-*`;
- collect exit status and structured logs;
- support `--backend drm|nested|headless` without silent fallback.

Expected process relationship:

```text
slopos-session
├── slopos-compositor
├── slopos-shell
├── slopos-settings-service
├── slopos-notification-service
├── slopos-visiond          # lazy or activatable
└── launched applications
```

The shell must not be the compositor’s process supervisor.

### B2. One authoritative window model

`slopos-compositor` must be the authority for:

- mapped toplevels;
- geometry;
- stacking;
- focus;
- activation;
- minimized state;
- zoom/fill/maximized state;
- fullscreen state;
- tiled state;
- output assignment;
- Space assignment;
- move/resize grabs;
- decoration policy;
- window rules.

Remove or isolate any duplicate shell-owned model of normal application windows. Shell-internal overlays and desktop UI may remain shell state, but normal application toplevels may not be painted as fake windows inside the desktop layer.

### B3. Input and interaction

Complete and runtime-test:

- visible default cursor;
- client cursor surfaces and hotspots;
- pointer focus and enter/leave;
- button serial handling;
- title-bar drag;
- edge and corner resize for all eight directions;
- touchpad and touch events where supported;
- keyboard focus;
- key repeat;
- modifier tracking;
- pointer constraints and relative pointer protocols;
- tablet protocol later if hardware is unavailable now;
- focus-follows-click as the default, with optional user policies only after correctness.

Interactive move and resize must be real pointer grabs. Direct mutation from a test helper does not count as verification.

### B4. Rendering loop

Remove all idle busy loops.

Required:

- event-driven dispatch;
- frame callbacks;
- damage accumulation;
- old/new geometry damage on move and resize;
- no full-screen repaint while idle;
- no unconditional `request_redraw()` loops;
- no solid placeholder rendering once a client has committed a usable buffer;
- frame timing statistics;
- no unbounded allocations per frame;
- idle CPU target near zero on GPU rendering and low single digits under LLVMpipe after the screen is static.

The exact threshold depends on the VM, but 100% CPU for an idle shell is a failure.

### B5. Surface protocols

Complete or validate:

- `xdg-shell`;
- `wlr-layer-shell`;
- `xdg-decoration`;
- output management;
- fractional scale;
- viewporter;
- presentation time;
- idle inhibit;
- activation;
- relative pointer and pointer constraints;
- text input/input method;
- primary selection and data device;
- screencopy/screenshot path;
- foreign toplevel listing/management or a SLOPOS equivalent;
- XWayland rootless window management.

Use standard Wayland protocols where they satisfy the requirement. Use versioned SLOPOS-private protocols only for genuinely SLOPOS-specific behavior.

---

## Phase C — Configurable window behavior and the SLOPOS zoom control

The current first-party SDK must not hardcode the title-bar zoom control or title-bar double-click to `set_maximized()`.

### C1. Window presentation state machine

Implement an authoritative compositor state model similar to:

```rust
pub enum WindowPresentationState {
    Normal,
    Minimized,
    SmartZoomed,
    Filled,
    Fullscreen,
    Tiled(TilePlacement),
}

pub struct WindowRestoreState {
    pub normal_geometry: LogicalRect,
    pub output_id: OutputId,
    pub space_id: SpaceId,
    pub previous_state: WindowPresentationState,
}
```

Rules:

- preserve normal geometry before any presentation transition;
- restore to the correct output and Space;
- handle output removal;
- respect minimum/maximum sizes and resize increments;
- avoid overlapping exclusive shell regions for Fill;
- fullscreen uses the entire output according to shell/fullscreen policy;
- tiled windows retain their tile group and restore geometry;
- minimize never destroys the client surface;
- closing and minimizing are separate operations.

### C2. User-configurable zoom/green-control policy

Add Settings → Desktop & Dock → Windows.

The user chooses the primary action for the SLOPOS zoom/green control:

- **Smart Zoom**: toggle between the user frame and an application-preferred useful size;
- **Fill**: fill the compositor work area while leaving menu/Dock reservations visible;
- **Full Screen**: occupy the output and optionally enter a dedicated Space;
- **Show Layout Menu**: clicking opens the window arrangement menu;
- **None**: no single-click action.

Also allow:

- alternate Option-click action;
- title-bar double-click action:
  - Smart Zoom;
  - Fill;
  - Minimize;
  - Full Screen;
  - None;
- per-application overrides;
- whether hovering, long-pressing, or right-clicking the control opens the arrangement menu;
- whether dragging to screen edges triggers Fill or tiling;
- whether moving a tiled window detaches it immediately or after a threshold.

Suggested settings keys:

```toml
[windows]
zoom_button_action = "smart_zoom"
zoom_button_alternate_action = "fill"
titlebar_double_click_action = "smart_zoom"
show_layout_menu_on_hover = true
edge_tiling = true
edge_fill = true
restore_last_geometry = true
```

Do not make first-party clients read the setting and independently decide geometry. The compositor owns policy.

### C3. Smart Zoom protocol

First-party SLOPOS applications may provide a preferred useful size or rectangle.

Examples:

- Finder: complete icon rows/columns without pointless empty area;
- TextEdit: preferred document/page width;
- Terminal: integral character-cell dimensions;
- Preview: image fit up to work-area bounds;
- Settings: size appropriate for the active pane.

Implement a small versioned SLOPOS protocol or approved IPC method for preferred zoom geometry. Validate client values. Clamp them to output work area and constraints.

For third-party applications that do not provide a preferred size, Smart Zoom falls back to Fill.

Do not expose raw compositor memory or trust arbitrary geometry blindly.

### C4. Arrangement menu

The zoom/green control arrangement menu should include only actions valid for the current output and window:

- Smart Zoom / Restore;
- Fill / Restore;
- Full Screen / Exit Full Screen;
- Tile Left;
- Tile Right;
- quarters when the output is large enough;
- move to another display;
- move to another SLOPOS Space;
- optionally pair with another window.

Keep the visual treatment SLOPOS-native and compact. Do not clone modern macOS artwork.

---

## Phase D — SLOPOS Spaces

Rename and evolve the existing virtual-workspace concept into **SLOPOS Spaces**.

Spaces are compositor-owned. The shell provides UI and commands, but must not maintain a divergent list of application-window assignments.

### D1. Dynamic Space model

Replace the fixed, anonymous eight-workspace assumption with a dynamic model.

Requirements:

- stable `SpaceId` values;
- user-configurable count, default chosen by product policy;
- configurable maximum, initially up to 16;
- create, remove, reorder, rename, and duplicate Space configuration;
- preserve at least one Space;
- persistent ordering and metadata;
- window assignment to exactly one Space by default;
- optional “All Spaces” windows for approved use cases;
- move active window to previous/next/specific Space;
- switch and follow the moved window as a separate command;
- wrap or stop at ends according to user setting;
- state survives shell restart;
- session restore remembers application/Space placement where policy permits.

Suggested model:

```rust
pub struct Space {
    pub id: SpaceId,
    pub name: String,
    pub order: u32,
    pub wallpaper: Option<WallpaperId>,
    pub kind: SpaceKind,
}

pub enum SpaceKind {
    Desktop,
    DedicatedFullscreen { owner: WindowId },
}
```

### D2. Multi-monitor policy in the user’s hands

Provide both modes:

- **Unified Spaces**: switching changes the desktop set across all displays together;
- **Separate Spaces per Display**: every output has an independent active Space.

Also expose:

- remember Spaces per monitor identity;
- choose whether newly connected displays clone or create fresh Space sets;
- move windows across displays and Spaces;
- safe behavior when a display disappears;
- whether full-screen applications get a dedicated Space;
- whether the menu bar and Dock appear on one display or follow focus.

Do not hardcode a macOS policy. The user decides.

### D3. Spaces Overview

Implement a SLOPOS-native overview comparable in utility to Mission Control without copying its visuals.

Entry methods, all configurable:

- keyboard shortcut;
- hot corner;
- trackpad gesture;
- Dock/menu command;
- dragging a window to a screen edge or top activation region.

Overview behavior:

- show every Space in order;
- show real live or recently captured window thumbnails;
- show all windows on the active Space;
- optionally show grouped-by-application view;
- add/remove/reorder/rename Spaces;
- drag windows between Spaces;
- drag windows to another display;
- create a tiled pair by dropping appropriately;
- enter a window or Space by click/keyboard;
- close windows only through an explicit affordance;
- support keyboard-only navigation and accessibility;
- respect reduced-motion preference.

Do not render fake thumbnails. If live thumbnails are not implemented, clearly classify the overview as incomplete rather than substituting colored rectangles and declaring success.

### D4. App-to-Space assignment

From Dock/application menus, support:

- All Spaces;
- Current Space;
- selected named Space;
- None / automatic placement.

Expose a setting equivalent in function to:

- when activating an application, switch to a Space containing one of its windows;
- or remain on the current Space and indicate the remote window;
- or move/open a new window in the current Space where the application supports it.

The user chooses this behavior.

### D5. Gestures and animations

Support configurable gestures through the compositor input stack:

- three/four-finger horizontal swipe to switch Spaces;
- upward swipe to open Spaces Overview;
- downward swipe to close it or reveal application windows, if enabled.

Provide:

- animation speed setting;
- reduced motion;
- disable gestures;
- invert direction;
- keyboard equivalents for every gesture.

Animations must be frame-timed and interruptible, not blocking sleeps.

---

## Phase E — Production font and text platform

User-installable and user-selectable fonts are explicitly part of SLOPOS-I.

This is consistent with the Macintosh tradition of user-managed fonts while modernizing it with safe validation, per-user installation, font roles, Unicode shaping, and live settings.

SLOPOS-I must not ship Apple’s San Francisco fonts unless their license explicitly permits redistribution for this product. Remove assumptions that `SF Pro Text` or `SF Mono` exist. If a user legally installs them, SLOPOS may discover and allow selection like any other font.

### E1. Shared font service/library

Create or finish a shared Rust font platform, for example:

```text
crates/slopos-fonts
```

Responsibilities:

- scan system and per-user font directories;
- parse TTF, OTF, TTC, and supported variable-font data;
- provide family/style/weight/stretch metadata;
- perform font matching and fallback;
- maintain a cache with invalidation;
- expose font roles to shell and applications;
- validate install candidates;
- detect duplicates and conflicts;
- activate/deactivate user fonts;
- emit change notifications;
- remain independent of Wayland and UI.

Use pure-Rust parsing and discovery where practical. Do not require fontconfig as the only implementation path. Platform adapters may use system facilities only when the portable core remains usable.

Search at least:

```text
$XDG_DATA_HOME/fonts/
$XDG_DATA_HOME/slopos-i/fonts/
$XDG_DATA_DIRS/fonts/
/usr/local/share/fonts/
/usr/share/fonts/
```

Add appropriate FreeBSD paths through configuration, not Linux-only hardcoding.

### E2. Font Manager

Implement either a dedicated first-party `SLOPOS Fonts` application or a full Settings → Fonts pane.

Required functions:

- preview family and styles;
- preview glyph repertoire;
- preview user-entered sample text;
- install for current user;
- request privileged all-users installation through an approved helper/polkit path;
- validate before installation;
- warn about malformed fonts and duplicates;
- activate/deactivate nonessential fonts;
- remove user-installed fonts;
- create collections/favorites;
- inspect license/name/version metadata;
- reveal font file in Finder;
- restore SLOPOS default font profile;
- never deactivate fonts required to draw the recovery UI.

Finder should open font files in the font preview/installer rather than blindly copying them.

### E3. Appearance font profiles

Expose selectable profiles:

- **Classic**: compact retro-oriented UI typography using a legally redistributable classic-style font or user-selected equivalent;
- **Modern**: clean contemporary system sans/mono using a permissively licensed bundled or system-available family;
- **Accessible**: readability-first family, larger defaults, stronger contrast;
- **Custom**: user-selected font roles.

Do not hardcode one font across every role.

Required roles:

```text
system_ui
menu
window_title
body
small
monospace
document_default
```

Settings must allow family, size, and where sensible weight for each role, with safe bounds and a reset button.

Theme files should reference semantic font roles and fallback stacks rather than proprietary family names.

### E4. Text shaping and rendering

Make `cosmic-text` or an equivalent pure-Rust shaping stack the authoritative layout engine.

Required:

- Unicode;
- grapheme clusters;
- script shaping;
- kerning;
- bidirectional layout;
- fallback fonts;
- emoji fallback where available;
- line breaking;
- text measurement;
- selection and caret geometry;
- IME composition;
- fractional scaling;
- high-DPI rendering;
- consistent metrics across shell and applications.

Replace character-by-character layout and one-rectangle-per-glyph-pixel production rendering.

Implement:

- glyph atlas or suitable cached text render path;
- subpixel or grayscale antialiasing according to renderer/output policy;
- physical-pixel rasterization at the actual scale;
- atlas invalidation on font/scale changes;
- measured-width ellipsis;
- no fixed character width except in verified monospaced contexts;
- stable baseline alignment;
- text rendering tests at 1.0, 1.25, 1.5, 1.75, and 2.0 scale.

The system must still have a tiny embedded recovery font for catastrophic font-database failure, but that is not the normal UI renderer.

### E5. Live propagation

Changing a system font or profile must:

- update shell surfaces;
- update first-party applications;
- invalidate text/layout caches;
- relayout windows safely;
- preserve user data and focus;
- not require logout unless technically unavoidable.

Use a versioned settings notification mechanism. Do not make every process poll a settings file continuously.

---

## Phase F — Appearance, themes, icons, and accessibility

### F1. Appearance profiles

Support:

- Classic light;
- Classic graphite;
- dark;
- high contrast;
- modern light/dark profiles;
- user themes with validated manifests;
- accent color;
- reduced transparency;
- reduced motion;
- UI density;
- font profile;
- icon size;
- cursor theme and size;
- sound theme.

Keep classic Mac inspiration in spacing, hierarchy, controls, and interaction—not in unlicensed copied assets.

### F2. Theme architecture

Create one authoritative theme state and broadcast it to clients.

Remove split-brain state between:

- shell ThemeManager;
- SDK global atomics;
- environment variables;
- per-window copies;
- Settings configuration.

Theme packages must be versioned and validated. Unknown fields should be handled safely.

### F3. Accessibility

Implement a real accessibility path for custom-rendered controls.

Minimum:

- semantic tree;
- roles, names, values, state, and actions;
- focus events;
- keyboard-only navigation;
- screen-reader bridge where available;
- high contrast;
- reduced motion;
- large text and cursor;
- sticky/slow/bounce keys where platform support permits;
- focus indicator that does not rely on color alone;
- captions/visual alternatives for system sounds.

Canvas-rendered applications must not appear as one inaccessible bitmap.

---

## Phase G — Shell and Finder completion

### G1. Shell

Complete:

- global menu server;
- Dock with running/minimized/attention states;
- application switching;
- window switching within an application;
- SLOPOS Spaces Overview;
- notifications;
- control/quick-settings center;
- clock/calendar;
- audio/network/Bluetooth/power indicators;
- Spotlight-style search;
- desktop icons and wallpaper per Space;
- hot corners;
- screen lock and session power menu;
- keyboard shortcut editor;
- first-run onboarding and recovery mode.

The shell must remain a client of the compositor, not a second window manager.

### G2. Finder

Make Finder a serious file manager while retaining spatial/classic influence.

Required:

- icon, list, and column views;
- robust measured layout at every scale;
- navigation/history;
- sidebar and mounted volumes;
- create, rename, duplicate, copy, move, trash, restore, delete;
- conflict resolution;
- progress and cancellation;
- drag-and-drop;
- file associations and Open With;
- application bundles;
- previews/thumbnails;
- search and metadata;
- permissions and properties;
- removable media;
- network locations later through adapters;
- no shell-command construction from untrusted paths;
- no synchronous long file operation on the UI thread.

Replace heuristic desktop detection and fixed label truncation with explicit layout modes and measured text.

### G3. Application bundles and Launch Services

Implement a real on-disk SLOPOS app bundle/manifest format.

Required:

- identifier;
- display name;
- executable;
- icon;
- version;
- MIME types;
- URL schemes where supported;
- permissions/portal declarations;
- supported architectures;
- signature/integrity metadata;
- localization metadata;
- uninstall data.

Scan configured application directories instead of hardcoding built-ins.

Use structured process spawning and stable application IDs.

---

## Phase H — Settings and system-service backends

Settings must show actual live state and apply changes through typed service interfaces.

Implement or complete:

- Appearance and Fonts;
- Desktop, Dock, Windows, and SLOPOS Spaces;
- Displays, scaling, arrangement, HDR, VRR, color profile, refresh rate;
- Wallpaper per Space/display;
- Sound input/output and volume;
- Network;
- Bluetooth;
- Keyboard, shortcuts, repeat, layout, input methods;
- Mouse and touchpad;
- Accessibility;
- Notifications;
- Privacy and permissions;
- Users/session where feasible;
- Date/time/locale;
- Power and battery;
- Storage and removable media;
- application defaults;
- SLOPOS Vision status and model packs;
- system information and diagnostics.

Do not represent settings as functional merely because a TOML value changes. A setting is live only when the owning service applies it and reports the resulting state.

Use adapters for system services. Keep policy and data types in first-party Rust crates. Best-effort CLI shell-outs may remain temporary fallbacks only when clearly classified and safely invoked.

---

## Phase I — Modern display and graphics stack

### I1. Outputs

Complete:

- multi-output discovery;
- hotplug;
- mode selection;
- layout and orientation;
- fractional scaling;
- per-output scale;
- transform;
- primary/focused output policy;
- safe rollback after an unusable display configuration;
- EDID identity and persistence;
- nested and DRM parity where possible.

### I2. VRR

Implement compositor-owned VRR policy:

- Off;
- Automatic;
- Fullscreen only;
- Always when supported.

Verify connector/property support, frame scheduling, and fallback behavior on real hardware. Do not claim VRR from configuration parsing alone.

### I3. HDR and color management

Complete the path beyond setting HDR metadata:

- output capability discovery;
- color-management protocol support;
- content color spaces;
- transfer functions;
- compositor working space;
- tone mapping;
- SDR content in HDR output;
- HDR content on SDR output;
- ICC profile metadata where supported;
- user controls and safe fallback;
- screenshots with defined color behavior.

HDR is hardware-verified only after testing on a suitable GPU, connector, display, and driver stack.

### I4. Presentation performance

Implement where Smithay/backend support permits:

- direct scanout;
- overlay/hardware cursor planes;
- explicit synchronization where required;
- tearing policy for games;
- presentation feedback;
- frame pacing;
- occlusion handling;
- minimized/inactive Space throttling.

---

## Phase J — Third-party application compatibility

A daily-driver SLOPOS session must run ordinary applications.

Required test classes:

- native Wayland toolkit clients;
- Electron/Chromium Wayland clients;
- Firefox;
- media players;
- terminal applications;
- games;
- X11 applications through XWayland;
- applications using portals;
- applications with client-side decorations;
- applications requesting server-side decorations.

Implement:

- XWayland rootless window mapping, focus, move/resize, clipboard, drag-and-drop, fullscreen, and Spaces assignment;
- xdg-desktop-portal backend integration for file chooser, screenshots, screencast, open URI, notifications, settings, inhibit, and related interfaces;
- PipeWire-based screencast path where appropriate;
- MIME/application defaults;
- activation tokens;
- app IDs and startup notification;
- tray/status-notifier compatibility only if it fits SLOPOS policy.

Do not promise that arbitrary client-side-decorated applications can be visually restyled into classic SLOPOS windows. First-party applications should be fully native; third-party applications must be functionally correct.

---

## Phase K — System applications

Finish existing applications before multiplying placeholders.

### Settings

Must use real backends and include Windows, Spaces, and Fonts.

### TextEdit

Complete:

- new/open/save/save-as;
- encoding handling;
- undo/redo;
- find/replace;
- selection/caret/IME;
- font choice;
- rich text only when a real document model exists;
- crash-safe save.

### Terminal

Complete:

- PTY lifecycle;
- resize;
- Unicode;
- selection/clipboard;
- scrollback;
- colors/font settings;
- shell exit behavior;
- safe command handling.

### App Store / software manager

Do not call it complete until it has:

- real catalog source or local repository format;
- transport with verification;
- signed manifests/package trust policy;
- atomic install/update/rollback;
- uninstall;
- permissions display;
- dependency handling policy;
- failure recovery;
- no removal of the current valid app before replacement is verified;
- symlink/hardlink/path traversal defense;
- no synchronous fake async operations.

A local-first package source is acceptable. Network functionality must be explicit and secure.

### Preview

Add a lightweight SLOPOS-native Preview application for images and supported documents. It should be the primary system client for SLOPOS Vision OCR and Lift Subject.

---

## Phase L — SLOPOS Vision audit and integration

OpenCode may have added Vision crates and applications. Audit rather than blindly replacing them.

SLOPOS Vision is the OS-facing form of the broader Loom Vision concept, but Loom remains a separate project.

The intended long-term boundary is:

```text
portable pure-Rust vision engine
├── SLOPOS Vision daemon/client/system integrations
└── Loom-specific creative adapters in the separate Loom project
```

Do not make SLOPOS depend on Loom or move the Loom suite into this workspace.

### L1. Audit current Vision work

Inspect for:

- added workspace crates;
- model loading;
- OCR implementation;
- subject segmentation;
- daemon/client protocol;
- Finder/Preview integration;
- model manifests and hashes;
- model/license files;
- network dependencies;
- placeholders, `todo!()`, fake output, and hardcoded results;
- UI-thread blocking;
- unsafe path handling;
- missing cancellation/resource limits;
- FreeBSD portability of pure crates.

Preserve correct work. Fix broken work incrementally.

### L2. Required Vision architecture

Prefer:

```text
slopos-vision             # portable pure-Rust inference/image-processing core
slopos-vision-protocol    # typed request/result/job structures
slopos-vision-client      # reusable application client
slopos-visiond            # session daemon, lazy model loading
apps/preview              # primary system UI
Finder integration
```

Base implementation requirements:

- local inference only;
- no image upload;
- no telemetry;
- no silent model download;
- no Python/C++ subprocess;
- pure-Rust baseline using appropriately licensed runtimes;
- model weights independently license-audited;
- lazy loading;
- bounded job queue;
- cancellation;
- image dimension limits;
- model hash verification;
- restrictive cache permissions;
- no arbitrary daemon writes to caller-selected paths;
- honest capability reporting when model packs are missing.

### L3. Functional acceptance

Do not mark OCR complete until it recognizes real fixture text and returns measured source coordinates.

Do not mark Lift Subject complete until it produces a real source-resolution alpha mask and transparent PNG from inference.

Do not mark Finder/Preview integration complete until the UI remains responsive and the operation can be cancelled.

Vision must not block compositor, shell, or UI event loops.

---

## Phase M — Security, privacy, reliability, and recovery

Implement:

- safe path handling;
- atomic writes;
- secure temporary files;
- bounded decompression and image decoding;
- permission prompts through portals/polkit;
- lock screen that protects session content;
- secrets/keyring integration through a first-party adapter;
- crash recovery for shell/session settings;
- safe-mode startup with embedded recovery font/theme;
- watchdog/restart policy for noncritical services;
- no world-writable control sockets;
- no arbitrary command interpolation;
- dependency and license scanning;
- structured logs with sensitive-data redaction;
- user-controlled diagnostics export.

Do not claim sandboxing until processes are actually isolated and portal access is enforced.

---

## Phase N — Packaging and installation

Support Ubuntu Server and Arch Linux as initial substrates without making either distribution’s desktop environment a dependency.

Required:

- reproducible build instructions;
- session desktop files;
- system/user service definitions where appropriate;
- package manifests;
- dependency list;
- default themes/fonts/assets with verified licenses;
- model packs separate from core where appropriate;
- uninstall path;
- upgrade/migration path;
- debug symbols/package option;
- recovery session entry;
- no hardcoded developer home paths.

Keep pure crates portable enough for FreeBSD where that is already a stated constraint, especially SLOPOS Vision and document/image utilities. Do not claim full FreeBSD compositor/runtime support unless actually implemented and tested.

---

# QA AND ACCEPTANCE

## 4. Automated test requirements

Add deterministic tests for:

- session environment separation;
- private socket validation;
- safe cleanup;
- compositor window state transitions;
- restore geometry;
- move/resize grab state;
- work-area calculation;
- dynamic Space creation/removal/reordering;
- window movement between Spaces;
- unified vs per-display Space policies;
- green/zoom-button settings parsing and behavior;
- title-bar double-click policy;
- font discovery/matching/fallback;
- font installation validation and duplicate detection;
- text shaping, measurement, bidi, fallback, ellipsis;
- theme/font live updates;
- app bundle parsing and launch safety;
- clipboard/drag-and-drop MIME handling;
- portal request validation;
- Vision manifest, hash, job, cancellation, cache, and image limits;
- atomic install/save/update paths.

Unit tests must not substitute for runtime interaction evidence.

## 5. Nested runtime acceptance

A valid nested QA run must show:

1. the host sees one SLOPOS compositor output window;
2. shell and applications connect to the private socket;
3. a visible cursor works over shell and application surfaces;
4. real mouse-driven move and eight-direction resize work;
5. minimize, restore, Smart Zoom, Fill, Full Screen, tiling, and restore work;
6. changing the zoom-button policy changes behavior without app-specific hacks;
7. SLOPOS Spaces can be created, renamed, reordered, switched, and removed;
8. windows can be dragged between Spaces in the overview;
9. unified and per-display policies behave correctly in a multi-output nested test;
10. fonts can be installed for the current user and selected for a UI role;
11. Unicode, fallback, and fractional-scale text remain aligned;
12. Finder performs real file operations with progress/cancellation;
13. a native Wayland third-party application works;
14. an XWayland application works;
15. clipboard and drag-and-drop work across first- and third-party apps;
16. Settings reports and applies live state;
17. idle CPU settles after animations stop;
18. closing the compositor terminates the complete session;
19. stale cleanup never unlinks the host compositor socket;
20. no audit field is marked PASS without corresponding evidence.

## 6. Hardware acceptance

On suitable physical hardware, separately verify:

- DRM/KMS session startup;
- multi-monitor hotplug;
- fractional scaling;
- hardware cursor;
- direct scanout;
- VRR;
- HDR and SDR interoperability;
- color-management behavior;
- suspend/resume;
- input devices;
- audio/network/Bluetooth/power adapters.

Hardware-only features remain `IMPLEMENTED` or `RUNTIME-VERIFIED` until hardware evidence exists.

## 7. Required evidence format

For each runtime test, capture:

- exact command;
- commit hash and dirty status;
- backend and environment;
- process tree;
- socket ownership;
- client environment;
- mapped surface/application IDs;
- before/after compositor state;
- screenshot or recording where interaction matters;
- CPU/memory sample duration;
- output artifact and hash;
- failure logs;
- honest classification.

Do not fabricate evidence documents from expected behavior.

---

# PRIORITY ORDER WHEN TOKEN OR TIME BUDGET IS LIMITED

Continue implementation in this exact order:

1. make the workspace compile and tests run;
2. preserve and audit OpenCode Vision changes;
3. finish session/compositor correctness and remove duplicate window authority;
4. fix cursor, real pointer move/resize, frame loop, and idle CPU;
5. implement the compositor window state machine;
6. implement configurable zoom/green-control behavior;
7. implement compositor-owned dynamic SLOPOS Spaces core;
8. implement Settings panes for Windows and Spaces;
9. implement Spaces Overview and multi-monitor policy;
10. implement production text shaping/font discovery;
11. implement user font installation and font-role settings;
12. finish Finder and app-bundle/launch services;
13. complete portals, XWayland, clipboard, drag-and-drop, and third-party compatibility;
14. complete display/HDR/VRR/color paths;
15. finish system applications and Vision UI integration;
16. packaging, recovery, accessibility, and performance hardening.

Do not spend the remaining budget on broad documentation while a higher-priority code defect remains.

If the execution budget ends, leave the repository buildable and create a concise `PATCH_STATUS.md` containing:

- exact completed changes;
- exact compile/test results;
- exact runtime evidence produced;
- known regressions;
- next source file/function to edit;
- no speculative PASS claims.

---

# DEFINITION OF COMPLETE

SLOPOS-I is complete for this goal only when it is a usable, sovereign desktop session rather than a themed prototype.

At minimum:

- `slopos-session` owns lifecycle;
- `slopos-compositor` owns all visible SLOPOS windows and outputs;
- window movement, resize, focus, minimize, restore, Smart Zoom, Fill, Full Screen, tiling, and close work;
- the user controls zoom-button and title-bar behavior;
- SLOPOS Spaces provides dynamic desktops, overview, window movement, gestures, app assignment, and user-selectable multi-monitor policy;
- user fonts can be installed, validated, activated, selected, and rendered correctly;
- no proprietary Apple font is assumed or redistributed without permission;
- text shaping and fallback are production quality;
- shell, Finder, Settings, Terminal, TextEdit, Preview, and software management have real workflows;
- native Wayland and XWayland applications are functional;
- portals, clipboard, drag-and-drop, notifications, screenshots, and screencast paths are real;
- multi-monitor, scaling, input, accessibility, and session controls work;
- HDR/VRR claims are limited to verified support;
- SLOPOS Vision remains local-first and uses real inference rather than placeholders;
- first-party code remains Rust/assembly and MIT-licensed;
- documentation has one canonical truth source;
- tests and evidence reflect actual execution.

Do not mark the goal complete because folders, enums, menu items, settings keys, tests, or documents exist. Completion requires human-usable end-to-end behavior inside the SLOPOS-owned compositor session.
