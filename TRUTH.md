# TRUTH.md — SLOPOS Audit and Evidence Ledger

**Purpose:** This is the sole factual status and audit document for the SLOPOS
project. Product requirements and execution rules live in `AGENTS.md`.
`README.md` remains the public introduction.

**Audited implementation head:** `db6cc01cb1a6f74b395fbcf24dd8cfddd7aaca2a`
(`docs/program-design`)  
**Current documentation head before this update:**
`9291befddb1306609683aeaaea6770ad3c93f181`  
**Audit date:** 2026-08-06  
**Audit type:** current-source review, commit-delta review, exact-commit GitHub
Actions evidence, and comparison with retained UTM/VM runtime evidence.  
**Current product generation:** SLOPOS-I  
**SLOPOS-II implementation status:** planned only; no kernel source exists.

The commits after `db6cc01` and before this audit update changed development
documentation and removed one-shot migration machinery. They did not alter the
audited product implementation.

---

## 1. Evidence language

| Label | Meaning |
|---|---|
| **PLANNED** | Accepted requirement only. |
| **SOURCE PRESENT** | Code exists, but the relevant runtime behavior is not proved. |
| **BUILD VERIFIED** | The named target compiled at the recorded commit. |
| **TEST VERIFIED** | Named automated tests passed at the recorded commit. |
| **RUNTIME OBSERVED** | A real process/runtime action produced retained evidence. |
| **HARDWARE VERIFIED** | Behavior ran on applicable graphics/display/input hardware. |

A source type, helper, test fixture, menu label, configuration value, screenshot
mock, or generated table does not by itself prove a working product feature.

### Audit confidence

- Source architecture and exact CI results: high confidence, approximately
  ±3 score points.
- Functional runtime ratings: moderate confidence, approximately ±5 points,
  because current-head interactive hardware QA was not repeated in this audit.
- Visual-polish ratings: lower confidence, approximately ±7 points, because the
  current exact head was not freshly exercised by a human visual QA session.

---

## 2. Exact current verification

GitHub Actions run `30779578542` completed successfully against implementation
head `db6cc01`:

- workspace build with all targets;
- workspace tests;
- workspace Clippy;
- Linux release build;
- lockfile regeneration consistency;
- rustfmt;
- exact-commit compositor source/build contract;
- SLOPOS-owned headless Wayland runtime protocol gate;
- compositor evidence upload.

The headless runtime gate verifies a private compositor socket, authenticated
readiness, registry access, abrupt client-disconnect recovery, XDG toplevel
configure, maximize, fullscreen and restore transitions, XDG popup configure,
and popup repositioning. It does **not** prove DRM/KMS scanout, physical input,
visual rendering, popup pointer grabs, XWayland, HDR, VRR, or multi-monitor
hardware behavior.

Retained earlier VM/UTM evidence demonstrates a real SLOPOS DRM session,
compositor-owned shell layers, first-party client windows, visible cursor,
focus/global-menu changes, Fill, minimize and restore. Those observations remain
useful but are not silently promoted to exact-current-head hardware proof.

---

## 3. Scoring scale

| Score | Meaning |
|---:|---|
| 90–100 | Complete, polished, release-quality subsystem with broad runtime evidence |
| 75–89 | Strong beta; substantial functionality and bounded remaining gaps |
| 60–74 | Functional alpha; real implementation with important incomplete paths |
| 40–59 | Credible prototype; useful but visibly/product-wise immature |
| 20–39 | Early implementation or disconnected scaffolding |
| 0–19 | Mostly requirement, experiment, or placeholder |

A 100 requires implementation, automated verification, runtime evidence,
failure-path coverage, supported-hardware evidence where relevant, and no known
contradictory defect.

---

## 4. Executive scorecard

