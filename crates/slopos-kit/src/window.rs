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
    pub is_active: bool,
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
            is_active: true,
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
                let content_rect = if self.title == "SLOPOS-I Desktop" {
                    rect
                } else {
                    Rect::new(
                        rect.x + 1.0,
                        rect.y + 25.0,
                        (rect.width - 2.0).max(0.0),
                        (rect.height - 26.0).max(0.0),
                    )
                };
                content.set_rect(content_rect);
                content.layout(LayoutConstraint::tight(Size::new(
                    content_rect.width,
                    content_rect.height,
                )))
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
        match event {
            Event::FocusIn => {
                self.is_active = true;
                EventResult::Handled
            }
            Event::FocusOut => {
                self.is_active = false;
                EventResult::Handled
            }
            // Positional events: a window is an opaque surface. Outside its
            // rect it is not a target at all. Inside, the *content* owns
            // routing — an app root sitting in an SDK window runs its own
            // whole event pipeline (the shell's WM policy + dispatcher), so
            // the window must delegate via `handle_event`, never walk the
            // content's subtree itself. Whatever the content ignores, the
            // window swallows — a click on a window's empty area must never
            // fall through to whatever is stacked underneath (the shell's
            // old click-through-to-desktop bug).
            Event::MouseDown { point, .. }
            | Event::MouseUp { point, .. }
            | Event::MouseMove { point, .. }
            | Event::DoubleClick { point, .. } => {
                if !self.rect().contains(*point) {
                    return EventResult::Ignored;
                }
                let result = if let Some(content) = &mut self.content {
                    content.handle_event(event)
                } else {
                    self.layout.handle_event(event)
                };
                match result {
                    EventResult::Ignored => EventResult::Handled,
                    other => other,
                }
            }
            _ => {
                if let Some(content) = &mut self.content {
                    content.handle_event(event)
                } else {
                    self.layout.handle_event(event)
                }
            }
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
            None => self.layout.children().iter().map(|c| c.as_ref()).collect(),
        }
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        match &mut self.content {
            Some(c) => vec![c.as_mut()],
            None => self
                .layout
                .children_mut()
                .iter_mut()
                .map(|c| &mut **c as &mut dyn Widget)
                .collect(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
