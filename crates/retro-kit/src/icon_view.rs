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
        let width = constraint.max_width;
        let height = constraint.max_height;
        let size = constraint.clamp(Size::new(width, height));
        let r = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(r);

        let is_desktop = size.width >= 600.0
            && size.height >= 360.0
            && self.items.iter().any(|item| item.label == "Hard Disk")
            && self.items.iter().any(|item| item.label == "Trash");
        let icon_size = self.icon_size;
        let spacing = self.spacing;
        if is_desktop {
            let right_x = r.x + size.width - icon_size - 28.0;
            let app_x = r.x + 24.0;
            let mut app_index = 0usize;
            for item in &mut self.items {
                let rect = match item.label.as_str() {
                    "Hard Disk" => Rect::new(right_x, r.y + 28.0, icon_size, icon_size + 22.0),
                    "Home" => Rect::new(right_x, r.y + 118.0, icon_size, icon_size + 22.0),
                    "Applications" => Rect::new(right_x, r.y + 208.0, icon_size, icon_size + 22.0),
                    "Trash" => Rect::new(
                        right_x,
                        r.y + size.height - icon_size - 34.0,
                        icon_size,
                        icon_size + 22.0,
                    ),
                    _ => {
                        let col = app_index % 4;
                        let row = app_index / 4;
                        app_index += 1;
                        Rect::new(
                            app_x + col as f32 * (icon_size + 38.0),
                            r.y + 36.0 + row as f32 * (icon_size + 38.0),
                            icon_size,
                            icon_size + 22.0,
                        )
                    }
                };
                item.rect = rect;
            }
        } else {
            let cols = (size.width / (icon_size + spacing)).max(1.0) as usize;
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
            Event::MouseDown {
                button: crate::event::MouseButton::Left,
                point,
                ..
            } => {
                let mut hit = false;
                for item in &mut self.items {
                    if item.rect.contains(*point) {
                        item.selected = true;
                        hit = true;
                    } else {
                        item.selected = false;
                    }
                }
                if hit {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
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
