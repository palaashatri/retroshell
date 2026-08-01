use ab_glyph::{Font as AbFont, FontArc, PxScale, ScaleFont};
use cosmic_text::{Family, FontSystem};
use fontdb::{Query, Stretch, Style, Weight};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, OnceLock};

pub struct RetroFont {
    pub font_system: Arc<Mutex<FontSystem>>,
}

impl Default for RetroFont {
    fn default() -> Self {
        Self::new()
    }
}

impl RetroFont {
    pub fn new() -> Self {
        Self {
            font_system: Arc::new(Mutex::new(FontSystem::new())),
        }
    }

    pub fn font_system(&self) -> Arc<Mutex<FontSystem>> {
        self.font_system.clone()
    }
}

static AB_FONT: OnceLock<Option<FontArc>> = OnceLock::new();

const SYSTEM_FONT_FALLBACKS: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/System/Library/Fonts/Supplemental/Helvetica.ttf",
    "/Library/Fonts/Arial.ttf",
];

fn load_ab_font() -> Option<FontArc> {
    let mut font_sys = FontSystem::new();
    let query = Query {
        families: &[Family::SansSerif],
        weight: Weight::NORMAL,
        stretch: Stretch::Normal,
        style: Style::Normal,
    };
    let font_id = font_sys.db_mut().query(&query);
    if let Some(id) = font_id {
        if let Some(data) = font_sys.db().with_face_data(id, |data, _| data.to_vec()) {
            if let Ok(font) = FontArc::try_from_vec(data) {
                return Some(font);
            }
        }
    }

    for path in SYSTEM_FONT_FALLBACKS {
        if let Ok(data) = fs::read(path) {
            if let Ok(font) = FontArc::try_from_vec(data) {
                return Some(font);
            }
        }
    }

    log::warn!("no usable system sans-serif font found; falling back to bitmap glyphs");
    None
}

/// Rasterized glyph data with exact typographic bearings and metrics.
#[derive(Debug, Clone)]
pub struct RasterGlyph {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub advance: f32,
    /// Horizontal left-side bearing (min x offset from glyph origin).
    pub bearing_x: f32,
    /// Vertical top bearing relative to baseline (min y offset from baseline).
    pub bearing_y: f32,
    /// Top offset relative to baseline (legacy alias for bearing_y).
    pub top: f32,
    /// Font ascent at this size (in pixels): distance from baseline to line top.
    pub ascent: f32,
    /// Font descent at this size (in pixels): distance from baseline to line bottom.
    pub descent: f32,
}

/// The cache key uses the exact physical pixel size used by `ab_glyph`.
/// Keeping the float bits avoids rounding two nearby scales into the same
/// raster while still giving the hash map a stable, comparable key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphCacheKey {
    ch: char,
    font_size_bits: u32,
}

impl GlyphCacheKey {
    fn new(ch: char, font_size: f32) -> Option<Self> {
        if !font_size.is_finite() || font_size <= 0.0 {
            return None;
        }

        Some(Self {
            ch,
            font_size_bits: font_size.to_bits(),
        })
    }
}

const GLYPH_CACHE_CAPACITY: usize = 4096;

#[derive(Default)]
struct GlyphCache {
    entries: HashMap<GlyphCacheKey, Option<RasterGlyph>>,
}

impl GlyphCache {
    fn lookup(&self, key: GlyphCacheKey) -> Option<Option<RasterGlyph>> {
        self.entries.get(&key).cloned()
    }

