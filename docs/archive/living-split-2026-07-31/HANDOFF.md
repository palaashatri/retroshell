# HANDOFF — continue SLOPOS-I (updated 2026-07-31 evening)

> You are a fresh coding agent taking over mid-effort. Read **[README.md](README.md)**
> first (doc map), then [PROGRAM.md](PROGRAM.md) (honesty + stages),
> [MATURITY.md](MATURITY.md) (gaps vs GNOME/KDE + fix phases), and [UI.md](UI.md)
> (visual SoT). Ops details for VMs are below.
>
> **Honesty contract:** a task/stage is done only when its acceptance command
> passes on the real VM, evidenced by a transcript or screenshot — never by
> reading code or self-scoring. Do not trust a subagent that claims a smithay
> compositor build “passed” on macOS/Windows — **build and run graphics on the
> Linux VM**. Do **not** claim GNOME/KDE parity.

---

## 0. Snapshot — where the code actually is

| Item | Truth |
|------|--------|
| Product name | **SLOPOS-I** (was RetroShell) |
| Crates / bins | `slopos-{render,kit,sdk,shell,bus,compositor}`; bins `slopos-shell`, `slopos-compositor`, `slopos-lock` |
| Env / config | `SLOPOS_*`, `~/.config/slopos-i` |
| Stages 0–3 | **VERIFIED** (see `qa/stage-*.md`) |
| Stage 4 | Packaging in-tree; **clean install / ISO DoD unverified** |
| Spotlight | **Paints** on UTM (`qa/v0.2.0/`) — was previously invisible (stub widgets + not in paint tree) |
| UI polish | Better chrome/icons/Graphite; **not** System7Components parity (`qa/ui-polish/`, [UI.md](UI.md)) |
| Defect H | `slopos-bus` still a facade |
| Defect J | Clicks proven Env B; not re-proven UTM |
| **vs GNOME/KDE** | **~15–25%** daily-driver — research DE, not a peer ([MATURITY.md](MATURITY.md)) |
| Branch | `docs/program-design` — rename + latest UI work may be **uncommitted** locally; check `git status` |
| Host folder | Still often named `retroshell` on disk; guest path **`~/slopos-i`** |
| UTM SSH key | Still `~/.ssh/retroshell_utm` (legacy filename only) |

**Do next (default priority unless user says otherwise):**

1. UI polish toward System 7 kits — edit `slopos-sdk` paint, capture into `qa/ui-polish/`, update [UI.md](UI.md) gaps only.
2. Or Phase A honesty from [MATURITY.md](MATURITY.md): wire/remove fake menus, Stage 4 DoD, UTM Defect J.
3. Broader “make it a real DE” → Phases B→C in MATURITY (bus, portals, DRM/XWayland, WM) — only if asked.
4. Never claim visual success without non-blank screenshots on the VM.

---

## 1. What this project is

SLOPOS-I is a classic-Mac-styled Linux **desktop environment** in Rust (Cargo
workspace): Wayland compositor (`crates/slopos-compositor`, smithay), shell
(`crates/slopos-shell`), widget kit (`crates/slopos-kit`), app SDK
(`crates/slopos-sdk`, winit+wgpu), first-party apps (`apps/*`).

Paint reality check: many kit `Widget::draw` impls are **empty stubs**. The
desktop/Spotlight you see is largely **`slopos-sdk::draw_widget`** walking the
shell’s widget tree. Spotlight only became visible when its widgets were put on
that path (`Panel` + children), not when stub `draw()` methods were “implemented”
in isolation.

---

## 2. Two supported environments

The **Rust source is architecture-independent**; VM lifecycle, SSH, screenshots,
and GL differ.

| Aspect | **A: macOS + UTM** (recent UI / Stage 3 / Spotlight) | **B: Windows + VirtualBox** (Stages 0–2 DRM proofs) |
|---|---|---|
| Host | macOS (Apple Silicon) | Windows x86_64 |
| Guest | Ubuntu 26.04 **aarch64**, VM name `Ubuntu` | Arch **x86_64**, VM name `slopos-i-arch` |
| GPU/KMS | virtio-gpu → `/dev/dri/card0` | VMSVGA+3D → `vmwgfx` → `/dev/dri/card0` |
| SSH | `~/.ssh/retroshell_utm`, `192.168.64.15:22` | `packaging/vm/qa_key`, `127.0.0.1:2222` |
| User | `ubuntu` / `ubuntu` (passwordless sudo) | `retro` / `retro` |
| Guest tree | **`~/slopos-i`** (`~/retroshell` may symlink) | typically `~/slopos-i` or legacy path — confirm |
| **Screenshot** | **sway headless + grim** (SIGUSR1 compositor dump **BLOCKED** on UTM) | **`VBoxManage screenshotpng`** |
| Software GL | **Required:** `LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe` | usually not needed |
| Disk | ~30G LVM — **easy to fill** with duplicate `target/` trees; exclude `target*` on rsync | larger disk typical |

