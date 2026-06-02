use crate::{
    Rect, Size, Visibility, CursorStyle, Event, EventResult,
    LayoutConstraint, AccessibilityNode,
    theme::ThemeContext,
};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    pub fn new() -> Self {
        Self(NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WidgetId({})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct WidgetState {
    pub id: WidgetId,
    pub rect: Rect,
    pub visibility: Visibility,
    pub enabled: bool,
    pub focused: bool,
    pub hovered: bool,
    pub cursor: CursorStyle,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            rect: Rect::ZERO,
            visibility: Visibility::Visible,
            enabled: true,
            focused: false,
            hovered: false,
            cursor: CursorStyle::Default,
        }
    }
}

pub trait Widget: Send {
    fn widget_state(&self) -> &WidgetState;
    fn widget_state_mut(&mut self) -> &mut WidgetState;

    fn id(&self) -> WidgetId { self.widget_state().id }

    fn rect(&self) -> Rect { self.widget_state().rect }
    fn set_rect(&mut self, rect: Rect) { self.widget_state_mut().rect = rect; }

    fn visibility(&self) -> Visibility { self.widget_state().visibility }
    fn set_visibility(&mut self, v: Visibility) { self.widget_state_mut().visibility = v; }

    fn enabled(&self) -> bool { self.widget_state().enabled }
    fn set_enabled(&mut self, enabled: bool) { self.widget_state_mut().enabled = enabled; }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size;
    fn draw(&self, theme: &ThemeContext);
    fn handle_event(&mut self, _event: &Event) -> EventResult { EventResult::Ignored }
    fn accessibility(&self) -> Option<AccessibilityNode> { None }
    fn children(&self) -> Vec<&dyn Widget> { vec![] }
    fn children_mut(&mut self) -> Vec<&mut dyn Widget> { vec![] }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
