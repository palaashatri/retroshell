# AGENTS.md — SLOPOS-I Development Source of Truth

**Authority:** This is the sole normative development document for SLOPOS-I.
It defines the product, architecture, engineering rules, implementation order,
and acceptance criteria. `TRUTH.md` records what is actually implemented and
verified. `README.md` is only the human-facing introduction and quick start.

## Documentation rule

The repository must contain exactly three Markdown files:

1. `README.md` — short project overview, setup, and links.
2. `AGENTS.md` — this development specification and execution contract.
3. `TRUTH.md` — current audit, evidence ledger, defects, and verification log.

Do not create another plan, roadmap, handoff, session summary, audit report,
capability matrix, task brief, or QA Markdown file. Put durable design and work
instructions here. Put factual results and evidence in `TRUTH.md`. Store raw
screenshots, logs, JSON, recordings, traces, and benchmark output under
`artifacts/qa/<date>-<slug>/`.

Legal notices and generated attribution may use `.txt`, TOML, JSON, SPDX, or
other machine-readable formats. They must not become competing project truth.

---

## 1. Project identity

SLOPOS-I is a sovereign, local-first, Linux-first desktop environment written in
Rust and, where justified, assembly. Its shared userland and desktop policy must
remain POSIX-portable so the same desktop can run on Linux and FreeBSD without a
fork. It combines the visual and interaction lineage of classic Macintosh System
7 / Platinum with the architecture and expected capabilities of a modern
KDE/GNOME-class desktop.

The goal is not a theme running on somebody else's desktop. SLOPOS-I owns its
session-facing user experience:

- compositor and window-management policy;
- desktop shell, global menu, Dock, search, notifications, lock screen;
- toolkit, SDK, renderer, text stack, font platform, accessibility;
- Settings, Finder, Terminal, TextEdit, Preview, App Store;
- application bundles, launch services, portals, clipboard and drag-and-drop;
- SLOPOS Spaces;
- SLOPOS Vision;
- packaging and session supervision.

The Linux or FreeBSD substrate, kernel drivers, Mesa, system services, and
permissively licensed Rust dependencies remain third-party components. The
honest ownership statement is:

> First-party SLOPOS-I code and original assets are owned by Palaash Atri and
> licensed under MIT. Third-party libraries, system components, fonts, codecs,
> and model weights retain their own licenses and notices.


### Product generations and release milestones

SLOPOS is one desktop product lineage with kernel support added in generations.
A generation is not permission to rewrite the desktop, abandon compatibility,
or reset already completed functionality.

#### SLOPOS-I — desktop-environment generation

The first release milestone is a complete, sovereign **Linux desktop
environment**. Linux is the Tier-1 reference platform and the compositor's
first 100/100 implementation target.

SLOPOS-I must also establish a real POSIX/Unix platform boundary and bring the
same desktop to FreeBSD. The order is:

1. **SLOPOS-I M1 — Linux desktop:** complete compositor, shell, toolkit,
   applications, session, packaging, accessibility and daily-driver QA on
   Linux. No third-party production compositor.
2. **SLOPOS-I M2 — portable desktop:** shared crates are POSIX-clean, required
   release scripts are POSIX `sh`, Linux-specific services are isolated behind
   platform interfaces, and a native FreeBSD backend builds and runs the same
   desktop experience.

Linux and FreeBSD are operating-system substrates for SLOPOS-I. SLOPOS-I does
not include a custom kernel.

#### SLOPOS-II — custom-kernel generation

The **only generational objective** of SLOPOS-II is to add a first-party custom
Rust kernel as a third supported kernel target. SLOPOS-II is not a UI redesign,
application rewrite, compatibility break, or excuse to regress Linux or FreeBSD.

The SLOPOS-II support matrix is mandatory:

| Kernel target | Required status |
|---|---|
| Linux | Remains fully supported and release-blocking |
| FreeBSD | Remains fully supported and release-blocking |
| SLOPOS kernel | New first-party Rust/assembly kernel and release-blocking target |

The desktop, shell, compositor policy, toolkit, SDK, applications, document
formats, accessibility semantics and user configuration must remain shared.
Kernel-specific code belongs behind platform and ABI adapters. Do not create
three application trees or three competing desktop implementations.