| Perspective | Score / 100 | Truth |
|---|---:|---|
| **Engineering foundation** | **76** | Strong Rust workspace, session/compositor ownership, growing tests and exact Linux CI. |
| **UI and UX implementation** | **59** | Distinct, coherent design language; rendering, typography, layout and animation remain prototype-grade. |
| **Product functionality** | **61** | Real shell, compositor, apps and Vision service paths, but many daily-driver workflows are incomplete. |
| **Linux desktop milestone readiness** | **51** | Credible custom desktop alpha, not yet a KDE/GNOME-class daily driver. |
| **Compositor strict completion** | **66** | Much stronger protocol/state foundation; hardware, multi-output, input, XWayland and compatibility gates remain. |
| **POSIX-portable SLOPOS-I architecture** | **22** | Requirement is now normative; current shared code still leaks Linux assumptions and FreeBSD is unproved. |
| **FreeBSD desktop support** | **8** | Design target only; no current native build/runtime evidence. |
| **SLOPOS-II custom kernel** | **0** | Intentionally not started. |
| **Overall SLOPOS-I product** | **63** | Strong custom desktop alpha with a large remaining productisation programme. |

The overall score is not an arithmetic claim of percent complete. It is the
current product-maturity judgment under the rubric above.

---

## 5. UI and UX audit

| UI area | Score | Current truth |
|---|---:|---|
| Visual identity | 77 | Recognisable Classic Macintosh/System 7/Platinum lineage rather than generic GTK/GNOME styling. |
| Design tokens and consistency | 71 | Shared semantic Classic palette and compact metrics exist; not every app fully consumes them. |
| Window chrome | 73 | Native SLOPOS chrome and typed compositor actions are real; policy configuration is not fully exposed in Settings. |
| Global menu experience | 66 | Compositor focus can drive the shell menu; many application commands remain labels without complete implementations. |
| Typography quality | 60 | Shaping APIs are present and used by the SDK, but font roles and live user profiles are not authoritative. |
| Text rendering performance | 39 | Immediate rendering and glyph-coverage rectangles remain much less efficient than a glyph atlas/batched text pipeline. |
| Image rendering | 32 | Preview currently expands a maximum 96×64 software thumbnail as a grid of `Panel` cells rather than presenting the original image through a GPU texture. |
| Layout and resizing | 62 | Core layout works, but many applications still use fixed logical heights, hand-authored coordinates and rigid sidebars/toolbars. |
| Keyboard focus/navigation | 72 | Shared `FocusManager`, focusable controls and keyboard activation are substantive improvements. Coverage is not universal. |
| Pointer dispatch/capture | 72 | Shared dispatcher, capture and rect-aware routing are real. Full compositor pointer-grab runtime QA remains incomplete. |
| Editing interaction | 64 | Text fields now support UTF-8-safe selection and caret insertion. Grapheme clusters, shaped caret geometry and IME are missing. |
| Accessibility UX | 46 | AT-SPI roles/actions/text/component scaffolding exists, but the source explicitly records snapshot trees, shallow nesting and incomplete Orca behavior. |
| Animation and motion | 27 | No coherent production animation system for Spaces, minimize/restore, overview, notifications or window transitions. |
| High-DPI/fractional-scale polish | 53 | Logical scale plumbing exists; complete mixed-scale visual evidence and pixel-quality review do not. |
| Theme/customisation UX | 57 | Several themes and accent choices exist. Fonts, Spaces and zoom behavior are not yet first-class Settings products. |
| Visual polish | 52 | Distinct and increasingly coherent, but still visibly an engineering alpha. |
| **UI/UX overall** | **59** | Strong identity and improving interaction architecture; substantial renderer and productisation work remains. |

### Highest-impact UI defects

1. Replace rectangle-per-glyph drawing with a glyph atlas, batched quads,
   retained resources and scale-aware cache invalidation.
2. Replace Preview's thumbnail-cell mosaic with a real GPU image/texture path,
   colour-correct scaling, large-image tiling and responsive zoom/pan.
3. Make `slopos-fonts` authoritative across shell, SDK and applications, with a
   native Font Manager and live role/profile updates.
