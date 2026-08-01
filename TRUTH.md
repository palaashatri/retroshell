# TRUTH.md — SLOPOS-I Audit and Evidence Ledger

**Purpose:** This is the sole factual status and audit document. It records what
exists in source, what has actually been built/tested/run, current defects, and
the next acceptance gate. Product requirements live in `AGENTS.md`.

**Original snapshot audited:** `retroshell(2).zip`
**Original archive Git HEAD:** `5ed6f74f700ead25ccfbd4a9c81ef3226ae73203`
**Current Git HEAD:** r16 evidence commit on `docs/program-design` (resolve the exact hash with `git log -1`)
**Current working tree:** clean after the r16 Spaces, font-discovery, Preview-output, and QA-evidence commit
**Audit date:** 2026-08-01  
**Audit type:** source review plus Ubuntu Server VM build/test/runtime verification and UTM visual QA.

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

The original archive audit environment did not contain `cargo` or `rustc`, so
that audit did not compile, test, launch, or benchmark the archive. The current
working tree was subsequently copied to and hash-matched against the Ubuntu VM
QA tree; current build and runtime evidence is recorded in section 11.

`cargo-deny` was not installed in the VM and was not run.

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

These particular checks are static facts; the current VM build, test, and
runtime evidence is recorded in section 11.

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

The current working-tree P1 slice also establishes the compositor-owned desktop
surface model: the shell uses full-output/background, full-width menu, Dock,
and popup layer surfaces; ordinary application windows remain compositor-owned
XDG toplevels; and the global menu follows the compositor's focused app.

Largest current blockers:

- pointer-driven move/resize, full popup compatibility, and third-party client
  protocol coverage still need dedicated runtime interaction evidence;
- new window-presentation policy types are not wired into all live SDK/compositor
  transitions;
- SLOPOS Spaces remains a fixed eight-workspace prototype;
- `slopos-fonts` is disconnected and the visible text renderer is not production-grade;
- Vision protocol/client/daemon and Preview now have typed/current source and
  focused tests, but successful model inference, packaged model import, and
  complete Finder/Preview UX remain open;
- XWayland movement/resizing and several portal/application paths remain incomplete;
- a clean clone has no implemented model-pack acquisition/install flow;
- current VM evidence does not prove physical hardware behavior, XWayland
  compatibility, or the complete application/Spaces/text requirements.

## 4. Current component matrix

| Component | Status | Evidence and current truth |
|---|---|---|
| Workspace/build | **BUILD VERIFIED, TEST VERIFIED** | Fresh current-source host verification for the r16 slice ran `cargo check --workspace --all-targets` with 0 errors, `cargo test --workspace` with 834 passed across 37 suites, and all-features Clippy with 0 errors. `cargo deny` is unavailable (`no such command: deny`). |
| `slopos-session` | **RUNTIME OBSERVED** | DRM session published a per-session runtime directory, readiness token, actual `1280x800` output dimensions, and private client socket; exact session termination removed its runtime directory and left no SLOPOS processes. |
| Private socket routing | **RUNTIME OBSERVED** | Compositor owned private `wayland-1`; shell and first-party clients were launched from readiness-provided private runtime variables. The artifact records the socket inode/owner and process tree. |
| Visible cursor | **RUNTIME OBSERVED** | UTM captures show a visible cursor over application content. |
| Native move/resize | **TEST VERIFIED; runtime interaction unverified** | Grab state, geometry/damage updates, and seat/serial ownership checks are present; the compositor test `pointer_grab_requires_live_same_surface_press_and_owned_seat` passed. The fresh r15 UTM run still has no human pointer-driven move/resize sequence because input capture reported `noWindowsAvailable`. |
| Focus/raise | **RUNTIME OBSERVED** | Focus artifacts show Settings and Terminal becoming active; `active-toplevel` and the global menu changed with compositor focus. |
| Minimize/restore | **RUNTIME OBSERVED, partial** | The fresh r15 Finder run sent typed `Minimize` and `Restore` requests to the exact session control socket; the window disappeared and returned in fresh UTM captures. Dock-click restore and broader lifecycle coverage remain incomplete. |
| Maximize/fullscreen | **RUNTIME OBSERVED, partial** | The fresh r15 run applied compositor-owned `Fill` and recorded `state=Filled`; fullscreen and the complete shared presentation policy remain unverified. |
| Configurable zoom control | **PLANNED / scaffolding** | `WindowPresentationState`, `ZoomPolicyConfig` and geometry helpers exist but are not wired through SDK, Settings and both compositor backends. |
| XDG popups | **SOURCE PRESENT, partial runtime** | Popup handling and shell popup overlay paths are present; the QA compositor mapped the popup overlay layer. Full third-party popup grabs/repositioning remain unverified. |
| Layer shell | **RUNTIME OBSERVED** | DRM logs show compositor-owned desktop background, full-width menu, Dock, and overlay surfaces; the output was `1280x800` and the UTM canvas had no surrounding shell window. |
| XWayland | **SOURCE PRESENT, TEST VERIFIED; runtime unverified** | The nested XWayland move/resize bridge and Linux edge-mapping regression test exist; no real X11 client exercised the bridge in UTM. |
| SLOPOS Spaces | **BUILD VERIFIED, TEST VERIFIED, model expanded** | The model now persists per-Space output metadata, supports shared-span versus independent-per-display policy, deterministic output restore/migration, and safe removal fallback. Shell/compositor overview UI, gestures, fullscreen Space policy, and live Settings integration remain open. |
| Renderer | **SOURCE PRESENT, immature** | wgpu/immediate drawing exists. Text still expands glyph coverage into many rectangles; expensive and not shaped. |
| Text platform | **PLANNED** | No end-to-end shaping, bidi, fallback, glyph atlas, IME or production selection geometry in the visible SDK path. |
| `slopos-fonts` | **BUILD VERIFIED, TEST VERIFIED, disconnected** | Discovery now recursively walks regular directories, preserves search-root precedence, skips symlinked files/directories, and canonical-path deduplicates overlapping roots. Font metadata, installation, role resolution, renderer integration and Settings remain open. |
| Shell | **RUNTIME OBSERVED** | Shell painted only shell chrome/overlays; no production fake Finder window was started. Global menu content followed the focused real client. |
| Finder | **RUNTIME OBSERVED** | `SLOPOS_START_APP=finder` launched the real Finder client; the UTM capture shows the Finder window on the compositor-owned desktop. |
| Settings | **RUNTIME OBSERVED, partial** | Settings rendered as a separate client in the multi-window UTM capture; live font, Spaces, zoom-policy, and complete display/color controls remain incomplete. |
| TextEdit | **RUNTIME OBSERVED, partial** | A separate TextEdit client was visible in the multi-window UTM capture; production text/editing/save/recovery remains incomplete. |
| Terminal | **RUNTIME OBSERVED, partial** | A separate Terminal client rendered and took focus in UTM; PTY, resize, Unicode, selection and lifecycle still need dedicated QA. |
| App Store | **RUNTIME OBSERVED, prototype** | A separate App Store client was visible in UTM; update/remove/confirm/signing/trust/atomic replacement remain incomplete. |
| Preview | **BUILD VERIFIED, TEST VERIFIED; RUNTIME OBSERVED, partial** | Current-source Preview is covered by the fresh workspace run and now bounds/sanitizes atomic Vision output persistence under SLOPOS-owned XDG storage. The r16 UTM Preview action was accepted through the exact app-control socket and visibly remained `Running`; no successful model output or saved artifact was observed. |
| Vision core | **BUILD VERIFIED, substantial V1** | Real PP-OCRv4/U2Netp inference path, decoding, mask processing and hashing compiled in the workspace; model inference output was not exercised in this QA run. |
| Vision protocol/client | **BUILD VERIFIED, TEST VERIFIED** | Current-source focused check and Clippy pass; protocol 8 tests and client 9 tests pass. The typed local-only job/asset/error path is source-backed, but no successful model job was observed through the daemon in this run. |
| `slopos-visiond` | **BUILD VERIFIED, TEST VERIFIED; RUNTIME OBSERVED, startup and active job** | Current-source daemon check, Clippy, and 3 tests pass. The r16 guest launched a session-scoped daemon, loaded the OCR engine, and kept the small Preview job active; the runtime `vision-artifacts` directory remained empty, so successful inference is not claimed. |
| Finder/Preview Vision UX | **SOURCE PRESENT, partial** | Preview contains native Vision request/result paths, but Finder context integration, successful inference display, clipboard/save output, packaged model import, and complete UX remain open. |
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