A kernel alone cannot honestly be called POSIX-compliant. POSIX conformance is
a system property involving kernel behavior, libc/API surfaces, the shell and
utilities. Therefore the SLOPOS-II program includes only the minimum companion
work required to expose and verify a POSIX-conformant system interface for the
custom kernel: processes and threads, virtual memory, filesystems and VFS,
permissions, signals, clocks/timers, pipes, Unix sockets, networking, device and
terminal interfaces, executable loading, system calls, libc bindings, and the
required command/runtime surface.

Use **POSIX-conformant target** until the relevant conformance suites pass. Use
**POSIX certified** only after formal certification has actually been obtained.
No documentation may infer certification from design intent or unit tests.

SLOPOS-II may begin only after SLOPOS-I has a stable Linux desktop, a frozen
portable platform contract, and a non-regression suite capable of running the
same desktop/application tests on Linux and FreeBSD. The custom kernel must then
join that same matrix; it must not replace either existing kernel target.

### Naming

| Kind | Canonical form |
|---|---|
| Product | **SLOPOS-I** |
| Former name | RetroShell, historical only |
| Crates and binaries | `slopos-*` |
| Environment prefix | `SLOPOS_*` |
| User config | `$XDG_CONFIG_HOME/slopos-i` |
| User data | `$XDG_DATA_HOME/slopos-i` |
| User cache | `$XDG_CACHE_HOME/slopos-i` |
| Session entry | `slopos-i.desktop` |
| System menu | **SLOPOS** |

Do not reintroduce `retro-*` names into new APIs, files, environment variables,
or product copy.

---

## 2. Non-negotiable architecture

### Production topology

```text
Display manager / TTY session
└── slopos-session
    ├── slopos-compositor
    │   ├── DRM/KMS output backend
    │   ├── renderer and presentation scheduler
    │   ├── input seats and cursor manager
    │   ├── window manager and SLOPOS Spaces
    │   ├── layer-shell policy and work areas
    │   ├── XWayland bridge
    │   └── private Wayland display socket
    ├── slopos-shell
    ├── first-party SLOPOS applications
    ├── third-party Wayland/XWayland applications
    └── session-scoped services such as slopos-visiond
```

### Nested development topology

```text
Host desktop compositor
└── one slopos-compositor nested output window
    ├── slopos-shell
    ├── Finder
    ├── Settings
    ├── TextEdit
    ├── Terminal
    ├── App Store
    ├── Preview
    └── test clients
```

Only `slopos-compositor` may connect to the host display in nested mode. Every
SLOPOS shell surface and application must connect to the compositor-owned
private socket. The host compositor must see one outer SLOPOS window, not every
inner application.

### Sovereignty rules

- No production fallback to labwc, Sway, KWin, Mutter, or another window manager.
- A host compositor is allowed only as the nested development display backend.
- Never silently fall back to a different backend. `drm`, `nested`, and
  `headless` modes must be explicit and fail clearly.
- Never scan arbitrary `wayland-*` sockets or delete them with a glob.
- Use a unique per-session runtime directory, readiness file, nonce, and exact
  private socket handle.
- The compositor is the sole authority for mapped-window geometry, focus,
  stacking, workspace/Space membership, output assignment, minimize, Fill,
  Zoom, tiling, fullscreen, and restore state.
- The shell paints desktop chrome and shell-only overlays. It must not maintain
  a second fake model of ordinary application windows.
- Applications may request semantic operations. They do not directly mutate
  compositor geometry or move host windows.


### POSIX and operating-system portability contract

POSIX does not specify Wayland, DRM/KMS, desktop composition, window controls,
SLOPOS Spaces, graphical applications or visual design. Do not describe those
GUI features as POSIX features. The enforceable goal is a POSIX-portable shared
userland with explicit operating-system backends.

#### Required architecture

```text
Shared SLOPOS desktop and POSIX/Unix layer
├── compositor policy and Wayland protocol state
├── shell and applications
├── toolkit, SDK, renderer-independent scene policy
├── file/process/IPC abstractions
├── configuration, bundles and document services
├── Vision protocol/client and portable inference core
└── platform traits
    ├── Linux backend
    ├── FreeBSD backend
    └── SLOPOS-kernel backend (SLOPOS-II only)
```

