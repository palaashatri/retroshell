use crate::{
    theme::ThemeContext, Event, EventResult, LayoutConstraint, Rect, Size, Widget, WidgetState,
};

pub struct ScrollView {
    state: WidgetState,
    pub content: Option<Box<dyn Widget>>,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scrollable_x: bool,
    pub scrollable_y: bool,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            content: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scrollable_x: false,
            scrollable_y: true,
        }
    }

    pub fn set_content(&mut self, widget: Box<dyn Widget>) {
        self.content = Some(widget);
    }
}

impl Widget for ScrollView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        let rect = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(rect);
        if let Some(content) = &mut self.content {
            let content_constraint =
                LayoutConstraint::loose(Size::new(size.width * 2.0, size.height * 2.0));
            content.layout(content_constraint);
            let content_rect = content.rect();
            content.set_rect(Rect::new(
                rect.x - self.scroll_x,
                rect.y - self.scroll_y,
                content_rect.width,
                content_rect.height,
            ));
        }
        size
    }

    fn draw(&self, theme: &ThemeContext) {
        if let Some(content) = &self.content {
            content.draw(theme);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::Scroll { delta, .. } => {
                if self.scrollable_y {
                    self.scroll_y = (self.scroll_y - delta.y).max(0.0);
                }
                if self.scrollable_x {
                    self.scroll_x = (self.scroll_x - delta.x).max(0.0);
                }
                EventResult::Handled
            }
            _ => {
                if let Some(content) = &mut self.content {
                    content.handle_event(event)
                } else {
                    EventResult::Ignored
                }
            }
        }
    }

    fn children(&self) -> Vec<&dyn Widget> {
        match &self.content {
            Some(c) => vec![c.as_ref()],
            None => vec![],
        }
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        match &mut self.content {
            Some(c) => vec![c.as_mut()],
            None => vec![],
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
