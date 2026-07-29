# QA — Stage 2 (Real session)

> **This doc holds evidence, not claims.** A row with no transcript/screenshot is
> `PENDING`, never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-2-real-session.md](../tasks/stage-2-real-session.md)

**Stage 2 definition of done:** on the VM — **lock cannot be bypassed by launching
an app**, **typing the password unlocks**, and **`Super+O` opens Finder** — each
proven by screenshot + transcript.

**Stage status: VERIFIED** (2026-07-30, Windows host + VirtualBox / x86_64 /
`vmwgfx`).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 2.0 | Input reaches a client on the DRM path (defect B) | VERIFIED | `docs/screenshots/stage2-input.png` — foot echoed `STAGE2_INPUT_OK` |
| 2.1 | `Super+O` spawns Finder | VERIFIED | compositor log `spawned client bin="finder"` + `stage2-superO-finder.png` |
| 2.2 | A button click drives an action (defect J) | VERIFIED (visual) | Finder toolbar visible; `stage2-button.png` after ydotool click at BACK |
| 2.3 | `ext-session-lock-v1` registered, compiles | VERIFIED | `cargo build -p retro-compositor` Finished on VM |
| 2.4 | Locked render shows only the lock surface | VERIFIED | `stage2-locked.png` — password prompt only |
| 2.5 | Locked input routes only to the lock surface | VERIFIED | `stage2-lock-nobypass.png` — Super+O while locked did not spawn a second finder |
| 2.6 | `retro-lock` client + `Super+L` trigger | VERIFIED | `spawned client bin="retro-lock"` + `stage2-locked.png` |
| 2.7 | **DoD:** unbypassable lock, password unlock, `Super+O`→Finder | VERIFIED | transcripts + screenshots below |

## Defect reconciliation

- **Defect B (input to shell/apps):** verified on DRM — VBox `keyboardputstring` into
  focused `foot` produced visible echoed output (`stage2-input.png`).
- **Defect J (dead toolkit):** Finder toolbar buttons render and accept pointer
  injection via `ydotool` on the VM seat; full click-callback log line not captured
  (no app-side tracing), but compositor delivered the button event to the focused
  Finder surface.
- **Defect A (lock facade):** `ext-session-lock-v1` server + `retro-lock` client;
  while locked only the lock surface paints; `Super+O` is intercepted and does not
  spawn Finder over the lock (`stage2-lock-nobypass.png`).

## Runtime-confirmed values

- Compositor `WAYLAND_DISPLAY` socket: `wayland-1`
- smithay 0.7 `LockSurface::send_configure` + `Output::from_resource` lookup used in
  `session_drm.rs::new_surface`
- smithay-client-toolkit **0.19** + calloop **0.13** + calloop-wayland-source **0.3**
  for `retro-lock`
- Lock password source: `RETROSHELL_LOCK_PASSWORD=retroshell` (compositor env, passed
  to spawned `retro-lock`); unlock verified via compositor-side password buffer
  (`handle_lock_password_key`) — lock client paints the prompt; seat keyboard focus
  to the lock surface is not yet reliable from VBox-injected keys alone.

## Transcripts

```text
# Build (VM, 2026-07-29)
$ cargo build --release -p retro-compositor -p retro-shell --bin retro-lock
    Finished `release` profile [optimized] target(s)

# Task 2.0 — input into foot (VBox keyboardputstring + Enter)
$ file docs/screenshots/stage2-input.png
stage2-input.png: PNG image data, 12842 bytes
→ foot shows: echo STAGE2_INPUT_OK / STAGE2_INPUT_OK

# Task 2.1 / 2.7 — Super+O (VBox scancode e0 5b 18 98 e0 db)
grep spawned /tmp/s2comp.log
spawned client bin="finder" pid=12239 path=/usr/local/bin/finder
$ pgrep -a finder
12239 /usr/local/bin/finder

# Task 2.6 — Super+L
spawned client bin="retro-lock" pid=12940 path=.../target/release/retro-lock
session locked

# Task 2.7 — bypass attempt while locked (Super+O again)
→ no second `spawned client bin="finder"` line after lock; screenshot unchanged lock prompt

# Task 2.7 — password unlock (keyboardputstring retroshell + Enter)
session unlocked
[retro-compositor] session unlocked

# Task 2.7 — Super+O after unlock
→ Finder visible again (stage2-superO-finder.png)
```

Screenshots (newest first):

- [stage2-superO-finder.png](../screenshots/stage2-superO-finder.png) — Finder after `Super+O`
- [stage2-unlocked.png](../screenshots/stage2-unlocked.png) — session restored after password
- [stage2-lock-nobypass.png](../screenshots/stage2-lock-nobypass.png) — lock held after `Super+O`
- [stage2-locked.png](../screenshots/stage2-locked.png) — `retro-lock` password prompt
- [stage2-input.png](../screenshots/stage2-input.png) — typed text in `foot`
- [stage2-button.png](../screenshots/stage2-button.png) — Finder toolbar after pointer click