Create or evolve explicit boundaries equivalent to:

```text
crates/slopos-platform
crates/slopos-platform-linux
crates/slopos-platform-freebsd
```

The future SLOPOS-II repository/program adds a SLOPOS-kernel implementation of
the same public platform contract. Names may change during implementation, but
the dependency direction may not: shared desktop crates depend on interfaces,
not Linux, FreeBSD or SLOPOS-kernel implementations.

Portable crates must not directly depend on `/proc`, `/sys`, udev, systemd,
logind, epoll, inotify, signalfd, memfd-specific behavior, Linux credential
structures, Linux DRM ioctls, NetworkManager, PipeWire, or Linux-only command
output. Those facilities are allowed only inside the Linux backend. FreeBSD and
future SLOPOS-kernel facilities receive their own implementations.

General Unix APIs may use `std::os::unix` and carefully reviewed `libc` calls.
Linux-only APIs must be under `cfg(target_os = "linux")` in Linux-owned modules.
FreeBSD-only APIs must be isolated likewise. A broad `cfg(unix)` is not proof
that behavior is portable.

#### Shell and command portability

Every script required to build, install, start, stop, recover, upgrade, package
or test a supported release must use POSIX shell syntax unless it is explicitly
platform-owned:

```sh
#!/bin/sh
set -eu
```

Do not require Bash arrays, `[[ ... ]]`, `${BASH_SOURCE[0]}`, process
substitution, `set -o pipefail`, GNU-only `stat`, GNU-only `sed`, `grep -P`,
`readlink -f`, `timeout`, or `seq` in the portable release path. A Linux-only
developer/QA script may use Bash, but it must be labelled as such and may not be
the sole route to build or operate SLOPOS-I on FreeBSD.

#### Portability gates

CI must grow to include:

- Linux glibc workspace build/test;
- Linux musl portability build where dependencies permit;
- FreeBSD workspace build/test on a native runner or VM;
- POSIX-shell validation under at least `dash` and BusyBox `ash` for portable
  scripts, plus FreeBSD `/bin/sh` when the runner exists;
- a dependency-boundary check that rejects Linux-only imports from portable
  crates;
- shared behavioral tests for filesystem, process, IPC, settings and session
  abstractions;
- identical first-party application tests across Linux and FreeBSD;
- in SLOPOS-II, the same non-regression suite against the SLOPOS kernel.

Do not claim FreeBSD support from `cargo check` alone. Full support requires a
native compositor/session, input, graphics, audio, power, networking, packaging
and application runtime evidence. Do not claim SLOPOS-II kernel support until a
real desktop session and the shared compatibility suite run on that kernel.

---

## 3. Current repository map

The single Cargo workspace currently contains:

### Core crates

| Path | Responsibility |
|---|---|
| `crates/slopos-session` | Session supervisor, readiness, process lifecycle |
| `crates/slopos-compositor` | Smithay compositor, DRM/nested/headless backends, WM |
| `crates/slopos-shell` | Desktop, menu bar, Dock, search, launch services, portals |
| `crates/slopos-render` | GPU/software rendering primitives and text plumbing |
| `crates/slopos-kit` | Widgets, layout, focus, controls, accessibility semantics |
| `crates/slopos-sdk` | First-party application framework and CSD integration |
| `crates/slopos-bus` | SLOPOS IPC abstractions; retain only where real and useful |
| `crates/slopos-fonts` | Font discovery, profiles, installation, role resolution |
| `crates/slopos-vision` | Platform-neutral local OCR and subject segmentation core |
| `crates/slopos-vision-protocol` | Typed Vision IPC protocol |
| `crates/slopos-vision-client` | Reusable Vision daemon client |
| `crates/slopos-visiond` | Session-scoped local Vision service |

### Applications

| Path | Product role |
|---|---|
| `apps/finder` | File manager and application/document entry point |
| `apps/settings` | System configuration frontend |
| `apps/textedit` | Native text editor |
| `apps/terminal` | Terminal emulator |
| `apps/appstore` | SLOPOS `.app` catalog and installer |
| `apps/preview` | Image/document viewer and Vision UI |

Keep the monorepo until stable SDK boundaries make an external app repository
useful. Do not split core crates merely for aesthetic reasons.

---

## 4. Product and visual doctrine

