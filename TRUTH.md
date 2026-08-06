# TRUTH.md — SLOPOS-I Audit and Evidence Ledger

**Purpose:** This is the sole factual status, score and defect ledger for
SLOPOS-I. Final requirements and execution rules live in `AGENTS.md`.
`README.md` is the public introduction.

**Audited product implementation:**
`c9b74951ee2a167967d807f64677a5064c8fc118`
**Audit date:** 2026-08-06
**Audit basis:** current-source review, commit-delta review, exact-commit GitHub
Actions evidence and retained VM/UTM runtime evidence.
**Public target:** a 100/100 production Linux desktop environment that genuinely
competes with KDE Plasma and GNOME as a daily driver.
**Current verdict:** **63/100 — functional custom desktop alpha.**

Documentation commits after the audited implementation do not change the
product score unless they are accompanied by implementation and evidence.

---

## 1. Evidence language

| Label | Meaning |
|---|---|
| **PLANNED** | Accepted requirement only |
| **SOURCE PRESENT** | Relevant code exists, but the user-visible/runtime behaviour is not proved |
| **BUILD VERIFIED** | The named target compiled at the recorded commit |
| **TEST VERIFIED** | Named automated tests passed at the recorded commit |
| **RUNTIME OBSERVED** | A real process or interaction produced retained evidence |
| **HARDWARE VERIFIED** | Behaviour ran on applicable graphics, display or input hardware |

A type, helper, menu entry, test fixture, generated table, screenshot mock or
successful build does not by itself prove a production feature.

### Audit confidence

- source architecture and exact CI: high confidence, approximately ±3 points;
- functional runtime ratings: moderate confidence, approximately ±5 points;
- visual-polish ratings: lower confidence, approximately ±7 points because the
  exact current product head was not freshly reviewed through a complete human
  screenshot and interaction matrix during this documentation pass.

---

## 2. Exact verified baseline

GitHub Actions run `30779578542` completed successfully against implementation
commit `db6cc01` and included:

- workspace build with all targets;
- workspace tests;
- workspace Clippy;
- Linux release build;
- lockfile consistency;
- rustfmt;
- exact-commit compositor source/build contract;
- SLOPOS-owned headless Wayland runtime protocol gate;
- compositor evidence upload.

The headless runtime gate verifies:

- a private compositor-owned socket;
- authenticated readiness;
- registry access;
- abrupt client-disconnect recovery;
- XDG toplevel configure;
- maximize, fullscreen and restore transitions;
- XDG popup configure;
- popup reposition acknowledgement.

It does **not** prove physical DRM/KMS rendering, real input devices,
multi-monitor hardware, popup pointer grabs, broad application compatibility,
XWayland, HDR, VRR or long-running stability.

Retained earlier VM/UTM evidence demonstrates a real SLOPOS DRM session,
compositor-owned shell layers, first-party windows, visible cursor, focus/global
menu changes, Fill, minimize and restore. That evidence remains valid for the
recorded commits but is not silently promoted to current-head hardware proof.

### Current implementation wave — output-aware presentation

Implementation commit `c6ce17e161ea9749cf7dd01dfa1c0f2a43f2f9ea` is **BUILD VERIFIED**, **TEST
VERIFIED** and covered by the existing **RUNTIME OBSERVED** headless compositor
gate. The new multi-output geometry itself remains unverified on physical
multi-monitor hardware.

This wave:

- normalises negative/offset nested output layouts while preserving relative
  monitor placement;
- computes true output-union bounds without assuming an origin of `(0, 0)`;
- assigns windows deterministically by greatest output overlap and nearest-output
  fallback;
- constrains XDG popups to the output that owns their root surface;
- applies Smart Zoom, Fill and fullscreen to one selected output instead of the
  complete multi-monitor canvas;
- stores the real connector/synthetic output name in restore state;
- clamps normal windows against the selected output's exclusive work area;
- adds pure tests for negative layouts, gaps, off-screen windows, overlap ties and
  integer-boundary safety;
