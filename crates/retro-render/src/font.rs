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

pub fn rasterize_char(ch: char, font_size: f32) -> Option<(Vec<u8>, u32, u32, f32)> {
    let font = AB_FONT.get_or_init(load_ab_font).as_ref()?;
    let glyph_id = AbFont::glyph_id(font, ch);
    if ch.is_control() {
        return None;
    }
    if glyph_id.0 == 0 && !ch.is_control() {
        return None;
    }
    let px_scale = PxScale::from(font_size * 1.4);
    let scaled_font = font.as_scaled(px_scale);
    let advance = scaled_font.h_advance(glyph_id).max(font_size * 0.35);
    let glyph = glyph_id.with_scale(px_scale);
    let Some(outlined) = AbFont::outline_glyph(font, glyph) else {
        return Some((Vec::new(), 0, 0, advance));
    };
    let bounds = outlined.px_bounds();
    let width = bounds.width().ceil() as u32;
    let height = bounds.height().ceil() as u32;
    if width == 0 || height == 0 {
        return Some((Vec::new(), 0, 0, advance));
    }
    let mut data = vec![0u8; (width * height) as usize];
    outlined.draw(|x, y, coverage| {
        let ix = x as usize;
        let iy = y as usize;
        if ix < width as usize && iy < height as usize {
            data[iy * width as usize + ix] = (coverage * 255.0) as u8;
        }
    });
    Some((data, width, height, advance))
}