SLOPOS-I should feel descended from classic Macintosh, not like macOS, GNOME,
elementary OS, or a generic retro skin.

### Preserve from classic Macintosh

- clear black-and-white or restrained Platinum hierarchy;
- compact menus and controls;
- strong window borders and direct-manipulation affordances;
- global application menu;
- spatial Finder behavior;
- visible state and predictable commands;
- restrained animation;
- user-installable fonts and system typography choice;
- smart Zoom as an application-aware alternative to blind maximize;
- one coherent design language across system apps.

### Add modern expectations

- Unicode shaping, fallback fonts, IME, bidi, variable fonts;
- fractional scaling and high-DPI rendering;
- multi-monitor layouts;
- VRR, HDR, wide-gamut and color management where hardware permits;
- dynamic virtual desktops as SLOPOS Spaces;
- accessibility, keyboard navigation, screen-reader semantics;
- Wayland, XWayland, portals, clipboard, drag-and-drop, notifications;
- crash recovery, autosave, atomic writes, secure app installation;
- local-first Vision/AI;
- user control over appearance and behavior rather than one rigid policy.

### Visual constraints

- Do not use Apple trademarks, logos, proprietary assets, or bundled Apple fonts.
- Do not copy modern macOS traffic-light visuals as the default design.
- Classic, Graphite, High Contrast, Modern, and custom appearance profiles may
  coexist, but all must remain recognizably SLOPOS-I.
- Every visual change requires screenshots at 1.0, 1.25, 1.5, and 2.0 scale
  where the backend supports those scales.
- An idle desktop must not continuously repaint.

---

## 5. Engineering truth contract

### Evidence levels

Use these exact labels in `TRUTH.md`:

1. **PLANNED** — desired behavior only.
2. **SOURCE PRESENT** — code exists; build not run after the relevant edit.
3. **BUILD VERIFIED** — relevant target compiled successfully in the named environment.
4. **TEST VERIFIED** — named automated tests ran and passed after the edit.
5. **RUNTIME OBSERVED** — a human or automated runtime action produced raw logs/artifacts.
6. **HARDWARE VERIFIED** — exercised on applicable DRM/GPU/display/input hardware.

Never upgrade a claim because a type, test helper, button, or documentation file
exists. Unit tests cannot prove a visible cursor, real pointer grab, HDR output,
VRR, display-manager login, application compatibility, or visual quality.

### Required task workflow

1. Read `AGENTS.md` and `TRUTH.md`.
2. Inspect the current source and `git status`; do not trust old summaries.
3. State a small plan and exact files to touch.
4. Preserve existing working code, especially uncommitted Vision changes.
5. Implement one coherent slice.
6. Run the strongest available format/build/test/runtime checks.
7. Save raw evidence under `artifacts/qa/<date>-<slug>/`.
8. Update `TRUTH.md` with commands, environment, results, failures, and remaining risk.
9. Update `AGENTS.md` only when architecture or accepted product requirements change.
10. Do not create another Markdown file.

### Prohibited behavior

- Fabricated PASS fields or generated “verified” matrices.
- Claiming runtime behavior from source inspection.
- Directly modifying geometry in a test and calling it pointer-driven dragging.
- Optimistic completion percentages without a reproducible rubric.
- Silent fallback to third-party compositors or cloud services.
- Replacing a hard architectural problem with a visual simulation.
- Deleting prior work before extracting any unique facts and preserving it in Git history.
- Adding broad features while foundational protocol or session correctness is broken.

---

## 6. Implementation priority

Work in this order unless the user explicitly changes it. A later phase may be
scaffolded, but it must not distract from an earlier broken invariant.

### P0 — Establish a reproducible build baseline

- Run `cargo fmt --all -- --check`.
- Run `cargo check --workspace --all-targets`.
- Run `cargo test --workspace`.
- Run Clippy when the baseline builds.
- Record exact toolchain, target, distro, GPU/backend, and command output in `TRUTH.md`.
- Audit and preserve OpenCode changes in `slopos-vision`; do not rewrite working
  inference code merely because it came from another agent.
- Add `license = "MIT"` consistently to first-party Cargo packages.
- Keep `Cargo.lock` committed.


### P0.5 — Freeze the portable platform boundary

