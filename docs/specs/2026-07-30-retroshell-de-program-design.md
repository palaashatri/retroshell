# RetroShell — Desktop Environment & Distribution Program Design

**Date:** 2026-07-30
**Status:** Approved design (spec). Implementation plans derive from this.
**Author:** Design cycle following the QA report `docs/QA_REPORT_2026-07-26.md`.

---

## 1. Purpose & honest framing

RetroShell is a classic-Mac-styled Linux desktop environment written in Rust —
its own Wayland compositor, shell, toolkit, SDK, IPC bus, and first-party apps —
in the spirit of [helloSystem](https://hellosystem.github.io/docs/). The
long-term ambition is a self-hosting desktop *and* distribution that a user could
daily-drive, in the same category as KDE Plasma / GNOME on Ubuntu/Fedora.

This document is the honest program plan. It exists because the prior
documentation (removed in commit `61f95a9`) claimed an "~85/100 daily-driver"
score for a compositor that, per `docs/QA_REPORT_2026-07-26.md`, **had never
built cleanly on Linux, never dispatched a Wayland client, and never released a
frame callback.** We do not repeat that mistake. The governing rule of this
program:

> **A stage is "done" only when its QA document passes on the real VM, evidenced
> by a screenshot or a command transcript — never by reading code or by
> self-assigned scores.**

### 1.1 What is realistic (stated plainly)

- Competing with KDE/GNOME/Ubuntu is a multi-year, multi-person effort. This
  program does not deliver that in one cycle. It delivers a **correct, provable
  foundation** and a **repeatable process** to build the rest.
- The intended executor for the atomic task docs is a small on-device model
  (Gemma-3n-E4B class, ~4B effective parameters). Such a model **cannot** design
  a compositor. It **can** execute tightly-scoped tasks — "write this function
  with this signature; make this command print this output." Therefore all
  architecture is decided *here* and in the plans; the executor only fills
  bounded gaps against a copy-paste acceptance test.
- Anything not yet verified on hardware is labelled "unverified." We never state
  a benchmark, score, or "it works" without a transcript behind it.

## 2. Current state (verified 2026-07-30 by code inspection)

| Area | Reality |
|---|---|
| Workspace | `cargo` workspace: `crates/{retro-render,retro-kit,retro-shell,retro-bus,retro-sdk,retro-compositor}` + `apps/{finder,settings,textedit,terminal,appstore}`. ~50k LOC. |
| Compositor | smithay 0.7, backends: `backend_x11` (nested) + `backend_drm`/`gbm`/`libinput`/`udev`/`session_libseat` (bare-metal KMS). Linux-only; stubs on macOS. |
| CI | `.github/workflows/ci.yml` **already builds + tests + clippies the workspace on `ubuntu-latest`** (debug + release). This is the safety net the QA report asked for; it exists. It does **not** build a VM image or run any live/graphical test. |
| App bundles | `crates/retro-shell/src/launch_services.rs` defines an `AppBundle` struct and search paths `/Applications`, `/User/Applications`, but `scan_applications()` is **stubbed** — it hardcodes 5 builtins and never reads disk. **No on-disk bundle format exists.** |
| App store | `apps/appstore` installs software by shelling out to `pacman`/`apt`/`brew`. It is **completely disconnected** from `launch_services` / the `.app` model. |
| VM tooling | `packaging/vm/*` targets **x86 VirtualBox** with `vmwgfx`. The host is **arm64 (Apple Silicon)** using **UTM**. The scripts are wrong for this machine and must be rewritten. |
| Docs | Only `docs/QA_REPORT_2026-07-26.md` survives. All prior docs were removed in `61f95a9`. |

### 2.1 Open defects inherited from the QA report (must be closed on the VM)

Critical/high items that require a running compositor to fix honestly:

- **A** — Lock screen does not lock a multi-client session (needs
  `ext-session-lock-v1`). Currently a security hole: apps draw over the lock.
- **B** — Keyboard input never reaches `retro-shell` on the labwc fallback path.
- **C** — DRM present **leaks a dumb buffer + framebuffer every frame**
  (`mem::forget`), ~1/sec — unbounded kernel memory leak.
- **D** — DRM session **discards all libinput events** — zero input on bare metal.
- **E/F/G** — portal string parsing garbage; screenshot/record are X11-only;
  display-arrange writes to the wrong process and fabricates output geometry.
- **H** — `retro-bus` transports are a facade (sends discarded, never receives).
- **I** — terminal VT parser missing cursor CSIs, ED 0/1, HT, DECSTBM scroll.
- **J** — toolkit interaction layer largely dead (buttons/scroll/focus inert).

## 3. Goals & non-goals

**Goals**

1. A reproducible **arm64 Arch Linux UTM VM** with real `virtio-gpu` KMS, that I
   can drive over SSH, on which the compositor can finally be exercised.
2. **Prove the live path**: `retro-compositor` displays at least one real
   first-party app window, with working input, on that VM.
3. A **documentation system** (`docs/PROGRAM.md`, `docs/tasks/`, `docs/qa/`) with
   atomic, Gemma-3n-executable task files and per-stage VM QA scripts.
4. A defined, self-contained **`.app` bundle format** and an app store that
   installs it (built in a later stage, specified here).
5. A path to a **bootable/installable distribution** (later stage, specified here).

**Non-goals (this cycle)**

- Writing atomic task docs for Stages 2–4 now (they depend on Stage 1 results).
- Feature parity with KDE/GNOME. HDR/VRR polish. Theming breadth beyond fixing
  the known dark-label bug. Third-party SDK stability guarantees.
- Any claim of "daily-driver readiness" until Stage 2's QA passes on the VM.

## 4. Staged program

Each stage is its own spec→plan→execute→QA cycle. `docs/PROGRAM.md` (produced in
the planning phase) is the living index; this section is the authoritative shape.

### Stage 0 — Foundation: VM + CI (unblocks everything)
- Rewrite `packaging/vm/*` for **arm64 UTM + virtio-gpu**. Produce:
  - Host GUI steps (human-only: creating the UTM VM, attaching the Arch ISO,
    enabling virtio-gpu, port-forwarding host→VM:22).
  - `arch-install.sh` rewritten for aarch64, GRUB/systemd-boot for UEFI,
    `virtio_gpu` in initramfs, autologin on a TTY → `scripts/start-retroshell`.
  - An **SSH bridge**: key-based access so the agent can drive the VM; an
    `rsync`/shared-folder path to push the working tree in.
- Extend CI: keep the existing Linux build gate; add a `cargo fmt`/backlog note.
  (Do **not** claim to add Linux CI — it exists.)
- **Definition of done:** `ssh retro@vm 'ls /dev/dri/card0 && cargo build
  --release --workspace'` succeeds, transcript captured in `docs/qa/stage-0.md`.

### Stage 1 — Prove the live path (the QA report's step zero)
- On the VM's real KMS via `session_drm.rs`, get **one app window painting**.
- Fix, against a running binary verified over SSH:
  - **C** DRM present leak, **D** DRM libinput input drop, and confirm **#3**
    frame callbacks fire on the DRM path.
- **Definition of done:** a screenshot (captured on the VM) of Finder rendered
  by `retro-compositor` — not labwc — with a visible cursor that moved in
  response to an injected input event. Transcript + image in `docs/qa/stage-1.md`.

### Stage 2 — Real session
- Keyboard/pointer routed to shell + apps (defect **B**); documented shortcuts
  actually fire; `ext-session-lock-v1` so the lock screen truly locks (defect
  **A**); fix the dead toolkit interaction layer enough to click a button
  (defect **J**).
- **Definition of done:** `docs/qa/stage-2.md`: lock → cannot bypass with a
  launched app (the exact scenario from QA screenshot `qa7/08`), unlock by typing
  the password, `Super+O` opens Finder. All on the VM.

### Stage 3 — `.app` bundles + app store
- Define the **self-contained `.app` bundle format** (see §5). Make
  `launch_services::scan_applications()` actually read `/Applications/*.app`.
  Rewrite the app store to download/verify/install bundles into `/Applications`
  (not shell out to `pacman`/`apt`). Package the 5 first-party apps as `.app`s.
- **Definition of done:** `docs/qa/stage-3.md`: store installs a `.app`, it
  appears in Finder/dock, and it launches — on the VM.

### Stage 4 — Distribution (two delivery paths, layer-first)
RetroShell is a **desktop environment on a normal Linux base**, so the primary
delivery path is layering it onto an existing distro; a bootable image is a
secondary convenience. (Ordering confirmed with user.)
- **Primary — install on an existing base.** An installer (script or native
  package: AUR/`.deb`) that adds RetroShell + its session files to a running
  **Arch** or **Ubuntu (incl. server)** system and registers it as a selectable
  session. This is how most users get it, and it reinforces §5.3: the base distro
  keeps its own package manager, reachable from the Terminal app.
- **Secondary — bootable image.** An archiso-derived ISO that boots straight into
  RetroShell, for evaluation / fresh installs. Built from the same session files
  as the layered path so the two cannot drift.
- **Definition of done:** (1) on a clean Arch VM *and* a clean Ubuntu-server VM,
  the layered installer produces a login-selectable RetroShell session that
  reaches the desktop; (2) the ISO boots a fresh VM into RetroShell. Transcripts
  + screenshots in `docs/qa/stage-4.md`.

## 5. The `.app` bundle format (self-contained; target for Stage 3)

Decision (confirmed with user): macOS/helloSystem model — **self-contained
bundles**, not package-manager wrappers.

### 5.1 On-disk layout

```text
<Name>.app/
  Resources/
    Info.toml            # required manifest (see 5.2)
    icon.png             # 512x512 app icon
    <other resources>
  bin/
    <executable>         # the launched binary (or a launcher script)
  lib/                   # optional: bundled shared libraries (rpath $ORIGIN/../lib)
```
Installed to `/Applications/<Name>.app` (system) or
`~/Applications/<Name>.app` / `/User/Applications` (per-user). `.app` is a
**directory**, matching the existing `launch_services` search paths.

### 5.2 `Info.toml` manifest (maps to the existing `AppBundle` struct)
```toml
bundle_id       = "com.retro.textedit"   # -> AppBundle.bundle_id
name            = "TextEdit"             # -> AppBundle.name
version         = "0.1.0"                # -> AppBundle.version
entrypoint      = "bin/textedit"         # -> AppBundle.entrypoint (path within bundle)
supported_types = ["txt", "md", "rtf"]   # -> AppBundle.supported_types
permissions     = ["files.read", "files.write"]  # -> AppBundle.permissions
```
`scan_applications()` walks each search path, reads every
`*.app/Resources/Info.toml`, and registers an `AppBundle` per §2 struct. Launch
resolves `path + "/" + entrypoint` and execs it as a Wayland client.

### 5.3 Store install flow (Stage 3)
1. Fetch a signed bundle archive (`.app` tarred) from a catalog URL.
2. Verify checksum/signature (design detail deferred to Stage 3 plan).
3. Extract into a staging dir, then atomically move into `/Applications`.
4. Trigger a `launch_services` rescan.

**The store is `.app`-only. There is no package-manager path in the store.**
(Decision confirmed with user.) A real Linux base is always present underneath,
so a user who wants `apt`/`pacman`/`yum`/`pkg` simply runs it in the **Terminal**
app — that is the escape hatch, and it needs no store support. Consequently the
current `apps/appstore` logic that shells out to `pacman`/`apt`/`brew` — including
`execute_transaction`, `install_async`, and the `package_changes_allowed()` gate
and its env var — is **removed** in Stage 3, not preserved. This closes QA
finding **#8** (the install button that bypassed the package-change gate) by
deleting the whole system-package path rather than re-gating it.

## 6. Documentation system (the primary deliverable)

Produced in the writing-plans phase; structure fixed here.

- **`docs/PROGRAM.md`** — master index: vision, the 5 stages, per-stage
  definition-of-done, links to task and QA files, and the honesty contract (§1).
- **`docs/tasks/NN-slug.md`** — atomic task files, one file/function each, ordered.
- **`docs/qa/stage-N.md`** — per-stage QA: exact commands + screenshot checklist
  to run **on the VM**, with a pass/fail table filled in from real runs.

### 6.1 Atomic task template (mandatory for every task file)

```text
# Task NN — <title>
Stage: <n>   Depends on: <task ids or "none">
Precondition: <what must already pass; a command whose success proves it>
File: <exact repo-relative path>
Signature: <exact fn/struct/trait signature to add or change>
Steps:
  1. <edit>
  2. <edit>
Acceptance:
  $ <command>
  → expect: <exact stdout / exit 0 / screenshot description>
DO NOT:
  - <the 2-3 scope/build traps a weak model would fall into>
Commit: <conventional-commit message to use>
```
Rationale: the `DO NOT` block is load-bearing for weak executors — it fences the
sandbox. The `Acceptance` block must be runnable verbatim; no "verify it looks
right." One task ≈ one commit.

## 7. Development environment

- **Host:** macOS on arm64 (Apple Silicon), UTM.
- **VM:** Arch Linux **aarch64** in UTM, `virtio-gpu` (gives `/dev/dri/card0`
  KMS — the capability WSL2/Docker lacked and why the compositor was never run).
- **Agent access:** UTM emulated network + host→VM:22 port-forward, key-based
  SSH. Working tree pushed via `rsync` over SSH (or a UTM shared directory).
- **Isolation:** All graphical/live work happens **in the VM**, never on the
  host. Host is for editing code and authoring docs only. (Matches the user's
  standing preference for isolated execution environments.)

## 8. Architecture map (existing crates — do not restructure without cause)

- `retro-render` — event loop + wgpu render plumbing (returns `Result` now).
- `retro-kit` — widget toolkit (interaction layer partly dead — defect J).
- `retro-sdk` — app framework: theming, layout, text; used by all 5 apps.
- `retro-shell` — desktop shell: menu bar, dock, workspaces, launch services,
  portals, session/lock policy, layer-shell client.
- `retro-bus` — IPC facade (currently non-functional — defect H).
- `retro-compositor` — smithay compositor; X11-nested + DRM/KMS backends.
- `apps/*` — finder, settings, textedit, terminal, appstore.

Keep boundaries as-is. Targeted fixes only where a stage requires them (e.g.
`session_drm.rs` in Stage 1, `launch_services.rs` + `appstore` in Stage 3). No
speculative refactors.

## 9. Risks & mitigations

- **virtio-gpu may not expose the DRM features smithay needs.** Mitigation:
  Stage 0's DoD explicitly probes `/dev/dri/card0` and a minimal KMS modeset
  before we build on it. If it fails, we fall back to UTM's other GPU options or
  document the gap honestly rather than proceeding on a false premise.
- **aarch64 build differences** (deps, atomics). Mitigation: CI stays x86 Linux;
  the VM is the aarch64 truth. Any arch-specific breakage is captured on the VM.
- **Weak executor drift.** Mitigation: the atomic template's `DO NOT` block +
  per-task acceptance command; no task may leave an architectural choice open.
- **Scope creep back into "score inflation."** Mitigation: the §1 honesty
  contract; QA docs record real transcripts only.

## 10. Open questions (resolve during planning, not blocking this spec)

1. Stage 3: bundle signing scheme and catalog hosting — defer to Stage 3 plan.
2. Stage 4: exact native-package form for the layered installer (AUR `PKGBUILD`
   for Arch; `.deb` vs. install script for Ubuntu) — defer to Stage 4 plan.

**Resolved during this review:**

- Store is `.app`-only; package managers are reached via the Terminal, not the
  store (§5.3).
- Distribution is layer-first (install onto existing Arch/Ubuntu), with a
  bootable ISO as a secondary path built from the same session files (§4 Stage 4).
