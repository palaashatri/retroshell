# QA — Stage 2 (Real session)

> **This doc holds evidence, not claims.** A row with no transcript/screenshot is
> `PENDING`, never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-2-real-session.md](../tasks/stage-2-real-session.md)

**Stage 2 definition of done:** on the VM — **lock cannot be bypassed by launching
an app**, **typing the password unlocks via `retro-lock`** (client calls
`ext-session-lock-v1` unlock, not a compositor-side password buffer), and
**`Super+O` opens Finder** with global menu mode (no in-window `MenuBar`).

**Stage status: VERIFIED** (2026-07-30, Windows host + VirtualBox `retroshell-arch`,
DRM/`vmwgfx`, `packaging/vm/_stage2-host.ps1` orchestration).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 2.0 | Input reaches a client on the DRM path (defect B) | PASS | `docs/screenshots/stage2-input.png` — compositor session + foot alive; keyboard drives clients (`Super+O`/`Super+L` below) |
| 2.1 | `Super+O` spawns Finder | PASS | `docs/screenshots/stage2-superO-finder.png`; compositor log `spawned client bin="finder" path=.../target/release/finder` |
| 2.2 | A button click drives an action (defect J) | PASS | `docs/screenshots/stage2-button-before-back.png` → `stage2-button.png` — pointer click changes Finder UI (Applications sidebar selection) |
| 2.3 | `ext-session-lock-v1` registered, compiles | PASS | compositor log `session locked` / `session unlocked` |
| 2.4 | Locked render shows only the lock surface | PASS | `docs/screenshots/stage2-locked.png` — full-screen lock prompt |
| 2.5 | Locked input routes only to the lock surface | PASS | `docs/screenshots/stage2-lock-nobypass.png` — lock screen persists after `Super+O`; no new finder spawn in compositor log between lock/unlock |
| 2.6 | `retro-lock` client unlocks session on password | PASS | `docs/screenshots/stage2-unlocked.png`; compositor log `session unlocked` after typed password |
| 2.7 | **DoD:** unbypassable lock, client unlock, `Super+O`→Finder | PASS | rows 2.0–2.6 + transcripts below |

## Fixes applied (2026-07-30)

- **`retro-lock` keyboard:** `SeatHandler::new_capability` → `get_keyboard()`; lock
  surfaces created in `locked()` callback.
- **Removed compositor password bypass** — unlock must come from the lock client.
- **`client_spawn.rs`:** `RETROSHELL_GLOBAL_MENU=1`, menu manifest dir, prefer
  `~/retroshell/target/release` over stale `/usr/local/bin`.
- **DRM session:** auto-spawn `retro-shell` for layer-shell chrome + global menu.
- **Shell chrome:** `session_output_size()` + `should_paint_kit_chrome()` — menu
  bar and dock span compositor output (1024×768).
- **QA scripts:** `_stage2-start.sh` + `_stage2-host.ps1` (host-driven; `pkill -f
  '[r]etro-compositor'` — `pkill -x retro-compositor` fails on Linux 15-char name
  limit); `ydotool` via SSH with `click 0xC0`.

## Session chrome architecture (menu bar + dock)

**Owned by `retro-shell`, not Finder or any app window.**

| Chrome | Owner | Where pixels are drawn today |
|---|---|---|
| Menu bar | `MenuServer` + `ShellDesktop.menu_bar` | Top of fullscreen `RetroShell Desktop` surface |
| Dock | `Dock` + `ShellDesktop.dock_view` | Bottom of same surface |
| Desktop wallpaper + icons | `ShellDesktop.desktop` | Same surface |
| External Finder (`Super+O`) | Wayland client | Separate toplevel; menus synced to shell menu bar |

Apps publish menu JSON when `RETROSHELL_GLOBAL_MENU=1`. No app embeds a dock.

## Transcripts

```text
# Host QA run 2026-07-30 (_stage2-host.ps1)
SESSION_READY
finder after Super+O: 1
retro-lock after Super+L: 1
finder while locked: 1          # pre-lock finder still in process list; no new spawn while locked
finder after unlock Super+O: 2

# Compositor (~/qa-stage2/compositor.log)
spawned client bin="retro-shell" path=/home/retro/retroshell/target/release/retro-shell
spawned client bin="finder" path=/home/retro/retroshell/target/release/finder
spawned client bin="retro-lock" path=/home/retro/retroshell/target/release/retro-lock
session locked
session unlocked
spawned client bin="finder" path=/home/retro/retroshell/target/release/finder
```

Orchestration:
```bash
powershell -File packaging/vm/_stage2-host.ps1
# → Stage 2 host orchestration complete.
```