- regenerates the workspace lockfile and passes compositor check, test, source
  contract and headless runtime gates before commit.

The overall product score remains **63/100**. The compositor score advances from
66 to **67/100**; physical hotplug, mixed-scale/refresh, per-output layer-shell
targeting, direct scanout and hardware evidence remain release blockers.

### Current implementation wave — per-output layer-shell ownership

Implementation commit `c9b74951ee2a167967d807f64677a5064c8fc118` is **BUILD VERIFIED**, **TEST
VERIFIED** and covered by the existing **RUNTIME OBSERVED** headless compositor
gate. Physical multi-monitor placement and hotplug remain unverified.

This wave:

- resolves a layer-shell client's requested `wl_output` back to the exact Smithay
  output and stores that owner on the mapped layer;
- computes menu-bar, Dock, notification and other layer geometry relative to the
  owning output rather than the full compositor canvas;
- scopes exclusive zones and normal-window work-area clamping to the owning
  output only;
- emits compositor-managed `wl_surface.enter` and `wl_surface.leave` membership
  as windows move or resize across outputs;
- constrains layer surfaces to one output membership and clears it on destroy;
- sends frame callbacks using each window or layer's selected output instead of
  routing every surface through the first output;
- adds pure multi-output membership tests and a permanent source/build contract;
- regenerates the workspace lockfile and passes workspace build/test/Clippy plus
  exact compositor source, release and headless runtime gates before commit.

The overall product score remains **63/100**. The compositor score advances from
67 to **68/100**. Runtime topology mutation, connector removal, mixed-scale
rendering, physical output evidence and DRM/KMS hotplug remain open.

---

## 3. Production scoring model

| Score | Meaning |
|---:|---|
| 0–19 | Requirement, experiment or placeholder |
| 20–39 | Early implementation or disconnected subsystem |
| 40–59 | Credible prototype |
| 60–74 | Functional alpha |
| 75–84 | Strong beta |
| 85–91 | Release candidate |
| 92–99 | Production-ready with bounded known gaps |
| 100 | Frozen acceptance contract completely satisfied |

The public target is 100/100. The current score remains evidence-based rather
than aspirational.

---

## 4. Executive scorecard

| Perspective | Score | Current truth |
|---|---:|---|
| Engineering foundation | **76** | Strong Rust workspace, session/compositor ownership, useful tests and exact Linux CI |
| UI and UX | **59** | Distinctive and coherent, but renderer, typography, image display, animation and integration remain alpha-grade |
| Product functionality | **61** | Real shell, compositor, applications and Vision paths; many daily-driver workflows are incomplete |
| Linux daily-driver readiness | **51** | Suitable for controlled development and QA, not yet for a non-technical user’s only desktop |
| Compositor strict completion | **68** | Strong protocol/state foundation; hardware, input, displays, XWayland and compatibility gates remain |
| Security and release readiness | **52** | Good session/filesystem hardening, incomplete sandbox, signing, packaging, upgrades and recovery |
| Accessibility readiness | **38** | Meaningful AT-SPI work, incomplete live tree and Orca operation |
| POSIX/FreeBSD portability | **22** | Direction is defined; implementation and native evidence remain early |
| **Overall SLOPOS-I** | **63** | Strong custom desktop alpha with substantial productionisation remaining |

The overall score is a maturity judgment, not a percentage of code written.

---

## 5. Why SLOPOS-I is not production-level today

A production desktop must remain dependable across hardware, applications,
input methods, displays, upgrades, crashes and days of continuous use. Current
SLOPOS-I still has release-blocking gaps in each of those areas.

The most important blockers are:

1. incomplete physical compositor/input/multi-monitor coverage;
2. prototype text and image rendering;
3. incomplete cross-application DnD and IME;
4. partial XWayland and third-party application compatibility;
5. SLOPOS Spaces model not yet connected to a complete user experience;
6. Settings not yet authoritative for all system services;
7. first-party applications remain incomplete for normal daily use;
8. accessibility is not yet live and Orca-complete;
9. sandbox, permissions, publisher trust and package signing are incomplete;
10. installation, upgrade, rollback, recovery and long soaks are not proven.

Passing CI proves engineering health. It does not erase these product gaps.

---

## 6. UI and UX audit

| UI area | Score | Current truth |
|---|---:|---|
| Visual identity | 77 | Recognisable classic Macintosh/System 7/Platinum lineage without generic GTK styling |
| Design-system consistency | 71 | Semantic palette and compact metrics exist; use is not universal |
| Window chrome | 73 | Native controls and typed compositor actions are real; policy UI is incomplete |
| Global menu | 66 | Focus-driven ownership exists; command completeness varies |
| Typography quality | 60 | Shaping APIs exist, but font roles and profiles are not authoritative everywhere |
| Text-rendering performance | 39 | Immediate glyph-coverage rectangles remain far behind a glyph-atlas/batched path |
| Image rendering | 32 | Preview still presents a maximum 96×64 proxy as panel cells instead of a real GPU texture |
| Layout and resizing | 62 | Core layout works; many applications retain fixed sizes and hand-authored geometry |
| Keyboard navigation | 72 | Shared focus management and keyboard activation are substantive |
| Pointer dispatch and capture | 72 | Shared dispatcher and capture are real; compositor interaction evidence remains incomplete |
| Editing interaction | 64 | UTF-8-safe selections and caret insertion exist; graphemes, bidi and IME remain open |
| Accessibility UX | 46 | AT-SPI structure exists; live-tree and assistive workflows remain incomplete |
| Animation and motion | 27 | No production transition system for Spaces, windows, Dock and notifications |
| Scaling polish | 53 | Logical scaling exists; mixed-scale visual matrix is incomplete |
| Theme/font customisation | 57 | Themes exist; font profiles and live Settings integration are incomplete |
| Visual polish | 52 | Distinctive but visibly an engineering alpha |
| **UI/UX overall** | **59** | Strong identity, incomplete production rendering and productisation |

### UI release blockers

- replace rectangle-per-glyph rendering with a glyph atlas and batched quads;
- replace Preview’s panel-per-pixel image path with GPU textures, large-image
  tiling and colour-correct scaling;
- make `slopos-fonts` authoritative across shell, SDK and applications;
- implement shaped grapheme/bidi caret geometry and IME;
- add restrained, reduced-motion-aware window and Spaces transitions;
- run a current-head screenshot matrix at 1.0, 1.25, 1.5 and 2.0 scale;
- remove clipping, rigid geometry, contrast and focus-ring inconsistencies.

---

## 7. Compositor and session audit

### Strict 100-point compositor contract

| Domain | Current | Target | Main remaining work |
|---|---:|---:|---|
| Session sovereignty and lifecycle | 9 | 10 | Display-manager, suspend/resume, lid and longer failure coverage |
| Core Wayland lifecycle | 12 | 14 | Broader popup, subsurface, transient and modal compatibility |
| Input correctness | 7 | 10 | Physical multi-device, touch, gestures, constraints, relative pointer and hotplug |
| Clipboard, DnD and IME | 4 | 8 | Cross-client DnD, drag icons, cancellation, large transfers and text-input/input-method |
| Rendering and frame scheduling | 9 | 12 | Direct scanout, occlusion, GPU recovery and physical pacing evidence |
| Displays and scaling | 7 | 12 | Hotplug, mixed scale/refresh, rotation, migration and topology recovery |
| External Wayland compatibility | 6 | 12 | GTK, Qt, Electron, browsers, office, media, games and popup-heavy apps |
| XWayland | 4 | 8 | Rootless scene, override-redirect, clipboard/DnD, DPI, restart and application matrix |
| HDR, VRR and colour | 3 | 6 | Physical capable hardware, metadata/presentation proof and full colour path |
| Security, stability and release QA | 7 | 8 | Soaks, resource plateaus, fuzzing and hostile-client breadth |
| **Total** | **68** | **100** | First subsystem targeted for a genuine 100 |