This work starts during SLOPOS-I rather than being deferred to SLOPOS-II:

- inventory every Linux-specific import, path, command, service and protocol;
- classify it as shared Unix/POSIX behavior or platform implementation;
- define typed platform interfaces for session/seat, device discovery, display,
  input, audio, power, networking, notifications, credentials and filesystem
  integration;
- move Linux implementations behind the interface without weakening the Linux
  compositor or replacing direct hardware support with stubs;
- add FreeBSD compile gates, then native runtime implementations and evidence;
- keep application and shell code free of direct Linux service invocation;
- convert release-critical scripts to POSIX `sh` or provide an equivalent
  FreeBSD-native path;
- record all remaining platform leakage in `TRUTH.md`.

Exit gate: the Linux desktop remains fully functional, portable crates contain
no accidental Linux dependencies, and the FreeBSD backend can be implemented
without changing public application or desktop policy APIs.

### P1 — Compositor and session correctness

#### Session supervisor

Implement a per-session private runtime directory such as:

```text
$XDG_RUNTIME_DIR/slopos-i/session-<nonce>/
├── readiness
├── client-wayland-display
├── token
└── logs/
```

Requirements:

- compositor child identity tied to the readiness handshake;
- no global readiness-file race;
- SIGINT/SIGTERM/SIGHUP-aware shutdown;
- process-group termination and reaping;
- session exits when the compositor dies;
- stale resource cleanup restricted to resources created by that session;
- clear logs for backend selection and child exits.

#### Wayland interaction correctness

- Validate `xdg_toplevel.move` and `resize` using the supplied seat and a valid
  implicit pointer-grab serial belonging to the requesting client.
- Implement real move/resize pointer grabs, old/new damage, configure events,
  and release behavior in shared backend-neutral code.
- Synchronize XDG `Activated`, `Resizing`, `Maximized`, and `Fullscreen` states.
- Hit-test surface trees and input regions, not only compositor rectangles.
- Complete XDG popup creation, positioning, constraint adjustment, grabs,
  repositioning, dismissal, and popup-tree rendering/input.
- Implement layer-shell anchors, requested size, margins, exclusive zones,
  keyboard interactivity, layer order, and authoritative work areas.
- Complete minimize/restore integration with the Dock.
- Bring XWayland focus, stacking, move, resize, transient windows, and clipboard
  behavior to an explicitly tested compatibility level.

#### Cursor

- Respect client cursor surfaces and hotspots.
- Always provide a visible fallback cursor.
- Support named themes and scale-aware cursor assets later.
- Verify nested software cursor, DRM composition, and hardware cursor plane
  independently; do not conflate them.

#### Rendering loop

- No unconditional redraw in `about_to_wait`, tick, or shell update loops.
- Render only for damage, input, animation, frame callbacks, output changes, or
  explicitly scheduled work.
- Target near-zero idle compositor/shell CPU; record real numbers rather than
  setting a fixed promise independent of backend and hardware.

### P2 — Production text and font platform

The current one-font, character-by-character, per-covered-pixel rectangle path
must be replaced before TextEdit, Finder, Terminal, accessibility, or modern
font profiles can be considered mature.

#### Shared text pipeline

Use a single shaped-text service built around the existing `cosmic-text` stack
or another permissively licensed Rust solution. It must provide:

- Unicode shaping and grapheme clusters;
- kerning and ligatures;
- bidi and script-aware fallback;
- line breaking, wrapping, measurement, selection and caret geometry;
- IME/text-input protocol integration;
- font fallback by run/glyph;
- fractional-scale rasterization;
- glyph atlas and batched quads;
- cache invalidation when fonts or scale change;
- deterministic screen/export metrics where required.

#### Font service and manager

`slopos-fonts` must become the authority for:

- recursive discovery in system and user font trees;
- TTF, OTF, TTC and variable-font metadata;
- family, style, weight, stretch, axes and script coverage;
- user installation, validation, duplicate detection, enable/disable and removal;
- safe atomic copies to `$XDG_DATA_HOME/fonts` or SLOPOS-owned font paths;
- change notification and live renderer refresh;
- role resolution for menu, title, body, small text, monospace and document fonts;
- per-user profiles and fallback chains;
- a guaranteed embedded recovery font.

User-facing profiles:

