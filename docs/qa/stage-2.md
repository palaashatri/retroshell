# QA — Stage 2 (Real session)

> **This doc holds evidence, not claims.** A row with no transcript/screenshot is
> `PENDING`, never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-2-real-session.md](../tasks/stage-2-real-session.md)

**Stage 2 definition of done:** on the VM — **lock cannot be bypassed by launching
an app**, **typing the password unlocks via `retro-lock`**, and **`Super+O` opens
Finder** with global menu mode (no in-window `MenuBar`). Root-level session chrome
(menu bar + dock) must be genuinely shell-owned, not an app window.

**Stage status: NEEDS REWORK** (re-audited 2026-07-30 on macOS + UTM `Ubuntu`
aarch64, DRM/`virtio-gpu`). The prior "VERIFIED" was **overclaimed** and is
retracted.

## ⚠️ Audit correction (2026-07-30)

The earlier all-PASS "VERIFIED" did not hold up. The evidence screenshots were
captured on a defective build and have been **deleted** (they were misleading).
Confirmed defects at re-audit, with root causes in code:

- **Desktop did not fill the screen.** The compositor forces every xdg-toplevel —
  including the RetroShell desktop — to a hardcoded 640×480 at (64,64)
  (`crates/retro-compositor/src/session_drm.rs` `new_toplevel`). The doc's claim
  that the menu bar and dock "**span compositor output**" was false in both code
  and pixels.
- **Chrome is not root-level.** Menu bar / dock / wallpaper are painted as kit
  widgets inside the shell's ordinary fullscreen toplevel; real layer-shell is
  stubbed (`chrome_protocol.rs::should_paint_kit_chrome` hardcoded `true`, gray
  placeholder buffers in `layer_shell_client.rs`). Being a normal client window is
  why the chrome read as "part of an app."
- **Font glyph corruption** (descenders: `p`→`r`, `g`→`s`) — **fixed** this session
  (retro-render baseline bearing + retro-sdk bitmap descenders).
- **Legacy in-window menu bar** shipped in every app (env-gated by
  `RETROSHELL_GLOBAL_MENU`, not removed) — **removed** (retro-sdk is now
  global-menu-only).

## Fixed & verified this session

Screenshot `docs/screenshots/qa-shell-xvfb.png` (retro-shell on the aarch64 VM):
fonts render correctly, the desktop fills 1280×800 when sized correctly, and there
is no in-window menu bar.

| Item | Status | Evidence |
|---|---|---|
| Fonts (descenders) render correctly | PASS | `docs/screenshots/qa-shell-xvfb.png` |
| No in-window menu bar (global-menu only) | PASS | same screenshot; `retro-sdk` `attach_menu_bar` removed |
| Workspace builds on aarch64 (Ubuntu 26.04) | PASS | `cargo build --release --workspace` → `Finished` on VM |
| Compositor runs headless DRM over SSH | PASS | seatd `seat0`, `/dev/dri/card0`, 1280×800 modeset, spawns shell |

## Remaining (blocks DoD)

- **Root-level chrome via layer-shell** — see
  [../tasks/stage-2b-layer-shell-chrome.md](../tasks/stage-2b-layer-shell-chrome.md).
  Until it lands, the desktop under the DRM compositor is not fullscreen and the
  chrome is not a real root surface.
- **Re-QA lock / unbypass / `Super+O`** on the reworked build (prior screenshots
  deleted; functional paths compile and the compositor spawns clients, but must be
  re-evidenced, not assumed).

## QA environment (current)

- VM: UTM `Ubuntu` aarch64 @ 192.168.64.15 (user `ubuntu`). `/dev/dri/card0` KMS ok.
- Run DRM headless over SSH: user in `video`,`render`,`input` groups;
  `LIBSEAT_BACKEND=seatd`; `LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe`
  (virtio hardware GL fails for wgpu clients; llvmpipe works).
- Screenshots: compositor SIGUSR1 readback is blocked (this GLES context rejects
  `glReadPixels` for all formats). Working method: **Xvfb + `import -window root`**.
