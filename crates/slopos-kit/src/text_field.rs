use crate::{
    event::{KeyCode, MouseButton},
    theme::ThemeContext,
    AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, Rect, Size, Widget,
    WidgetState,
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

    /// Places the cursor at the byte offset of the `char_index`-th character
    /// (or the end, if `text` is shorter) — bridges a click's "N characters
    /// in" position to the byte index `cursor_position` actually stores,
    /// without ever landing inside a multi-byte character.
    fn set_cursor_to_char_index(&mut self, char_index: usize) {
        let byte_pos = self
            .text
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        self.set_cursor_position(byte_pos);
    }

    /// Move the cursor back one full character (may be multi-byte) — the
    /// same char-boundary logic `Backspace` uses, just without deleting.
    fn move_cursor_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }
        let prev = self.text[..self.cursor_position]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.cursor_position = prev;
    }

    /// Move the cursor forward one full character (may be multi-byte).
    /// `cursor_position` is always on a char boundary already (the
    /// invariant `set_cursor_position` maintains), so slicing from it and
    /// reading the next `char` can never panic.
    fn move_cursor_right(&mut self) {
        if self.cursor_position >= self.text.len() {
            return;
        }
        if let Some(c) = self.text[self.cursor_position..].chars().next() {
            self.cursor_position += c.len_utf8();
        }
    }
}

impl Widget for TextField {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    // Was: no override, so every `TextField` was focusable-in-spirit but
    // never actually joined the tab order and nothing ever gated input on
    // it (see docs/TOOLKIT_REMEDIATION.md). Text input is exactly the case
    // `focusable()` exists for. Hidden or disabled fields (e.g. a closed
    // find bar) stay out of the tab order.
    fn focusable(&self) -> bool {
        self.state.enabled && self.state.visibility == crate::Visibility::Visible
    }

