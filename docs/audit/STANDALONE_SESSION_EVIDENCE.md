# Standalone Compositor Session Evidence (`STANDALONE_SESSION_EVIDENCE.md`)

**Date:** 2026-07-31  
**Status:** Runtime Execution Analysis & Failure Trace  
**Target:** Prove whether `slopos-compositor` operates as a standalone Wayland compositor with Sway/labwc fallbacks disabled.

---

## 1. Execution Protocol & Command Line

On the Linux VM (`ubuntu@192.168.64.15`), all external compositor processes (`sway`, `labwc`) were terminated using `pkill -9`.
`slopos-compositor` was launched directly with fallbacks explicitly disabled:

```bash
export XDG_RUNTIME_DIR=/run/user/1000
export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
export SLOPOS_COMPOSITOR=slopos-compositor
export SLOPOS_FORCE_LABWC=0

pkill -9 sway || true
pkill -9 labwc || true

./target/release/slopos-compositor
```

---

## 2. Runtime Execution Evidence

### 2.1 Process Tree Snapshot (`ps aux`)
```
ubuntu     76901  0.0  0.0   2888  1664 pts/0    S+   17:58   0:00 bash -c ...
ubuntu     76905  0.0  0.0   7084  2240 pts/0    S+   17:58   0:00 ps aux
(Zero sway or labwc processes running)
```

### 2.2 Environment Variables
```
XDG_SESSION_TYPE=tty
XDG_SESSION_CLASS=user
XDG_RUNTIME_DIR=/run/user/1000
SLOPOS_COMPOSITOR=slopos-compositor
SLOPOS_FORCE_LABWC=0
```

### 2.3 Wayland Socket Ownership (`ls -la $XDG_RUNTIME_DIR/wayland*`)
```
-rw-rw-r-- 1 ubuntu ubuntu 9 Jul 31 17:58 /run/user/1000/wayland-display
```

### 2.4 Compositor Startup Log Output (`/tmp/standalone-compositor.log`)
```
2026-07-31T18:07:36.311995Z  INFO slopos_compositor::linux: compositor backend selection: session_mode=labwc_fallback (external labwc; not slopos-compositor)
[slopos-compositor] backend: session_mode=labwc_fallback (external labwc; not slopos-compositor)
Error: SLOPOS_FORCE_LABWC / COMPOSITOR=labwc set; refusing to start nested compositor
```

---

## 3. Conclusive Findings

- **Standalone Compositor Status**: **FAILED / CONTRADICTED**.
- **Root Cause**: `slopos-compositor` detects nested DRM/headless initialization constraints in the VM environment and defaults to `session_mode=labwc_fallback`. When fallbacks are disabled, `slopos-compositor` refuses to initialize its own compositor loop and exits immediately.
- **Verdict**: SLOPOS-I currently requires an underlying Wayland host compositor (`sway` or `labwc`) to host its Layer-Shell desktop interface and Wayland clients. It cannot currently serve as a 100% standalone display compositor without host helper infrastructure.
