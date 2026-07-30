# HANDOFF — continue RetroShell (updated 2026-07-30)

> You are a fresh coding agent taking over mid-effort. Read this top to bottom,
> then `docs/PROGRAM.md` (honesty contract) and
> `docs/tasks/stage-2b-layer-shell-chrome.md` (the active workstream).
> **Honesty contract governs everything:** a task/stage is done only when its
> acceptance command passes on the real VM, evidenced by a transcript or
> screenshot — never by reading code or self-scoring. Do not mark anything
> "verified" you have not actually run. Do not trust a subagent that claims a
> build passed on the macOS/Windows host — the workspace only builds on the Linux VM.

## 1. What this project is
RetroShell is a classic-Mac-styled Linux **desktop environment** in Rust (Cargo
workspace). Own Wayland compositor (`crates/retro-compositor`, smithay), own shell
(`crates/retro-shell`), a widget kit (`crates/retro-kit`), an app SDK
(`crates/retro-sdk`, winit+wgpu), and first-party apps (`apps/*`).

## 2. Two supported environments — pick whichever machine you're on
The **Rust source and all fixes are architecture-independent**; only the VM
lifecycle, SSH, screenshot method, and GL specifics differ. Both are documented.

| Aspect | **A: macOS + UTM** (set up, used this session) | **B: Windows + VirtualBox** (Cursor's setup) |
|---|---|---|
| Host | macOS (Apple Silicon) | Windows x86_64 |
| Guest | Ubuntu 26.04 **aarch64**, VM name `Ubuntu` | Arch **x86_64**, VM name `retroshell-arch` |
| GPU/KMS | virtio-gpu → `/dev/dri/card0` | VMSVGA+3D → `vmwgfx` → `/dev/dri/card0` |
| Disk | LVM | `/dev/sda` |
| SSH | key `~/.ssh/retroshell_utm`, `192.168.64.15:22` | key `packaging/vm/qa_key`, `127.0.0.1:2222` |
| User | `ubuntu` / `ubuntu` (passwordless sudo) | `retro` / `retro` |
| **Screenshot** | **sway+grim / Xvfb** (SIGUSR1 dump BLOCKED) | **`VBoxManage screenshotpng`** (works directly) |
| Software GL | **required** (`LIBGL_ALWAYS_SOFTWARE=1`) — virtio hw GL fails wgpu | usually NOT needed (vmwgfx renders wgpu) |
| VM create | UTM app / `utmctl` | `packaging/vm/create-vm.ps1` |

Both VMs give a real `/dev/dri/card0` with KMS + render node. Ignore the repo's
Mac-only arm64 *Arch* scripts (`arch-install-arm64.sh`, `provision-arm64.sh`) —
those were an earlier dead-end; the UTM VM used this session is a plain Ubuntu VM.

## 3A. Environment A — macOS + UTM (currently provisioned & running)
- Start VM: `/Applications/UTM.app/Contents/MacOS/utmctl start Ubuntu`. If the IP
  changed: `arp -a | grep 192.168.64`. (No guest agent, so `utmctl ip-address` fails.)
- SSH: `ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15`.
- Already provisioned: rustup, build-essential, wayland/drm/seatd/libinput/gbm/egl
  dev libs, fonts-dejavu, xvfb, imagemagick, sway, grim, libxkbcommon-x11-0, 4G
  swap. User in `video,render,input` groups (the `seat` group does not exist here).
- **Edit-on-host / build-on-VM** loop:
  ```bash
  rsync -az --exclude target --exclude .git --exclude docs/screenshots \
    -e "ssh -i ~/.ssh/retroshell_utm" ./ ubuntu@192.168.64.15:/home/ubuntu/retroshell/
  ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15 \
    'cd ~/retroshell && source ~/.cargo/env && cargo build --release -p <crate>'
  ```
- **Run compositor (real DRM/KMS, headless over SSH):**
  ```bash
  export XDG_RUNTIME_DIR=/run/user/1000 LIBSEAT_BACKEND=seatd \
         LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
  ./target/release/retro-compositor        # gets seat0 + card0 + 1280x800 + spawns shell
  ```
- **Screenshots (SIGUSR1 dump is BLOCKED here — glReadPixels rejected by the driver):**
  - Layer-shell UI: run under **sway headless + grim** — exact recipe in
    `docs/tasks/stage-2b-layer-shell-chrome.md` (§QA); see `qa-layer-desktop.png`.
  - Plain winit UI: `Xvfb :99 -screen 0 1280x800x24 &`, then `DISPLAY=:99`
    (unset `WAYLAND_DISPLAY`) run the binary, then `DISPLAY=:99 import -window root out.png`.

## 3B. Environment B — Windows + VirtualBox (rebuild from scratch if you use this)
`create-vm.ps1` handles KMS (VMSVGA+3D → real vmwgfx). A standard x86_64 Arch ISO
from archlinux.org works.
1. Host prereqs: VirtualBox (`C:\Program Files\Oracle\VirtualBox\VBoxManage.exe`),
   PowerShell 7 (`pwsh`), Git, OpenSSH client. Download x86_64 Arch ISO.
2. Create VM:
   ```powershell
   pwsh -File packaging\vm\create-vm.ps1 -IsoPath C:\path\to\archlinux-x86_64.iso -Recreate
   ```
   → `retroshell-arch`: 8192 MB / 4 CPU / 60 GB, EFI, VMSVGA+3D, NAT host `2222`→guest `22`.
3. Host file server + key (installer fetches itself):
   ```powershell
   ssh-keygen -t ed25519 -N '""' -f packaging\vm\qa_key -C retroshell-vm
   cd packaging\vm ; python -m http.server 8000
   ```
   (`arch-install.sh` may not install your pubkey — set the `retro` password at the
   console and use password SSH, or add the key step.)
4. From the VirtualBox live console: `curl -sL http://10.0.2.2:8000/arch-install.sh | bash`
   — partitions `/dev/sda`, installs x86_64 Arch + deps, user `retro`/`retro`,
   `cargo build --release --workspace`, session files, tty1 autologin, reboots.
5. From the Windows host, sync edits + build (analogous to §3A but with
   `packaging\vm\qa_key` and `-p 2222 retro@127.0.0.1`).
- **Screenshots are easy here:** `VBoxManage controlvm retroshell-arch screenshotpng out.png`
  (see `packaging/vm/qa-live.sh`) captures the real scanout — no sway/Xvfb needed.
  So on VBox you can screenshot `retro-compositor` running the desktop directly.
- Software GL is usually unnecessary (vmwgfx rendered Cursor's wgpu clients). If a
  client can't get a GPU, fall back to `LIBGL_ALWAYS_SOFTWARE=1`.

## 4. Common to both — the layer-shell desktop
- The reworked desktop is gated behind env `RETROSHELL_LAYER_SHELL_CHROME=1`. Set
  it in the environment of `retro-compositor` (the spawned shell inherits it), or
  when running `retro-shell` directly under any wlr-layer-shell compositor.
- When unset, the shell uses the original winit xdg-toplevel path (unchanged).
- Milestone proof so far (Env A, under sway): `docs/screenshots/qa-layer-desktop.png`
  — fullscreen root-level background layer, full-width menu bar, correct fonts,
  desktop icons, Finder window, dock.

## 5. What was done this session (branch `docs/program-design`, 8 commits)
Re-audited Cursor's Stage 2 ("VERIFIED" was overclaimed); fixed and evidenced:
- **Fonts fixed** (retro-render baseline bearing/ascent + retro-sdk bitmap
  descenders) — `docs/screenshots/qa-shell-xvfb.png`.
- **Global-menu-only** — removed the legacy in-window `MenuBar` from retro-sdk.
- **Audit corrections** — `docs/qa/stage-2.md` retracts "VERIFIED"; deleted
  misleading old screenshots; added `docs/FUTURE.md` (backlog + HIG constraints)
  and `docs/tasks/stage-2b-layer-shell-chrome.md`.
- **Layer-shell rework — Phase 2b rendering DONE:** `retro_sdk::RawSurfaceRenderer`
  (wgpu from raw wl handles) + `retro_sdk::UiRuntime` (backend-agnostic render/input
  core; `tick()` drives `ShellDesktop::update()` → dock) + `retro-shell/src/layer_desktop.rs`
  (wlr-layer-shell background surface driver). Verified under sway+grim.
- Compositor also got a (non-firing) app_id fullscreen attempt and the blocked
  SIGUSR1 screenshot tool.

Commits: `c9d17c3` handoff · `2552fc6` polish (chromeless+dock) · `119abc5`
evidence · `a7d5e37` layer_desktop · `db41171` UiRuntime · `1bd616c`
RawSurfaceRenderer · `441b6a6` scope · `aea7014` fonts/menu/audit. **Not pushed to
origin yet.**

## 6. NEXT STEPS (priority order)
1. **DONE (Env B / VBox):** layer desktop under `retro-compositor` with
   `RETROSHELL_LAYER_SHELL_CHROME=1` — see `docs/screenshots/qa-layer-desktop-vbox.png`
   and `qa-layer-input-click.png` (menu opens on click). Compositor now hit-tests
   layer surfaces; smithay focus Point is surface **origin**, not pointer-local.
2. **DONE (Phase 3 exclusive chrome):** menu → Top exclusive, dock → Bottom exclusive,
   wallpaper/icons → Background. Gray PoC live-bind removed; kit chrome gated.
   Evidence: `qa-phase3-exclusive-chrome.png`.
3. **DONE (keyboard + Stage 2 Re-QA on Env B):** layer_desktop maps letters/modifiers
   and emits `Char` for lock typing; compositor Super+O/L already xkb-backed.
   Re-QA under layer chrome: `stage2-reqa-*.png` + STATUS
   (`FINDER_AFTER_SUPER_O=YES`, `LOCK_CLIENT=YES`, `FINDER_WHILE_LOCKED=1`, unlock
   restores desktop).
4. **DONE (menu Overlay):** open dropdowns paint on `retroshell-menu-popup` Overlay
   (exact size + margins); evidence `qa-phase3-menu-dropdown.png`.
5. **DONE (polish):** menu-bar inter-item gap; live clock via 1s `poll` timeout;
   evidence `qa-polish-menu-spacing.png`, `qa-polish-live-clock.png`.
6. **IN PROGRESS (Stage 3 `.app` bundles):** Tasks 3.0–3.8 done on Env B —
   disk scan, launch entrypoint, packaging scripts (5 apps), package-manager path
   removed, sha256 installer + install button wired, shell rescan via
   `~/Applications/.retroshell-rescan`. Next: Task 3.10 VM DoD (store install →
   Finder/dock → launch + screenshots). Optional 3.9 HTTP fetch skipped.

## 7. Gotchas
- Some docs (e.g. `docs/qa/stage-2.md`) had **CRLF** line endings from the Windows
  work — if a line-based Edit fails to match, use Write.
- `RETROSHELL_LAYER_SHELL_CHROME` gates the `layer_desktop` multi-surface path.
- Any Cargo.toml **dependency** change forces a ~15 min feature re-unification
  rebuild on the VM; code-only changes are fast.
- Keep the winit default path working — it is the fallback and how `apps/*` render.
