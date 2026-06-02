use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Event, EventResult,
    LayoutConstraint, Rect, Size, Widget, WidgetState,
};

pub struct ListView {
    state: WidgetState,
    pub items: Vec<String>,
    pub selected_index: Option<usize>,
    pub multi_select: bool,
    pub on_select: Option<Box<dyn FnMut(Option<usize>) + Send>>,
}

impl Default for ListView {
    fn default() -> Self {
        Self::new()
    }
}

impl ListView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: vec![],
            selected_index: None,
            multi_select: false,
            on_select: None,
        }
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn add_item<S: Into<String>>(&mut self, item: S) {
        self.items.push(item.into());
    }
}

impl Widget for ListView {
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

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        let mut node = AccessibilityNode::new(AccessibilityRole::List, "list");
        for item in &self.items {
            node.children
                .push(AccessibilityNode::new(AccessibilityRole::ListItem, item));
        }
        Some(node)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
