# SLOPOS-I Compositor & Layer Architecture (`COMPOSITOR_ARCHITECTURE.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Specification  
**Scope:** Architecture document defining layer ordering, window clipping, damage tracking, and surface compositing across SLOPOS-I.

---

## 1. Authoritative 10-Tier Layer Order

All visual elements across `slopos-compositor` and `slopos-shell` adhere to a strict, non-bypassable 10-tier z-index hierarchy:

```
[Layer 10] Pointer & Debug Overlays      (Top-most: Mouse Cursor, Bounds Diagnostics)
[Layer  9] Spotlight Overlay Search     (Super+Space Global Input Field)
[Layer  8] Modal Dialog Windows          (System Dialogs, Alert Boxes)
[Layer  7] Modal Dimming Backdrop        (Semi-transparent backdrop layer)
[Layer  6] Bottom Desktop Dock           (System 7 Beveled Taskbar Dock)
[Layer  5] Global Top Menu Bar & Popups  (System Menu, Active App Menus, Dropdowns)
[Layer  4] Active (Focused) Window       (Top-most managed xdg_toplevel window)
[Layer  3] Normal Managed Windows        (Stack of open application windows)
[Layer  2] Desktop Icons & Labels        (Grid icons: Hard Disk, Home, Applications, etc.)
[Layer  1] Desktop Background Wallpaper  (Base desktop wallpaper & pattern canvas)
```

---

## 2. Window Clipping & Occlusion Boundaries

Every managed window (`ShellWindow` / `Window`) enforces 4 explicit bounding regions:

```
+-------------------------------------------------------------+
| Frame Rectangle (Outer Bevel & Shadows)                     |
|  +-------------------------------------------------------+  |
|  | Decoration Clip (Titlebar, Close/Zoom Box, Rail Grips) |  |
|  +-------------------------------------------------------+  |
|  | Client Rectangle (Inner Application Workspace)        |  |
|  |  +-------------------------------------------------+  |  |
|  |  | Content Clip (Child Widgets Viewport)           |  |  |
|  |  +-------------------------------------------------+  |  |
|  +-------------------------------------------------------+  |
+-------------------------------------------------------------+
```

### 2.1 Regional Boundaries Definition
1. **Frame Rectangle**: Total bounding box including outer 1px System 7 borders and bevels.
2. **Decoration Clip**: Contains titlebar (19px height), grips, close box (11x11px), zoom box (11x11px), and status bar (18px height).
3. **Client Rectangle**: Inner application workspace bounded by $(X_{\text{frame}} + 1, Y_{\text{frame}} + 20, W_{\text{frame}} - 2, H_{\text{frame}} - 39)$.
4. **Content Clip**: Viewport clip pushed to `Canvas` (`Canvas::push_clip`) during widget rendering. Child widgets CANNOT paint outside Content Clip.

---

## 3. Occlusion & Damage Tracking Rules

1. **Opaque Window Occlusion**: Any managed window that is opaque ($A = 1.0$) subtracts its Frame Rectangle from the background repaint region.
2. **Desktop Icon Label Occlusion**: Desktop icon text labels sitting beneath an opaque window frame MUST NOT be repainted during damage updates.
3. **Overlay Opacity**: Spotlight overlay (Layer 9) and Dropdown popups (Layer 5) must paint an opaque solid base rectangle (`[0.9, 0.9, 0.9, 1.0]`) before drawing drop-shadows or text, guaranteeing zero background text leak-through.
4. **Stale Damage Clearing**: Before painting any damaged region, the surface buffer must clear the damaged rect using the resolved active theme background color.
5. **No Double Resampling**: App surfaces render directly at target client physical resolution; no post-render scaling is applied to app surfaces.