- **Classic** — period-appropriate metrics using legally distributable or user-provided fonts;
- **Modern** — clean contemporary sans/mono pair using audited permissive fonts;
- **Accessible** — high-legibility defaults and larger metrics;
- **Custom** — independently selectable font roles and sizes.

Users may select legally installed Apple fonts, but SLOPOS-I must not distribute
San Francisco, Chicago, Geneva, Monaco, or other proprietary fonts without the
necessary rights.

Settings must include a native Font Manager with preview, validation, install,
remove, enable/disable, duplicates, variable axes, and profile controls.

### P3 — Window presentation and configurable zoom control

Create one shared compositor state machine used by every backend:

```rust
pub enum WindowPresentationState {
    Normal,
    Minimized,
    Zoomed,
    Filled,
    Fullscreen,
    Tiled(TilePlacement),
}
```

Restore data must retain normal geometry, output, Space, stacking intent, and
previous state. State transitions must survive output and work-area changes.

#### User-selectable primary zoom-control action

Settings lets the user choose:

- Smart Zoom;
- Fill available work area;
- Full Screen;
- Show Layout Menu;
- Minimize;
- No action.

Configure separately:

- alternate/Option-click behavior;
- title-bar double-click behavior;
- whether fullscreen creates a dedicated Space;
- whether the global menu and Dock hide in fullscreen;
- animation duration or reduced-motion behavior.

#### Smart Zoom

First-party apps may advertise a preferred content-aware size. Examples:

- Finder fits useful rows/columns without pointless empty space;
- TextEdit fits page width and useful document height;
- Terminal snaps to whole character cells;
- Preview fits the image within the work area;
- Settings fits the current pane.

The compositor clamps the result to minimum/maximum constraints and work area.
For third-party apps without a preferred size, Smart Zoom falls back to Fill.
The SDK sends semantic requests; it must not call host-native maximize logic as
the final implementation.

#### Layout menu

A hold, hover, or configured action may expose SLOPOS-native placements:

- left/right half;
- top/bottom half;
- quadrants;
- centered useful size;
- move to another display;
- move to another Space.

Placement is compositor-owned and work-area aware.

### P4 — SLOPOS Spaces

Evolve fixed workspaces into dynamic, persistent, user-controlled Spaces.

Each Space needs:

- stable ID, user name and order;
- output assignment according to the active multi-monitor policy;
- optional wallpaper/appearance override;
- normal/fullscreen classification;
- ordered window membership;
- persistence and safe migration;
- creation, deletion and reordering rules;
- restore behavior when a display disappears.

User-selectable policies:

- one shared Space spans all displays;
- each display owns independent Spaces;
- activating an app switches to an existing Space or brings a window here;
- fullscreen uses the current Space or a dedicated fullscreen Space;
- applications/documents may be assigned to all, current, or named Spaces.

Implement a SLOPOS Spaces Overview owned by the shell/compositor:

- thumbnails for all Spaces;
- create, rename, reorder and remove;
- drag windows between Spaces;
- keyboard and pointer navigation;
- configurable animation and reduced-motion mode;
- multi-monitor representation matching the selected policy.

Gestures are optional until normal input is reliable, but the state model must
not depend on a specific touchpad library.

### P5 — Shell and Finder completion

#### Shell

Complete:

- authoritative global menu routing;
- Dock launch/running/minimized indicators and restore;
- notifications and notification history;
- search/launcher with applications, files, settings and commands;
- lock screen and session actions;
- wallpaper and appearance propagation;
- Spaces Overview;
- accessible semantic nodes for all shell controls.

#### Finder

Complete:

- robust local filesystem browsing;
- list, icon and column views where specified;
- file operations with conflict handling, undo and progress;
- Trash, removable devices, mounts and network locations;
- MIME/open-with and application associations;
- thumbnails, metadata and search;
- application bundles and document icons;
- safe structured launch requests;
- Vision actions for supported images;
- explicit desktop mode rather than label/dimension heuristics.

### P6 — Settings and system integration

Settings must expose live, truthful control of:

- appearance, themes, font profiles and accessibility;
- zoom-control/title-bar behavior;
- Spaces and multi-monitor policy;
- displays, scale, orientation, refresh, HDR and VRR availability;
- input devices and keyboard shortcuts;
- audio, networking, Bluetooth, power, storage, users and date/time;
- default applications and file associations;
- privacy, portals and Vision model status;
- software updates and session information.