4. Replace fixed character-width caret placement with shaped cluster/grapheme
   geometry and integrate Wayland text-input/IME.
5. Add one restrained, reduced-motion-aware transition system for windows,
   Spaces, overlays, notifications and Dock state.
6. Run a current-head screenshot matrix at 1.0, 1.25, 1.5 and 2.0 scale and
   eliminate spacing, clipping, contrast and focus-ring inconsistencies.

---

## 6. Compositor and session audit

### Strict completion score

| Domain | Current | Target | Remaining evidence/work |
|---|---:|---:|---|
| Session sovereignty and lifecycle | 9 | 10 | Display-manager, suspend/resume and longer failure/soak coverage. |
| Core Wayland window lifecycle | 12 | 14 | Broader third-party popup/subsurface/transient/modal compatibility. |
| Input correctness | 7 | 10 | Physical multi-device, touch, gestures, constraints, relative pointer and hotplug matrix. |
| Clipboard, DnD and IME | 4 | 8 | Cross-client DnD, drag icons, cancellation, large transfers, text-input/input-method. |
| Rendering and frame scheduling | 9 | 12 | Physical pacing/occlusion/direct-scanout/GPU-loss and long idle proof. |
| Multiple displays and scaling | 5 | 12 | Real hotplug, mixed scale/refresh, rotation, migration, lid and topology recovery. |
| External Wayland compatibility | 6 | 12 | GTK/Qt/Electron/Firefox/Chromium/LibreOffice/Steam test matrix. |
| XWayland | 4 | 8 | Rootless scene, override-redirect, clipboard/DnD, DPI, restart and app matrix. |
| HDR, VRR and colour path | 3 | 6 | Physical capable displays/GPUs, metadata/presentation proof and colour pipeline. |
| Security, stability and release QA | 7 | 8 | Long soaks, resource-leak plateaus, fuzzing and broader hostile-client tests. |
| **Total** | **66** | **100** | Linux compositor remains the first subsystem targeted for a genuine 100. |

### What is now strong

- The session supervisor owns a unique private runtime and compositor socket.
- Runtime-directory ownership is pinned with an open directory descriptor and
  verified by device/inode identity before recursive cleanup.
- The compositor registers the Wayland display poll source instead of relying
  only on the listening socket.
- XDG presentation state has reversible normal/Fill/fullscreen/minimize/tiling
  policy with output-change clamping.
- Work-area, frame-pacing, HDR/VRR capability, DRM property and screenshot paths
  have materially stronger validation and failure handling.
- The exact CI runtime client exercises XDG toplevel state transitions, popup
  creation/reposition and abrupt disconnect recovery.
- The CI compositor contract is release-blocking rather than an aspirational
  script stored in the tree.

### What prevents 100

- No current exact-head physical DRM/input/multi-monitor compatibility matrix.
- No exhaustive third-party Wayland application suite.
- Cross-application drag-and-drop and production IME/text-input remain open.
- XWayland is not demonstrated as a complete first-class scene path.
- HDR/VRR hooks are capability-driven, but supported physical hardware evidence
  is still required.
- Direct scanout, mixed-output timing and GPU-reset recovery are not release
  proven.
- 24-hour idle/mixed-workload soaks, aggressive client fuzzing and leak gates
  are not complete.

---

## 7. Shell and desktop product audit

