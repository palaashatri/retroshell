use crate::{
    Widget, WidgetState, Rect, Size, Event, EventResult,
    Layout, LayoutConstraint, AccessibilityNode, AccessibilityRole,
    theme::ThemeContext,
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

    pub fn title(&self) -> &str { &self.title }
    pub fn set_title<S: Into<String>>(&mut self, title: S) { self.title = title.into(); }
}

impl Widget for Window {
    fn widget_state(&self) -> &WidgetState { &self.state }
    fn widget_state_mut(&mut self) -> &mut WidgetState { &mut self.state }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = self.layout.layout_size(constraint);
        self.set_rect(Rect::new(self.rect().x, self.rect().y, size.width, size.height));
        self.layout.arrange(self.rect());
        size
    }

    fn draw(&self, theme: &ThemeContext) {
        let _bg = theme.color(crate::ThemeToken::WindowBackground);
        let _border = theme.color(crate::ThemeToken::WindowBorder);
    }

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::Window, &self.title))
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

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
