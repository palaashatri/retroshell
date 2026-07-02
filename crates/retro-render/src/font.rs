use ab_glyph::{FontArc, Font as AbFont, PxScale};
use cosmic_text::{Family, FontSystem};
use fontdb::{Query, Stretch, Style, Weight};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::OnceLock;

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

static AB_FONT: OnceLock<FontArc> = OnceLock::new();

fn load_ab_font() -> FontArc {
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
                return font;
            }
        }
    }
    FontArc::try_from_slice(include_bytes!(
        "../../../assets/DejaVuSans.ttf"
    ))
    .expect("Failed to load embedded DejaVuSans.ttf font for ab_glyph rasterization")
}

pub fn rasterize_char(ch: char, font_size: f32) -> Option<(Vec<u8>, u32, u32)> {
    let font = AB_FONT.get_or_init(load_ab_font);
    let glyph_id = AbFont::glyph_id(font, ch);
    if glyph_id.0 == 0 && !ch.is_control() {
        return None;
    }
    let px_scale = PxScale::from(font_size * 1.4);
    let glyph = glyph_id.with_scale(px_scale);
    let outlined = AbFont::outline_glyph(font, glyph)?;
    let bounds = outlined.px_bounds();
    let width = bounds.width().ceil() as u32;
    let height = bounds.height().ceil() as u32;
    if width == 0 || height == 0 {
        return None;
    }
    let mut data = vec![0u8; (width * height) as usize];
    outlined.draw(|x, y, coverage| {
        let ix = x as usize;
        let iy = y as usize;
        if ix < width as usize && iy < height as usize {
            data[iy * width as usize + ix] = (coverage * 255.0) as u8;
        }
    });
    Some((data, width, height))
}
