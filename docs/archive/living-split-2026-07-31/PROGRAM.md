# SLOPOS-I — Program Index

> **Doc map:** [README.md](README.md) is the entry point. UI SoT is [UI.md](UI.md).
> Ops / VM loop: [HANDOFF.md](HANDOFF.md).  
> **DE maturity vs GNOME/KDE + fix plan:** [MATURITY.md](MATURITY.md).

SLOPOS-I is a classic-Mac-styled Linux desktop environment and distribution,
written in Rust, in the spirit of [helloSystem](https://hellosystem.github.io/docs/).
Formerly **RetroShell** — crates are `slopos-*`, env prefix `SLOPOS_*`, config
`~/.config/slopos-i`. Full rationale:
[specs/2026-07-30-slopos-i-de-program-design.md](specs/2026-07-30-slopos-i-de-program-design.md).

## The honesty contract (read this first)

This project previously carried an "~85/100 daily-driver" score for a compositor
that had never built cleanly, never dispatched a client, and never painted a
window (see [archive/QA_REPORT_2026-07-26.md](archive/QA_REPORT_2026-07-26.md)).
Later agents claimed Spotlight/theme “complete” with **blank PNGs**. We do not
repeat that. The single governing rule:

> **A task is "done" only when its acceptance command passes.
> A stage is "done" only when its QA doc passes on the real VM — evidenced by a
> screenshot or a command transcript, never by reading code or self-scoring.**

Every doc here obeys three rules:

1. **No unverified claims.** If it hasn't been run, it says "unverified."
2. **Evidence or it didn't happen.** QA results are transcripts/screenshots.
3. **No fabricated work.** We never write a task to "fix" something already fixed.

## Current focus (honest — 2026-07-31 evening)

| Track | Status | Evidence |
|-------|--------|----------|
| Project rename RetroShell → SLOPOS-I | **DONE in-tree** (host folder/SSH key filenames still legacy) | crates/`slopos-*`, `start-slopos-i`, docs |
| Stages 0–3 | **VERIFIED** on real VMs | `qa/stage-0.md` … `qa/stage-3.md` |
| Stage 4 distribution | Packaging **authored**; clean-VM install/ISO DoD **unverified** | `qa/stage-4.md` |
| v0.2.0 bare bar (Spotlight + themed desktop) | **Visually proven** on UTM | `qa/v0.2.0/` |
| UI polish vs System 7 kits | **Improved, still far from kit-parity** | `qa/ui-polish/` + [UI.md](UI.md) |
| Defect J (toolkit clicks) | Proven on Env B Stage 2; **not re-proven on UTM** | `qa/stage-2.md` |
| Defect H (`slopos-bus`) | **Still a facade** (sends discarded) | code + spec §defects |
| HDR / VRR / compositor roadmap | Must **not regress** while polishing UI | compositor sources |
| **DE maturity vs GNOME/KDE** | **~15–25%** daily-driver; research DE, not a peer | [MATURITY.md](MATURITY.md) |

**Active agent work right now:** keep UI quality moving toward System7Components /
Figma kits ([UI.md](UI.md)), without inventing new session markdown. Closing the
GNOME/KDE gap follows the phased plan in [MATURITY.md](MATURITY.md) (honesty →
integration spine → compositor → apps). Stage 4 VM DoD and Defect H are Phase A/B
unless the user prioritizes them.

Do **not** treat archived “theme complete” / old roadmaps as current status.

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
| 0 | VM with real KMS + SSH; workspace builds | **VERIFIED** (2026-07-30, Env B VBox/`vmwgfx`) | [HANDOFF.md](HANDOFF.md) · [tasks/stage-0-vm-foundation.md](tasks/stage-0-vm-foundation.md) · [qa/stage-0.md](qa/stage-0.md) |
| 1 | Live path: app window paints on compositor DRM | **VERIFIED** (2026-07-30, Finder on `slopos-compositor`) | [tasks/stage-1-prove-live-path.md](tasks/stage-1-prove-live-path.md) · [qa/stage-1.md](qa/stage-1.md) |
| 2 | Real session: input, shortcuts, lock, clickable chrome | **VERIFIED** (2026-07-30 Env B; layer-shell re-QA same day) | [tasks/stage-2-real-session.md](tasks/stage-2-real-session.md) · [qa/stage-2.md](qa/stage-2.md) · [tasks/stage-2b-layer-shell-chrome.md](tasks/stage-2b-layer-shell-chrome.md) |
| 3 | `.app` bundles + store install → discover → launch | **VERIFIED** (2026-07-31, Env A UTM Task 3.10) | [tasks/stage-3-app-bundles.md](tasks/stage-3-app-bundles.md) · [qa/stage-3.md](qa/stage-3.md) |
| 4 | Layer onto Arch/Ubuntu + bootable ISO | **CODE-COMPLETE / DoD UNVERIFIED** | [tasks/stage-4-distribution.md](tasks/stage-4-distribution.md) · [qa/stage-4.md](qa/stage-4.md) |

**Env note:** Recent UI / Spotlight / Stage 3 work ran on **Env A (macOS + UTM
Ubuntu aarch64)**. Stages 0–2 primary DRM proofs were on **Env B (Windows +
VirtualBox Arch x86_64)**. Both remain valid; pick the machine you have — see
[HANDOFF.md](HANDOFF.md).

Stage 4 packaging (Tasks 4.0–4.4 / 4.7 authored: `install.sh`, PKGBUILD, `.deb`,
archiso profile, verify harness) is in-tree. **Tasks 4.5 / 4.6 / 4.8** (clean Arch
install, clean Ubuntu install, ISO boot screenshots) have **no transcripts yet**.

## Definition of done, per stage

- **Stage 0:** over SSH from the host, `ls /dev/dri/card0` succeeds **and**
  `cargo build --release --workspace` succeeds inside the VM. (`qa/stage-0.md`)
- **Stage 1:** VM screenshot of Finder rendered by `slopos-compositor` (not only
  labwc), or an evidenced diagnosis of why it does not paint. (`qa/stage-1.md`)
- **Stage 2:** lock cannot be bypassed by launching an app; typing the password
  unlocks; `Super+O` opens Finder — all on the VM.
- **Stage 3:** the store installs a `.app`, it appears in Finder/dock, it launches.
- **Stage 4:** the layered installer yields a login-selectable SLOPOS-I session
  on a clean Arch VM **and** a clean Ubuntu-server VM; the ISO boots into it.

## Architecture map

- `crates/slopos-render` — event loop + wgpu render plumbing.
- `crates/slopos-kit` — widget toolkit. Many `Widget::draw` methods are **stubs**;
  real paint is often in `slopos-sdk` walking the widget tree. Interaction layer
  partly proven (Defect J on Env B only).
- `crates/slopos-sdk` — app framework + **primary paint path** (windows, menu,
  dock, icons, Spotlight widgets via `draw_widget`).
- `crates/slopos-shell` — shell: menu bar, dock, workspaces, launch services,
  portals, session/lock, Spotlight UI, layer-shell client (`layer_desktop.rs`).
- `crates/slopos-bus` — IPC facade (**Defect H** — transports discard sends;
  type name may still say `RetroBus` in places).
- `crates/slopos-compositor` — smithay compositor; Nested X11 + DRM/KMS backends;
  HDR/VRR hooks on DRM path.
- `apps/{finder,settings,textedit,terminal,appstore}` — first-party apps.

### Important runtime gates / QA hooks

| Knob | Meaning |
|------|---------|
| `SLOPOS_LAYER_SHELL_CHROME=1` | Multi-surface layer-shell desktop (menu Top / dock Bottom / wallpaper Background) |
| `SLOPOS_QA_SPOTLIGHT=<query>` | One-shot open Spotlight for screenshots (no input injector required) |
| `LIBGL_ALWAYS_SOFTWARE=1` + `GALLIUM_DRIVER=llvmpipe` | **Required on UTM** virtio-gpu for wgpu |
| Config | `~/.config/slopos-i/settings.conf` (theme, lock password, etc.) |

## Known defects (still open)

| ID | Summary | Status | Plan |
|----|---------|--------|------|
| **H** | `slopos-bus` is a facade — no real IPC receive path | Open; blocks bus-driven theme hot-swap | [MATURITY.md](MATURITY.md) Phase B |
| **J** | Toolkit / chrome clicks | Pass on Env B Stage 2; **not** re-run on UTM | [MATURITY.md](MATURITY.md) Phase A |

Broader DE gaps (portals, PAM, XWayland-on-DRM, decorative menus, thin app suite,
Stage 4 ship path) are catalogued with fix phases in **[MATURITY.md](MATURITY.md)**.
Do not claim GNOME/KDE parity.

## Development environment

The VM must expose a **real DRM/KMS device with a render node** — the capability
WSL2/Docker/Xvfb alone lack for the session compositor path. Two host paths work:

- **Env A — macOS arm64 + UTM (active for recent UI/Spotlight/Stage 3):** Ubuntu
  **aarch64**, virtio-gpu → `/dev/dri/card0`. Software GL required. Guest tree:
  `~/slopos-i`. SSH key on host: `~/.ssh/retroshell_utm` (legacy filename).
- **Env B — Windows x86_64 + VirtualBox (Stages 0–2 DRM proofs):** Arch **x86_64**,
  VMSVGA+3D → `vmwgfx`. Screenshots via `VBoxManage screenshotpng`.

Details, recipes, and gotchas: **[HANDOFF.md](HANDOFF.md)**.
All graphical/live work happens **in the VM**, never as a smithay session on macOS/Windows hosts.
