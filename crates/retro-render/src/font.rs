use cosmic_text::{FontSystem, SwashCache};
use std::sync::Arc;
use parking_lot::Mutex;

pub struct Font {
    pub(crate) font_system: Arc<Mutex<FontSystem>>,
    pub(crate) swash_cache: Arc<Mutex<SwashCache>>,
}

impl Font {
    pub fn new() -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        Self {
            font_system: Arc::new(Mutex::new(font_system)),
            swash_cache: Arc::new(Mutex::new(swash_cache)),
        }
    }

    pub fn font_system(&self) -> Arc<Mutex<FontSystem>> {
        self.font_system.clone()
    }

    pub fn swash_cache(&self) -> Arc<Mutex<SwashCache>> {
        self.swash_cache.clone()
    }
}
