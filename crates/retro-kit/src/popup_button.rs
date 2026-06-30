use crate::theme::ThemeContext;
use crate::{
    AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, Rect, Size, Widget,
    WidgetState,
};
use std::any::Any;

pub struct PopupButton {
    state: WidgetState,
    pub items: Vec<String>,
    pub selected_index: usize,
    pub open: bool,
}

impl Default for PopupButton {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupButton {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: vec![],
            selected_index: 0,
            open: false,
        }
    }

    pub fn add_item(&mut self, item: &str) {
        self.items.push(item.to_string());
    }

    pub fn select_item(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.selected_index = index;
            true
        } else {
            false
        }
    }

    pub fn selected_title(&self) -> Option<&str> {
        self.items.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
}

impl Widget for PopupButton {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(150.0, 26.0));
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
        match event {
            Event::MouseDown { .. } => {
                self.toggle();
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        let title = self.selected_title().unwrap_or("Popup Button");
        Some(AccessibilityNode::new(AccessibilityRole::ComboBox, title))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