Use typed service adapters and capability detection. Do not present a control as
functional when it only writes a private config file that the live subsystem
never reads. Linux-specific service adapters must be isolated from portable
policy/data crates. FreeBSD remains a real portability constraint.

### P7 — Third-party application compatibility

Implement and test:

- XDG shell toplevels and popups;
- layer shell;
- xdg-decoration negotiation;
- data device, primary selection and drag-and-drop;
- text-input-v3 and input-method integration;
- fractional scale, viewporter and presentation-time;
- relative pointer, pointer constraints and tablet protocols as needed;
- idle inhibit, session lock and shortcuts inhibit;
- foreign-toplevel/application activation where appropriate;
- portals: file chooser, open URI, screenshots, screen cast, notifications,
  settings, inhibit and background behavior;
- XWayland for representative X11 applications.

Maintain an executable compatibility suite of representative clients. Record
actual launch, map, input, popup, clipboard, fullscreen, file-dialog and exit
behavior in `TRUTH.md` or raw QA artifacts, never a guessed matrix.

### P8 — Modern display and graphics stack

#### Outputs and scaling

- hotplug, enable/disable, mode selection, orientation and position;
- fractional scale with correct logical/physical coordinate conversion;
- per-output work areas and shell placement;
- atomic KMS commits, page-flip/error recovery and multi-GPU awareness;
- reliable nested, headless and DRM test paths.

#### VRR

- detect connector/driver support;
- user policy: off, automatic, fullscreen only, always where safe;
- scheduling and direct-scanout behavior compatible with VRR;
- clear fallback and diagnostics;
- verify on physical hardware.

#### HDR and color

- output HDR capability and metadata handling;
- color-management protocol support;
- ICC profiles and per-output transforms;
- SDR baseline, wide-gamut and HDR content paths;
- transfer functions, tone mapping and SDR compatibility;
- user-visible capability/status controls;
- hardware proof before claiming support.

HDR is not complete merely because an enum or DRM property exists.

### P9 — SLOPOS Vision

SLOPOS Vision is the operating-system form of the same portable capability that
may later be consumed by the separate Loom project. Loom itself is not part of
this repository.

Architecture:

```text
slopos-vision              pure Rust OCR/segmentation/image processing
slopos-vision-protocol     typed request/job/result structures
slopos-vision-client       shared async client
slopos-visiond             local session daemon, lazy model loading
Finder / Preview / apps    user-facing integrations
```

Baseline rules:

- local-only; no telemetry, cloud fallback, or silent model download;
- CPU implementation is authoritative;
- Linux and FreeBSD-friendly pure-Rust runtime where practical;
- model files verified by manifest hash before loading;
- model licenses and redistribution rights recorded;
- models distributed as explicit packages or imported model packs;
- no arbitrary unvalidated output paths from IPC clients;
- bounded image dimensions, encoded input bytes, worker queue, memory and cache;
- cooperative cancellation described honestly;
- no image or recognized text in normal logs.

Required products:

1. **Extract Text** — OCR with lines, words, bounds and confidence where available.
2. **Lift Subject** — segmentation, mask cleanup, alpha compositing, clipboard/save.
3. **Preview** — native viewer with zoom/pan, OCR overlays and subject lifting.
4. Finder context actions and MIME integration.

Before adding UI, audit the current `slopos-vision` implementation, validate
model output shapes and errors, resolve packaged model paths, and preserve real
working inference. Do not leave protocol/client/daemon/Preview placeholders.

### P10 — Applications, packaging, security and release

#### TextEdit

- production text pipeline;
- open/save/revert, undo/redo, find/replace;
- atomic save and recovery;
- common text formats without false compatibility claims;
- print/PDF when the platform service exists.

#### Terminal

- reliable PTY lifecycle, resize, UTF-8, selection, clipboard and scrollback;
- configurable fonts/colors and accessible output;
- shell process cleanup and safe URI/file handling.

#### App Store

The SLOPOS store manages signed/verified SLOPOS `.app` bundles, not the base
distro package manager.

Bundle layout:

