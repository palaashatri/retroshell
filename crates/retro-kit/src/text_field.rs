use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Event, EventResult,
    LayoutConstraint, Rect, Size, Widget, WidgetState,
};

pub struct TextField {
    state: WidgetState,
    pub text: String,
    pub placeholder: String,
    pub is_password: bool,
    pub on_change: Option<Box<dyn FnMut(String) + Send>>,
    cursor_position: usize,
}

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}

impl TextField {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            text: String::new(),
            placeholder: String::new(),
            is_password: false,
            on_change: None,
            cursor_position: 0,
        }
    }

    pub fn with_placeholder<S: Into<String>>(mut self, text: S) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }
}

impl Widget for TextField {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = constraint.max_width.min(200.0);
        let height = 26.0;
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
        match event {
            Event::KeyDown {
                key: crate::event::KeyCode::Backspace,
                ..
            } => {
                if self.cursor_position > 0 {
                    self.text.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    if let Some(cb) = &mut self.on_change {
                        (cb)(self.text.clone());
                    }
                }
                EventResult::Handled
            }
            Event::Char { character } => {
                self.text.insert(self.cursor_position, *character);
                self.cursor_position += 1;
                if let Some(cb) = &mut self.on_change {
                    (cb)(self.text.clone());
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(
            AccessibilityNode::new(AccessibilityRole::TextField, &self.text)
                .with_description(&self.placeholder),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
