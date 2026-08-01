# TRUTH.md — SLOPOS-I Audit and Evidence Ledger

**Purpose:** This is the sole factual status and audit document. It records what
exists in source, what has actually been built/tested/run, current defects, and
the next acceptance gate. Product requirements live in `AGENTS.md`.

**Snapshot audited:** `retroshell(2).zip`  
**Git HEAD:** `5ed6f74f700ead25ccfbd4a9c81ef3226ae73203`  
**Commit:** `5ed6f74 feat(slopos-i): implement window presentation state machine, zoom policy, and shared font service`  
**Branch in snapshot:** `docs/program-design`  
**Audit date:** 2026-08-01  
**Audit type:** source/archive/static validation; no independent Rust build or runtime execution in the audit environment.

## 1. Evidence language

| Label | Meaning |
|---|---|
| **PLANNED** | Requirement only; no implementation claim. |
| **SOURCE PRESENT** | Relevant code exists, but was not built after the audited edit. |
| **BUILD VERIFIED** | Named build command passed in a recorded environment. |
| **TEST VERIFIED** | Named tests passed after the relevant edit. |
| **RUNTIME OBSERVED** | Raw runtime evidence demonstrates the behavior. |
| **HARDWARE VERIFIED** | Behavior was exercised on applicable physical hardware. |

A higher label requires actual evidence. Source, unit tests, generated tables,
or state mutation in a test do not prove visible runtime interaction.

## 2. Independent audit limits and static checks

The audit environment did not contain `cargo` or `rustc`, so it did not compile,
test, launch, or benchmark this snapshot. Runtime claims inherited from prior
agent reports remain historical claims until repeated with raw artifacts.

The following checks were independently completed on this snapshot:

- repository and commit inspection;
- source tracing across compositor, session, SDK, renderer, fonts, Vision,
  shell, apps, portals, App Store and packaging;
- `git diff --check` passed;
- Bash syntax validation passed for 25 shell scripts;
- Python bytecode compilation passed for both Python scripts;
- TOML parsing passed for 42 TOML files;
- all four bundled Vision support/model files matched the SHA-256 values in
  `models/vision/manifest.toml`;
- approximately 61,747 Rust source lines were present under `crates/` and `apps/`.

These are static facts, not a successful Rust build.

## 3. Executive status

SLOPOS-I has a credible early compositor/session foundation and a substantial
local Vision core, but it is not yet a complete window manager, Spaces product,
font/text platform, Vision service, or KDE/GNOME-class daily-driver desktop.

Strongest current source progress:

- `slopos-session` replaces the old shell-supervises-compositor topology and
  avoids glob-deleting host Wayland sockets.
- the compositor has real cursor composition and move/resize geometry paths in
  nested and DRM code;
- private compositor socket routing was previously reported across first-party
  clients;
- `slopos-vision` contains real OCR, U2Netp segmentation, mask post-processing,
  alpha compositing, manifest/hash validation and cancellation checks.

Largest current blockers:

- move/resize request validation ignores the supplied Wayland seat/serial;
- XDG popups and correct layer-shell work-area layout are incomplete;
- new window-presentation policy types are not wired into live SDK/compositor behavior;
- SLOPOS Spaces remains a fixed eight-workspace prototype;
- `slopos-fonts` is disconnected and the visible text renderer is not production-grade;
- Vision protocol/client/daemon and Preview are placeholders;
- XWayland movement/resizing and several portal/application paths remain incomplete;
- a clean clone has no implemented model-pack acquisition/install flow;
- no independent build/runtime result exists for this exact snapshot.

## 4. Current component matrix