| Component | Score | Status and major gaps |
|---|---:|---|
| Session supervisor | 86 | Strong private lifecycle and cleanup; platform abstraction and display-manager breadth remain. |
| Desktop shell | 66 | Real layer surfaces, menu, Dock and overlays; settings/service integration and polish remain incomplete. |
| Global menu routing | 68 | Focus-driven ownership is credible; command completeness varies by app. |
| Dock | 59 | Visible launcher/minimize concepts exist; production restore, indicators, ordering, DnD and multi-monitor policy need work. |
| Notifications | 48 | Infrastructure exists; grouping, actions, persistence, quiet modes and polished UX are incomplete. |
| Lock/session UX | 55 | Lock UI and session actions exist; authentication, display-manager and security review are not production-complete. |
| Search/launcher | 45 | Early local functionality; indexing, ranking, actions and broad data-source integration remain. |
| Portals | 47 | Source is present; broad third-party interoperability and permission behavior are not proved. |
| Clipboard | 65 | Real selection paths exist; large transfer/cancellation/format breadth need more QA. |
| Cross-app drag and drop | 51 | In-app Finder movement is real; complete Wayland data-device workflows remain. |
| SLOPOS Spaces model | 71 | Dynamic naming/order/membership/output policy and persistence tests exist. |
| SLOPOS Spaces product UI | 25 | Shell still lacks the requested overview, drag between Spaces, gestures, Settings and polished transitions. |
| Multi-monitor desktop product | 43 | Policy/types exist; complete live topology behavior is not established. |
| **Shell/desktop overall** | **57** | A real custom shell alpha, not a finished desktop product. |

---

## 8. Toolkit, renderer, text, fonts and accessibility

| Area | Score | Current truth |
|---|---:|---|
| Widget toolkit | 67 | Real layout, focus, pointer dispatch, capture and many controls; inconsistent depth and painter coupling remain. |
| SDK/application framework | 65 | Real window/menu/control routing and event wake paths; large monolithic presenter and platform leakage remain. |
| General renderer | 49 | WGPU/immediate path works; retained resources, image textures, batching and sophisticated clipping/effects are incomplete. |
| Unicode shaping API | 66 | `cosmic-text`-backed shaping APIs are present and consumed; not yet the sole authoritative layout/editing system. |
| Text editing model | 61 | UTF-8-safe selections and insertion exist; grapheme, bidi, IME, visual lines and shaped caret geometry remain. |
| Font infrastructure | 70 | Recursive discovery, install/hash/duplicate handling, enable state, roles and profiles are substantial. |
| Font product integration | 36 | No Settings Font Manager, no authoritative live renderer role resolution and no complete metadata/variable-axis product. |
| Accessibility infrastructure | 55 | AT-SPI mapping/actions/events/component/text code exists. |
| Accessibility daily-driver usability | 38 | Snapshot tree, shallow nesting, incomplete live updates and incomplete Orca workflows. |
| **Platform layer overall** | **58** | Good foundations, but renderer/text/font/accessibility integration is a major release blocker. |

---

## 9. First-party application audit

| Application | Functional | UI/UX | Overall | Current truth |
|---|---:|---:|---:|---|
| Finder | 63 | 59 | **61** | Real navigation, directory listing, file operations, trash and in-window drag-to-folder. Missing mature list/column/gallery modes, search, mounts, thumbnails, associations, conflict UI and undo. The sidebar still incorrectly uses the Apple product name `AirDrop`; it must become SLOPOS Share/Nearby Sharing. |
| Settings | 48 | 53 | **50** | Broad categories and atomic persistence; many values are advertised policy rather than authoritative live service control. No Fonts, Spaces or zoom-policy product panels. |
| TextEdit | 67 | 61 | **64** | Selection-aware cut/copy/paste, caret insertion, find selection, save/recovery, focus and undo/redo now work in source/tests. Undo remains whole-string snapshots; no shaped multiline editor, IME, rich text or mature document model. |
| Terminal | 72 | 65 | **69** | Real PTY, parser, tabs, alternate screen, selection, resize, event-loop wake and child shutdown. Cell model is still one Rust `char` per cell; CJK width, combining marks, graphemes and full terminal compatibility remain. |
| App Store | 46 | 48 | **47** | Searchable local catalogue and hardened archive installation exist. Catalogue is still explicitly a stub; signing, trust, publisher authenticity, network delivery, updates, removal and rollback UX are incomplete. Product name also needs a trademark review. |
| Preview | 57 | 39 | **48** | Bounded decoding, zoom state, Vision client and artifact handling are real. Image display is a stretched 96×64 panel mosaic, making it unsuitable as a production viewer. |
| **Application suite** | **59** | **54** | **57** | Useful native alpha applications, not daily-driver replacements yet. |