### Strong current compositor work

- private session runtime and socket ownership;
- verified readiness token and process identity;
- Wayland display polling and client dispatch;
- reversible presentation states;
- output-change geometry clamping;
- popup configuration and reposition testing;
- abrupt-client-disconnect recovery;
- frame-pacing and work-area tests;
- capability-driven HDR/VRR policy rather than fabricated support;
- exact-commit CI contract.

### Remaining proof before 100

- physical DRM/input/multi-monitor matrix on current code;
- touch, touchpad gestures and multiple-device hotplug;
- production DnD and IME;
- broad Wayland client matrix;
- first-class XWayland;
- HDR/VRR on capable displays;
- direct scanout and GPU-reset recovery;
- 24-hour idle and mixed-workload soaks;
- memory/file-descriptor plateaus and fuzzing.

---

## 8. Shell and desktop product audit

| Component | Score | Current truth |
|---|---:|---|
| Session supervisor | 86 | One of the strongest components; broader lifecycle and platform abstraction remain |
| Desktop shell | 66 | Real layer surfaces, menu, Dock and overlays; integration and polish remain incomplete |
| Global menu routing | 68 | Focus-driven ownership is credible; app command coverage varies |
| Dock | 59 | Launcher/minimize foundations exist; indicators, ordering, DnD and multi-monitor policy need work |
| Notifications | 48 | Infrastructure exists; actions, grouping, history and quiet modes are incomplete |
| Lock/session UX | 55 | UI and session actions exist; production authentication and lifecycle proof remain |
| Search/launcher | 45 | Early local functionality; indexing, ranking and actions remain incomplete |
| Portals | 47 | Source exists; compatibility and permission behaviour are not broadly proved |
| Clipboard | 65 | Real selection paths; large/cancelled/format-diverse transfers need QA |
| Cross-app drag-and-drop | 51 | In-app movement exists; complete data-device workflow remains |
| SLOPOS Spaces model | 71 | Dynamic model, persistence and output policy are substantive |
| SLOPOS Spaces UX | 25 | No complete overview, gestures, drag-between-Spaces or Settings product |
| Multi-monitor desktop UX | 43 | Policy/types exist; complete live topology behaviour is not established |
| **Shell/desktop overall** | **57** | Real custom shell alpha, not finished product |

---

## 9. Renderer, toolkit, text, fonts and accessibility

| Area | Score | Current truth |
|---|---:|---|
| Widget toolkit | 67 | Real layout, focus, dispatch, capture and controls; inconsistent depth remains |
| SDK/application framework | 65 | Real window/menu/event routing; central presenter and platform leakage remain |
| General renderer | 49 | WGPU immediate path works; retained resources, image textures and batching remain incomplete |
| Unicode shaping API | 66 | `cosmic-text` shaping exists; it is not yet the sole authoritative editing/layout path |
| Text editing model | 61 | UTF-8-safe selection and insertion; grapheme, bidi, IME and visual lines remain |
| Font infrastructure | 70 | Discovery, install, hashes, duplicates, enable state, roles and profiles are substantial |
| Font product integration | 36 | No complete Font Manager or live role resolution across the desktop |
| Accessibility infrastructure | 55 | AT-SPI roles, actions, events, component and text work exist |
| Accessibility daily-driver usability | 38 | Snapshot tree, shallow nesting and incomplete Orca workflows remain |
| **Platform layer overall** | **58** | Good foundations, major release-blocking integration work |

The current accessibility source itself records best-effort D-Bus events,
snapshot trees, shallow nesting and incomplete live text updates. Production
claims are prohibited until assistive-technology workflows are demonstrated.