| Component | Status | Evidence and current truth |
|---|---|---|
| Workspace/build | **SOURCE PRESENT** | 18 workspace members in `Cargo.toml`; exact snapshot not independently compiled. |
| `slopos-session` | **SOURCE PRESENT** | Starts compositor, waits for private socket, launches shell, separates host/client display, process-group teardown. Needs per-session nonce directory and signal handling. |
| Private socket routing | **RUNTIME OBSERVED, historical report** | Prior VM report claimed all six clients on `wayland-2` and host saw one outer window. Not independently repeated after HEAD `5ed6f74`. |
| Visible cursor | **SOURCE PRESENT** | Nested and DRM cursor-state/composition paths exist. Human-visible behavior on current HEAD not independently observed. |
| Native move/resize | **SOURCE PRESENT** | Grab state and geometry/damage updates exist. Seat/serial validation is incorrect; manual pointer sequence unverified. |
| Focus/raise | **SOURCE PRESENT** | Geometry/stack focus code exists; XDG Activated synchronization needs correction. |
| Minimize/restore | **SOURCE PRESENT, incomplete** | Hidden/minimized Boolean exists; Dock model and robust restore flow are incomplete. |
| Maximize/fullscreen | **SOURCE PRESENT, basic** | Old raw-output geometry path exists. Work-area-aware Fill and shared state machine are not live. |
| Configurable zoom control | **PLANNED / scaffolding** | `WindowPresentationState`, `ZoomPolicyConfig` and geometry helpers exist but are not wired through SDK, Settings and both compositor backends. |
| XDG popups | **PLANNED** | `new_popup`, `grab`, and `reposition_request` handlers are empty in the audited nested path. |
| Layer shell | **SOURCE PRESENT, incomplete** | Shell surfaces map, but anchors, margins, requested dimensions and exclusive zones are not implemented authoritatively. |
| XWayland | **SOURCE PRESENT, incomplete** | Basic integration exists; interactive move/resize handlers remain empty. |
| SLOPOS Spaces | **SOURCE PRESENT, prototype** | Fixed `WORKSPACE_COUNT = 8`, switch/filter mapping. No dynamic model, overview, names, assignment, fullscreen Spaces, gestures or Settings. |
| Renderer | **SOURCE PRESENT, immature** | wgpu/immediate drawing exists. Text still expands glyph coverage into many rectangles; expensive and not shaped. |
| Text platform | **PLANNED** | No end-to-end shaping, bidi, fallback, glyph atlas, IME or production selection geometry in the visible SDK path. |
| `slopos-fonts` | **SOURCE PRESENT, disconnected** | Profiles/discovery structures exist. No consumers; discovery is non-recursive; no font database/installation/Settings/render integration. |
| Shell | **SOURCE PRESENT** | Desktop/menu/Dock/search/portals are substantial but contain incomplete paths and must be re-QA'd under compositor sovereignty. |
| Finder | **SOURCE PRESENT** | Native app and shell Finder-like views exist. File operations, explicit desktop view, MIME/thumbnail/application integration need systematic QA. |
| Settings | **SOURCE PRESENT, partial** | Existing panels and best-effort backends. No live font, Spaces, zoom-policy or complete display/color controls. |
| TextEdit | **SOURCE PRESENT, partial** | App exists; blocked by production text/editing/save/recovery requirements. |
| Terminal | **SOURCE PRESENT, partial** | App exists; PTY, resize, Unicode, selection and lifecycle require runtime QA. |
| App Store | **SOURCE PRESENT, prototype** | Catalog and install-related code exists; update/remove/confirm/signing/trust/atomic replacement remain incomplete. |
| Preview | **PLANNED** | Current `apps/preview` logs “not yet implemented” and exits. |
| Vision core | **SOURCE PRESENT, substantial V1** | Real PP-OCRv4/U2Netp inference path, decoding, mask processing and hashing. Exact build/output not independently reproduced. |
| Vision protocol/client | **PLANNED** | Placeholder crates without the required typed job API. |
| `slopos-visiond` | **PLANNED** | Logs “not yet implemented” and exits. |
| Finder/Preview Vision UX | **PLANNED** | No complete daemon, overlay, clipboard or context-menu flow. |
| Packaging | **SOURCE PRESENT, unverified** | Arch, Debian, session and ISO artifacts exist; clean install/login/ISO boot are not verified for current HEAD. |
| HDR/VRR | **PLANNED / hooks only** | No current physical-hardware evidence. Do not advertise as working. |
| FreeBSD | **PLANNED portability target** | No runtime evidence; cross-build status for current HEAD is unknown. |

## 5. Critical source findings

### 5.1 Session supervisor

Useful source behavior:

- compositor is started before shell clients;
- shell receives explicit private `WAYLAND_DISPLAY`;
- host `DISPLAY` is removed from clients by default;
- known readiness file is removed rather than every `wayland-*` socket;
- compositor and shell use process groups and one side tears down when the other exits.

