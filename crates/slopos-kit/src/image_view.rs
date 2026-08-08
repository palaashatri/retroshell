use crate::{AccessibilityNode, AccessibilityRole, LayoutConstraint, Size, Widget, WidgetState};
use std::any::Any;
use std::sync::Arc;

/// A decoded RGBA8 image owned by the widget tree.
///
/// The SDK painter uploads this source to retained GPU tile textures. Keeping
/// the bytes behind an `Arc` lets every visible tile share one immutable source
/// without copying the image on each frame.
pub struct ImageView {
    state: WidgetState,
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl ImageView {
    /// Creates an image view from tightly packed RGBA8 pixels.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("image dimensions must be non-zero".to_string());
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| "image dimensions overflow RGBA8 storage".to_string())?;
        if pixels.len() != expected {
            return Err(format!(
                "RGBA8 image has {} bytes; expected {}",
                pixels.len(),
                expected
            ));
        }
        Ok(Self {
            state: WidgetState::new(),
            width,
            height,
            pixels: Arc::from(pixels),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixels_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.pixels)
    }
}

impl Widget for ImageView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(self.width as f32, self.height as f32));
        let rect = self.rect();
        self.set_rect(crate::Rect::new(rect.x, rect.y, size.width, size.height));
        size
    }

    fn draw(&self, _theme: &crate::ThemeContext) {}

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::Image, "Image"))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::ImageView;

    #[test]
    fn rejects_invalid_rgba8_storage() {
        assert!(ImageView::new(2, 2, vec![0; 15]).is_err());
        assert!(ImageView::new(0, 2, vec![]).is_err());
    }

    #[test]
    fn retains_exact_source_dimensions_and_bytes() {
        let pixels = vec![7; 3 * 2 * 4];
        let view = ImageView::new(3, 2, pixels.clone()).unwrap();
        assert_eq!(view.width(), 3);
        assert_eq!(view.height(), 2);
        assert_eq!(view.pixels(), pixels.as_slice());
    }
}
