use crate::theme::ThemeContext;
use crate::{
    AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, Rect, Size, Widget,
    WidgetState,
};
use std::any::Any;

pub struct Tab {
    pub id: String,
    pub title: String,
    pub content: Option<Box<dyn Widget>>,
}

pub struct TabView {
    state: WidgetState,
    pub tabs: Vec<Tab>,
    pub selected_tab_index: usize,
}

impl Default for TabView {
    fn default() -> Self {
        Self::new()
    }
}

impl TabView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            tabs: vec![],
            selected_tab_index: 0,
        }
    }

    pub fn add_tab(&mut self, id: &str, title: &str, content: Box<dyn Widget>) {
        self.tabs.push(Tab {
            id: id.to_string(),
            title: title.to_string(),
            content: Some(content),
        });
    }

    pub fn remove_tab(&mut self, id: &str) -> bool {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(pos);
            if self.selected_tab_index >= self.tabs.len() && !self.tabs.is_empty() {
                self.selected_tab_index = self.tabs.len() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn select_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.selected_tab_index = index;
            true
        } else {
            false
        }
    }

    pub fn selected_content(&self) -> Option<&dyn Widget> {
        if let Some(t) = self.tabs.get(self.selected_tab_index) {
            if let Some(ref c) = t.content {
                return Some(c.as_ref());
            }
        }
        None
    }

    pub fn selected_content_mut(&mut self) -> Option<&mut dyn Widget> {
        if let Some(t) = self.tabs.get_mut(self.selected_tab_index) {
            if let Some(ref mut c) = t.content {
                return Some(c.as_mut());
            }
        }
        None
    }
}

impl Widget for TabView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let header_height = 30.0;
        let mut child_size = Size::ZERO;

        if let Some(tab) = self.tabs.get_mut(self.selected_tab_index) {
            if let Some(content) = &mut tab.content {
                let child_constraint = LayoutConstraint {
                    min_width: constraint.min_width,
                    max_width: constraint.max_width,
                    min_height: (constraint.min_height - header_height).max(0.0),
                    max_height: (constraint.max_height - header_height).max(0.0),
                };
                child_size = content.layout(child_constraint);
            }
        }

        let size = constraint.clamp(Size::new(
            child_size.width.max(constraint.min_width),
            child_size.height + header_height,
        ));
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
        if let Some(tab) = self.tabs.get_mut(self.selected_tab_index) {
            if let Some(content) = &mut tab.content {
                return content.handle_event(event);
            }
        }
        EventResult::Ignored
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(
            AccessibilityRole::TabGroup,
            "Tab View",
        ))
    }

    fn children(&self) -> Vec<&dyn Widget> {
        let mut result = vec![];
        for tab in &self.tabs {
            if let Some(ref c) = tab.content {
                result.push(c.as_ref());
            }
        }
        result
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        let mut result = vec![];
        for tab in &mut self.tabs {
            if let Some(ref mut c) = tab.content {
                let r: &mut dyn Widget = c.as_mut();
                let r_extended: &mut dyn Widget = unsafe { &mut *(r as *mut dyn Widget) };
                result.push(r_extended);
            }
        }
        result
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