The current working tree adds a unique session nonce/runtime directory, binds
readiness to the compositor PID/token, forwards actual output dimensions, and
keeps shell/application clients on the exact private socket. A post-fix DRM run
also exercised compositor death and `SIGHUP`: both paths tore down the shell and
client process groups and removed the unique session directory, leaving only
the shared `slopos-i` root.

Remaining defects:

- backend naming and full display-manager/hardware lifecycle coverage remain
  separate acceptance work.

Next acceptance gate: validate the same session contract under nested mode and a
real display-manager launch, then capture pointer-driven window interaction.

### 5.2 Compositor protocols and input

The current working tree validates the requesting seat and live pointer-press
serial for interactive move/resize, keeps input routing separate from shell
paint routing, and updates mapped-window app IDs when clients announce them
after mapping. It also configures compositor-owned desktop/menu/Dock/overlay
layer surfaces from the actual output dimensions, publishes authoritative
focus through `active-toplevel`, and routes global-menu changes from focus
instead of a timer. The DRM compositor no longer launches `slopos-shell`; the
session supervisor is the sole shell owner.

The r15 runtime logs observed all four shell layer surfaces and the UTM captures
showed a real Finder client, focus/menu chrome, Fill, Minimize, Restore, and
scale behavior. The automated compositor tests covered pointer-grab ownership
and surface-tree paint/input separation. A full human pointer move/resize
sequence, third-party popup grab/reposition compatibility, and XWayland
interaction remain unverified.

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

The current source now contains typed local-only protocol/client/daemon paths and
a native Preview viewer. Current-source focused checks cover 26 tests across
Preview, the client, protocol, and daemon, plus 63 Vision-core tests. These
checks do not prove successful inference with the packaged weights or visible
Vision output.

Remaining issues:

- the core `VisionEngineConfig::default()` remains repository-relative; the
  daemon/session path prefers `$XDG_DATA_HOME/slopos-i/models/vision`, then
  `$HOME/.local/share/slopos-i/models/vision`, with a relative fallback;
- ignored weights have no complete model-pack acquisition/import flow, and
  model attribution/notice packaging is incomplete;
- real OCR/segmentation output, bounded worker/resource behavior under load,
  and cancellation during a long inference call remain unverified;
- Finder has no complete Vision context-action integration; Preview does not
  yet prove successful overlay, clipboard, and save output through the daemon.

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

The r3 run superseded this historical sample with a 60-second multi-client
sample: shell CPU was approximately 5.1–5.6% and compositor CPU approximately
8.4–8.6%, with active application clients present. This is evidence of a
measured run, not a near-zero idle claim; a clean idle benchmark remains open.