```text
Name.app/
├── Resources/
│   ├── Info.toml
│   └── icons/assets
├── bin/
└── lib/            optional
```

Implement catalog transport, signatures, trust roots, staging, path/symlink
validation, atomic install/update, rollback, removal, permissions, quarantine,
and launch-service rescan. Never delete the installed version before a complete
replacement has been validated and atomically committed.

#### Packaging

- layered installation on Arch and Ubuntu Server/Desktop;
- login-selectable Wayland session;
- optional bootable image from the same package/session definitions;
- FreeBSD build/port status reported separately and honestly;
- source archive export without `.git`, local agent settings, `.DS_Store`,
  `__MACOSX`, ignored secrets or accidentally bundled model weights;
- dependency and model attribution bundles generated from exact locked inputs.

#### Security and reliability

- no shell command construction from untrusted paths;
- canonicalize and validate filesystem operations;
- atomic config/document/package writes;
- bounded queues and resources;
- crash-safe cleanup and recovery;
- compositor/session services enforce client ownership and serial rules;
- no telemetry or network dependency for core desktop operation;
- audit unsafe Rust and justify every unsafe block.

---

## 7. Build and QA contract

### Static/build checks

Run from repository root where available:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash -n scripts/start-slopos-i
python3 -m py_compile scripts/*.py
cargo deny check licenses advisories bans sources
```

Do not claim success if a command was skipped or run before the final edit.

### Nested acceptance gate

A valid nested test must show:

- host compositor exposes exactly one SLOPOS outer window;
- shell and all test apps use the SLOPOS private socket;
- visible cursor over shell and app surfaces;
- pointer-driven move and resize with valid serial/grab logs;
- popup menu creation and dismissal;
- focus/raise and correct Activated state;
- minimize to Dock and restore;
- Fill, Smart Zoom, fullscreen and restore when implemented;
- Space switching and overview when implemented;
- clipboard and drag-and-drop;
- idle CPU/memory sample of at least 60 seconds;
- clean shutdown of the whole session.

### Hardware acceptance gate

Run separately on suitable Linux hardware:

- DRM/KMS login session;
- input and cursor, including cursor plane where supported;
- output hotplug and multi-monitor;
- fractional scale;
- VRR policy on a VRR-capable monitor;
- HDR metadata/output on an HDR-capable monitor;
- suspend/resume, lock/unlock and session termination;
- representative Wayland and XWayland applications.

### Evidence format

Each raw QA directory should contain machine-readable or raw artifacts such as:

```text
artifacts/qa/2026-08-01-private-socket/
├── environment.txt
├── commands.log
├── process-tree.txt
├── sockets.txt
├── compositor.log
├── geometry-before.json
├── geometry-after.json
├── cpu-memory.csv
├── screenshot.png
└── recording.webm
```

`TRUTH.md` records a concise result and points to the artifact path. Do not
encode fixed PASS values in a script without executing the measured operation.

---

## 8. Definition of complete

SLOPOS-I may be called a daily-driver desktop only after all of these have
runtime evidence:

- display-manager login into a compositor-owned session;
- stable shell, cursor, input, popups, focus, window management and shutdown;
- usable Finder, Settings, Terminal, TextEdit, Preview and App Store flows;
- production text and user-manageable fonts;
- dynamic SLOPOS Spaces and configurable window presentation behavior;
- clipboard, drag-and-drop, MIME/open-with, notifications and portals;
- representative Wayland and XWayland applications;
- multi-monitor and fractional scale;
- accessible keyboard and semantic navigation;
- secure, atomic application installation/update/removal;
- local Vision OCR and subject lifting through the daemon and UI;
- package installation on supported base systems;
- acceptable idle/resource behavior;
- no severe data-loss, session-security, or compositor-protocol defect;
- HDR/VRR claims separately hardware-verified where advertised.

Until then, describe it as an experimental or developing desktop environment
and report component status from `TRUTH.md`.

---

## 9. Maintenance of this document

Change `AGENTS.md` only for accepted architecture, product requirements,
engineering policy, phase order, or definition of done. Do not append daily
logs. When a requirement is completed, leave the requirement here and update
its factual status in `TRUTH.md`.

All agents must finish by checking that the repository still has exactly:

```text
AGENTS.md
README.md
TRUTH.md
```

as Markdown files.
