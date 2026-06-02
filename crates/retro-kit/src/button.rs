use crate::{
    Widget, WidgetState, Rect, Size, Event, EventResult,
    LayoutConstraint, AccessibilityNode, AccessibilityRole,
    theme::ThemeContext, event::MouseButton,
};

pub struct Button {
    state: WidgetState,
    pub label: String,
    pub checked: bool,
}

impl Button {
    pub fn new<S: Into<String>>(label: S) -> Self {
        Self {
            state: WidgetState::new(),
            label: label.into(),
            checked: false,
        }
    }

    pub fn label(&self) -> &str { &self.label }
    pub fn set_label<S: Into<String>>(&mut self, label: S) { self.label = label.into(); }
}

impl Widget for Button {
    fn widget_state(&self) -> &WidgetState { &self.state }
    fn widget_state_mut(&mut self) -> &mut WidgetState { &mut self.state }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = self.label.len() as f32 * 10.0 + 24.0;
        let height = 28.0;
        let size = constraint.clamp(Size::new(width, height));
        self.set_rect(Rect::new(self.rect().x, self.rect().y, size.width, size.height));
        size
    }

    fn draw(&self, theme: &ThemeContext) {
        let _bg = if self.state.hovered {
            theme.color(crate::ThemeToken::ButtonHighlight)
        } else {
            theme.color(crate::ThemeToken::ButtonBackground)
        };
        let _text_color = theme.color(crate::ThemeToken::ButtonText);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseDown { button: MouseButton::Left, .. } => {
                self.state.hovered = true;
                EventResult::Handled
            }
            Event::MouseEnter => { self.state.hovered = true; EventResult::Handled }
            Event::MouseLeave => { self.state.hovered = false; EventResult::Handled }
            _ => EventResult::Ignored,
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::Button, &self.label))
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
