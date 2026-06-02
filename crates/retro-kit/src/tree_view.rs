use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, LayoutConstraint, Rect, Size,
    Widget, WidgetState,
};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub icon: Option<String>,
}

impl TreeNode {
    pub fn new<S: Into<String>>(label: S) -> Self {
        Self {
            label: label.into(),
            children: vec![],
            expanded: false,
            icon: None,
        }
    }
}

pub struct TreeView {
    state: WidgetState,
    pub roots: Vec<TreeNode>,
    pub selected_path: Option<Vec<usize>>,
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            roots: vec![],
            selected_path: None,
        }
    }
}

impl Widget for TreeView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = constraint.max_width.min(200.0);
        let height = constraint.max_height.min(300.0);
        let size = constraint.clamp(Size::new(width, height));
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
        Some(AccessibilityNode::new(AccessibilityRole::Tree, "files"))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
