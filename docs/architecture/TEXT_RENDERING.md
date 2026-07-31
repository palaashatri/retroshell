# SLOPOS-I Text Rendering & Typography Specification (`TEXT_RENDERING.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Specification  
**Scope:** Architecture document defining unified text metrics, glyph bearing, baseline alignment, and rasterization rules across SLOPOS-I.

---

## 1. Unified Text Rendering Architecture

All text rendering across SLOPOS-I (Menu bar, Window titles, Buttons, Desktop icon labels, Finder, Settings, App Store, TextEdit, Terminal, Spotlight, Status bars, Dialogs) is powered by a unified text-rendering API in `slopos-sdk` (`Canvas::text`, `Canvas::glyph`, `Canvas::measure_text`) calling `slopos_render::rasterize_char`.

```
                        +--------------------------------+
                        |  App / Shell UI Component      |
                        +--------------------------------+
                                        |
                                        v
                        +--------------------------------+
                        | Canvas::text / Canvas::glyph   |  <-- Single Shared API
                        +--------------------------------+
                                        |
                                        v
                        +--------------------------------+
                        | slopos_render::rasterize_char  |  <-- Font Rasterizer
                        +--------------------------------+
                                        |
                                        v
                        +--------------------------------+
                        | ab_glyph / System Fallback TTF |
                        +--------------------------------+
```

---

## 2. Text Metrics & Glyph Bearing Mathematical Model

### 2.1 RasterGlyph Data Structure
Each rasterized character returns exact typographic metrics:
```rust
pub struct RasterGlyph {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub advance: f32,
    /// Horizontal left-side bearing (min x bound from glyph outline)
    pub bearing_x: f32,
    /// Vertical top bearing relative to baseline (min y bound from glyph outline)
    pub bearing_y: f32,
    /// Font ascent at this size (distance from baseline to line top)
    pub ascent: f32,
}
```

### 2.2 Glyph Positioning & Advance Equation
For a character `ch` rendered at text baseline origin $(X_0, Y_{\text{baseline}})$:

1. **Baseline Y Coordinate**:
   $$Y_{\text{baseline}} = Y_0 + \text{ascent}$$

2. **Pixel Position for Glyph Bitmap Cell $(col, row)$**:
   $$X_{\text{pixel}} = \lfloor X_{\text{cursor}} + \text{bearing\_x} + col \rceil$$
   $$Y_{\text{pixel}} = \lfloor Y_{\text{baseline}} + \text{bearing\_y} + row \rceil$$

3. **Horizontal Cursor Advance**:
   $$X_{\text{cursor\_next}} = X_{\text{cursor}} + \text{advance}$$

---

## 3. Strict Typography Rules

1. **No Artificial Scaling Multipliers**: `rasterize_char(ch, font_size)` must rasterize glyphs at the exact requested `font_size` (e.g. 13.0px), never scaling by internal multipliers like `1.4x`.
2. **Horizontal Side Bearings Enforced**: `bearing_x` (`bounds.min.x`) must be included in glyph pixel placement. Omitting `bearing_x` breaks kerning and creates word-splitting gaps ("Applicati ons", "SLOPOS- I").
3. **No Arbitrary Advance Clamping**: `advance` must be the exact font advance (`scaled_font.h_advance(glyph_id)`), without imposing artificial minimum limits like `max(4.0)` which distort narrow glyphs.
4. **Single-Source Text Measurement**: `measure_text` and `text` must use the exact same `RasterGlyph.advance` calculation. Never estimate text width using `len() * 7.0`.
5. **Terminal Cell Baseline**: Terminal cell glyphs must align to a stable cell baseline:
   $$Y_{\text{cell\_baseline}} = Y_{\text{cell\_top}} + \text{ascent}$$
   Terminal cell height must equal:
   $$\text{CellHeight} = \text{ascent} + \text{descent} + \text{line\_gap}$$
6. **Subpixel & Antialiasing Thresholding**: Glyph alpha coverage is blended cleanly using:
   $$\alpha_{\text{final}} = \alpha_{\text{glyph}} \cdot \alpha_{\text{color}}$$
