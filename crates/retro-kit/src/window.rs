use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Event, EventResult, Layout,
    LayoutConstraint, Rect, Size, Widget, WidgetState,
};

pub struct Window {
    state: WidgetState,
    pub title: String,
    pub content: Option<Box<dyn Widget>>,
    pub layout: Layout,
    pub is_dark: bool,
    pub has_toolbar: bool,
}

impl Window {
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self {
            state: WidgetState::new(),
            title: title.into(),
            content: None,
            layout: Layout::vertical(0.0),
            is_dark: false,
            has_toolbar: false,
        }
    }

    pub fn set_content(&mut self, widget: Box<dyn Widget>) {
        self.content = Some(widget);
    }

    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn set_title<S: Into<String>>(&mut self, title: S) {
        self.title = title.into();
    }
}

impl Widget for Window {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        if self.content.is_some() {
            let proposed = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
            self.set_rect(Rect::new(
                self.rect().x,
                self.rect().y,
                proposed.width,
                proposed.height,
            ));
            let rect = self.rect();
            if let Some(content) = &mut self.content {
                content.set_rect(rect);
                content.layout(LayoutConstraint::tight(proposed))
            } else {
                proposed
            }
        } else {
            let size = self.layout.layout_size(constraint);
            self.set_rect(Rect::new(
                self.rect().x,
                self.rect().y,
                size.width,
                size.height,
            ));
            self.layout.arrange(self.rect());
            size
        }
    }

    fn draw(&self, theme: &ThemeContext) {
        let _bg = theme.color(crate::ThemeToken::WindowBackground);
        let _border = theme.color(crate::ThemeToken::WindowBorder);
        if let Some(content) = &self.content {
            content.draw(theme);
        } else {
            self.layout.draw(theme);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Some(content) = &mut self.content {
            content.handle_event(event)
        } else {
            self.layout.handle_event(event)
        }
    }

    fn update(&mut self) {
        if let Some(content) = &mut self.content {
            content.update();
        } else {
            self.layout.update();
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(
            AccessibilityRole::Window,
            &self.title,
        ))
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