### Earlier “verified” stages and matrices

Legacy documents made broad stage, compatibility and readiness claims. Several
were based on unit tests, source presence, blank/tiny screenshots, or a harness
that hardcoded PASS-like fields without launching apps. Those labels are not
carried forward. Git history preserves the documents for archaeology.

## 7. Immediate next work order

1. Validate the private-session contract in nested mode and through a
   display-manager launch.
2. Capture a real pointer-driven move/resize sequence, popup grab/reposition
   sequence, and representative XWayland interaction.
3. Record a clean idle compositor/shell benchmark after the current runtime fix.
4. Protect and audit current Vision changes before merging other branches.
5. Wire the shared presentation state machine through SDK, nested, DRM and Settings.
6. Replace the text renderer and connect `slopos-fonts` before visual font profiles.
7. Build dynamic SLOPOS Spaces after the core WM state is reliable.
8. Exercise the current Vision protocol/client/daemon with a verified model pack,
   then complete Preview/Finder Vision UX and packaged model provisioning.

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

## 11. 2026-08-01 — compositor-owned desktop/session architecture fix and UTM QA

Environment: Ubuntu 26.04 LTS, aarch64, UTM guest on the macOS host; DRM
backend; `1280x800` output; llvmpipe renderer; Rust 1.97.1 / Cargo 1.97.1.
The 13 modified source files were SHA-256 matched between the host working tree
and `/home/ubuntu/qa/2026-08-01-compositor-p1` before verification. Git HEAD was
`afef105d2b2dd0c45bc269a16930ad04d3185b56`; the fix remains uncommitted.

Commands: `artifacts/qa/2026-08-01-utm-architecture-fix-r3/verification-vm.txt`;
runtime logs, readiness, socket ownership, process tree, focus transitions, and
60-second CPU/RSS sample in the same directory. Visual evidence:
`utm-finder.jpeg`, `utm-multi-window-settings.jpeg`, and
`utm-focused-terminal.jpeg`.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED**.

- The compositor published `wayland-1` in a unique session directory and
  readiness included the compositor PID/token and actual `1280x800` dimensions.
- DRM logs showed four compositor-owned layer surfaces: full-output desktop
  background, full-width global menu, Dock, and menu-popup overlay.
- `SLOPOS_START_APP=finder` started the real Finder client. Finder, Settings,
  TextEdit, Terminal, and App Store appeared as separate compositor-managed
  clients; no fake Finder XDG desktop window was used.
- The UTM captures show the desktop canvas filling the guest output, a full-width
  global menu, visible cursor, bottom Dock, and no surrounding `640x480` shell
  surface. The Finder menu showed Finder-specific controls; the focused Terminal
  capture showed Terminal-specific controls; the Settings capture showed the
  Settings menu title after focus changed.
- Focus artifacts and `active-toplevel` showed Settings then Terminal focus;
  the global menu changed with focus without the previous two-second throttle.
- The 60-second multi-client sample recorded approximately 5.1–5.6% shell CPU
  and 8.4–8.6% compositor CPU. This is not a clean idle benchmark.
- Exact guest shutdown left no matching SLOPOS/application processes and removed
  the session runtime directory.

Known failures and limitations: guest `grim` could not capture because the
compositor does not implement `wlr-screencopy-unstable-v1`; the SIGUSR1
compositor capture failed on llvmpipe because `AR24` framebuffer readback is
unsupported. The session log also records the VM's missing AT-SPI registry and
portal-registration warning. `cargo-deny` was unavailable. This run did not
prove a human pointer-driven move/resize, full third-party popup grab/reposition
behavior, XWayland interaction, nested backend behavior, physical DRM hardware,
or completion of the remaining P2–P10 product requirements.

Changed truth: P1 session identity/cleanup, compositor-owned shell layer layout,
real-client startup, app-ID propagation, authoritative focus publication, and
focus-driven global-menu routing are now source- and VM-runtime-backed for this
working tree. The desktop remains experimental rather than a complete daily
driver until the remaining acceptance gates are exercised.

## 12. 2026-08-01 — session lifecycle and single-shell correction

Environment: Ubuntu 26.04 LTS, aarch64 UTM guest; DRM backend; llvmpipe;
`1280x800`; Rust 1.97.1 / Cargo 1.97.1. The updated
`crates/slopos-compositor/src/session_drm.rs` was hash-matched between host and
guest before the post-fix build.

Change: removed the unconditional DRM compositor call that spawned
`slopos-shell`. `slopos-session` is now the sole shell owner; compositor key
actions may still launch explicit clients such as Finder or the lock screen.

Commands and raw evidence:
`artifacts/qa/2026-08-01-utm-p1-shell-single-r1/verification.txt`,
`session.log`, `readiness.txt`, `process-tree.txt`, `runtime-files.txt`,
`topology.log`, `duplicate-shell-warning.txt`, and
`shutdown-verification.txt`. The lifecycle artifacts from the compositor-death
and SIGHUP runs are under
`artifacts/qa/2026-08-01-utm-session-lifecycle-r1/`.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED**.

- VM format, workspace check, workspace tests, workspace build, and Clippy
  with `-D warnings` all exited 0 after the edit.
- The fresh DRM session published a private `wayland-1`, actual `1280x800`
  dimensions, exactly one shell child, one real Finder client, and all four
  compositor-owned shell layers. No duplicate `slopos-shell` spawn warning was
  emitted.
