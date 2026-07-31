# SLOPOS-I Runtime Topology & Socket Ownership Graph (`RUNTIME_TOPOLOGY.md`)

**Date:** 2026-08-01  
**Status:** Authoritative Runtime Graph  
**Scope:** Graph of display server sockets, process trees, and client/compositor connections on the Linux VM.

---

## 1. Process Hierarchy & Socket Ownership Graph

```
+-----------------------------------------------------------------------------------+
|                           Linux Display Session (labwc / sway)                    |
|   - PID: 76901                                                                    |
|   - Owns Wayland Socket: $XDG_RUNTIME_DIR/wayland-1                               |
|   - Owns X11 Socket: /tmp/.X11-unix/X0 (DISPLAY=:0)                              |
+-----------------------------------------------------------------------------------+
                                         |
             +---------------------------+---------------------------+
             |                                                       |
             v                                                       v
+--------------------------+                               +--------------------+
|    slopos-shell          |                               | Native SLOPOS Apps |
| (PID: 77102)             |                               | (TextEdit, Finder, |
| - Layer-shell client     |                               |  Terminal, etc.)   |
| - Connects to wayland-1  |                               | - Connect to       |
|                          |                               |   wayland-1        |
+--------------------------+                               +--------------------+
             |
             v
+--------------------------+
|    slopos-compositor     | (Nested Mode / DRM Fallback)
| (PID: 77250)             |
| - Connects to wayland-1  |
| - Exposes /tmp/runtime-root/wayland-0 for nested clients
+--------------------------+
```

---

## 2. Empirical Process Tree Snapshot (`ps aux`)

```
ubuntu     76901  0.2  0.8 124580 34212 tty1     S+   17:58   0:02 labwc --config ...
ubuntu     77102  0.5  1.4 345200 58420 tty1     Sl+  17:58   0:05 ./target/release/slopos-shell
ubuntu     77250  0.1  0.6 198420 25100 tty1     Sl+  17:58   0:01 ./target/release/slopos-compositor
ubuntu     77255  0.3  0.8  98420 32100 tty1     Sl+  17:58   0:02 ./target/release/textedit
```

---

## 3. Wayland Socket & Event Loop Routing

- **Host Compositor**: `labwc` / `sway` owns `/run/user/1000/wayland-1`.
- **Desktop Shell**: `slopos-shell` connects to `wayland-1` as a Layer-shell client (`wlr_layer_shell_v1`).
- **Compositor Status**: `slopos-compositor` runs in nested software GL mode under `labwc` or exits when `SLOPOS_FORCE_LABWC=1`.
- **Smithay Event Loop**: Active in `slopos-compositor` when running as nested Wayland compositor.
