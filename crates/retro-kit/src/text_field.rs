use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Event, EventResult,
    LayoutConstraint, Rect, Size, Widget, WidgetState,
};

pub struct TextField {
    state: WidgetState,
    pub text: String,
    pub placeholder: String,
    pub is_password: bool,
    pub multiline: bool,
    pub expands_horizontally: bool,
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
            multiline: false,
            expands_horizontally: false,
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
    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
    }

    pub fn set_expands_horizontally(&mut self, expands: bool) {
        self.expands_horizontally = expands;
    }
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
        self.cursor_position = self.text.len();
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Clamps to the text length and snaps down to the nearest UTF-8 char
    /// boundary — the cursor is a byte offset and must never sit inside a
    /// multi-byte character.
    pub fn set_cursor_position(&mut self, pos: usize) {
        let mut pos = pos.min(self.text.len());
        while pos > 0 && !self.text.is_char_boundary(pos) {
            pos -= 1;
        }
        self.cursor_position = pos;
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
        let (width, height) = if self.multiline {
            (
                constraint.max_width.max(constraint.min_width),
                constraint.max_height.max(constraint.min_height),
            )
        } else if self.expands_horizontally {
            (constraint.max_width.max(constraint.min_width), 26.0)
        } else {
            (constraint.max_width.min(200.0), 26.0)
        };
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
                    // Step back one full character (may be multi-byte).
                    let prev = self.text[..self.cursor_position]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.text.remove(prev);
                    self.cursor_position = prev;
                    if let Some(cb) = &mut self.on_change {
                        (cb)(self.text.clone());
                    }
                }
                EventResult::Handled
            }
            Event::Char { character } => {
                self.text.insert(self.cursor_position, *character);
                self.cursor_position += character.len_utf8();
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