### Application naming/legal defect

The current Finder sidebar contains a visible `AirDrop` string. It must not ship
as the name of a SLOPOS feature. The native feature should be called **SLOPOS
Share** or **Nearby Sharing**, with SLOPOS-to-SLOPOS discovery, authenticated
encrypted transfer, consent UI, resume, integrity verification and safe atomic
save. Optional Apple-device interoperability must be labelled experimental and
kept separate from the native protocol.

`Finder` and `App Store` should also receive a deliberate product-name/trademark
review before public release rather than being assumed safe because they are
familiar labels.

---

## 10. SLOPOS Vision audit

| Vision area | Score | Current truth |
|---|---:|---|
| OCR/segmentation core | 74 | Real PP-OCR/U2Netp preprocessing, inference, decoding, masks and compositing. |
| Model manifest/integrity | 68 | Hash and manifest validation exist; clean-install acquisition and redistributability workflow remain incomplete. |
| Protocol | 72 | Typed local job/asset/error protocol with bounded structures. |
| Client | 70 | Reusable local client and polling paths exist. |
| Daemon | 72 | Session-local Unix socket, bounded jobs/artifacts and local-only operation are substantive. |
| Preview integration | 60 | Real asynchronous requests and result paths; successful polished end-to-end output is not broadly proved. |
| Finder integration | 18 | Requested native context actions and workflow are not complete. |
| Accuracy/evaluation | 22 | No sufficiently documented labelled benchmark, calibration or demographic/device evaluation. |
| Performance/acceleration | 31 | CPU path exists; production acceleration, memory and cancellation benchmarks are incomplete. |
| Model distribution | 34 | No complete clean-clone import/download/licence acceptance/update path. |
| **Vision product overall** | **58** | Serious local subsystem alpha; product proof and distribution remain the bottlenecks. |

SLOPOS Vision remains separate from Loom. A neutral portable engine may later be
shared, but Loom itself is not part of the SLOPOS-I workspace or desktop
architecture.

---

## 11. Security, quality and release engineering

| Area | Score | Current truth |
|---|---:|---|
| Rust architecture | 72 | Clear crate intent and improving shared policy; several central files remain very large. |
| Error handling | 72 | Session, filesystem, installer, Vision and compositor failure handling have improved materially. |
| Filesystem safety | 78 | Atomic writes, bounded paths, symlink checks, hashes, rollback and runtime identity hardening are common. |
| Session isolation | 82 | Private runtime/socket/token/process-group design is one of the strongest areas. |
| Application sandbox/permissions | 27 | No mature general application sandbox or capability permission model. |
| Package trust/signing | 24 | Integrity checks exist; publisher authenticity and trust-chain product do not. |
| Automated testing | 82 | Broad unit/integration coverage and exact-commit compositor tests. |
| CI quality | 87 | Linux build/test/Clippy/release/fmt/lockfile and headless compositor runtime gates are green. |
| Runtime QA breadth | 61 | Useful UTM/VM evidence; current-head hardware/app compatibility matrix is incomplete. |
| Performance engineering | 54 | Frame scheduling and idle intent improved; rendering architecture remains expensive. |
| Packaging/install/upgrade | 45 | Artefacts exist; clean install, login, upgrade, recovery and uninstall gates need current evidence. |
| Documentation discipline | 73 | Three-source rule is correct; this update replaces a materially stale TRUTH ledger. |
| **Quality/release overall** | **65** | Stronger engineering discipline than product maturity. |

---

## 12. POSIX, FreeBSD and SLOPOS-II truth

### SLOPOS-I portability