    fn insert(&mut self, key: GlyphCacheKey, glyph: Option<RasterGlyph>) {
        if !self.entries.contains_key(&key) && self.entries.len() >= GLYPH_CACHE_CAPACITY {
            // Clearing at a fixed entry count keeps memory bounded without
            // depending on hash-map iteration order for eviction.
            self.entries.clear();
        }
        self.entries.insert(key, glyph);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

static RASTER_CACHE: OnceLock<Mutex<GlyphCache>> = OnceLock::new();

fn raster_cache() -> &'static Mutex<GlyphCache> {
    RASTER_CACHE.get_or_init(|| Mutex::new(GlyphCache::default()))
}

pub fn rasterize_char(ch: char, font_size: f32) -> Option<RasterGlyph> {
    if ch.is_control() {
        return None;
    }
    let key = GlyphCacheKey::new(ch, font_size)?;
    if let Some(cached) = raster_cache().lock().lookup(key) {
        return cached;
    }

    let glyph = rasterize_char_uncached(ch, font_size);
    raster_cache().lock().insert(key, glyph.clone());
    glyph
}

fn rasterize_char_uncached(ch: char, font_size: f32) -> Option<RasterGlyph> {
    let font = AB_FONT.get_or_init(load_ab_font).as_ref()?;
    let glyph_id = AbFont::glyph_id(font, ch);
    if glyph_id.0 == 0 && !ch.is_control() {
        return None;
    }
    let px_scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(px_scale);
    let advance = scaled_font.h_advance(glyph_id);
    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();
    let glyph = glyph_id.with_scale(px_scale);
    let Some(outlined) = AbFont::outline_glyph(font, glyph) else {
        return Some(RasterGlyph {
            data: Vec::new(),
            width: 0,
            height: 0,
            advance,
            bearing_x: 0.0,
            bearing_y: -ascent,
            top: -ascent,
            ascent,
            descent,
        });
    };
    let bounds = outlined.px_bounds();
    let width = bounds.width().ceil() as u32;
    let height = bounds.height().ceil() as u32;
    let bearing_x = bounds.min.x;
    let bearing_y = bounds.min.y;
    let top = bounds.min.y;
    if width == 0 || height == 0 {
        return Some(RasterGlyph {
            data: Vec::new(),
            width: 0,
            height: 0,
            advance,
            bearing_x,
            bearing_y,
            top,
            ascent,
            descent,
        });
    }
    let mut data = vec![0u8; (width * height) as usize];
    outlined.draw(|x, y, coverage| {
        let ix = x as usize;
        let iy = y as usize;
        if ix < width as usize && iy < height as usize {
            data[iy * width as usize + ix] = (coverage * 255.0) as u8;
        }
    });
    Some(RasterGlyph {
        data,
        width,
        height,
        advance,
        bearing_x,
        bearing_y,
        top,
        ascent,
        descent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_glyph() -> RasterGlyph {
        RasterGlyph {
            data: vec![255],
            width: 1,
            height: 1,
            advance: 1.0,
            bearing_x: 0.0,
            bearing_y: 0.0,
            top: 0.0,
            ascent: 1.0,
            descent: 0.0,
        }
    }

    #[test]
    fn cache_key_is_stable_and_scale_sensitive() {
        let base = GlyphCacheKey::new('A', 13.0).expect("valid size");

        assert_eq!(base, GlyphCacheKey::new('A', 13.0).expect("valid size"));
        assert_ne!(base, GlyphCacheKey::new('A', 13.25).expect("valid size"));
        assert_ne!(base, GlyphCacheKey::new('B', 13.0).expect("valid size"));
    }

    #[test]
    fn cache_reuses_hits_and_preserves_missing_glyphs() {
        let mut cache = GlyphCache::default();
        let present_key = GlyphCacheKey::new('A', 13.0).expect("valid size");
        let missing_key = GlyphCacheKey::new('\u{1f600}', 13.0).expect("valid size");
        let glyph = sample_glyph();

        cache.insert(present_key, Some(glyph.clone()));
        cache.insert(missing_key, None);

        let Some(Some(cached)) = cache.lookup(present_key) else {
            panic!("present glyph was not cached");
        };
        assert_eq!(cached.data, glyph.data);
        assert_eq!(cached.advance, glyph.advance);
        assert!(matches!(cache.lookup(missing_key), Some(None)));
        assert!(cache
            .lookup(GlyphCacheKey::new('A', 13.25).expect("valid size"))
            .is_none());
    }

    #[test]
    fn rasterize_char_populates_each_exact_scale_entry() {
        let first_key = GlyphCacheKey::new('Q', 13.0).expect("valid size");
        let second_key = GlyphCacheKey::new('Q', 13.25).expect("valid size");

        let _ = rasterize_char('Q', 13.0);
        assert!(raster_cache().lock().lookup(first_key).is_some());

        let _ = rasterize_char('Q', 13.25);
        assert!(raster_cache().lock().lookup(second_key).is_some());
    }

    #[test]
    fn cache_capacity_is_bounded() {
        let mut cache = GlyphCache::default();
        let glyph = sample_glyph();

        for index in 0..=GLYPH_CACHE_CAPACITY {
            let ch = char::from_u32(0x1000 + index as u32).expect("test character");
            let key = GlyphCacheKey::new(ch, 13.0).expect("valid size");
            cache.insert(key, Some(glyph.clone()));
        }

        assert!(cache.len() <= GLYPH_CACHE_CAPACITY);
    }

    #[test]
    fn invalid_font_sizes_are_rejected_before_font_lookup() {
        for size in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(GlyphCacheKey::new('A', size).is_none());
            assert!(rasterize_char('A', size).is_none());
        }
    }
}
