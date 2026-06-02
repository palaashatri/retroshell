use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Button, LayoutConstraint, Rect,
    Size, Widget, WidgetState,
};

pub struct Dialog {
    state: WidgetState,
    pub title: String,
    pub message: String,
    pub buttons: Vec<Button>,
}

impl Dialog {
    pub fn new<S: Into<String>>(title: S, message: S) -> Self {
        Self {
            state: WidgetState::new(),
            title: title.into(),
            message: message.into(),
            buttons: vec![],
        }
    }

    pub fn add_button(&mut self, label: &str) {
        self.buttons.push(Button::new(label));
    }
}

impl Widget for Dialog {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = 400.0;
        let height = 150.0;
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
        Some(AccessibilityNode::new(
            AccessibilityRole::Dialog,
            &self.title,
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
