//! Integration tests for retro-compositor geometry logic.
//!
//! `MappedWindow` and `RetroCompositor` are not exported — they live inside
//! the `linux` cfg-gated module and require a live Wayland display.
//! We test the *pure geometry logic* (the `contains` predicate and window
//! ordering semantics) by replicating the minimal struct locally.
//!
//! This file compiles and runs on all platforms, including macOS.

/// Minimal replica of the compositor-space window rectangle used by
/// `MappedWindow::contains` in `src/main.rs`.
#[derive(Clone, Debug)]
struct WindowRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl WindowRect {
    /// Equivalent to `MappedWindow::contains` in src/main.rs.
    fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

// ---------------------------------------------------------------------------
// contains() tests
// ---------------------------------------------------------------------------

#[test]
fn contains_point_inside_window() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    // Centre of the window
    assert!(win.contains(60, 60));
}

#[test]
fn contains_point_at_top_left_corner_is_inside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    assert!(win.contains(10, 20));
}

#[test]
fn contains_point_at_bottom_right_exclusive_boundary_is_outside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    // x = 10 + 100 = 110, y = 20 + 80 = 100 — both exclusive upper bounds
    assert!(!win.contains(110, 100));
}

#[test]
fn contains_point_one_before_bottom_right_exclusive_boundary_is_inside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    assert!(win.contains(109, 99));
}

#[test]
fn contains_point_left_of_window_is_outside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    assert!(!win.contains(9, 60));
}

#[test]
fn contains_point_above_window_is_outside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    assert!(!win.contains(60, 19));
}

#[test]
fn contains_point_right_of_window_is_outside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    assert!(!win.contains(111, 60));
}

#[test]
fn contains_point_below_window_is_outside() {
    let win = WindowRect { x: 10, y: 20, w: 100, h: 80 };
    assert!(!win.contains(60, 101));
}

#[test]
fn zero_size_window_contains_no_points() {
    let win = WindowRect { x: 5, y: 5, w: 0, h: 0 };
    assert!(!win.contains(5, 5));
    assert!(!win.contains(4, 4));
    assert!(!win.contains(6, 6));
}

// ---------------------------------------------------------------------------
// Window z-order / focus tests (pure Vec manipulation, mirrors focus_window)
// ---------------------------------------------------------------------------

/// Equivalent to `RetroCompositor::focus_window`: removes the window at `idx`
/// and appends it to the back of the list (top of z-order).
fn focus_window(windows: &mut Vec<WindowRect>, idx: usize) {
    let win = windows.remove(idx);
    windows.push(win);
}

#[test]
fn focus_window_moves_target_to_end() {
    let mut windows = vec![
        WindowRect { x: 0, y: 0, w: 100, h: 100 },  // idx 0
        WindowRect { x: 10, y: 10, w: 80, h: 60 },  // idx 1
        WindowRect { x: 20, y: 20, w: 60, h: 40 },  // idx 2
    ];
    focus_window(&mut windows, 0);
    // Window originally at idx 0 (x=0) should now be last
    assert_eq!(windows.last().unwrap().x, 0);
    assert_eq!(windows.len(), 3);
}

#[test]
fn focus_window_on_last_element_is_identity() {
    let mut windows = vec![
        WindowRect { x: 0, y: 0, w: 50, h: 50 },
        WindowRect { x: 99, y: 99, w: 50, h: 50 },
    ];
    let last_x_before = windows.last().unwrap().x;
    let last_idx = windows.len() - 1;
    focus_window(&mut windows, last_idx);
    assert_eq!(windows.last().unwrap().x, last_x_before);
    assert_eq!(windows.len(), 2);
}

#[test]
fn focus_window_preserves_other_order() {
    let mut windows = vec![
        WindowRect { x: 1, y: 0, w: 50, h: 50 },  // idx 0 — will be focused
        WindowRect { x: 2, y: 0, w: 50, h: 50 },  // idx 1
        WindowRect { x: 3, y: 0, w: 50, h: 50 },  // idx 2
    ];
    focus_window(&mut windows, 0);
    // After focusing idx 0, order should be [2, 3, 1] by x
    assert_eq!(windows[0].x, 2);
    assert_eq!(windows[1].x, 3);
    assert_eq!(windows[2].x, 1);
}

// ---------------------------------------------------------------------------
// prune_dead_windows logic (pure Vec retain, no live surfaces needed)
// ---------------------------------------------------------------------------

#[test]
fn prune_retains_alive_windows_and_removes_dead_ones() {
    // Simulate alive/dead tracking with a bool flag — mirrors the
    // `w.toplevel.alive()` predicate in RetroCompositor::prune_dead_windows.
    struct TrackableWindow {
        rect: WindowRect,
        alive: bool,
    }

    let mut windows: Vec<TrackableWindow> = vec![
        TrackableWindow { rect: WindowRect { x: 0, y: 0, w: 10, h: 10 }, alive: true },
        TrackableWindow { rect: WindowRect { x: 1, y: 0, w: 10, h: 10 }, alive: false },
        TrackableWindow { rect: WindowRect { x: 2, y: 0, w: 10, h: 10 }, alive: true },
        TrackableWindow { rect: WindowRect { x: 3, y: 0, w: 10, h: 10 }, alive: false },
    ];

    windows.retain(|w| w.alive);

    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].rect.x, 0);
    assert_eq!(windows[1].rect.x, 2);
}
