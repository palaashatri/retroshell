# HANDOFF — continue RetroShell on a new machine

> **You are a fresh Claude Code instance with no memory of the prior session.**
> This file is your entry point. Everything you need is in the repo (git is the
> transfer mechanism). Read this top to bottom, then `docs/PROGRAM.md`.

## 0. How to resume

```bash
git fetch origin
git checkout docs/program-design      # the working branch (not merged to main yet)
git pull --ff-only origin docs/program-design
```

Read, in order: this file → [PROGRAM.md](PROGRAM.md) (honesty contract + stage
map) → [tasks/README.md](tasks/README.md) (task format) →
[tasks/stage-0-vm-foundation.md](tasks/stage-0-vm-foundation.md).

The SDD progress ledger lived in a git-ignored dir on the old machine and does
**not** transfer — §4 below is the authoritative "what's done" list instead.

## 1. What this project is (30 seconds)

RetroShell is a classic-Mac-styled Linux **desktop environment and distribution**
in Rust (Cargo workspace, ~50k LOC), in the spirit of helloSystem — own Wayland
compositor (smithay), own shell, and a macOS-like self-contained `.app` store.

**The honesty contract governs everything** (full text in PROGRAM.md §"honesty
contract"): a prior session scored a compositor "85/100 daily-driver" that had
never built cleanly, never dispatched a client, never painted a window. We do not
repeat that. **A task is done only when its acceptance command passes; a stage is
done only when its QA doc passes on the real VM — evidenced by a transcript or
screenshot, never by reading code or self-scoring.** No unverified claims, no
fabricated "fix" tasks for things already fixed.

## 2. Environment change (this is the important part)

The prior session set Stage 0 up for **macOS + UTM + arm64 + virtio-gpu**. You are
now on **Windows + VirtualBox**, which means **x86_64** (VirtualBox only runs on
x86_64 hosts) and a **VMSVGA → `vmwgfx`** virtual GPU.

| Aspect | Old (Mac, dormant) | **Now (Windows — active)** |
|---|---|---|
| Host | macOS arm64 | **Windows x86_64** |
| Hypervisor | UTM | **VirtualBox** |
| Guest arch | aarch64 | **x86_64** |
| Virtual GPU / KMS | virtio-gpu → `virtio_gpu` | **VMSVGA + 3D → `vmwgfx`** |
| Disk device | `/dev/vda` | **`/dev/sda`** |
| GRUB target | `arm64-efi` | **`x86_64-efi`** |
| Install script | `arch-install-arm64.sh` | **`arch-install.sh`** (original x86) |
| VM creation | UTM GUI | **`packaging/vm/create-vm.ps1`** |

**Consequence:** the arm64/UTM scripts the prior session wrote
(`packaging/vm/arch-install-arm64.sh`, `packaging/vm/provision-arm64.sh`, and the
Path-A tasks 0.1A/0.3A/0.4A in the Stage-0 doc) are **Mac-only and now dormant** —
do not run them. Use the repo's **original VirtualBox/Windows pipeline** instead
(Path B in the Stage-0 doc, plus the PowerShell scripts). Those original scripts
target exactly this environment and need no rewrite. Leave the arm64 scripts in
the repo (useful if the Mac is ever used again, and for the Stage 4 ISO).

## 3. Stage 0 on Windows + VirtualBox (the active runbook)

Everything KMS-related is already handled by `create-vm.ps1` (VMSVGA + 3D → real
`vmwgfx` DRM device with KMS *and* a render node — the capability WSL2/Docker
lacked). Unlike the aarch64 ISO hunt, a standard **x86_64 Arch ISO** from
archlinux.org works directly.

1. **Prereqs (Windows host):** install VirtualBox (default path
   `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe`), PowerShell 7 (`pwsh`),
   Git, and OpenSSH client (`ssh`, built into Windows 10/11). Download an
   **x86_64** Arch ISO from https://archlinux.org/download/.
2. **Create the VM:**
   ```powershell
   pwsh -File packaging\vm\create-vm.ps1 -IsoPath C:\path\to\archlinux-x86_64.iso -Recreate
   ```
   This makes `retroshell-arch`: 8192 MB / 4 CPU / 60 GB, EFI, **VMSVGA + 3D**,
   NAT with host `2222`→guest `22`, ISO attached, boot-from-DVD.
3. **Host file server + SSH key** (so the installer can fetch itself and your key):
   ```powershell
   ssh-keygen -t ed25519 -N '""' -f packaging\vm\qa_key -C retroshell-vm
   cd packaging\vm ; python -m http.server 8000    # leave running; Ctrl-C when done
   ```
   (`qa_key*` is git-ignored — generate a fresh one here; it did not transfer.)
   **Note:** the current `arch-install.sh` does **not** install your SSH public
   key (the arm64 variant did). Either add that step (see §7) or set the `retro`
   password at the console and use password SSH.
4. **Start the VM and run the installer** — from the VirtualBox live console
   (GUI window; `startvm retroshell-arch` shows it):
   ```bash
   curl -sL http://10.0.2.2:8000/arch-install.sh | bash
   ```
   `arch-install.sh` partitions `/dev/sda`, installs x86_64 Arch + all deps,
   creates user `retro`/`retro`, builds `cargo build --release --workspace`,
   installs binaries + the `retroshell.desktop` session, sets tty1 autologin,
   and reboots. Detach the ISO on reboot (or rely on boot order = disk after DVD).
5. **KMS gate (Stage 0 definition of done):** from the Windows host,
   ```powershell
   ssh -i packaging\vm\qa_key -p 2222 retro@127.0.0.1 "ls /dev/dri/card0 && lsmod | grep vmwgfx"
   ssh -i packaging\vm\qa_key -p 2222 retro@127.0.0.1 "cd ~/retroshell && cargo build --release --workspace && echo STAGE0-DOD-PASS"
   ```
   → expect `/dev/dri/card0`, `vmwgfx` loaded, and `STAGE0-DOD-PASS`. Record the
   transcript in [qa/stage-0.md](qa/stage-0.md) and mark Stage 0 VERIFIED.

**QA helper scripts already fit this environment** (no rewrite needed):
`packaging/vm/qa-vm.sh` greps for `vmwgfx`; `packaging/vm/qa-live.sh` uses
`VBoxManage ... screenshotpng`; `packaging/vm/qa-compositor.sh` drives the
compositor. These were written for VirtualBox and are correct here.

## 4. What is already done (repo-side, committed on `docs/program-design`)

| Commit | What |
|---|---|
| `3719928` | Design spec: `docs/specs/2026-07-30-retroshell-de-program-design.md` |
| `0cf8211` | PROGRAM.md, task-format README, Stage 0 + Stage 1 atomic tasks, QA docs |
| `7c1b4d3` | `arch-install-arm64.sh` (Mac/UTM — dormant here) |
| `2983fa7` | Stage 0 Task 0.8 **VERIFIED**: Linux CI builds the workspace |
| `76805a0` `f28551d` `a105f7c` | Path A (UTM prebuilt image) — Mac-only, dormant here |

**Verified on this Windows+VBox machine (2026-07-30):** Stage 0 DoD
(`card0`/`vmwgfx` + `STAGE0-DOD-PASS`) and Stage 1 DoD **(a)** (Finder painted by
`retro-compositor` on DRM — see `docs/screenshots/stage1-finder.png` and
`docs/qa/stage-1.md`). Mac/UTM Path-A tasks remain dormant/UNVERIFIED.

## 5. Stage 1 (after Stage 0 passes) — verification-first

Do **not** write "fix" tasks for QA defects C/D/#3 (present-buffer leak, discarded
libinput events, missing frame callbacks) — they were already fixed by commit
`868b9c5`; writing fixes for them would be fabricated work (spec §2.1). Stage 1
**observes** whether `retro-compositor` actually paints a client on real KMS. The
one real known gap is a code comment at
`crates/retro-compositor/src/session_drm.rs:894` ("the DRM path does not yet
composite client buffers to scanout") — but whether that blank-scanout path or the
GL `DrmCompositor` path runs on this GPU is unknown until observed. Full tasks:
[tasks/stage-1-prove-live-path.md](tasks/stage-1-prove-live-path.md). DoD: a VM
screenshot of Finder painted by `retro-compositor` (not labwc) **or** an evidenced
diagnosis of exactly why it doesn't. **Done** — Stage 1 passed DoD (a).

## 5a. The road ahead — Stages 2–4 are now specced (authored 2026-07-30)

Atomic, executable task docs + QA docs exist for the remaining stages. Do them in
order; each opens with a re-ground/verify task and every task ends in a copy-paste
acceptance command. **All are UNVERIFIED until run on the VM.**

- **Stage 2 — Real session:** [tasks/stage-2-real-session.md](tasks/stage-2-real-session.md)
  · [qa/stage-2.md](qa/stage-2.md). Honest surprise from grounding: input already
  works on the DRM path and `Widget::draw` is a no-op abstraction — so defects B/J
  are *verification* tasks, and the real work is `ext-session-lock-v1` (2.3–2.6,
  compositor/protocol — use a strong model, not a 4B one). DoD: lock unbypassable,
  password unlock, `Super+O` opens Finder.
- **Stage 3 — `.app` bundles + store:** [tasks/stage-3-app-bundles.md](tasks/stage-3-app-bundles.md)
  · [qa/stage-3.md](qa/stage-3.md). Mostly host-testable (`cargo test`) until the
  VM DoD. Uses spec §5.2 `Info.toml` (not the older `App.toml`).
- **Stage 4 — Distribution:** [tasks/stage-4-distribution.md](tasks/stage-4-distribution.md)
  · [qa/stage-4.md](qa/stage-4.md). Primary path is a layered `install.sh` reusing
  the existing `scripts/install-session-files.sh`; needs a clean Arch VM **and** a
  clean Ubuntu-server VM, plus an archiso ISO.

## 6. Decisions carried over (memory did not transfer)

- **Repo stays a single Cargo workspace monorepo.** Do not split into per-component
  repos (gershwin-desktop style) now — the Cargo workspace gives one build graph,
  one lockfile, atomic cross-crate refactors, one CI. The split seam is Stage 3's
  `.app` format: once `retro-sdk`'s API is stable, first-party `apps/*` can peel
  off. (Also in spec §10.)
- **App store is `.app`-only.** Package managers (pacman/apt) are reached via the
  Terminal app, not the store (spec §5.3).
- **Distribution is layer-first** (install onto existing Arch/Ubuntu incl. server),
  bootable ISO secondary (spec §4 Stage 4).

## 7. Open items to decide/handle on Windows

- **SSH key install in `arch-install.sh`:** the x86 installer does not fetch
  `qa_key.pub` (the arm64 one did). To get key-based SSH like the plan assumes,
  add before the reboot, inside the user-clone chroot block:
  ```bash
  install -d -m 700 -o $USERNAME -g $USERNAME /home/$USERNAME/.ssh
  curl -sL http://10.0.2.2:8000/qa_key.pub -o /home/$USERNAME/.ssh/authorized_keys
  chown $USERNAME:$USERNAME /home/$USERNAME/.ssh/authorized_keys
  chmod 600 /home/$USERNAME/.ssh/authorized_keys
  ```
  Otherwise set a password and use password SSH. (User `retro` / pass `retro`.)
- **User identity is not an open question here:** `arch-install.sh` already creates
  `retro`/`retro` in groups `wheel,video,input,seat`. (The root-vs-retro question
  from the Mac path only applied to the prebuilt image, which you are not using.)

## 8. How the prior session was executing

Subagent-driven: dispatch a Haiku subagent per delegatable task, review, commit;
the user drives the VM GUI/console steps a subagent cannot. If your instance has
the `superpowers` plugin, `subagent-driven-development` was the skill in use. If
not, execute inline — the task docs are self-contained and each ends in a
copy-paste acceptance command. Keep using Haiku for mechanical/transcription
tasks; reserve stronger models for the Stage 1 diagnosis judgment.
