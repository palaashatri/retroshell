use crate::{theme::ThemeContext, LayoutConstraint, Rect, Size, Widget, WidgetState};

pub enum SplitDirection {
    Horizontal,
    Vertical,
}

pub struct SplitView {
    state: WidgetState,
    pub first: Option<Box<dyn Widget>>,
    pub second: Option<Box<dyn Widget>>,
    pub direction: SplitDirection,
    pub divider_position: f32,
    pub divider_size: f32,
}

impl SplitView {
    pub fn new(direction: SplitDirection) -> Self {
        Self {
            state: WidgetState::new(),
            first: None,
            second: None,
            direction,
            divider_position: 0.5,
            divider_size: 4.0,
        }
    }

    pub fn set_first(&mut self, widget: Box<dyn Widget>) {
        self.first = Some(widget);
    }
    pub fn set_second(&mut self, widget: Box<dyn Widget>) {
        self.second = Some(widget);
    }
}

impl Widget for SplitView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        let r = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(r);

        match self.direction {
            SplitDirection::Horizontal => {
                let first_w = r.width * self.divider_position;
                let second_w = r.width - first_w - self.divider_size;
                if let Some(child) = &mut self.first {
                    child.set_rect(Rect::new(r.x, r.y, first_w, r.height));
                }
                if let Some(child) = &mut self.second {
                    child.set_rect(Rect::new(
                        r.x + first_w + self.divider_size,
                        r.y,
                        second_w,
                        r.height,
                    ));
                }
            }
            SplitDirection::Vertical => {
                let first_h = r.height * self.divider_position;
                let second_h = r.height - first_h - self.divider_size;
                if let Some(child) = &mut self.first {
                    child.set_rect(Rect::new(r.x, r.y, r.width, first_h));
                }
                if let Some(child) = &mut self.second {
                    child.set_rect(Rect::new(
                        r.x,
                        r.y + first_h + self.divider_size,
                        r.width,
                        second_h,
                    ));
                }
            }
        }

        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn children(&self) -> Vec<&dyn Widget> {
        let mut result = vec![];
        if let Some(ref f) = self.first {
            result.push(f.as_ref());
        }
        if let Some(ref s) = self.second {
            result.push(s.as_ref());
        }
        result
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        let mut result: Vec<&mut dyn Widget> = vec![];
        if let Some(f) = &mut self.first {
            result.push(f.as_mut());
        }
        if let Some(s) = &mut self.second {
            result.push(s.as_mut());
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
