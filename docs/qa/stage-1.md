# QA — Stage 1 (Prove the Live Path)

> **This doc holds evidence, not claims.** The QA report's core lesson: a
> compositor was scored "85/100 daily-driver" while never having painted a window.
> Stage 1 exists to never do that again. See [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-1-prove-live-path.md](../tasks/stage-1-prove-live-path.md)

**Stage 1 definition of done — one of:**
- **(a)** a VM screenshot of Finder rendered by `retro-compositor` (not labwc), or
- **(b)** an evidenced diagnosis of exactly why it does not paint, citing the
  backend, client-bind status, and the specific scanout code path.

**Stage status: VERIFIED — DoD (a)** (2026-07-30, Windows host + VirtualBox /
x86_64 / `vmwgfx`). Screenshot: [../screenshots/stage1-finder.png](../screenshots/stage1-finder.png).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 1.1 | Compositor bring-up log captured on KMS | VERIFIED | Transcripts below |
| 1.2 | Which backend actually ran | VERIFIED | **DRM** (`session_drm`) |
| 1.3 | Client launched; screen captured; painted vs blank | VERIFIED | foot painted — `docs/screenshots/stage1-screen.png` |
| 1.4 | Finder screenshot = DoD (a) | VERIFIED | `docs/screenshots/stage1-finder.png` |
| 1.5 | evidenced diagnosis = DoD (b) | N/A | DoD (a) met |

## Backend determination (Task 1.2)

- Backend that ran: **DRM**
- Proving log lines:
  ```text
  compositor backend selection: session_mode=session_drm (DRM/KMS seat path)
  [retro-compositor] starting DRM/KMS session path (session_mode=session_drm …)
  [retro-compositor] opening DRM node /dev/dri/card0
  DrmCompositor initialized; GL composition active
  [retro-compositor] WAYLAND_DISPLAY=wayland-1 (DRM session)
  GL Vendor: "VMware, Inc." / GL Renderer: "SVGA3D; build: RELEASE;  LLVM;"
  ```
- Note: smithay warns `Unable to become drm master, assuming unprivileged mode`
  when launched over SSH `setsid` (not on the seat's controlling VT). Modeset +
  scanout still worked on this VBox/`vmwgfx` guest; foot and Finder both painted.

## Compositing observation (Task 1.3)

- Client used for sanity: `foot`
- Screen result: **client window painted**
- Screenshot: [../screenshots/stage1-screen.png](../screenshots/stage1-screen.png)
  (VBox `screenshotpng` while `FOOT=YES MAPPED=YES`)
- Compositor log: `toplevel mapped at (64,64) title=Untitled`
- `grim` failed (no wlr-screencopy on our compositor); host VBox screenshot used
  as evidence (allowed fallback in the Stage-1 task doc).

## Finder DoD (Task 1.4)

- Binary: `~/retroshell/target/release/finder`
- Hold state at capture: `COMPOSITOR_UP=YES CLIENT=YES MAPPED=YES SUBMISSIONS=11`
- Client: `Application 'Finder' started`; wgpu via Vulkan **llvmpipe**
- Screenshot shows Finder UI (sidebar, `/home/retro` icons, toolbar) on grey DRM
  clear color — **not** labwc, **not** a TTY.

## Diagnosis (Task 1.5 — N/A)

Skipped — DoD (a) met. Observation for Stage 2 planning:

- The comment at `session_drm.rs:894` ("DRM path does not yet composite client
  buffers to scanout") is **outdated on this GPU**: the GL `DrmCompositor` path
  ran (`composition_active=true`, `scanout_armed=false`) and **did** composite
  both `foot` (wl_shm) and Finder (wgpu/Vulkan llvmpipe).
- Remaining gaps for Stage 2 are session quality (DRM master when not on seat VT,
  screencopy/`grim`, decorations, input routing, lock), not "never painted."

## Transcripts

### Task 1.1 / 1.2 — bring-up (SSH `setsid`, 2026-07-30)

```text
COMPOSITOR_STILL_ALIVE=YES
session_mode=session_drm (DRM/KMS seat path)
libseat seat=seat0
opening DRM node /dev/dri/card0
WARN Unable to become drm master, assuming unprivileged mode
DrmDevice initializing
EGL platform PLATFORM_GBM_KHR / EGL Initialized
GL Version: OpenGL ES 3.0 Mesa 26.1.5-arch1.1
GL Vendor: VMware, Inc. / GL Renderer: SVGA3D; build: RELEASE;  LLVM;
DRM modeset plan: connector=Virtual-1 1280x800@60000mhz
DRM GL compositor ready (1280x800) — client surfaces will be composited
DrmCompositor initialized; GL composition active
WAYLAND_DISPLAY=wayland-1 (DRM session)
DRM session loop running (…; scanout_armed=false)
```

### Task 1.3 — foot

```text
COMPOSITOR_UP=YES FOOT=YES MAPPED=YES SOCK=wayland-1 MASTER_WARN=1
[retro-compositor/drm] toplevel mapped at (64,64) title=Untitled
[retro-compositor/drm] workspace active=0/8 windows=1 visible=1 …
```

### Task 1.4 — Finder

```text
COMPOSITOR_UP=YES CLIENT=YES MAPPED=YES SUBMISSIONS=11
Application 'Finder' started
Adapter Vulkan AdapterInfo { name: "llvmpipe (LLVM 22.1.8, 256 bits)", … }
Device::maintain: waiting for submission index 11+ (continuing past capture)
VBox screenshot → docs/screenshots/stage1-finder.png (16902 bytes)
```
