# SLOPOS-I Rendering Pipeline Audit (`RENDERING_AUDIT.md`)

**Date:** 2026-07-31  
**Status:** Audit & Root Cause Analysis Complete  
**Scope:** `slopos-render`, `slopos-sdk`, `slopos-kit`, `slopos-shell`, `slopos-compositor`, `finder`, `textedit`, `terminal`, `settings`, `appstore`

---

## 1. Defect & Root Cause Mapping

| # | Reported Rendering Defect | Root Cause Analysis | Responsible File(s) | Responsible Function(s) |
| :-: | :--- | :--- | :--- | :--- |
| **1** | Irregular character spacing | `rasterize_char` multiplied input font size by `1.4x` (`PxScale::from(font_size * 1.4)`), while `measure_text` and `glyph` caller loops assumed unscaled 13px glyph advances. In addition, horizontal left-side bearing (`bounds.min.x`) was omitted during glyph blitting. | [`crates/slopos-render/src/font.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-render/src/font.rs)<br>[`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs) | `rasterize_char`<br>`Canvas::glyph`<br>`Canvas::measure_text` |
| **2** | Words visually breaking apart ("Applicati ons", "Setti ngs", "SLOPOS- I") | Omission of `bounds.min.x` (glyph bearing offset) caused characters with non-zero left bearings (such as 'W', 'A', 'S', '-', 'I') to render at `x + col` instead of `x + bearing_x + col`. When combined with forced `glyph.advance.max(4.0)`, glyphs drifted horizontally relative to word boundaries. | [`crates/slopos-render/src/font.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-render/src/font.rs)<br>[`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs) | `rasterize_char`<br>`Canvas::glyph` |
| **3** | Clipped glyphs and truncated text | Text containers calculated line heights using fixed integer estimations (`len() * 7.0`) instead of font metric bounds (`ascent + descent + line_gap`). Labels clipped descenders (like 'g', 'p', 'y') against bottom container bounds. | [`crates/slopos-kit/src/label.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/label.rs)<br>[`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs) | `Label::draw`<br>`Canvas::text` |
| **4** | Inconsistent font baselines | Components rounded `y` positions independently (`(baseline_y + glyph.top).round()`) without a unified baseline snapping function, causing neighboring labels in toolbars, tab bars, and dialogs to offset vertically by 1px. | [`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs) | `Canvas::glyph`<br>`Canvas::text` |
| **5** | Terminal text clipped against top edge | Terminal row Y coordinate was computed as `row_idx * cell_height` with fixed `y + 4.0` offset, without adding font `ascent` or cell padding, placing row 0 glyph caps directly against the top window border. | [`apps/terminal/src/main.rs`](file:///Users/palaashatri/Code/retroshell/apps/terminal/src/main.rs) | `TerminalView::draw`<br>`draw_cell` |
| **6** | Controls using inconsistent text metrics | Buttons, text fields, popups, and tab bars used differing font measurement logic (`text.len() * 7.0` vs `measure_text`), producing misaligned button labels and uneven padding. | [`crates/slopos-kit/src/button.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/button.rs)<br>[`crates/slopos-kit/src/text_field.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/text_field.rs) | `Button::draw`<br>`TextField::draw` |
| **7** | Labels from obscured desktop icons remaining visible around foreground windows | Desktop background icon rendering lacked occlusion checking against open managed window bounds, painting icon labels across window borders when layer damage repainted. | [`crates/slopos-shell/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-shell/src/lib.rs) | `ShellDesktop::draw`<br>`draw_desktop_icons` |
| **8** | Borders and one-pixel lines appearing uneven | 1px stroke lines and rect edges used un-snapped floating point coordinates (`x as f32`) without half-pixel raster alignment (`(x + 0.5).floor()`), causing GPU rasterization anti-aliasing to feather 1px lines into 2px blurry lines. | [`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs) | `Canvas::rect`<br>`Canvas::line` |
| **9** | Window chrome and app content using different scaling | Application surfaces pre-scaled internal widgets while `ShellWindow` chrome applied logical-to-physical transforms independently, leading to double-scaled scrollbars and borders. | [`crates/slopos-kit/src/window.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/window.rs)<br>[`crates/slopos-render/src/renderer.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-render/src/renderer.rs) | `Window::draw`<br>`Renderer::render` |
| **10** | Incomplete or incorrect clipping | Child widget views lacked explicit `Canvas::push_clip` region enforcement, allowing scrolling lists and text fields to bleed content into adjacent toolbars or window frames. | [`crates/slopos-kit/src/tree_view.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/tree_view.rs)<br>[`crates/slopos-kit/src/window.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/window.rs) | `Window::set_content`<br>`TreeView::draw` |
| **11** | Overlays not fully occluding background content | Spotlight overlay and popup windows used semi-transparent background fill colors (`[0.9, 0.9, 0.9, 0.95]`) without painting a solid opaque base rect or establishing an explicit occlusion clip. | [`crates/slopos-shell/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-shell/src/lib.rs) | `draw_spotlight_overlay`<br>`draw_popup_menu` |
| **12** | Dark mode applying inconsistently | Certain window frames, background clear colors, and third-party app widgets read global theme settings from different sources (`ThemeManager` vs `render_dark_mode()`), leaving light background elements in dark mode. | [`crates/slopos-shell/src/theme_manager.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-shell/src/theme_manager.rs)<br>[`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs) | `ThemeManager::apply_theme`<br>`render_dark_mode` |
| **13** | Off-by-one errors around title bars, status bars, and resize corners | Window titlebar height (19px), status bar height (18px), and resize grip bounds (12x12px) used hardcoded static offsets (`height - 18`) without accounting for 1px outer bevel borders. | [`crates/slopos-sdk/src/lib.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-sdk/src/lib.rs)<br>[`crates/slopos-kit/src/window.rs`](file:///Users/palaashatri/Code/retroshell/crates/slopos-kit/src/window.rs) | `draw_classic_titlebar`<br>`draw_window_frame` |

---

## 2. Remediation Strategy & Execution Roadmap

1. **Unify Text Rasterization & Metrics (`slopos-render` & `slopos-sdk`)**:
   - Fix `rasterize_char` scale factor to exact 1.0x specified `font_size`.
   - Add `bearing_x` and `bearing_y` to `RasterGlyph`.
   - Update `Canvas::glyph` to position glyph pixels at `(x + bearing_x + col).round()` and `(baseline_y + bearing_y + row).round()`.
2. **Centralize Pixel Snapping & Scale Factor**:
   - Provide `snap_point_to_pixel`, `snap_rect_to_pixel`, and `snap_stroke_1px` in `slopos-sdk`.
   - Enforce single-pass DPI scale factor calculation across shell chrome and app surfaces.
3. **Enforce 10-Tier Layer Order & Occlusion Clipping**:
   - Ensure opaque window rectangles remove obscured desktop icon label regions from repaint loops.
   - Enforce strict `push_clip` / `pop_clip` bounds around all client content areas.
