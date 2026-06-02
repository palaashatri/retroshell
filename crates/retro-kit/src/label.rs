use crate::{
    Widget, WidgetState, Rect, Size, LayoutConstraint, AccessibilityNode,
    AccessibilityRole, theme::ThemeContext,
};

pub struct Label {
    state: WidgetState,
    pub text: String,
}

impl Label {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self { state: WidgetState::new(), text: text.into() }
    }
}

impl Widget for Label {
    fn widget_state(&self) -> &WidgetState { &self.state }
    fn widget_state_mut(&mut self) -> &mut WidgetState { &mut self.state }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = self.text.len() as f32 * 8.0;
        let height = 20.0;
        let size = constraint.clamp(Size::new(width, height));
        self.set_rect(Rect::new(self.rect().x, self.rect().y, size.width, size.height));
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::StaticText, &self.text))
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
