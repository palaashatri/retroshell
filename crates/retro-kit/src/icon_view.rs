use crate::{
    theme::ThemeContext, AccessibilityNode, Event, EventResult, LayoutConstraint, Rect, Size,
    Widget, WidgetState,
};

#[derive(Debug, Clone)]
pub struct IconItem {
    pub label: String,
    pub icon: Option<String>,
    pub selected: bool,
    pub rect: Rect,
}

pub struct IconView {
    state: WidgetState,
    pub items: Vec<IconItem>,
    pub icon_size: f32,
    pub spacing: f32,
    pub on_double_click: Option<Box<dyn FnMut(usize) + Send>>,
}

impl Default for IconView {
    fn default() -> Self {
        Self::new()
    }
}

impl IconView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: vec![],
            icon_size: 64.0,
            spacing: 8.0,
            on_double_click: None,
        }
    }
}

impl Widget for IconView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let width = constraint.max_width.min(400.0);
        let height = constraint.max_height.min(300.0);
        let size = constraint.clamp(Size::new(width, height));
        let r = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(r);

        let cols = (size.width / (self.icon_size + self.spacing)).max(1.0) as usize;
        let icon_size = self.icon_size;
        let spacing = self.spacing;
        for (i, item) in self.items.iter_mut().enumerate() {
            let col = i % cols;
            let row = i / cols;
            item.rect = Rect::new(
                r.x + col as f32 * (icon_size + spacing),
                r.y + row as f32 * (icon_size + spacing),
                icon_size,
                icon_size + 20.0,
            );
        }
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::DoubleClick { point, .. } => {
                for (i, item) in self.items.iter().enumerate() {
                    if item.rect.contains(*point) {
                        if let Some(cb) = &mut self.on_double_click {
                            (cb)(i);
                        }
                        return EventResult::Handled;
                    }
                }
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
