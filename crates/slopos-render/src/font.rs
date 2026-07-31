use ab_glyph::{Font as AbFont, FontArc, PxScale, ScaleFont};
use cosmic_text::{Family, FontSystem};
use fontdb::{Query, Stretch, Style, Weight};
use parking_lot::Mutex;
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

pub fn rasterize_char(ch: char, font_size: f32) -> Option<RasterGlyph> {
    let font = AB_FONT.get_or_init(load_ab_font).as_ref()?;
    let glyph_id = AbFont::glyph_id(font, ch);
    if ch.is_control() {
        return None;
    }
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
