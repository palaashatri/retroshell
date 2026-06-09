use retro_kit::*;

#[test]
fn test_widget_creation() {
    let button = Button::new("Click me");
    assert_eq!(button.label, "Click me");
    assert!(button.enabled());
}

#[test]
fn test_status_bar() {
    let mut bar = StatusBar::new();
    bar.add_item("Running", StatusBarAlignment::Left, 100.0);
    assert_eq!(bar.items.len(), 1);
}

#[test]
fn test_tab_view() {
    let mut tab_view = TabView::new();
    let btn = Button::new("Inner");
    tab_view.add_tab("tab-1", "Tab 1", Box::new(btn));
    assert_eq!(tab_view.tabs.len(), 1);
    assert!(tab_view.selected_content().is_some());
}

#[test]
fn test_popup_button() {
    let mut pop = PopupButton::new();
    pop.add_item("Option 1");
    pop.add_item("Option 2");
    assert_eq!(pop.items.len(), 2);
    assert_eq!(pop.selected_index, 0);
    assert!(pop.select_item(1));
    assert_eq!(pop.selected_index, 1);
}

#[test]
fn test_clipboard() {
    Clipboard::copy("Hello Clipboard");
    assert_eq!(Clipboard::paste(), "Hello Clipboard");
    Clipboard::clear();
    assert_eq!(Clipboard::paste(), "");
}

#[test]
fn test_dnd() {
    let session = DragSession {
        payload: DragData::Text("DnD text".to_string()),
        current_position: Point::new(10.0, 20.0),
    };
    match session.payload {
        DragData::Text(t) => assert_eq!(t, "DnD text"),
        _ => panic!("Expected text payload"),
    }
}

#[test]
fn test_text_field_set_text_places_cursor_at_end() {
    let mut field = TextField::new();
    field.set_text("abc");
    let result = field.handle_event(&Event::Char { character: 'd' });
    assert!(matches!(result, EventResult::Handled));
    assert_eq!(field.text(), "abcd");

    let result = field.handle_event(&Event::KeyDown {
        key: retro_kit::event::KeyCode::Backspace,
        modifiers: retro_kit::event::Modifiers::NONE,
    });
    assert!(matches!(result, EventResult::Handled));
    assert_eq!(field.text(), "abc");
}

struct FixedWidget {
    state: WidgetState,
    size: Size,
    handled_events: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl FixedWidget {
    fn new(width: f32, height: f32) -> Self {
        Self {
            state: WidgetState::new(),
            size: Size::new(width, height),
            handled_events: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    fn with_counter(
        width: f32,
        height: f32,
        handled_events: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> Self {
        Self {
            state: WidgetState::new(),
            size: Size::new(width, height),
            handled_events,
        }
    }
}

impl Widget for FixedWidget {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(self.size);
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        self.handled_events
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        EventResult::Handled
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[test]
fn test_layout_padding_is_applied_once() {
    let mut horizontal = Layout::horizontal(5.0).padding(10.0);
    horizontal.add(Box::new(FixedWidget::new(50.0, 20.0)));
    horizontal.add(Box::new(FixedWidget::new(50.0, 20.0)));
    let size = horizontal.layout_size(LayoutConstraint::UNBOUNDED);
    assert_eq!(size.width, 125.0);
    assert_eq!(size.height, 40.0);

    let mut vertical = Layout::vertical(5.0).padding(10.0);
    vertical.add(Box::new(FixedWidget::new(50.0, 20.0)));
    vertical.add(Box::new(FixedWidget::new(50.0, 20.0)));
    let size = vertical.layout_size(LayoutConstraint::UNBOUNDED);
    assert_eq!(size.width, 70.0);
    assert_eq!(size.height, 65.0);
}

#[test]
fn test_grid_edge_cases_do_not_panic_or_underflow() {
    let mut zero_column_grid = Layout::grid(0, 4.0).padding(3.0);
    zero_column_grid.add(Box::new(FixedWidget::new(50.0, 20.0)));
    let size = zero_column_grid.layout_size(LayoutConstraint::UNBOUNDED);
    assert_eq!(size.width, 6.0);
    assert_eq!(size.height, 6.0);
    zero_column_grid.arrange(Rect::new(0.0, 0.0, 100.0, 100.0));

    let mut empty_grid = Layout::grid(3, 4.0).padding(3.0);
    let size = empty_grid.layout_size(LayoutConstraint::UNBOUNDED);
    assert_eq!(size.width, 6.0);
    assert_eq!(size.height, 6.0);
}

#[test]
fn test_scroll_view_forwards_child_events() {
    let handled_events = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut scroll = ScrollView::new();
    scroll.set_content(Box::new(FixedWidget::with_counter(
        50.0,
        20.0,
        handled_events.clone(),
    )));

    let result = scroll.handle_event(&Event::MouseEnter);
    assert!(matches!(result, EventResult::Handled));
    assert_eq!(handled_events.load(std::sync::atomic::Ordering::SeqCst), 1);
}
