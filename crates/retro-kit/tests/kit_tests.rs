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