Remaining defects:

- one global readiness path can race across concurrent same-user sessions;
- readiness does not cryptographically or structurally prove it came from the
  exact child process just launched;
- no complete signal-aware supervisor shutdown path was found;
- backend naming still conflates `nested`/`winit` with the actual Smithay X11 nested backend.

Next acceptance gate: unique per-session runtime directory and nonce, signal
handling, clean process reaping, then a clean socket/process/runtime capture.

### 5.2 Compositor protocols and input

Move/resize code is now more than a fake shell rectangle, but handlers accept a
request based on a global left-button state while ignoring the supplied seat
and serial. This violates the intended Wayland implicit-grab contract and can
accept stale or unrelated requests.

Other blockers:

- XDG popup creation, grab and reposition handlers are empty;
- layer surfaces are configured without complete anchor/margin/exclusive-zone layout;
- authoritative work area is therefore unavailable for Fill/tiling;
- focus does not reliably clear/set XDG Activated state with configures;
- hit testing is based mainly on stored top-level rectangles rather than full
  surface trees, input regions, subsurfaces and popups;
- XWayland move/resize requests remain empty.

### 5.3 Window presentation

`crates/slopos-compositor/src/window_state.rs` defines useful policy structures:

- `WindowPresentationState`;
- `WindowRestoreState`;
- `ZoomAction`;
- `ZoomPolicyConfig`;
- presentation geometry helpers and tiling placements.

In the audited live code these values are initialized but do not drive real
state transitions. The existing maximize/fullscreen function still stores an
older restore rectangle and expands to raw output geometry. The SDK green/zoom
box and title-bar double click still directly toggle `winit` maximize behavior.
The DRM path does not share the new model.

Status: **scaffolding, not end-to-end functionality**.

### 5.4 SLOPOS Spaces

Current behavior is fixed virtual desktops with visibility filtering. It is not
the requested user-controlled Spaces product. Missing: dynamic creation,
naming, order, overview, drag between Spaces, app assignment, fullscreen
Spaces, per-display/spanning policy, persistence, gestures and Settings UI.

### 5.5 Text and fonts

`slopos-fonts` currently has no consumers. Discovery reads only one directory
level, so common nested Linux font directories are missed. There is no complete
font metadata database, TTC face enumeration, variable axes, validation,
installation, duplicates, enable/disable, role resolution, fallback chain or
live Settings integration.

The visible renderer still selects one generic sans face globally and the SDK
iterates characters individually. Covered glyph pixels are emitted as drawing
rectangles, causing poor scaling and high vertex cost. `cosmic-text` is present
as a dependency but is not the authoritative visible text path.

### 5.6 SLOPOS Vision

The core crate is real and should be preserved. It includes:

- PP-OCRv4 detection preprocessing and DB post-processing;
- perspective rectification and recognition preprocessing;
- CTC decoding and line grouping;
- U2Netp inference;
- mask resize/cleanup/feathering and RGBA compositing;
- manifest loading, hash checks and guarded image dimensions.

Model/support files in the audited archive matched:

```text
ch_PP-OCRv4_det_infer.onnx  d2a7720d45a54257208b1e13e36a8479894cb74155a5efe29462512d42f49da9
ch_PP-OCRv4_rec_infer.onnx  48fc40f24f6d2a207a2b1091d3437eb3cc3eb6b676dc3ef9c37384005483683b
ppocr_keys_v1.txt           28b2362ad4ab2dc38769aa72feb535e3a9ddb3fd2a7585a05920e6393b1dc7f7
u2netp.onnx                 309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8
```

Remaining issues:

- protocol/client/daemon/Preview are placeholders;
- model default path is repository-relative and unsuitable for packaged sessions;
- ignored weights have no implemented model-pack acquisition/import script;
- output tensor dimensions need strict validation to avoid assertion/panic paths;
- cancellation is cooperative, not interruptible during a long inference call;
- `model_status` suppresses some verification errors;
- encoded input file size needs a bound before reading the entire file;
- model attribution/notice packaging is incomplete.

### 5.7 Applications and services

- App Store update/remove/confirm/signature/trust flows are incomplete.
- Screen-cast and other portal paths include protocol-level scaffolding without
  full production streams.
- Settings has multiple private/config/CLI paths and must prove that each visible
  control affects the authoritative live subsystem.