- Killing the compositor produced the expected supervisor error path and
  cleaned the shell/client group. Sending `SIGHUP` to the supervisor produced
  the expected shutdown path. Both removed the unique session directory while
  preserving only the shared `slopos-i` root.
- The fresh UTM screenshot shows the full desktop canvas, global menu, real
  Finder window, cursor, desktop icons, and Dock: `utm-finder-post-fix.jpeg`.

Remaining risk: UTM Computer Use returned `noWindowsAvailable` for mouse
actions, and the guest has no input-injection utility, so pointer-driven
move/resize and popup grab/reposition remain unverified. This is recorded as an
evidence limitation, not a PASS. Nested/display-manager behavior, XWayland,
Spaces, production text/fonts, and later P2–P10 work remain open.

## 13. 2026-08-01 — scaled work-area clamp and Classic Macintosh kit QA

Environment: Ubuntu 26.04 LTS, aarch64, UTM guest on the macOS host; explicit
DRM backend; `1280x800` virtual output; llvmpipe; Rust 1.97.1 / Cargo 1.97.1.
Git HEAD was `afef105d2b2dd0c45bc269a16930ad04d3185b56`; the working tree remains
uncommitted. The guest source hashes for the compositor, session, design-token,
and SDK files matched the host before the run.

Commands and raw evidence: all build/test output, process/runtime snapshots,
readiness records, key compositor lines, and shutdown checks are under
`artifacts/qa/2026-08-01-scale-clamp-r8/evidence/`. Fresh UTM captures are
`utm-scale-100.jpeg`, `utm-scale-125.jpeg`, `utm-scale-150.jpeg`, and
`utm-scale-200.jpeg` in that directory.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED**.

- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
  `CARGO_BUILD_JOBS=1 cargo test --workspace`, `cargo build --workspace`,
  and Clippy with `-D warnings` all exited with status 0 in the guest. The
  compositor library test run also passed independently.
- The Figma Classic Macintosh UI Kit Community file was reviewed directly
  (file key `LGMlwNCoVdakZxDBvPKg1W`, example screen node `73:5016`). The shared
  native-kit path now carries the reference's compact 19 px menu bar, 16 px
  menu rows, hard-edged System 7 window treatment, left close/right zoom
  controls, restrained desktop pattern, and palette/token relationships in
  `crates/slopos-kit/src/design_tokens.rs` and the SDK renderer.
- At 1.0× and 1.25× the real Finder client remained at its requested
  `640x480` geometry because it fit the larger logical work area. At 1.5× and
  2.0× the compositor mapped it at `(0,19)` with `640x317`, after the global
  menu and Dock exclusive zones were applied. The fresh captures show the
  full-width global menu, full desktop backdrop, real Finder content, visible
  cursor, and Dock without the client covering the Dock.
- Every scale run used the private compositor socket and recorded the four
  compositor-owned shell layers: desktop background, global menu, Dock, and
  menu-popup overlay. Each run was terminated with `SIGTERM`; the shutdown
  artifacts show no remaining SLOPOS process or per-session runtime directory.

Remaining risk: this is runtime evidence for the DRM/llvmpipe UTM guest, not
physical display hardware. The captures do not prove pointer-driven
move/resize, third-party popup grabs, XWayland, nested mode, or production
text/font behavior. `slopos-kit` remains renderer-neutral while `slopos-sdk`
owns the current native pixel paint path; the shared token and SDK styling are
the current Figma-inspired visual foundation, not a claim that P2 text/font
work is complete.

## 14. 2026-08-01 — native kit global-menu wake path and UTM post-fix QA

Environment: Ubuntu 26.04 LTS, aarch64 UTM guest on the macOS host; explicit
DRM backend; `1280x800` virtual output; llvmpipe; Rust 1.97.1 / Cargo 1.97.1.
Git HEAD was `afef105d2b2dd0c45bc269a16930ad04d3185b56`; the working tree
remains uncommitted. The guest verification copy was
`/home/ubuntu/qa/2026-08-01-native-kit-r9`.

Change: application control sockets now wake SDK clients that are parked in
`ControlFlow::Wait`, so focused global-menu actions are delivered to the real
application event loop. The shared native kit continues to own the
Classic-inspired palette and metrics; the SDK consumes those tokens for the
current native pixel presentation path. The visual direction was reviewed
against the supplied Classic Macintosh UI Kit Community file
(`LGMlwNCoVdakZxDBvPKg1W`, example node `73:5016`) without copying proprietary
assets or fonts.

Commands and raw evidence:
`artifacts/qa/2026-08-01-utm-native-kit-r9/evidence/` contains the guest
format/check/test/build/Clippy logs, environment, session log, private-socket
listing, runtime files, CPU/RSS sample, app logs, and fresh UTM captures.
The scale matrix remains under
`artifacts/qa/2026-08-01-scale-clamp-r8/evidence/`.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED**.

