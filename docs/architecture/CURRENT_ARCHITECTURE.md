# SLOPOS-I Architecture & Component Map (`CURRENT_ARCHITECTURE.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Architecture Document  
**Scope:** Complete component breakdown, login flow, process hierarchy, and D-Bus integration map.

---

## 1. System Architecture Diagram

```
+-----------------------------------------------------------------------------------+
|                                  Display Manager                                  |
|                      (slopos-greeter / GDM / LightDM / tty login)                 |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
|                        Session Entry Point (start-slopos-i)                       |
+-----------------------------------------------------------------------------------+
                                         |
             +---------------------------+---------------------------+
             |                                                       |
             v                                                       v
+--------------------------+                               +--------------------+
|    slopos-compositor     | (Primary Standalone)          | labwc / sway       | (Fallback)
| (Smithay Wayland Server) |                               | (Host Compositor)  |
+--------------------------+                               +--------------------+
             |                                                       |
             +---------------------------+---------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
|                          slopos-shell (Layer Shell Client)                         |
|  - Top Global Menu Bar (`Layer::Top`)     - Desktop Background (`Layer::Background`)  |
|  - Bottom Dock (`Layer::Bottom`)          - Spotlight Overlay (`Layer::Overlay`)     |
+-----------------------------------------------------------------------------------+
          |                                                               |
          v                                                               v
+----------------------------+                                +---------------------+
| SLOPOS-I Native Apps       |                                | External Linux Apps |
| - Finder (com.slopos.finder)|                               | - Firefox (Wayland) |
| - TextEdit                 |                                | - MPV (OpenGL/VAAPI)|
| - Terminal (PTY / VT100)   |                                | - Doom (SDL2)       |
| - Settings (conf / D-Bus)  |                                | - LibreOffice       |
| - AppStore (catalog.json)  |                                | - GTK / Qt / Java   |
+----------------------------+                                +---------------------+
          |                                                               |
          +-------------------------------+-------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                            System D-Bus Services & Subsystems                     |
| - NetworkManager (Wi-Fi)                   - PipeWire / PulseAudio (Sound)        |
| - BlueZ (Bluetooth)                        - UPower & logind (Battery / Power)    |
| - UDisks2 (Removable Media)                - XDG Desktop Portals (FileChooser)    |
+-----------------------------------------------------------------------------------+
```

---

## 2. Ownership Analysis Matrix

| Subsystem Component | Codebase Owner | Implementation Status | Host Reliance / Delegation |
| :--- | :--- | :--- | :--- |
| **Wayland Compositor** | `slopos-compositor` | Standalone Smithay server (`DRM/KMS`, `winit` nested). | Falls back to Sway/labwc if DRM initialization fails. |
| **Desktop Shell** | `slopos-shell` | Custom layer-shell client (`wlr_layer_shell_v1`). | Fully owned by SLOPOS-I. |
| **IPC Bus** | `slopos-bus` | System & session message bus (`/tmp/slopos-bus.sock`). | Custom socket IPC; bridges to D-Bus. |
| **Window Management** | `slopos-compositor` | XDG-shell, foreign toplevel client, floating window placement. | Owned by SLOPOS-I. |
| **Terminal Emulator** | `apps/terminal` | Built-in PTY terminal emulator with VT100 parser. | Fully owned by SLOPOS-I. |
| **File Manager** | `apps/finder` | Built-in Finder with System 7 spatial directory navigation. | Fully owned by SLOPOS-I. |
| **Power Management** | `slopos-shell` | Idle config & lock screen integration (`slopos-lock`). | Delegates shutdown/reboot to systemd `org.freedesktop.login1`. |
| **Network Control** | `apps/settings` | Static config editor. | Delegates interface scanning to `NetworkManager`. |
| **Audio Subsystem** | `slopos-shell` | Volume bar IPC. | Delegates audio routing to `PipeWire` / `PulseAudio`. |
| **Portals** | `slopos-portal` | Custom `xdg-desktop-portal` implementation. | Exposes FileChooser and Screencast interfaces. |
