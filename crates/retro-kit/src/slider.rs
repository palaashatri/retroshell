use crate::{
    Widget, WidgetState, Rect, Size,
    LayoutConstraint, AccessibilityNode, AccessibilityRole,
    theme::ThemeContext,
};

pub struct Slider {
    state: WidgetState,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Slider {
    pub fn new() -> Self {
        Self { state: WidgetState::new(), value: 0.5, min: 0.0, max: 1.0, step: 0.01 }
    }
}

impl Widget for Slider {
    fn widget_state(&self) -> &WidgetState { &self.state }
    fn widget_state_mut(&mut self) -> &mut WidgetState { &mut self.state }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = constraint.max_width.min(150.0);
        let height = 24.0;
        let size = constraint.clamp(Size::new(width, height));
        self.set_rect(Rect::new(self.rect().x, self.rect().y, size.width, size.height));
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::Slider, "slider"))
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