- Guest `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
  `cargo test --workspace`, `cargo build --workspace`, and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  completed successfully in the post-fix logs.
- The DRM session used a private `wayland-1` socket under a unique runtime
  directory and logged compositor-owned desktop, global-menu, Dock, and
  menu-popup layer surfaces. Finder, Settings, and Terminal were real mapped
  clients rather than shell-painted application windows.
- The fresh captures `utm-finder-postfix.png`,
  `utm-settings-menu-action-postfix.png`, `utm-settings-minimized.png`, and
  `utm-terminal-postfix.png` visibly show a full-width global menu, a
  full-output patterned desktop, client windows constrained to the work area,
  a visible cursor, and a separate bottom Dock. The screenshots were visually
  inspected from the UTM guest captures.
- The Settings global-menu action produced a visible General-pane change and
  `application handled global menu action` in `settings-postfix.log`. The
  Settings Hide/Minimize action produced `state=Minimized` in the compositor
  session log and removed the Settings window from the canvas in the fresh
  capture.
- The 1.0x, 1.25x, 1.5x, and 2.0x scale captures and runtime logs show the same
  full-width shell chrome and work-area clamping at each supported scale.
- The session received SIGTERM and the post-fix log ended with client
  disconnects. A separate restore request did restore the currently focused
  Finder rather than the hidden Settings window; Dock restore therefore
  remains incomplete.

Known failures and limitations: UTM Computer Use returned
`noWindowsAvailable` for pointer actions and the guest had no input-injection
utility, so human pointer-driven move/resize and popup grab/reposition remain
unverified. The active-client CPU sample is not a clean idle benchmark.
Nested mode, display-manager launch, XWayland, physical hardware, production
text/fonts, Spaces, Vision UI/daemon, and later P2–P10 requirements remain
open. `cargo deny` was unavailable in the guest.

Changed truth: the native kit is now a shared, source-backed Classic-inspired
visual foundation used by kit geometry/menu behavior and SDK presentation;
the compositor-owned desktop/window/menu architecture and global-menu action
wake path have UTM runtime evidence. This does not elevate SLOPOS-I to a
complete daily-driver desktop.

## 15. 2026-08-01 — Wayland display-source repair and compositor restore-target QA

Environment: Ubuntu 26.04 LTS, aarch64, UTM guest on the macOS host; explicit
DRM backend; `1280x800` virtual output; llvmpipe; Rust 1.97.1 / Cargo 1.97.1.
The guest source at `/home/ubuntu/qa/2026-08-01-backend-transport-r10` was
hash-matched to the host before the run. The working tree remains uncommitted.

Change: both nested and DRM backends now register the Wayland server display
poll fd with calloop, so client requests are dispatched while the compositor
waits for events. The shared compositor presentation path also retains the
most recently minimized window id. A generic `Restore` request therefore
restores the window that was minimized even after focus has moved to another
client, and returns focus to the restored client.

Commands and raw evidence:
`artifacts/qa/2026-08-01-wayland-display-source-r11/evidence/wayland-display-source-r11/`
contains the guest format, check, test, build, Clippy, focused regression,
environment, readiness, socket, process, resource, application, session, and
shutdown artifacts. Fresh UTM captures are `utm-ubuntu-r11.png`,
`utm-final-r11.png`, `utm-settings-minimized-restorefix-r11.png`, and
`utm-settings-restored-r11.png` in the same QA directory.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED**.

- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
  `cargo test --workspace`, `cargo build --workspace`, and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  all exited 0 after the restore-target edit in the Ubuntu guest.
- The focused Wayland regression command ran
  `wayland_display_source_dispatches_client_requests_when_fd_is_ready` and
  reported `1 passed; 0 failed; 96 filtered out`.
- The fresh DRM session published a unique readiness-bound `wayland-1` socket.
  Logs show compositor-owned Background, Top global-menu, Bottom Dock, and
  Overlay menu-popup layer surfaces, followed by a real Finder toplevel and a
  real Settings toplevel. The UTM captures show the desktop canvas filling the
  guest output with no grey bootstrap-only frame, a full-width app-specific
  global menu, hard-edged native window chrome, visible cursor, patterned
  desktop, and Dock.
- A namespaced Settings menu request sent to the exact private application
  control socket woke the parked SDK event loop; `settings-restore.log`
  records `application handled global menu action`, and the final capture shows
  the Settings global menu and General pane while Finder remains a separate
  compositor-managed window behind it.
- A compositor `Minimize` request removed Settings from the canvas. After focus
  moved to Finder, a generic `Restore` request restored Settings and published
  `app_id=com.slopos.settings` as the active toplevel. The post-fix minimized
  and restored captures are visually inspected UTM window captures.
- The active multi-client 60-second sample contains 12 samples for session,
  compositor, shell, Finder, and Settings in `cpu-memory-active-60s.csv`.
  This is an active-client sample, not a clean idle benchmark.
- SIGTERM stopped the session and direct Settings client; the shutdown artifact
  reports no remaining matching SLOPOS/application process and
  `runtime_exists=no`.

Remaining risk: this is runtime evidence for the UTM DRM/llvmpipe guest, not
physical display hardware. The run still does not prove human pointer-driven
move/resize, third-party popup grabs, XWayland interaction, nested mode,
display-manager launch, production text/font behavior, dynamic Spaces, Vision
daemon/UI, or later P2–P10 requirements. `cargo deny` remains unavailable in
the guest.

Changed truth: Wayland client transport is now regression-tested and observed
in the DRM guest; the global menu is visibly focused-client-specific; and the
generic minimize/restore path no longer follows the wrong newly focused
window. SLOPOS-I remains experimental rather than a complete daily-driver
desktop.

## 16. 2026-08-01 — nested XWayland grab bridge and full UTM QA

Environment: Ubuntu 26.04 LTS, aarch64, UTM guest on the macOS host; explicit
DRM backend; UTM virtual output; llvmpipe; Rust 1.97.1 / Cargo 1.97.1. Git
HEAD was `afef105d2b2dd0c45bc269a16930ad04d3185b56`; the working tree remains
uncommitted. The fresh guest copy was
`/home/ubuntu/qa/2026-08-01-full-qa-r13`.

Change: `crates/slopos-compositor/src/main.rs` now records the XWayland
window-to-Wayland-surface association and routes nested XWayland
`move_request`/`resize_request` through the existing compositor interactive
grab path. The path reuses the same client seat, live pointer-press serial,
same-surface, and button-held authorization checks as native XDG toplevel
interaction. The edge mapping helper has a Linux binary test.

Commands and raw evidence: the fresh runtime/process/socket/readiness,
application logs, CPU/RSS sample, scale-session logs, and shutdown records are
under `artifacts/qa/2026-08-01-full-qa-r13/evidence/`. Fresh UTM captures are
`utm-finder-r13.jpeg`, `utm-multi-window.jpeg`,
`utm-multi-workspace-2.jpeg`, `utm-multi-workspace-1-recheck.jpeg`,
`utm-scale-125.jpeg`, `utm-scale-150.jpeg`, and `utm-scale-200.jpeg` in the
same directory.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED** for the
scoped claims below; the full workspace remains resource-limited in this VM.

- `cargo fmt --all -- --check` passed. The guest `cargo check --workspace
  --all-targets` passed before the final XWayland-only test visibility fix;
  the final guest `cargo check -p slopos-compositor --all-targets` passed.
- Guest `cargo test --workspace --lib` ran 656 tests with zero failures before
  the final XWayland binary-test-only edit. Final targeted tests then ran
  `cargo test -p slopos-compositor --bin slopos-compositor` with 5 passed and
  `cargo test -p slopos-compositor --lib` with 97 passed. Targeted compositor
  Clippy with `-D warnings` passed.
- The exact full `cargo test --workspace` attempt exited 101 while writing
  `winit` because the 30 GB guest filesystem reached `No space left on
  device`. A later full workspace binary build reached the same limit while
  linking TextEdit. The scoped runtime build succeeded with
  `cargo build -p slopos-session -p slopos-compositor -p slopos-shell -p
  finder -p settings -p textedit -p terminal -p appstore`.
- The fresh DRM session published `wayland-1` under a unique readiness-bound
  runtime with `1280x800` output. The process tree and environment records
  show the session, compositor, shell, Finder, TextEdit, Terminal, Settings,
  and App Store clients using the private socket; no host `DISPLAY` was
  passed to the shell/client path. The UTM multi-window capture visibly shows
  five real application clients with compositor-owned classic window chrome,
  global menu, patterned desktop, cursor, and Dock.
- With UTM input capture enabled, `Super+2` switched to an empty workspace and
  `Super+1` restored the application windows. This is runtime evidence for the
  existing fixed workspace shortcut path, not evidence for dynamic SLOPOS
  Spaces.
- Fresh scale runs produced `1280x800` readiness at 1.25× and `640x400`
  readiness at 1.5× and 2.0×, matching the current integer buffer-scale
  quantization policy. The inspected captures show the Finder client and Dock
  remaining visible at each run; 1.0× is represented by the fresh initial
  `utm-finder-r13.jpeg` capture.
- The active five-client resource sample contains 31 samples over 60 seconds.
  Average CPU was 4.42% for the shell, 0.50% for the compositor, 0.34% each
  for Finder and TextEdit, 0.41% for Terminal, 0.28% for Settings, and 0.31%
  for App Store. This is an active-client sample, not a clean idle benchmark.
- The fresh session and scale runs received SIGTERM; the per-session runtime
  directories were removed. The final shutdown record has zero matching
  SLOPOS/application processes and one remaining child under the shared
  `slopos-i` root, which is expected shared runtime state.

Known failures and limitations: after enabling UTM capture, the Computer Use
service rejected the pointer drag with `noWindowsAvailable`; releasing capture
required the UTM shortcut plus a harmless key. Therefore no pointer-driven
move/resize, popup grab/reposition, or Dock click claim is made. The six
additional application clients in this run were launched directly over the
private socket for protocol/visual coverage, then terminated explicitly before
session shutdown; this does not replace a shell-launched lifecycle test. The
new XWayland bridge is source- and Linux-test-verified but was not exercised by
a real X11 client. Nested mode, display-manager login, physical hardware,
production text/fonts, dynamic Spaces, Vision daemon/UI, and later P2–P10 work
remain open.

Changed truth: nested XWayland move/resize now has a shared source path and
Linux edge-mapping regression coverage; UTM DRM visual coverage now includes a
fresh multi-client run, keyboard workspace switching, and current scale
captures. SLOPOS-I remains an experimental/developing desktop and is not a
complete daily-driver.

## 17. 2026-08-01 — current-source Vision checks and fresh UTM r15 QA

Environment: current host source checks used macOS with Homebrew Rust 1.97.1 /
Cargo 1.97.1. Runtime QA used an Ubuntu 26.04 aarch64 UTM guest with the
explicit DRM backend, `Virtual-1` at `1280x800`, seatd, and llvmpipe/virgl.
The guest runtime source tree was `/home/ubuntu/qa/2026-08-01-vision-session-r14`.
Its session/compositor snapshot predates the current `slopos-visiond` and
Preview viewer hashes, so the current-source Vision checks below are distinct
from the guest runtime claims. The r15 source snapshot was recorded before the
r16 integration; current working-tree status is recorded in section 18.

Current-source commands and raw evidence:
`artifacts/qa/2026-08-01-utm-pointer-r15/evidence/` contains
`check-current-source-vision-preview-r15.log`,
`test-current-source-vision-preview-r15.log`,
`test-current-source-vision-core-r15.log`, and
`clippy-current-source-vision-preview-r15.log`.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED** for the
scoped claims below.

- Current-source `cargo check -p slopos-visiond -p slopos-vision-client
  -p slopos-vision-protocol -p preview` exited 0.
- Current-source tests passed with 26 total tests: Preview 6, Vision client 9,
  Vision protocol 8, and Vision daemon 3. `cargo test -p slopos-vision
  --all-targets` passed 63 tests. Current-source targeted Clippy passed with
  `-D warnings`. The commands emitted the existing Rust future-incompatibility
  warning for `block v0.1.6`, but no test or Clippy errors.
- The fresh 1.0× UTM run started the real Finder client through the session
  supervisor. Logs show a session-scoped Vision daemon, compositor-owned
  `wayland-1`, Background/Top/Bottom/Overlay layer-shell surfaces, and a real
  Finder toplevel. The UTM capture visibly shows the patterned desktop,
  full-width global menu, classic Finder chrome, cursor, desktop icons, and
  Dock.
- Typed requests sent to the exact session `control.sock` produced
  compositor log states `Fill -> Filled`, `Minimize -> Minimized`, and
  `Restore -> Normal`. Fresh captures show Finder filling the work area,
  disappearing on minimize, and returning at its original geometry on restore.

The fresh visual captures are stored at
`artifacts/qa/2026-08-01-utm-pointer-r15/`:
`utm-finder-current-r15.jpeg`, `utm-finder-fill-current-r15.jpeg`,
`utm-finder-minimized-current-r15.jpeg`, `utm-finder-restored-current-r15.jpeg`,
`utm-finder-scale-125-current-r15.jpeg`,
`utm-finder-scale-150-current-r15.jpeg`, and
`utm-finder-scale-200-current-r15.jpeg`. They are direct UTM captures, are
1173x768 JPEGs, and were visually inspected from disk after copying.

| Requested scale | Effective buffer scale | Logical canvas | Runtime evidence |
|---|---:|---:|---|
| 1.0× | 1 | 1280x800 | `session-visual-current-r15.log` |
| 1.25× | 1 | 1280x800 | `session-visual-scale-125-current-r15.log` |
| 1.5× | 2 | 640x400 | `session-visual-scale-150-current-r15.log` |
| 2.0× | 2 | 640x400 | `session-visual-scale-200-current-r15.log` |

Each scale run used the private `wayland-1` socket, mapped Finder, and was
terminated with SIGTERM. The guest post-run checks found no SLOPOS process and
no per-session runtime directory. The matrix demonstrates the current
integer-buffer-scale quantization; it is not evidence of true fractional
rasterization. The compositor SIGUSR1 screenshot hook remains broken on this
UTM renderer: the raw r15 diagnostic log records `Unsupported pixel format:
DrmFourcc(AR24)`, so those hook failures are not counted as visual captures.

Remaining risk: UTM input capture still could not produce a human pointer press
through drag, valid serial, geometry delta, and release sequence; therefore
pointer-driven move/resize, popup grabs/repositioning, Dock clicks, and true
input-driven focus changes remain unverified. Nested mode, display-manager
login, physical hardware, XWayland clients, clean idle profiling, full current
workspace verification, real Vision inference, model-pack import, Finder
Vision integration, production text/fonts, dynamic Spaces, and later P2–P10
requirements remain open. SLOPOS-I remains experimental/developing rather than
a complete daily-driver desktop.

## 18. 2026-08-01 — current-source P2/P4/P9 slice and fresh UTM r16 Preview QA

Environment: current host source checks used macOS with Rust 1.97.1 / Cargo
1.97.1. Runtime QA used an Ubuntu 26.04 aarch64 UTM guest with the explicit DRM
backend, `Virtual-1` at `1280x800`, and llvmpipe. The guest source snapshot was
`/home/ubuntu/qa/2026-08-01-luna-max-r16`; its hashes matched the host for the
three edited source files and `Cargo.lock` before runtime launch.

Source changes in this slice:

- `slopos-fonts` now recursively discovers regular font files, keeps search
  root ordering, skips symlinked files/directories, and deduplicates overlapping
  roots by canonical path.
- `SpacesModel` now retains stable output assignment metadata, validates output
  identifiers, models shared-span versus independent-per-display policy, and
  provides deterministic restore, migration, and removal fallback APIs.
- Preview bounds and sanitizes Vision text/PNG persistence beneath an absolute
  SLOPOS-owned XDG data/cache directory, uses collision-free atomic writes, and
  validates encoded size, PNG metadata, dimensions, and media type before save.

Raw current-source evidence is under
`artifacts/qa/2026-08-01-luna-max-r16/evidence/`: host check, test, format,
Clippy, syntax, diff, environment, and cargo-deny logs; guest Preview/session
logs; process tree; private-runtime file listing; Vision-artifact listing; and
shutdown record. Fresh UTM captures are
`utm-desktop-initial-r16.png`, `utm-preview-open-r16.png`,
`utm-preview-vision-running-r16.png`, and
`utm-preview-vision-small-input-r16.png`; the last capture was visually
inspected from disk and shows the real Preview window, desktop menu/Dock, and
the visible `Extract Text submitted (Running); no output is available yet.`
status.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED** for the
scoped claims below.

- Fresh host `cargo check --workspace --all-targets` completed with 0 errors;
  `cargo test --workspace` reported 834 passed across 37 suites; and
  all-features Clippy reported 0 errors. `cargo deny` was attempted and is not
  installed in the host environment.
- The DRM session published a unique readiness-bound private `wayland-1` and
  launched session, compositor, shell, Vision daemon, and Preview processes.
  The process/runtime evidence records the exact session directory and socket;
  the session was terminated with SIGTERM and the post-run check reports
  `runtime_exists=no` with no matching SLOPOS/Preview process.
- The namespaced `com.slopos.preview.vision.extract_text` request was sent
  through the exact Preview app-control socket. Preview visibly accepted it as
  `Running`, the Vision daemon log recorded `OCR engine loaded`, and no file
  appeared in the session `vision-artifacts` directory during the observation.
  Successful OCR inference, saved text, clipboard output, and lifted-subject
  output are therefore not claimed.

Known limitations: UTM input capture stayed enabled and the Computer Use
service could not synthesize the modifier-only release chord, so pointer-driven
move/resize, popup grabs/repositioning, and Dock-click behavior remain
unverified. Mesa/DRI warnings were present in the guest logs but the explicit
DRM/llvmpipe session rendered the fresh captures. This run does not elevate
SLOPOS-I beyond an experimental/developing desktop or prove physical hardware,
true fractional rasterization, production text/font integration, dynamic
Spaces UI, or complete Vision inference.

## 19. 2026-08-01 — current-source parallel slices and UTM r17 Vision wake QA

Environment: host verification used macOS with Rust 1.97.1 / Cargo 1.97.1.
Runtime verification used the Ubuntu aarch64 UTM guest at `192.168.64.15`,
explicit DRM backend, `Virtual-1` at `1280x800`, and llvmpipe. The guest source
snapshot was `/home/ubuntu/qa/2026-08-01-luna-max-r16`. The SDK and Preview
source files used for this run have matching host/guest SHA-256 values recorded
in `artifacts/qa/2026-08-01-luna-max-r17/evidence/source-hashes-r17.txt`.
The compositor, shell, and session binaries were the already-built r17 guest
snapshot; this is not a claim that every guest source file was re-copied during
this run.

Current source changes in this slice include:

- `slopos-sdk` now exposes a thread-safe `EventLoopWaker`; application-owned
  background work wakes the Winit loop, and user events mark one redraw dirty
  without introducing an idle polling loop.
- Preview Vision watcher events now wake the SDK for status, terminal, and
  timeout events; stale submissions remain generation-scoped and watcher
  threads are invalidated when the view is replaced or dropped.
- Preview keeps a finite ten-minute local Vision deadline. The longer bound is
  required for real CPU OCR on this UTM guest; it remains bounded and does not
  block the UI thread.
- The parallel implementation slices added bounded XCursor theme resolution,
  exact-scale glyph caching, truthful portal capability/lifecycle state,
  and TextEdit atomic-save/recovery handling. Their targeted agent evidence was
  independently checked in the current worktree before integration.

Raw host and guest evidence is under
`artifacts/qa/2026-08-01-luna-max-r17/evidence/`. Fresh UTM captures are
`utm-preview-wake-vision-missing-r17.jpeg`,
`utm-preview-wake-baseline-r17.jpeg`,
`utm-preview-wake-running-10m-r17.jpeg`,
`utm-preview-wake-timeout-r17.jpeg`, and
`utm-preview-wake-completed-10m-r17.jpeg`; the baseline, running, timeout, and
completed images were visually inspected from the UTM window. They show the
SLOPOS global menu, patterned desktop, desktop icons, real Preview chrome, and
Dock. The first missing-daemon capture is retained as a diagnostic, not as a
pass claim.

Result: **BUILD VERIFIED**, **TEST VERIFIED**, and **RUNTIME OBSERVED** for the
scoped claims below.

- Fresh `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
  `cargo test --workspace`, all-features workspace Clippy, shell/Python syntax,
  and `git diff --check` completed without errors. Workspace tests reported
  `860 passed` across 37 suites. The existing `block v0.1.6`
  future-incompatibility warning remains. `cargo deny check licenses advisories
  bans sources` was attempted and is unavailable in this host environment.