---

## 10. First-party application audit

| Application | Functional | UI/UX | Overall | Current truth |
|---|---:|---:|---:|---|
| File manager | 63 | 59 | **61** | Real navigation, file operations, trash and drag-to-folder; missing mature views, search, mounts, thumbnails, associations, conflicts and undo |
| Settings | 48 | 53 | **50** | Broad categories and persistence; many controls are not authoritative live services; no complete Fonts, Spaces or zoom-policy panels |
| TextEdit | 67 | 61 | **64** | Selection-aware clipboard, caret insertion, find, save/recovery and undo/redo; no production multiline shaping, IME, rich text or scalable transactions |
| Terminal | 72 | 65 | **69** | Real PTY, parser, tabs, alternate screen, selection, resize and child shutdown; cell model lacks complete CJK/combining/grapheme correctness |
| Software manager | 46 | 48 | **47** | Hardened local archive installation; catalogue, signing, publisher trust, network delivery, updates and removal are incomplete |
| Preview | 57 | 39 | **48** | Decode bounds, zoom state and Vision client paths exist; image presentation remains a stretched low-resolution panel mosaic |
| **Application suite** | **59** | **54** | **57** | Useful native alpha applications, not daily-driver replacements |

### Naming defect

The visible `AirDrop` label must be removed. The native nearby-transfer feature
is **SLOPOS Share**. It requires independent SLOPOS-to-SLOPOS discovery,
authenticated encryption, consent, resume, integrity checking and atomic save.

Other inherited application names require deliberate public naming and legal
review before release.

---

## 11. SLOPOS Vision audit

| Area | Score | Current truth |
|---|---:|---|
| OCR/segmentation core | 74 | Real preprocessing, inference, decoding, masks and compositing |
| Model integrity | 68 | Hash and manifest validation exist; acquisition and redistributability workflow remain incomplete |
| Protocol | 72 | Typed local job/asset/error protocol |
| Client | 70 | Reusable local client and polling paths |
| Daemon | 72 | Session-local socket, bounded jobs and local-only operation are substantive |
| Preview integration | 60 | Real asynchronous request/result paths; polished successful workflow is not broadly proved |
| File-manager integration | 18 | Native context actions and output workflow remain incomplete |
| Accuracy/evaluation | 22 | No sufficient labelled benchmark and documented calibration |
| Performance/acceleration | 31 | CPU path exists; production acceleration and memory/cancellation benchmarks remain |
| Model distribution | 34 | No complete clean-install model-pack workflow |
| **Vision product overall** | **58** | Serious local subsystem alpha; distribution and measured product proof remain bottlenecks |

---

## 12. Security, quality and release engineering

| Area | Score | Current truth |
|---|---:|---|
| Rust architecture | 72 | Clear crate intent; several central files remain large |
| Error handling | 72 | Session, filesystem, installer, Vision and compositor handling improved |
| Filesystem safety | 78 | Atomic writes, path bounds, symlink checks, hashes and rollback are common |
| Session isolation | 82 | Private runtime/socket/token/process-group design is strong |
| Application sandbox/permissions | 27 | No mature general sandbox or capability permission product |
| Package trust/signing | 24 | Integrity exists; publisher authenticity and trust chain do not |
| Automated testing | 82 | Broad tests and exact compositor contract |
| CI quality | 87 | Strong Linux build/test/release/fmt/lockfile/runtime gates at audited product head |
| Runtime QA breadth | 61 | Useful VM/UTM evidence; current-head hardware/app matrix incomplete |
| Performance engineering | 54 | Frame scheduling improved; renderer remains expensive |
| Packaging/install/upgrade | 45 | Artefacts exist; clean lifecycle and recovery need current evidence |
| Documentation discipline | 84 | Three-file structure and production target are now explicit |
| **Quality/release overall** | **65** | Engineering discipline is ahead of product maturity |

---

## 13. POSIX and FreeBSD truth