- Finder/SDK icon and desktop logic still includes brittle label/size heuristics.
- `UiRuntime::tick()` still calls window update on every wake; previous agent
  measurements reported high shell CPU under LLVMpipe, so current idle behavior
  needs a fresh profile.

## 6. Historical runtime claims retained with caution

These values came from prior agent-generated reports on the Linux VM and were
not independently reproduced during the latest static audit. They are retained
so useful evidence is not lost when old Markdown files are deleted.

### Private socket routing report, 2026-07-31

Reported topology:

- `slopos-compositor` PID `103581` owned `/run/user/1000/wayland-2`;
- `slopos-shell`, Finder, TextEdit, Terminal, Settings and App Store reported
  `WAYLAND_DISPLAY=wayland-2` and socket ownership by PID `103581`;
- host `swaymsg -t get_tree` reportedly showed one outer `slopos-compositor` window;
- compositor reportedly mapped five XDG toplevels plus the layer shell.

Reported interactions included a Finder geometry delta from `(64,64,720,480)`
to `(264,164,720,480)`, TextEdit resize to `850x560`, focus/raise, minimize,
maximize and close state changes. No retained recording proved that these were
caused by a valid human pointer sequence, and the current serial-validation
defect weakens the claim.

Reported CPU after an early redraw fix:

- compositor around `1.3%`;
- shell around `98.7%` under LLVMpipe.

That shell result is still unacceptable for an idle desktop and must be rerun
after the current HEAD.

### Earlier “verified” stages and matrices

Legacy documents made broad stage, compatibility and readiness claims. Several
were based on unit tests, source presence, blank/tiny screenshots, or a harness
that hardcoded PASS-like fields without launching apps. Those labels are not
carried forward. Git history preserves the documents for archaeology.

## 7. Immediate next work order

1. Build and test exact HEAD in the Linux VM; capture raw output.
2. Protect and audit current Vision changes before merging other branches.
3. Fix per-session runtime directory/nonce and supervisor signals.
4. Fix seat/serial validation for move/resize.
5. Implement XDG popups and correct layer-shell layout/work areas.
6. Synchronize XDG Activated and surface-tree hit testing.
7. Wire the shared presentation state machine through SDK, nested, DRM and Settings.
8. Replace the text renderer and connect `slopos-fonts` before visual font profiles.
9. Build dynamic SLOPOS Spaces after the core WM state is reliable.
10. Implement Vision protocol/client/daemon, model-pack paths, then Preview/Finder UX.

Do not start broad UI polish, HDR marketing, App Store expansion, or another
capability matrix before steps 1–6 are stable.

## 8. Required next verification commands

Run on the Linux VM from a clean working tree after reviewing uncommitted work:

```bash
git status --short
git rev-parse HEAD
rustc --version
cargo --version
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Then start a clean nested session and capture:

```text
process tree
PID/PPID/start time/command line
per-process WAYLAND_DISPLAY and SLOPOS_* display variables
Unix socket inodes and server owner
host toplevel tree
compositor surface and popup tree
pointer press serial + move/resize request serial
geometry/state before and after input
60-second CPU/RSS/frame/damage sample
clean shutdown and remaining process list
```

Save raw output under `artifacts/qa/<date>-<slug>/` and summarize the result in
this file.

## 9. Audit update template

Append concise entries here rather than creating a new report:

```text
### YYYY-MM-DD — <change or verification>
Environment: <distro, arch, backend, GPU/renderer, toolchain>
Commit: <hash>
Commands: <exact commands or artifacts/qa path>
Result: <evidence label and factual outcome>
Failures: <exact failing tests/runtime defects>
Changed truth: <matrix rows updated>
Remaining risk: <what this evidence does not prove>
```

## 10. Documentation consolidation record

On 2026-08-01, legacy plans, matrices, reports, task briefs, session summaries,
architecture notes and agent-memory Markdown files were consolidated into
`AGENTS.md` and `TRUTH.md`. The old files are intentionally deleted from the
working tree. Their prior contents remain available through Git history.

The repository documentation policy is now:

- `README.md` — introduction and quick start;
- `AGENTS.md` — normative development source of truth;
- `TRUTH.md` — factual audit/evidence source of truth;
- no other Markdown files.