Ignore Mac-only *Arch* arm64 install scripts (`arch-install-arm64.sh`,
`provision-arm64.sh`) for the current UTM Ubuntu guest.

---

## 3A. Environment A — macOS + UTM

- Start: `/Applications/UTM.app/Contents/MacOS/utmctl start Ubuntu`. IP drift:
  `arp -a | grep 192.168.64`.
- SSH: `ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15`
- Provisioned: rustup, build-essential, wayland/drm/seatd/libinput/gbm/egl,
  fonts-dejavu, xvfb, imagemagick, sway, grim, libxkbcommon-x11-0, swap.
  User in `video,render,input`.

### Edit-on-host / build-on-VM

```bash
rsync -az --exclude target --exclude target-docker --exclude .git \
  --exclude 'docs/qa/**/*.png' --exclude docs/screenshots \
  -e "ssh -i ~/.ssh/retroshell_utm" \
  ./ ubuntu@192.168.64.15:/home/ubuntu/slopos-i/

ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15 \
  'cd ~/slopos-i && source ~/.cargo/env && cargo build --release -p <crate>'
```

**Disk gotcha:** do not rsync `target`/`target-docker`. If `df` shows ~100% on `/`,
delete guest `target*` and duplicate checkouts before syncing again.

### Run compositor (DRM over SSH)

```bash
export XDG_RUNTIME_DIR=/run/user/1000 LIBSEAT_BACKEND=seatd \
       LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe \
       SLOPOS_LAYER_SHELL_CHROME=1
./target/release/slopos-compositor
```

### Screenshots on UTM (use this, not ImageMagick `import` alone)

SIGUSR1 GL readback is blocked. Prefer **sway headless + grim**:

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)
export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
export SLOPOS_LAYER_SHELL_CHROME=1
# sway config: output * { resolution 1280x800 }
sway -c /tmp/sway-headless.conf &
export SWAYSOCK=... WAYLAND_DISPLAY=wayland-1
./target/release/slopos-shell &
# optional Spotlight QA: SLOPOS_QA_SPOTLIGHT=vol ./target/release/slopos-shell
sleep 7
grim docs/qa/ui-polish/01-desktop.png
```

Full Spotlight recipe + honesty notes: `docs/qa/v0.2.0/QA-RESULTS.md`.

Reject any PNG that is tiny/blank (~hundreds of bytes / 1-bit empty). Prior false
“QA passed” used those — they are invalid.

---

## 3B. Environment B — Windows + VirtualBox

`create-vm.ps1` → VMSVGA+3D → real `vmwgfx`.

1. Prereqs: VirtualBox, PowerShell 7, Git, OpenSSH; x86_64 Arch ISO.
2. Create:
   ```powershell
   pwsh -File packaging\vm\create-vm.ps1 -IsoPath C:\path\to\archlinux-x86_64.iso -Recreate
   ```
3. Host file server + key under `packaging\vm\` as documented in older Stage 0 tasks.
4. Live console: `curl -sL http://10.0.2.2:8000/arch-install.sh | bash`
5. Sync/build with `qa_key` and `-p 2222 retro@127.0.0.1`.

Screenshots: `VBoxManage controlvm slopos-i-arch screenshotpng out.png`
(see `packaging/vm/qa-live.sh`). This captures real compositor scanout — easier
than UTM for DRM-path evidence.

---

## 4. Layer-shell desktop (common)

- Gate: `SLOPOS_LAYER_SHELL_CHROME=1` (inherited by shell when set for compositor).
- Unset → legacy winit xdg-toplevel desktop path (keep working for apps).
- Layout: wallpaper/icons Background; menu Top exclusive; dock Bottom exclusive;
  open menus on Overlay (`slopos-i-menu-popup`).
- Evidence (historical): `docs/screenshots/qa-layer-desktop.png`,
  `qa-phase3-exclusive-chrome.png`, `qa-phase3-menu-dropdown.png`, plus Env B
  `qa-layer-desktop-vbox.png` / click / Stage 2 re-QA set.

---

## 5. What landed recently (honest changelog)