    fn wants_click_focus(&self) -> bool {
        self.focusable()
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

    // Was: `Char`/`Backspace` mutated `text` unconditionally, with no rect
    // check and no focus gate at all — every `TextField` in the tree
    // consumed every keystroke (see docs/TOOLKIT_REMEDIATION.md). Now:
    // `MouseDown` inside the rect click-to-focuses (and only this field —
    // nothing else on the tree loses focus here, that's `FocusManager`'s
    // job once an app wires it up), and every keyboard branch refuses to
    // act unless `focused` is already set.
    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseDown {
                button: MouseButton::Left,
                point,
                ..
            } => {
                if !self.rect().contains(*point) {
                    return EventResult::Ignored;
                }
                self.widget_state_mut().focused = true;
                // ~7px per glyph is the same rough monospace advance the SDK
                // painter uses elsewhere (see `draw_dialog`'s button sizing);
                // good enough to land the caret near the click without real
                // text shaping here.
                const CHAR_WIDTH: f32 = 7.0;
                let clicked_chars =
                    ((point.x - self.rect().x) / CHAR_WIDTH).round().max(0.0) as usize;
                self.set_cursor_to_char_index(clicked_chars);
                EventResult::Handled
            }
            Event::KeyDown {
                key: KeyCode::Backspace,
                ..
            } => {
                if !self.widget_state().focused {
                    return EventResult::Ignored;
                }
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
            Event::KeyDown {
                key: KeyCode::ArrowLeft,
                ..
            } => {
                if !self.widget_state().focused {
                    return EventResult::Ignored;
                }
                self.move_cursor_left();
                EventResult::Handled
            }
            Event::KeyDown {
                key: KeyCode::ArrowRight,
                ..
            } => {
                if !self.widget_state().focused {
                    return EventResult::Ignored;
                }
                self.move_cursor_right();
                EventResult::Handled
            }
            Event::KeyDown {
                key: KeyCode::Home, ..
            } => {
                if !self.widget_state().focused {
                    return EventResult::Ignored;
                }
                self.set_cursor_position(0);
                EventResult::Handled
            }
            Event::KeyDown {
                key: KeyCode::End, ..
            } => {
                if !self.widget_state().focused {
                    return EventResult::Ignored;
                }
                self.set_cursor_position(self.text.len());
                EventResult::Handled
            }
            Event::Char { character } => {
                if !self.widget_state().focused {
                    return EventResult::Ignored;
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{event::Modifiers, Point};

    fn key(key: KeyCode) -> Event {
        Event::KeyDown {
            key,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn focusable_is_true() {
        assert!(TextField::new().focusable());
    }

    #[test]
    fn char_and_backspace_are_ignored_when_not_focused() {
        let mut field = TextField::new();
        field.set_text("abc");
        assert!(!field.widget_state().focused);

        assert!(matches!(
            field.handle_event(&Event::Char { character: 'x' }),
            EventResult::Ignored
        ));
        assert_eq!(field.text(), "abc");

        assert!(matches!(
            field.handle_event(&key(KeyCode::Backspace)),
            EventResult::Ignored
        ));
        assert_eq!(field.text(), "abc");
    }

    #[test]
    fn arrow_keys_are_ignored_when_not_focused() {
        let mut field = TextField::new();
        field.set_text("abc");
        field.set_cursor_position(1);

        assert!(matches!(
            field.handle_event(&key(KeyCode::ArrowLeft)),
            EventResult::Ignored
        ));
        assert_eq!(field.cursor_position(), 1);
    }

    #[test]
    fn click_inside_rect_focuses_and_places_cursor_near_the_click() {
        let mut field = TextField::new();
        field.set_text("héllo");
        field.set_rect(Rect::new(100.0, 0.0, 200.0, 26.0));
        assert!(!field.widget_state().focused);

        // ~2 characters in from the left edge of the rect.
        let point = Point::new(100.0 + 2.0 * 7.0, 10.0);
        let result = field.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point,
            modifiers: Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert!(field.widget_state().focused);
        assert_eq!(
            field.cursor_position(),
            3,
            "lands after 'h' + 'é' (3 bytes), never mid-character"
        );
    }

    #[test]
    fn click_outside_rect_is_ignored_and_does_not_focus() {
        let mut field = TextField::new();
        field.set_rect(Rect::new(100.0, 0.0, 200.0, 26.0));

        let result = field.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(5.0, 5.0),
            modifiers: Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Ignored));
        assert!(!field.widget_state().focused);
    }

    #[test]
    fn click_far_past_the_text_clamps_cursor_to_the_end() {
        let mut field = TextField::new();
        field.set_text("hi");
        field.set_rect(Rect::new(0.0, 0.0, 200.0, 26.0));

        let result = field.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(190.0, 10.0),
            modifiers: Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(field.cursor_position(), field.text().len());
    }

    #[test]
    fn arrow_keys_move_by_whole_characters_not_bytes() {
        let mut field = TextField::new();
        field.widget_state_mut().focused = true;
        field.set_text("héllo"); // cursor starts at the end (byte 6)
        assert_eq!(field.cursor_position(), 6);

        assert!(matches!(
            field.handle_event(&key(KeyCode::ArrowLeft)),
            EventResult::Handled
        ));
        assert_eq!(field.cursor_position(), 5, "before the trailing 'o'");

        assert!(matches!(
            field.handle_event(&key(KeyCode::ArrowLeft)),
            EventResult::Handled
        ));
        assert_eq!(
            field.cursor_position(),
            4,
            "steps over one 'l', not into the middle of 'é'"
        );

        assert!(matches!(
            field.handle_event(&key(KeyCode::ArrowRight)),
            EventResult::Handled
        ));
        assert_eq!(field.cursor_position(), 5);
    }

    #[test]
    fn home_and_end_move_cursor_to_bounds() {
        let mut field = TextField::new();
        field.widget_state_mut().focused = true;
        field.set_text("héllo");
        field.set_cursor_position(3);

        let _ = field.handle_event(&key(KeyCode::Home));
        assert_eq!(field.cursor_position(), 0);

        let _ = field.handle_event(&key(KeyCode::End));
        assert_eq!(field.cursor_position(), field.text().len());
    }

    #[test]
    fn multibyte_insert_and_backspace_does_not_panic_and_tracks_byte_cursor() {
        let mut field = TextField::new();
        field.widget_state_mut().focused = true;

        // 'é' is 2 bytes in UTF-8: insertion must advance by full
        // characters and backspace must remove exactly one, never leaving
        // the cursor mid-codepoint.
        let _ = field.handle_event(&Event::Char { character: 'h' });
        let _ = field.handle_event(&Event::Char { character: 'é' });
        assert_eq!(field.text(), "hé");
        assert_eq!(field.cursor_position(), 3); // 1 + 2 bytes

        let _ = field.handle_event(&key(KeyCode::Backspace));
        assert_eq!(field.text(), "h");
        assert_eq!(field.cursor_position(), 1);
    }
}
