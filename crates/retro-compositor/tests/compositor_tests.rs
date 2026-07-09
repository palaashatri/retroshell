use retro_compositor::{
    cascade_position, move_to_top, next_cascade_offset, topmost_window_at, OutputConfig,
    WindowGeometry, CASCADE_WRAP, DEFAULT_OUTPUT_H, DEFAULT_OUTPUT_W, DEFAULT_WINDOW_H,
    DEFAULT_WINDOW_W,
};

#[test]
fn window_contains_point_inside() {
    let win = WindowGeometry::new(10, 20, 100, 80);

    assert!(win.contains_f64(60.0, 60.0));
}

#[test]
fn window_contains_top_left_corner() {
    let win = WindowGeometry::new(10, 20, 100, 80);

    assert!(win.contains_f64(10.0, 20.0));
}

#[test]
fn window_rejects_bottom_right_exclusive_boundary() {
    let win = WindowGeometry::new(10, 20, 100, 80);

    assert!(!win.contains_f64(110.0, 100.0));
}

#[test]
fn zero_sized_window_contains_nothing() {
    let win = WindowGeometry::new(5, 5, 0, 0);

    assert!(!win.contains_f64(5.0, 5.0));
    assert!(!win.contains_f64(4.0, 4.0));
    assert!(!win.contains_f64(6.0, 6.0));
}

#[test]
fn topmost_window_at_prefers_later_z_order() {
    let windows = vec![
        WindowGeometry::new(0, 0, 200, 200),
        WindowGeometry::new(50, 50, 200, 200),
        WindowGeometry::new(100, 100, 200, 200),
    ];

    assert_eq!(topmost_window_at(&windows, 125.0, 125.0), Some(2));
    assert_eq!(topmost_window_at(&windows, 75.0, 75.0), Some(1));
    assert_eq!(topmost_window_at(&windows, 25.0, 25.0), Some(0));
    assert_eq!(topmost_window_at(&windows, 500.0, 500.0), None);
}

#[test]
fn move_to_top_moves_target_to_end_and_preserves_other_order() {
    let mut windows = vec![
        WindowGeometry::new(1, 0, 50, 50),
        WindowGeometry::new(2, 0, 50, 50),
        WindowGeometry::new(3, 0, 50, 50),
    ];

    move_to_top(&mut windows, 0);

    assert_eq!(windows.iter().map(|win| win.x).collect::<Vec<_>>(), vec![2, 3, 1]);
}

#[test]
fn cascade_position_uses_classic_offset_and_wraps() {
    assert_eq!(cascade_position(0), (64, 64));
    assert_eq!(cascade_position(32), (96, 96));
    assert_eq!(next_cascade_offset(CASCADE_WRAP - 32), 0);
}

#[test]
fn output_config_uses_defaults_for_missing_or_invalid_values() {
    assert_eq!(OutputConfig::from_env_values(None, None), OutputConfig::default());
    assert_eq!(
        OutputConfig::from_env_values(Some("wide".into()), Some("-1".into())),
        OutputConfig::default()
    );
}

#[test]
fn output_config_accepts_positive_env_values() {
    assert_eq!(
        OutputConfig::from_env_values(Some("1280".into()), Some("800".into())),
        OutputConfig {
            width: 1280,
            height: 800,
        }
    );
}

#[test]
fn defaults_match_runtime_contract() {
    assert_eq!((DEFAULT_OUTPUT_W, DEFAULT_OUTPUT_H), (1024, 768));
    assert_eq!((DEFAULT_WINDOW_W, DEFAULT_WINDOW_H), (640, 480));
}
