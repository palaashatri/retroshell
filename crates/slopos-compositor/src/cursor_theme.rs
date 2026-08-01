//! Fallback cursor bitmap for `CursorImageStatus::Named`.
//!
//! A real cursor theme is an XCursor file on disk (e.g.
//! `/usr/share/icons/*/cursors/left_ptr`) and parsing one is a new dependency
//! this crate does not have. Rather than pull one in just to draw a pointer,
//! this module draws a classic arrow procedurally: a filled-black shape with a
//! white outline, so a client that leaves the cursor `Named` (never sets its
//! own surface, see `session_drm.rs`'s `CursorImageStatus::Surface` handling)
//! still has a visible pointer. See `AGENTS.md`, phase P1 (Cursor).
//!
//! Wiring this into the DRM render-element list is a later step; this module
//! only produces the pixels.

#![cfg(target_os = "linux")]

/// A small cursor image plus its hotspot, ready to hand to a renderer as raw
/// ARGB8888 bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorBitmap {
    pub width: i32,
    pub height: i32,
    /// Pixel offset from the image's top-left to the point the cursor
    /// represents (where clicks actually land). `(0, 0)` for [`default_arrow`]:
    /// the arrow's tip is the image's first pixel.
    pub hotspot: (i32, i32),
    /// `width * height * 4` bytes, row-major, one pixel per 4 bytes in
    /// `B, G, R, A` order — the in-memory byte layout of `DRM_FORMAT_ARGB8888`
    /// on a little-endian host, matching the `Argb8888` format `session_drm.rs`
    /// already negotiates for cursor plane buffers.
    pub argb: Vec<u8>,
}

const WIDTH: i32 = 24;
const HEIGHT: i32 = 24;

/// Classic filled-black, white-outlined arrow pointer, hotspot at its tip `(0, 0)`.
///
/// Shape: a triangular arrowhead with its apex at the hotspot, plus a small
/// tail rectangle hanging off the triangle's base — the familiar desktop
/// pointer silhouette. Drawn pixel-by-pixel below rather than loaded from an
/// asset, so this crate needs no new dependency to have *a* visible pointer.
pub fn default_arrow() -> CursorBitmap {
    let mut argb = vec![0u8; (WIDTH * HEIGHT * 4) as usize];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = ((y * WIDTH + x) * 4) as usize;
            if arrow_fill(x, y) {
                write_pixel(&mut argb, idx, 0x00, 0x00, 0x00); // opaque black
            } else if touches_fill(x, y) {
                write_pixel(&mut argb, idx, 0xFF, 0xFF, 0xFF); // opaque white outline
            }
            // else: left as zeroed bytes, i.e. fully transparent.
        }
    }

    CursorBitmap {
        width: WIDTH,
        height: HEIGHT,
        hotspot: (0, 0),
        argb,
    }
}

/// Write one opaque BGRA pixel at byte offset `idx`.
fn write_pixel(buf: &mut [u8], idx: usize, b: u8, g: u8, r: u8) {
    buf[idx] = b;
    buf[idx + 1] = g;
    buf[idx + 2] = r;
    buf[idx + 3] = 0xFF;
}

/// True when `(x, y)` is part of the arrow's solid black silhouette.
///
/// Two pieces, sharing the row `y == 16` as a seam so they read as one shape:
/// - a right triangle, apex `(0, 0)`, right edge widening linearly down to
///   `(12, 16)` — the arrowhead;
/// - a narrow rectangle `x in [3, 7], y in [16, 21]` — the tail.
fn arrow_fill(x: i32, y: i32) -> bool {
    let in_head = (0..=16).contains(&y) && x >= 0 && x <= (y * 12) / 16;
    let in_tail = (3..=7).contains(&x) && (16..=21).contains(&y);
    in_head || in_tail
}

/// True when `(x, y)` is itself outside the silhouette but 8-connected to a
/// pixel that is inside it — i.e. it belongs on the 1px outline ring.
fn touches_fill(x: i32, y: i32) -> bool {
    if arrow_fill(x, y) {
        return false;
    }
    for dy in -1..=1 {
        for dx in -1..=1 {
            if (dx, dy) != (0, 0) && arrow_fill(x + dx, y + dy) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_are_24x24_with_zero_hotspot() {
        let c = default_arrow();
        assert_eq!(c.width, 24);
        assert_eq!(c.height, 24);
        assert_eq!(c.hotspot, (0, 0));
    }

    #[test]
    fn buffer_length_matches_width_times_height_times_4() {
        let c = default_arrow();
        assert_eq!(c.argb.len(), (c.width * c.height * 4) as usize);
    }

    #[test]
    fn hotspot_pixel_is_opaque() {
        let c = default_arrow();
        // Hotspot (0, 0) is the image's first pixel: bytes [0..4), alpha at index 3.
        assert_eq!(c.argb[3], 0xFF, "hotspot pixel must be opaque");
    }

    #[test]
    fn image_contains_both_black_and_white_pixels() {
        let c = default_arrow();
        let mut has_black = false;
        let mut has_white = false;
        for px in c.argb.chunks_exact(4) {
            let (b, g, r, a) = (px[0], px[1], px[2], px[3]);
            if a == 0xFF && b == 0x00 && g == 0x00 && r == 0x00 {
                has_black = true;
            }
            if a == 0xFF && b == 0xFF && g == 0xFF && r == 0xFF {
                has_white = true;
            }
        }
        assert!(has_black, "expected at least one opaque black fill pixel");
        assert!(
            has_white,
            "expected at least one opaque white outline pixel"
        );
    }

    #[test]
    fn no_pixel_is_both_black_and_white() {
        // arrow_fill and touches_fill must be mutually exclusive by construction;
        // guard the invariant directly rather than only via the color scan above.
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                assert!(!(arrow_fill(x, y) && touches_fill(x, y)));
            }
        }
    }

    #[test]
    fn shape_is_non_trivial_and_bounded() {
        let c = default_arrow();
        let opaque = c.argb.chunks_exact(4).filter(|px| px[3] == 0xFF).count();
        // Comfortably more than a stray pixel, comfortably less than the whole canvas.
        assert!(opaque > 20);
        assert!(opaque < (c.width * c.height) as usize);
    }
}
