# SLOPOS-I Coordinate Systems Specification (`COORDINATE_SYSTEMS.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Specification  
**Scope:** Architecture document defining all 10 coordinate spaces and their mathematical conversions across SLOPOS-I.

---

## 1. Coordinate Space Matrix

| Coordinate Space | Origin (0, 0) | Unit | Scale Factor | Primary Usage |
| :--- | :--- | :--- | :--- | :--- |
| **1. Desktop-Global** | Top-left corner of primary monitor display (0, 0) | Logical Pixels | 1.0x (unscaled) | Window positioning, desktop icon placement, global menu bar placement, Wayland `wlr_layer_surface_v1` surfaces. |
| **2. Physical Framebuffer** | Top-left of GPU hardware framebuffer / swapchain | Physical Pixels | `device_pixel_ratio` ($S \ge 1.0$) | WGPU swapchain rendering, DRM scanout, raw pixel buffer blitting. |
| **3. Logical UI** | Top-left of application or shell root layout canvas | Logical DIPs | 1.0x (unscaled) | Widget bounding rects (`Rect { x, y, width, height }`), layout constraints, margin/padding. |
| **4. Window-Client** | Top-left of window client content area (inside titlebar & borders) | Logical Pixels | 1.0x (unscaled) | Child widget positioning inside `Window::set_content`, scroll offsets, client-relative clicks. |
| **5. Application-Local** | Top-left of specific widget bounding box | Local Pixels | 1.0x (unscaled) | Self-contained component drawing (`Button`, `TextField`, `TreeView`, `TerminalView`). |
| **6. Texture / Surface** | Top-left of GPU texture backing store | Physical Pixels | `scale_factor` | Offscreen rendering, glyph atlas textures, layer-shell surface buffers. |
| **7. Device-Scaled** | Scaled layout coordinates | Scaled DIPs | $S = \text{DPI} / 96.0$ | High-DPI UI scaling conversion intermediate step. |
| **8. Text-Layout** | Text baseline origin $(x_{\text{start}}, y_{\text{baseline}})$ | Subpixel float | 1.0x | Glyph advance accumulation, baseline snapping, text selection bounding boxes. |
| **9. Pointer-Input** | Top-left of screen (global pointer) | Logical Pixels | 1.0x | Mouse cursor event dispatching, drag-and-drop tracking, hit-testing. |
| **10. Clipping** | Current active clip rectangle origin | Logical Pixels | 1.0x | Canvas viewport clipping stack (`Canvas::push_clip` / `Canvas::pop_clip`). |

---

## 2. Transformations & Conversion Equations

### 2.1 Logical UI to Physical Framebuffer
$$\begin{aligned}
X_{\text{physical}} &= \lfloor X_{\text{logical}} \cdot S \rceil \\
Y_{\text{physical}} &= \lfloor Y_{\text{logical}} \cdot S \rceil \\
W_{\text{physical}} &= \lfloor W_{\text{logical}} \cdot S \rceil \\
H_{\text{physical}} &= \lfloor H_{\text{logical}} \cdot S \rceil
\end{aligned}$$
*Rule:* Layout calculations take place exclusively in **Logical UI** coordinates. Framebuffer dimensions are derived **exactly once** using rounding to nearest physical pixel integer.

### 2.2 Pointer-Input to Window-Client
$$\begin{aligned}
X_{\text{client}} &= X_{\text{global}} - X_{\text{window}} - W_{\text{border}} \\
Y_{\text{client}} &= Y_{\text{global}} - Y_{\text{window}} - H_{\text{titlebar}} - H_{\text{border}}
\end{aligned}$$
*Rule:* Pointer coordinates are converted **exactly once** upon entering `PointerDispatcher`.

### 2.3 1-Pixel Stroke Pixel Snapping
For 1-pixel strokes (`Canvas::line`, `Canvas::rect` border), rasterization requires half-pixel offsets to align with pixel centers:
$$\begin{aligned}
X_{\text{stroke\_snap}} &= \lfloor X \rfloor + 0.5 \\
Y_{\text{stroke\_snap}} &= \lfloor Y \rfloor + 0.5
\end{aligned}$$
For filled rectangles (`Canvas::fill_rect`):
$$\begin{aligned}
X_{\text{fill\_snap}} &= \lfloor X \rceil \\
Y_{\text{fill\_snap}} &= \lfloor Y \rceil
\end{aligned}$$

---

## 3. Strict DPI Invariants

1. **Single Scale Application**: Scaling transforms must NEVER be nested. A widget must not apply scale factor $S$ if its parent container canvas has already applied scale factor $S$.
2. **Text Target Scale**: Text must be rasterized at final physical resolution $S$ directly into the framebuffer texture without post-rasterization bilinear/bicubic resampling.
3. **Integer Surface Bounds**: Every Wayland surface, offscreen texture, and framebuffer must have integer physical dimensions ($W_{\text{physical}} \in \mathbb{Z}^+, H_{\text{physical}} \in \mathbb{Z}^+$).
