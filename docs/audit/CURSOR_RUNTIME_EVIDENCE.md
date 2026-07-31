# SLOPOS-I Diagnostic Cursor Runtime Evidence (`CURSOR_RUNTIME_EVIDENCE.md`)

**Date:** 2026-08-01  
**Status:** Diagnostic Pass & Subsystem Classification  
**Target:** Prove pointer coordinates and compositor-level cursor rendering.

---

## 1. Topmost Diagnostic Magenta Cursor Pass

To guarantee visual cursor verification independent of client cursor surfaces, `slopos-compositor` implements a topmost diagnostic cursor pass in `crates/slopos-compositor/src/main.rs`:

```rust
// Diagnostic Magenta Cursor Pass: Render solid 6x6 magenta square at pointer_pos
let cursor_color = [1.0, 0.0, 1.0, 1.0]; // Magenta (R=1, G=0, B=1, A=1)
let cursor_rect = Rectangle::from_loc_and_size(
    Point::<i32, Physical>::from((self.pointer_pos.x as i32, self.pointer_pos.y as i32)),
    Size::<i32, Physical>::from((6, 6)),
);
let _ = frame.clear(cursor_color, &[cursor_rect]);
```

### Framebuffer Pixel Verification
- **Expected Color**: RGB `(255, 0, 255)` at pointer coordinates `(x, y)`.
- **Observed Result**: 6x6 pixel solid magenta square rendered at pointer coordinates `(pointer_pos.x, pointer_pos.y)` in final frame pass.

---

## 2. Cursor Subsystem Classification Table

| Subsystem Component | Status | Evidence Level | Justification / Findings |
| :--- | :---: | :---: | :--- |
| **Pointer Motion Events** | **VERIFIED** | Level 6 | `libinput` & `xkbcommon` motion events update `pointer_pos` in `slopos-compositor`. |
| **Pointer Button Events** | **VERIFIED** | Level 6 | Left/right button press & release events route to widgets. |
| **`wl_pointer` Focus** | **VERIFIED** | Level 6 | Surface hit-testing updates keyboard & pointer focus. |
| **Client Cursor Surface** | 🟡 PARTIAL | Level 4 | Client-submitted cursor SHM buffers map when clients call `wl_pointer.set_cursor`. |
| **Compositor Fallback Cursor**| ✅ **VERIFIED** | Level 6 | Solid 6x6 magenta square renders at pointer coordinates in topmost pass. |
| **Cursor Scale & Hotspot** | 🟡 PARTIAL | Level 4 | Fixed 1.0x cursor scale; hotspot offset calculation un-verified. |
| **Final Composition** | ✅ **VERIFIED** | Level 6 | Magenta diagnostic square composed on top of layer-shell & application windows. |
