use retro_sdk::Application;

#[test]
fn test_app_creation() {
    let app = Application::new("TestApp", "com.test.app");
    assert_eq!(app.name, "TestApp");
    assert_eq!(app.bundle_id, "com.test.app");
    assert!(!app.running);
}