Consolidate claims against evidence — not commit mythology.

### Verified program stages

- **0–2:** Env B DRM path (build, Finder paint, lock/shortcuts/clicks) — `qa/stage-0.md`…`stage-2.md`.
- **2b layer-shell:** exclusive chrome + menu overlay + keyboard for lock — Env B re-QA + UTM sway captures.
- **3:** `.app` scan/launch/packaging/store install DoD on **Env A** — `qa/stage-3.md` (`INSTALLED-VIA-STORE`).
- **4:** `install.sh`, Arch PKGBUILD, Debian metadata, archiso profile, verify scripts **authored**;
  clean-VM / ISO boot rows in `qa/stage-4.md` still **PENDING**.

### v0.2.0 / Spotlight (UTM)

- Prior “Spotlight complete” was **false** (stub draws; overlay not in paint tree;
  blank PNGs). Fixed path: `slopos-kit` `Panel` + SDK `draw_widget` + shell children;
  QA hook `SLOPOS_QA_SPOTLIGHT`. Evidence: `qa/v0.2.0/*.png` + `QA-RESULTS.md`.

### UI polish (still open-ended)

- System7Components palette, multi-layer bevels, frame title bar, fixed 32×32 icons,
  trademark-safe per-app glyphs, Graphite theme helpers, desktop nameplates.
- Evidence: `qa/ui-polish/`. Gaps remain (fonts, pixel icons, dark menu fidelity,
  full control port) — see [UI.md](UI.md).

### Rename RetroShell → SLOPOS-I

- Crates, binaries, env, session files, docs, packaging renamed.
- Release build on UTM after rename: `slopos-compositor`, `slopos-shell`,
  `slopos-lock`, `finder`, `settings` succeeded.
- Session packaging scripts (`verify_session_packaging.sh`,
  `verify_greeter_session.sh`) PASS with `DesktopNames=SLOPOS-I`.
- Leftovers to be aware of: host directory name, SSH key filename, possible
  `RetroBus` type inside `slopos-bus`, archived docs under `docs/archive/`.

### Docs consolidation

Living SoT only: `docs/README.md`, `PROGRAM.md`, `UI.md`, `HANDOFF.md`,
`FUTURE.md`, plus `tasks/`, `qa/`, `specs/`. Session sprawl → `docs/archive/`.

---

## 6. NEXT STEPS (priority)

1. **UI quality (default):** iterate `slopos-sdk` / kit paint vs System7Components +
   Figma; fresh `qa/ui-polish/` screenshots; update [UI.md](UI.md) gap list only.
2. **Do not regress** HDR/VRR / DRM compositor paths while polishing.
3. **Phase A honesty** ([MATURITY.md](MATURITY.md)): wire or remove decorative menus;
   Stage 4 DoD (`qa/stage-4.md`); UTM Defect J re-proof.
4. **Phase B+ when asked:** Defect H, live settings, clipboard, portals, workspace
   sync, then DRM scanout / XWayland / PAM / PipeWire (Phases C–D).
5. **Commit / push** rename + docs when the user asks — check `git status` first.

Full gap register + acceptance ideas: **[MATURITY.md](MATURITY.md)**.

---

## 7. Gotchas

- **Blank screenshots lie.** Reject tiny/empty PNGs; prefer grim under sway.
- `SLOPOS_LAYER_SHELL_CHROME` gates multi-surface chrome.
- Kit `draw()` stubs ≠ on-screen widgets — follow the SDK paint tree.
- Cargo **dependency** changes → long feature unification rebuilds on the VM;
  code-only changes are faster.
- Keep the **winit** app path working — first-party apps still use it.
- Some older `docs/qa/*.md` may have **CRLF** from Windows — Edit may fail to match.
- Desktop title **`SLOPOS-I Desktop`** is special-cased in `draw_window` for
  chromeless desktop; renaming that string without updating the special case
  breaks chrome-less layout.
- Product display strings use **SLOPOS-I**; Rust type for the shell may be
  `SloposI` — do not “fix” identifiers into invalid `SLOPOS-I` tokens.

---

## 8. Quick verification commands

```bash
# Packaging / greeter files (host or VM)
./scripts/verify_session_packaging.sh
./scripts/verify_greeter_session.sh

# Core release build (VM)
cd ~/slopos-i && source ~/.cargo/env
cargo build --release -p slopos-compositor -p slopos-shell -p finder

# Lib tests (VM; counts drift — don't hardcode forever)
cargo test -p slopos-kit -p slopos-sdk -p slopos-shell --lib --release
```
