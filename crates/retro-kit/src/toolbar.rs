use crate::{theme::ThemeContext, LayoutConstraint, Rect, Size, Widget, WidgetState};

pub struct Toolbar {
    state: WidgetState,
    pub items: Vec<Box<dyn Widget>>,
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            items: vec![],
        }
    }

    pub fn add(&mut self, widget: Box<dyn Widget>) {
        self.items.push(widget);
    }
}

impl Widget for Toolbar {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let height = 32.0;
        let mut width = 0.0;
        for child in &mut self.items {
            let size = child.layout(LayoutConstraint::loose(Size::new(100.0, height)));
            width += size.width;
        }
        let size = constraint.clamp(Size::new(width.max(constraint.min_width), height));
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn children(&self) -> Vec<&dyn Widget> {
        let mut result: Vec<&dyn Widget> = vec![];
        for w in self.items.iter() {
            result.push(w.as_ref());
        }
        result
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        let mut result: Vec<&mut dyn Widget> = vec![];
        for w in self.items.iter_mut() {
            result.push(w.as_mut());
        }
        result
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
