use retro_sdk::Application;

#[test]
fn test_app_creation() {
    let app = Application::new("TestApp", "com.test.app");
    assert_eq!(app.name, "TestApp");
    assert_eq!(app.bundle_id, "com.test.app");
    assert_eq!(app.initial_size.width, 960.0);
    assert_eq!(app.initial_size.height, 640.0);
    assert!(!app.running);
}

#[test]
fn test_app_initial_size_can_be_configured() {
    let mut app = Application::new("TestApp", "com.test.app");

    app.set_initial_size(retro_kit::Size::new(1280.0, 800.0));

    assert_eq!(app.initial_size.width, 1280.0);
    assert_eq!(app.initial_size.height, 800.0);
}

#[test]
fn test_mouse_button_mapping() {
    assert_eq!(
        retro_sdk::winit_to_retro_mouse_button(winit::event::MouseButton::Left),
        Some(retro_kit::event::MouseButton::Left)
    );
    assert_eq!(
        retro_sdk::winit_to_retro_mouse_button(winit::event::MouseButton::Other(42)),
        None
    );
}

#[test]
fn test_scroll_delta_mapping() {
    let line_delta = retro_sdk::winit_to_retro_scroll_delta(
        winit::event::MouseScrollDelta::LineDelta(1.0, -2.0),
    );
    assert_eq!(line_delta.x, 16.0);
    assert_eq!(line_delta.y, -32.0);

    let pixel_delta = retro_sdk::winit_to_retro_scroll_delta(
        winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(3.5, -4.0)),
    );
    assert_eq!(pixel_delta.x, 3.5);
    assert_eq!(pixel_delta.y, -4.0);
}