| Portability area | Score | Truth |
|---|---:|---|
| Portable architecture requirement | 90 | Now normative in `AGENTS.md`. |
| Actual platform abstraction | 24 | No complete `slopos-platform` boundary yet. |
| POSIX-shell release surface | 20 | Existing QA/release scripts still include Bash/GNU assumptions. |
| Linux backend | 68 | Real compositor/session implementation, still incomplete as a product. |
| FreeBSD compile support | 10 | No current verified native build matrix. |
| FreeBSD runtime backend | 3 | No native compositor/session/application evidence. |
| Shared cross-kernel non-regression suite | 5 | Not established. |
| **SLOPOS-I portability overall** | **22** | Correct direction is now frozen; implementation has barely begun. |

The shared desktop currently contains direct Linux assumptions, including
Linux-only compositor modules and application/SDK paths that inspect Linux
facilities such as `/sys`. These must move behind typed platform adapters rather
than being scattered under broad `cfg(unix)` branches.

### SLOPOS-II

| SLOPOS-II area | Score | Truth |
|---|---:|---|
| Scope definition | 90 | The sole objective is now clearly defined: add a first-party Rust kernel while retaining Linux and FreeBSD. |
| Kernel implementation | 0 | No kernel source. |
| POSIX system interface/libc | 0 | No implementation. |
| Drivers/filesystems/networking | 0 | No implementation. |
| Desktop boot/runtime | 0 | No implementation. |
| Linux non-regression under SLOPOS-II | 0 | Future gate. |
| FreeBSD non-regression under SLOPOS-II | 0 | Future gate. |
| **SLOPOS-II implementation** | **0** | Planned generation only. |

A custom kernel will be called a **POSIX-conformant target** until the kernel,
libc/API surface, shell and required utilities pass applicable conformance
suites. “POSIX certified” is prohibited unless formal certification is actually
obtained.

---

## 13. Correct next implementation order

1. **Finish the Linux compositor to the frozen 100/100 contract.** Complete
   physical input, DnD/IME, multi-output, XWayland, third-party clients,
   HDR/VRR-capable hardware, direct scanout/recovery and long soaks.
2. **Replace the renderer's prototype text/image paths.** Glyph atlas, image
   textures, batching, retained resources, shaped editing geometry and IME.
3. **Connect the product models.** Dynamic Spaces overview/Settings, live font
   profiles/manager, configurable zoom behavior and authoritative settings
   services.
4. **Complete core applications.** Finder view modes/search/associations,
   TextEdit production editor, Terminal Unicode width/compatibility, Preview
   real image path, App Store signing/trust/update/remove.
5. **Complete SLOPOS Vision distribution and evaluation.** Clean model-pack
   install, licences, benchmarks, successful Finder/Preview workflows.
6. **Freeze the platform boundary.** Isolate Linux services, convert portable
   release scripts to POSIX `sh`, add Linux musl and FreeBSD CI.
7. **Ship SLOPOS-I on Linux, then FreeBSD without a fork.** SLOPOS-II must not
   begin by destabilising unfinished SLOPOS-I foundations.
8. **Start SLOPOS-II as a separate kernel programme using the frozen platform
   contract.** Linux, FreeBSD and the SLOPOS kernel all remain release-blocking.

---

## 14. Bottom line

SLOPOS-I is now a **63/100 custom desktop environment alpha**.

It has a credible sovereign compositor/session architecture, a distinctive UI,
real first-party applications, a serious local Vision subsystem and strong
Linux CI discipline. The last development wave materially improved compositor
state contracts, headless runtime proof, terminal lifecycle and TextEdit
selection/editing.

It is not yet a complete Linux desktop environment. The largest blockers are
production rendering, complete compositor hardware/application compatibility,
Spaces product UX, font integration, accessibility, application depth,
packaging and long-term reliability.

SLOPOS-II is correctly scoped but remains **0/100 implemented**. Its future task
is to add a POSIX-conformant first-party Rust kernel as a third target while the
same SLOPOS desktop continues to pass release gates on Linux and FreeBSD.