| Area | Score | Current truth |
|---|---:|---|
| Normative portability architecture | 90 | Required boundary is documented |
| Implemented platform abstraction | 24 | No complete `slopos-platform` boundary yet |
| POSIX-shell release surface | 20 | Existing release/QA scripts still contain Bash/GNU assumptions |
| Linux backend | 68 | Real implementation, still incomplete as a production desktop |
| FreeBSD compile support | 10 | No current native build matrix |
| FreeBSD runtime backend | 3 | No native compositor/session/application evidence |
| Cross-platform non-regression suite | 5 | Not established |
| **Portability implementation** | **22** | Correct direction, early implementation |

Linux remains the first production target. Portability work must not weaken or
fork the Linux desktop.

---

## 14. Required path from 63 to 100

### Gate 1 — Compositor 100

Complete and prove physical input, DnD/IME, multi-output, scaling, XWayland,
third-party applications, HDR/VRR hardware, direct scanout, GPU recovery, soaks
and fuzzing.

### Gate 2 — Production renderer

Ship glyph atlas, authoritative shaping, grapheme/bidi/IME editing geometry,
real image textures, retained resources, batching and scale-aware caches.

### Gate 3 — Complete desktop product

Connect SLOPOS Spaces, font profiles, zoom policy, Dock, notifications, search,
lock/session and multi-monitor policies to authoritative compositor/services.

### Gate 4 — Authoritative Settings and services

Replace preference-only or shell-command paths with typed state, application,
failure reporting and rollback for displays, input, audio, network, Bluetooth,
power, accessibility, fonts, Spaces, permissions and defaults.

### Gate 5 — Finish first-party applications

Complete the file manager, text editor, Terminal, Preview and software manager
for every advertised workflow; hide commands that are not implemented.

### Gate 6 — Ecosystem compatibility

Pass GTK, Qt, Electron, browsers, office, media, communication, development,
gaming, portals, Flatpak and XWayland matrices.

### Gate 7 — Accessibility and localisation

Live AT-SPI tree, Orca workflows, keyboard-only completion, high contrast,
reduced motion, locale extraction, bidi UI and translation QA.

### Gate 8 — Security and trust

Sandbox/permission strategy, portal enforcement, signed bundles, publisher
identity, safe received files, threat model and security regression suite.

### Gate 9 — Reliability and release

Performance budgets, 24-hour soaks, leak plateaus, crash recovery, clean install,
upgrade, rollback, uninstall, configuration migration and signed release
artefacts.

### Gate 10 — Production declaration

The product may call itself production-ready only when the weighted score reaches
at least 92, no release-blocking domain remains incomplete, normal users can
install and operate it without development tools, and this file contains current
exact evidence without contradiction.

The aspirational endpoint remains 100/100.

---

## 15. Immediate next implementation order

1. finish the Linux compositor acceptance matrix;
2. implement production text and image rendering;
3. connect Spaces, fonts and zoom policy to the live desktop;
4. make Settings authoritative;
5. complete first-party applications;
6. finish portals, XWayland and third-party compatibility;
7. complete accessibility and localisation;
8. finish security, packaging, performance, recovery and long soaks;
9. implement the POSIX platform boundary and native FreeBSD support.

Do not divert core effort into decorative features while an earlier
release-blocking invariant remains broken.

---

## 16. Bottom line

SLOPOS-I has enough real implementation to be taken seriously as a custom Linux
desktop project. It has a sovereign compositor/session foundation, a distinct
interface, useful first-party applications, local Vision functionality and
strong Linux CI.

It is still **63/100** because production readiness is determined by complete
user workflows, hardware/application compatibility, accessibility, security,
installation, recovery and long-term reliability—not by repository size or the
number of implemented types.

The public mission is now unambiguous: **finish SLOPOS-I to a genuine 100/100
production desktop environment competitive with KDE Plasma and GNOME, while
keeping every progress claim tied to evidence.**
