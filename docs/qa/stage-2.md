# QA — Stage 2 (Real session)

> **This doc holds evidence, not claims.** The QA report's lesson: a compositor was
> scored "85/100" while never painting a window. A row with no transcript/screenshot
> is `PENDING`, never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-2-real-session.md](../tasks/stage-2-real-session.md)

**Stage 2 definition of done (spec §4):** on the VM — **lock cannot be bypassed by
launching an app**, **typing the password unlocks**, and **`Super+O` opens
Finder** — each proven by a screenshot + transcript.

**Stage status: PENDING** (authored 2026-07-30, not yet executed).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 2.0 | Input reaches a client on the DRM path (defect B) | PENDING | _screenshot: typed text in client_ |
| 2.1 | `Super+O` spawns Finder | PENDING | _`pgrep finder` + screenshot_ |
| 2.2 | A button click drives an action (defect J) — or evidenced diagnosis | PENDING | _screenshot + outcome_ |
| 2.3 | `ext-session-lock-v1` registered, compiles | PENDING | _`cargo build -p retro-compositor` Finished_ |
| 2.4 | Locked render shows only the lock surface | PENDING | _build + Task 2.7 proof_ |
| 2.5 | Locked input routes only to the lock surface | PENDING | _build + Task 2.7 proof_ |
| 2.6 | `retro-lock` client + `Super+L` trigger | PENDING | _screenshot: locked prompt only_ |
| 2.7 | **DoD:** unbypassable lock, password unlock, `Super+O`→Finder | PENDING | _3 screenshots + narrative_ |

## Defect reconciliation (fill from evidence)

- **Defect B (input to shell/apps):** Stage 1 evidence says input works on DRM.
  Task 2.0 result: _____ (verified / diagnosis).
- **Defect J (dead toolkit):** `Button::handle_event` implemented+tested;
  `Widget::draw` is a no-op abstraction (not the render path). Task 2.2 result:
  _____ (click works / where the event is lost).
- **Defect A (lock facade):** was a client-side `bool` only. After Tasks 2.3–2.6,
  Task 2.7 proves the compositor enforces it. Result: _____.

## Runtime-confirmed values

- Compositor `WAYLAND_DISPLAY` socket used for spawned clients: _____
- smithay 0.7 `LockSurface` configure/Output-lookup API actually used (Task 2.3
  CONFIRM AT RUNTIME): _____
- smithay-client-toolkit session-lock API + version pinned for `retro-lock`: _____
- Lock password source (env `RETROSHELL_LOCK_PASSWORD` / config): _____

## Transcripts

_Raw command output + screenshots, newest first. Do not summarize._

```text
(none yet — Stage 2 has not been run on a VM)
```
