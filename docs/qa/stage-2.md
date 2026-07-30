# QA — Stage 2 (Real session)

> **This doc holds evidence, not claims.** A row with no transcript/screenshot is
> `PENDING`, never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-2-real-session.md](../tasks/stage-2-real-session.md)

**Stage 2 definition of done:** on the VM — **lock cannot be bypassed by launching
an app**, **typing the password unlocks via `retro-lock`**, and **`Super+O` opens
Finder** with global menu mode (no in-window `MenuBar`). Root-level session chrome
(menu bar + dock) must be genuinely shell-owned, not an app window.

**Stage status: IN PROGRESS on Env B** (re-audited 2026-07-30; functional DoD
items re-evidenced under layer-shell chrome — see Env B table). Not re-marked
`VERIFIED` until Overlay menu polish is settled and Env A is optionally refreshed.

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
  stubbed (`chrome_protocol.rs::should_paint_kit_chrome` was hardcoded `true`, gray
  placeholder buffers in `layer_shell_client.rs`). **Addressed on Env B:** exclusive
  Top/Bottom/Background layers + kit chrome gated when bound (Phase 3).
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

- Menu dropdown overlays currently clip to the 24px Top surface — needs Overlay
  surface or expand-on-open (polish).
- Full sctk `KeyboardHandler` (like `retro-lock`) would replace the KEY_*/xkb-mask
  map in `layer_desktop` for layout-correct text input.

## Env B progress (Windows + VirtualBox, 2026-07-30)

| Item | Status | Evidence |
|---|---|---|
| Layer desktop under `retro-compositor` | PASS | `docs/screenshots/qa-layer-desktop-vbox.png` |
| Pointer → layer surface → UI action | PASS | `docs/screenshots/qa-layer-input-click.png` |
| Phase 3 exclusive Top/Bottom/Background | PASS | `docs/screenshots/qa-phase3-exclusive-chrome.png` — menu Top, dock Bottom, no dual menu; compositor log: `retroshell-menu` Top, `retroshell-dock` Bottom, `retroshell-desktop` Background |
| Top menu receives pointer | PASS | `docs/screenshots/qa-phase3-menu-click.png` — View menu highlighted |
| Gray PoC removed; kit chrome gated | PASS | `should_paint_kit_chrome(bound)=!bound`; `try_map_layer_shell_chrome` noop |
| `Super+O` opens Finder | PASS | `docs/screenshots/stage2-reqa-superO-finder.png` — peer Finder window; STATUS `FINDER_AFTER_SUPER_O=YES` |
| Lock via `Super+L` (`retro-lock`) | PASS | `docs/screenshots/stage2-reqa-locked.png` — password prompt; STATUS `LOCK_CLIENT=YES` |
| Lock cannot be bypassed by `Super+O` | PASS | `docs/screenshots/stage2-reqa-lock-nobypass.png` still locked; STATUS `FINDER_WHILE_LOCKED=1` (no new finder) |
| Password unlock restores session | PASS | `docs/screenshots/stage2-reqa-unlocked.png` — desktop + Finder visible again |

Orchestration: `packaging/vm/_stage2b-start.sh` / `_stage2-reqa.sh` + `RETROSHELL_LAYER_SHELL_CHROME=1`.
Note: ydotool absolute coords on this guest appear ~2× compositor logical coords.
After unlock, `retro-lock` may briefly remain as a defunct zombie in `pgrep`; the
scanout screenshot is the unlock evidence.

## QA environments

- **Env A:** UTM `Ubuntu` aarch64 (Claude session) — sway+grim / Xvfb screenshots.
- **Env B:** VirtualBox `retroshell-arch` x86_64 — `VBoxManage screenshotpng`.
