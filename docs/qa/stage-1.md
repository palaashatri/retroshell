# QA — Stage 1 (Prove the Live Path)

> **This doc holds evidence, not claims.** The QA report's core lesson: a
> compositor was scored "85/100 daily-driver" while never having painted a window.
> Stage 1 exists to never do that again. See [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-1-prove-live-path.md](../tasks/stage-1-prove-live-path.md)

**Stage 1 definition of done — one of:**
- **(a)** a VM screenshot of Finder rendered by `retro-compositor` (not labwc), or
- **(b)** an evidenced diagnosis of exactly why it does not paint, citing the
  backend, client-bind status, and the specific scanout code path.

Either outcome makes Stage 1 VERIFIED — the *verification* is the deliverable, not
a guaranteed green result.

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 1.1 | Compositor bring-up log captured on KMS | PENDING | _paste `stage1-bringup.log` grep_ |
| 1.2 | Which backend actually ran (DRM/X11/labwc/failed) | PENDING | _name backend + quote proving lines_ |
| 1.3 | Client launched; screen captured; painted vs blank | PENDING | _attach `stage1-screen.png`; state result_ |
| 1.4 | (if painted) Finder screenshot = DoD (a) | PENDING | _attach `docs/screenshots/stage1-finder.png`_ |
| 1.5 | (if blank) evidenced diagnosis = DoD (b) | PENDING | _fill diagnosis section below_ |

## Backend determination (Task 1.2)

- Backend that ran: _____ (one of: DRM, X11, labwc-fallback, failed)
- Proving log lines:
  ```text
  (paste the exact lines)
  ```

## Compositing observation (Task 1.3)

- Client used for sanity: `foot`
- Screen result: _____ (one of: "client window painted", "blank/black scanout,
  client alive in logs", "compositor crashed", "client failed to connect")
- Screenshot: `stage1-screen.png` (attach / describe)

## Diagnosis (Task 1.5 — only if it did not paint)

Answer each with quoted evidence:

- **Backend:** _____
- **Did the compositor accept the client connection?** (grep for `wl_registry` /
  `xdg_surface` bind) _____
- **Was scanout blank due to the known gap** (blank framebuffer presented, clients
  kept alive — comment at `crates/retro-compositor/src/session_drm.rs:894`)
  **or a different failure** (DRM error / GL init failure on virtio-gpu / panic)?
  _____
- **Is the GL `DrmCompositor` path reached** (gated behind `composition_active`)
  **or does the blank-buffer path run?** _____
- **Single next fix the evidence points to** (becomes the Stage-2 compositing spec
  problem statement): _____

## Transcripts

_Raw logs and command output, newest first._

```text
(none yet — Stage 1 has not been run on a VM)
```
