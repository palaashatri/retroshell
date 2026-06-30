use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, Rect, Size,
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

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::MouseDown { button: crate::event::MouseButton::Left, point, .. } = event {
            if self.rect().contains(*point) {
                let relative_y = point.y - self.rect().y;
                let height = self.rect().height;
                let index = if relative_y < height * 0.3 {
                    vec![0, 3] // Desktop
                } else if relative_y < height * 0.6 {
                    vec![0, 4] // Documents
                } else {
                    vec![0, 5] // Downloads
                };
                self.selected_path = Some(index);
                return EventResult::Handled;
            }
        }
        EventResult::Ignored
    }

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
