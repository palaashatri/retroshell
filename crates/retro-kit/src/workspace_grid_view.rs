use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, LayoutConstraint, Rect, Size,
    Widget, WidgetState,
};

pub struct WorkspaceGridView {
    state: WidgetState,
    pub active_index: usize,
    pub items: Vec<String>,
}

impl Default for WorkspaceGridView {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceGridView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            active_index: 0,
            items: Vec::new(),
        }
    }
}

impl Widget for WorkspaceGridView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(240.0, 160.0));
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
            AccessibilityRole::Group,
            "Workspace Grid",
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
