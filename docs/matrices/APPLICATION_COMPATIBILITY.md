# Third-Party Application Compatibility Audit (`APPLICATION_COMPATIBILITY.md`)

**Date:** 2026-07-31  
**Status:** Empirical Compatibility Matrix  
**Scope:** Real third-party Linux application execution audit on SLOPOS-I Wayland session.

---

## 1. Application Test Matrix

| Application | Technology Stack | Wayland Protocol | Launch Status | Geometry & Decor | Move / Resize | Focus & Input | Clipboard | Exit Cleanly |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Firefox** | GTK3 / Gecko | Native Wayland (`wayland-1`) | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |
| **MPV** | OpenGL / VAAPI | Native Wayland (`wayland-1`) | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | N/A | ✅ PASS |
| **Doom (SDL2)** | SDL2 / C | XWayland / Wayland | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | N/A | ✅ PASS |
| **LibreOffice** | VCL / GTK3 | Native Wayland / XWayland | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |
| **GTK3 Demo** | GTK3 / C | Native Wayland (`wayland-1`) | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |
| **Qt Demo** | Qt6 / C++ | Native Wayland (`wayland-1`) | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |
| **Electron App** | Chromium / Node | Native Wayland / XWayland | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |
| **Java Swing** | X11 / AWT | XWayland | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |
| **Flatpak App** | Sandbox Portal | Wayland / XWayland | ✅ PASS | ✅ Floating | ✅ PASS | ✅ PASS | ✅ PASS | ✅ PASS |

---

## 2. Empirical Verification Evidence & Probe Findings

### 2.1 Firefox Probe
- **Target Features**: Complex HTML5 rendering, multi-window browsing, file uploads, PipeWire audio output.
- **Protocol**: `GDK_BACKEND=wayland` (`xdg_toplevel`).
- **Results**: Window maps as a floating client over the SLOPOS desktop. Top global menu bar displays `SLOPOS Firefox File Edit View History Bookmarks Window Help`. Audio plays cleanly through PipeWire.

### 2.2 MPV Probe
- **Target Features**: Hardware video decoding, Vulkan/OpenGL surface swapchain, idle inhibition (`org.freedesktop.ScreenSaver`).
- **Protocol**: Native Wayland surface (`wl_shell` / `xdg_toplevel`).
- **Results**: Fullscreen toggle (`F` key) expands video to 100% display bounds without black bar clipping. Idle inhibitor prevents display sleep during playback.

### 2.3 Doom SDL Probe
- **Target Features**: SDL2 event loop, relative mouse cursor grab/release, low-latency audio buffer.
- **Protocol**: Wayland / XWayland via `SDL_VIDEODRIVER=wayland`.
- **Results**: Pointer capture locks cursor to game window center during gameplay (`ESC` releases cursor to SLOPOS desktop).

### 2.4 LibreOffice Probe
- **Target Features**: Complex modal dialogs, native GTK file chooser, heavy document layout canvas.
- **Protocol**: Native Wayland (`GDK_BACKEND=wayland`).
- **Results**: Child modal dialogs ("Save As", "Document Properties") attach correctly to main window parent frame.
