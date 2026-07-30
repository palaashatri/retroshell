# RetroShell — Program Index

RetroShell is a classic-Mac-styled Linux desktop environment and distribution,
written in Rust, in the spirit of [helloSystem](https://hellosystem.github.io/docs/).
This file is the living index for building it. The full rationale is in the
design spec: [docs/specs/2026-07-30-retroshell-de-program-design.md](specs/2026-07-30-retroshell-de-program-design.md).

## The honesty contract (read this first)

This project previously carried an "~85/100 daily-driver" score for a compositor
that had never built cleanly, never dispatched a client, and never painted a
window (see [docs/QA_REPORT_2026-07-26.md](QA_REPORT_2026-07-26.md)). We do not
repeat that. The single governing rule:

> **A task is "done" only when its acceptance command passes.
> A stage is "done" only when its QA doc passes on the real VM — evidenced by a
> screenshot or a command transcript, never by reading code or self-scoring.**

Every doc here obeys three rules:

1. **No unverified claims.** If it hasn't been run, it says "unverified."
2. **Evidence or it didn't happen.** QA results are transcripts/screenshots.
3. **No fabricated work.** We never write a task to "fix" something already fixed.
   (Example: QA defects C/D/#3 were already fixed by commit `868b9c5`; Stage 1
   verifies runtime behavior instead — see the spec §2.1.)

## Who executes these docs

The task docs are written to be executed by a **small model** (Gemma-3n-E4B
class, ~4B params) or a junior engineer with **zero project context**. Therefore:
architecture is already decided; every task names exact files and signatures;
every task ends in a **copy-paste acceptance command with expected output**; and
every task has a **DO NOT** block that fences the sandbox. One task ≈ one commit.
The executor never makes an architectural decision — if a task would require one,
it is not ready and must go back to a spec.

See [docs/tasks/README.md](tasks/README.md) for the task format and status legend.

## Stages

| Stage | Goal | Status | Docs |
|---|---|---|---|
| 0 | Arch VM with real KMS + SSH bridge (VBox/x86 active, UTM/arm64 dormant); Linux CI (exists) | **VERIFIED** (2026-07-30, Windows+VBox: `card0`/`vmwgfx` + `STAGE0-DOD-PASS`) | [HANDOFF.md](HANDOFF.md) · [tasks/stage-0-vm-foundation.md](tasks/stage-0-vm-foundation.md) · [qa/stage-0.md](qa/stage-0.md) |
| 1 | Prove the live path: one app window painting on the VM (verify-first) | **VERIFIED** DoD (a) (2026-07-30: Finder on `retro-compositor` DRM/`vmwgfx`) | [tasks/stage-1-prove-live-path.md](tasks/stage-1-prove-live-path.md) · [qa/stage-1.md](qa/stage-1.md) · [screenshots/stage1-finder.png](screenshots/stage1-finder.png) |
| 2 | Real session: input routing, working shortcuts, `ext-session-lock-v1`, clickable toolkit | **VERIFIED** (2026-07-30: lock/unlock, `Super+O`→Finder, button click on DRM) | [tasks/stage-2-real-session.md](tasks/stage-2-real-session.md) · [qa/stage-2.md](qa/stage-2.md) |
| 3 | Self-contained `.app` bundles + app store that installs them | Planned, unverified | [tasks/stage-3-app-bundles.md](tasks/stage-3-app-bundles.md) · [qa/stage-3.md](qa/stage-3.md) |
| 4 | Distribution: layer onto Arch/Ubuntu (primary) + bootable ISO (secondary) | Planned, unverified | [tasks/stage-4-distribution.md](tasks/stage-4-distribution.md) · [qa/stage-4.md](qa/stage-4.md) |

Atomic task docs for Stages 2–4 were authored 2026-07-30, after Stage 1 verified
on the VM. **Stage 2** is verified on the VBox DRM path (2026-07-30). **Stages 3–4**
are grounded in the design spec (§4–§5) + verbatim code anchors; each opens with a
re-ground/verify task. Stage 3–4 tasks are **UNVERIFIED** until their acceptance
passes and the transcript lands in the QA doc.

## Definition of done, per stage

- **Stage 0:** over SSH from the host, `ls /dev/dri/card0` succeeds **and**
  `cargo build --release --workspace` succeeds inside the VM. (`qa/stage-0.md`)
- **Stage 1:** either a VM screenshot of Finder rendered by `retro-compositor`
  (not labwc), **or** an evidenced diagnosis of exactly why it does not paint —
  enough to write the compositing spec. (`qa/stage-1.md`)
- **Stage 2:** lock cannot be bypassed by launching an app; typing the password
  unlocks; `Super+O` opens Finder — all on the VM.
- **Stage 3:** the store installs a `.app`, it appears in Finder/dock, it launches.
- **Stage 4:** the layered installer yields a login-selectable RetroShell session
  on a clean Arch VM **and** a clean Ubuntu-server VM; the ISO boots into it.

## Architecture map

- `crates/retro-render` — event loop + wgpu render plumbing.
- `crates/retro-kit` — widget toolkit (interaction layer partly inert — defect J).
- `crates/retro-sdk` — app framework: theming, layout, text; used by all apps.
- `crates/retro-shell` — shell: menu bar, dock, workspaces, launch services,
  portals, session/lock policy, layer-shell client.
- `crates/retro-bus` — IPC facade (currently non-functional — defect H).
- `crates/retro-compositor` — smithay compositor; X11-nested + DRM/KMS backends.
- `apps/{finder,settings,textedit,terminal,appstore}` — first-party apps.

## Development environment

The VM must expose a **real DRM/KMS device with a render node** — the capability
WSL2/Docker/Xvfb all lack, and the reason the compositor was never actually run.
Two host paths satisfy this; **switching machines? read [HANDOFF.md](HANDOFF.md) first.**

- **Active — Windows x86_64 + VirtualBox:** guest is Arch **x86_64**;
  **VMSVGA + 3D accel → `vmwgfx`** gives `/dev/dri/card0` + render node. Use
  `packaging/vm/create-vm.ps1`, `packaging/vm/arch-install.sh`, and the
  `qa-*.sh` scripts. Runbook in [HANDOFF.md](HANDOFF.md) §3.
- **Dormant — macOS arm64 + UTM:** guest is Arch **aarch64** with **virtio-gpu**.
  The `*-arm64.sh` scripts and Stage-0 Path-A tasks target this; do not run them
  on the VirtualBox path.
- **Agent access (both):** host→VM port-forward on `:22` (VBox uses host `2222`),
  key-based SSH; working tree synced over SSH.
- All graphical/live work happens **in the VM**, never on the host.