- The exact namespaced request
  `com.slopos.preview.vision.extract_text` was sent through the unique
  per-session Preview control socket. The first rebuilt run proved the wake
  path through the visible timeout transition, but its 60-second bound expired
  before the slow daemon job finished. The raw daemon result later reported
  `job-1` succeeded with two OCR lines, so the bound was extended to ten
  minutes and the run was repeated.
- In the final run the session launched `slopos-session`, `slopos-compositor`,
  `slopos-shell`, `slopos-visiond`, and Preview. The exact runtime was
  `/run/user/1000/slopos-i/session-18c7bfb950aa2b32-365227`, with private
  `wayland-1`, Preview control socket, and Vision socket. The daemon result
  recorded `full_text` as `日且\n1` across two lines. Preview visibly reached
  `Extract Text completed: 4 characters across 2 lines` and saved
  `/home/ubuntu/.local/share/slopos-i/preview/vision/input-small-vision-text-365735-1785606794752521726-0.txt`,
  whose captured contents are the same two OCR lines.
- The session was terminated with SIGTERM. Post-run process inspection found no
  matching SLOPOS, Preview, or Vision process, and no readiness file remained
  under `/run/user/1000/slopos-i`; the shared menu directory remained as
  expected session-scoped shared state.

Remaining risk: UTM input capture still could not produce a human pointer press
through drag, valid serial, geometry delta, and release sequence; pointer-driven
move/resize, popup grabs/repositioning, Dock clicks, and true input-driven focus
changes remain unverified. The ten-minute deadline reflects measured slow CPU
inference on this VM and is not a performance claim. Nested mode,
display-manager login, physical hardware, XWayland clients, clean idle
profiling, true fractional rasterization, production text/font integration,
dynamic Spaces UI, Lift Subject UI, Finder Vision integration, and later P2–P10
requirements remain open. SLOPOS-I remains experimental/developing rather than
a complete daily-driver desktop.
