use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, LayoutConstraint, Rect, Size,
    Widget, WidgetState,
};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct DockViewItem {
    pub label: String,
    pub icon: String,
    pub is_focused: bool,
    pub is_running: bool,
}

pub struct DockView {
    state: WidgetState,
    pub items: Vec<DockViewItem>,
}

impl Default for DockView {
    fn default() -> Self {
        Self::new()
    }
}

impl DockView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: vec![],
        }
    }
}

impl Widget for DockView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, 64.0));
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(
            AccessibilityRole::Toolbar,
            "dock",
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
