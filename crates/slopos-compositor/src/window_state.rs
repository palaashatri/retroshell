//! Single-authority window presentation state machine and zoom policy for slopos-compositor.
//!
//! Copyright (c) 2026 Palaash Atri
//! SPDX-License-Identifier: MIT

use crate::WindowGeometry;
use std::collections::HashMap;

/// Specific tile placement on the screen output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TilePlacement {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Single-authority presentation state of a mapped window.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum WindowPresentationState {
    #[default]
    Normal,
    Minimized,
    SmartZoomed,
    Filled,
    Fullscreen,
    Tiled(TilePlacement),
}

/// How a presentation transition should treat the window's compositor-owned
/// stacking position.  Geometry changes do not implicitly raise or reorder a
/// window; the backend may apply a separate focus/stacking operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WindowStackingIntent {
    Preserve,
    RestoreAt(usize),
}

/// Geometry and location recorded prior to a presentation state transition
/// (zoom, fill, fullscreen, tiling).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowRestoreState {
    pub normal_geometry: WindowGeometry,
    pub previous_state: WindowPresentationState,
    pub output_id: String,
    pub space_id: usize,
    pub stacking_intent: WindowStackingIntent,
}

/// Result of one compositor-owned presentation transition.
///
/// Backends apply this value to their Wayland surface/configure state, but do
/// not make their own geometry or restore decisions. Keeping the transition
/// pure is what lets nested and DRM use exactly the same state machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationTransition {
    pub state: WindowPresentationState,
    pub geometry: WindowGeometry,
    pub restore_state: Option<WindowRestoreState>,
    /// Restore metadata consumed when a non-normal window returns to Normal.
    /// This is deliberately separate from the active restore record, which is
    /// cleared once the normal geometry has been selected.
    pub restored_from: Option<WindowRestoreState>,
}

/// Transition one window presentation state while preserving normal geometry.
///
/// `work_area` excludes compositor-owned shell exclusive zones for Fill/Smart
/// Zoom. `output_area` is the complete output used by Fullscreen. Once a
/// window enters a non-normal presentation state, its first normal geometry
/// and Space/output identity are retained until it returns to `Normal`.
#[allow(clippy::too_many_arguments)]
pub fn transition_presentation_state(
    current_state: WindowPresentationState,
    current_geometry: WindowGeometry,
    current_restore_state: Option<&WindowRestoreState>,
    target_state: WindowPresentationState,
    work_area: WindowGeometry,
    output_area: WindowGeometry,
    preferred_size: Option<(i32, i32)>,
    output_id: impl Into<String>,
    space_id: usize,
) -> PresentationTransition {
    // A normal window has no active restore record.  Dropping a stale record
    // here prevents a repeated Normal transition from unexpectedly replacing
    // a user-resized geometry with an old presentation snapshot.
    let mut restore_state = if current_state == WindowPresentationState::Normal {
        None
    } else {
        current_restore_state.cloned()
    };

    if target_state != WindowPresentationState::Normal && restore_state.is_none() {
        restore_state = Some(WindowRestoreState::new(
            current_geometry,
            current_state,
            output_id,
            space_id,
        ));
    }

    if target_state == WindowPresentationState::Normal {
        let geometry = if current_state == WindowPresentationState::Normal {
            current_geometry
        } else {
            restore_state
                .as_ref()
                .map(|restore| restore.normal_geometry)
                .unwrap_or(current_geometry)
        };
        return PresentationTransition {
            state: WindowPresentationState::Normal,
            geometry: clamp_geometry_to_area(geometry, work_area),
            restore_state: None,
            restored_from: restore_state,
        };
    }

    if target_state == WindowPresentationState::Minimized {
        // Minimize changes visibility, not geometry. Keep the restore record so
        // restoring a window after a presentation transition returns to its
        // original normal geometry.
        return PresentationTransition {
            state: WindowPresentationState::Minimized,
            geometry: current_geometry,
            restore_state,
            restored_from: None,
        };
    }

    let normal_geometry = restore_state
        .as_ref()
        .map(|restore| restore.normal_geometry)
        .unwrap_or(current_geometry);
    let area = if target_state == WindowPresentationState::Fullscreen {
        output_area
    } else {
        work_area
    };

    PresentationTransition {
        state: target_state,
        geometry: calculate_presentation_geometry(
            area,
            target_state,
            preferred_size,
            normal_geometry,
        ),
        restore_state,
        restored_from: None,
    }
}

impl WindowRestoreState {
    pub fn new(
        normal_geometry: WindowGeometry,
        previous_state: WindowPresentationState,
        output_id: impl Into<String>,
        space_id: usize,
    ) -> Self {
        Self {
            normal_geometry,
            previous_state,
            output_id: output_id.into(),
            space_id,
            stacking_intent: WindowStackingIntent::Preserve,
        }
    }

    pub fn with_stacking_intent(mut self, stacking_intent: WindowStackingIntent) -> Self {
        self.stacking_intent = stacking_intent;
        self
    }
}

/// Configurable action for zoom button clicks or titlebar double-clicks.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ZoomAction {
    SmartZoom,
    Fill,
    FullScreen,
    ShowLayoutMenu,
    Minimize,
    None,
}

impl ZoomAction {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "smart_zoom" | "smartzoom" | "smart-zoom" => Self::SmartZoom,
            "fill" => Self::Fill,
            "full_screen" | "fullscreen" | "full-screen" => Self::FullScreen,
            "show_layout_menu" | "layout_menu" | "menu" => Self::ShowLayoutMenu,
            "minimize" => Self::Minimize,
            _ => Self::None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SmartZoom => "smart_zoom",
            Self::Fill => "fill",
            Self::FullScreen => "fullscreen",
            Self::ShowLayoutMenu => "layout_menu",
            Self::Minimize => "minimize",
            Self::None => "none",
        }
    }
}

/// User-configurable window management and zoom button policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoomPolicyConfig {
    pub zoom_button_action: ZoomAction,
    pub zoom_button_alternate_action: ZoomAction,
    pub titlebar_double_click_action: ZoomAction,
    pub show_layout_menu_on_hover: bool,
    pub edge_tiling: bool,
    pub edge_fill: bool,
    pub restore_last_geometry: bool,
}

impl Default for ZoomPolicyConfig {
    fn default() -> Self {
        Self {
            zoom_button_action: ZoomAction::SmartZoom,
            zoom_button_alternate_action: ZoomAction::Fill,
            titlebar_double_click_action: ZoomAction::SmartZoom,
            show_layout_menu_on_hover: true,
            edge_tiling: true,
            edge_fill: true,
            restore_last_geometry: true,
        }
    }
}

impl ZoomPolicyConfig {
    /// Parse policy configuration from settings map (e.g. `settings.conf` key-value pairs).
    pub fn from_settings_map(map: &HashMap<String, String>) -> Self {
        let mut config = Self::default();
        if let Some(v) = map.get("zoom_button_action") {
            config.zoom_button_action = ZoomAction::parse(v);
        }
        if let Some(v) = map.get("zoom_button_alternate_action") {
            config.zoom_button_alternate_action = ZoomAction::parse(v);
        }
        if let Some(v) = map.get("titlebar_double_click_action") {
            config.titlebar_double_click_action = ZoomAction::parse(v);
        }
        if let Some(v) = map.get("show_layout_menu_on_hover") {
            config.show_layout_menu_on_hover = parse_bool(v, config.show_layout_menu_on_hover);
        }
        if let Some(v) = map.get("edge_tiling") {
            config.edge_tiling = parse_bool(v, config.edge_tiling);
        }
        if let Some(v) = map.get("edge_fill") {
            config.edge_fill = parse_bool(v, config.edge_fill);
        }
        if let Some(v) = map.get("restore_last_geometry") {
            config.restore_last_geometry = parse_bool(v, config.restore_last_geometry);
        }
        config
    }
}

fn parse_bool(s: &str, default: bool) -> bool {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => default,
    }
}

/// Compute target geometry for a window given work area, presentation state, and preferred size.
pub fn calculate_presentation_geometry(
    work_area: WindowGeometry,
    state: WindowPresentationState,
    preferred_size: Option<(i32, i32)>,
    normal_geometry: WindowGeometry,
) -> WindowGeometry {
    let target = match state {
        WindowPresentationState::Normal | WindowPresentationState::Minimized => normal_geometry,
        WindowPresentationState::Filled | WindowPresentationState::Fullscreen => {
            normalize_area(work_area, 1, 1)
        }
        WindowPresentationState::SmartZoomed => {
            let work_area = normalize_area(work_area, 1, 1);
            if let Some((pref_w, pref_h)) = preferred_size {
                let min_width = 200.min(work_area.width);
                let min_height = 150.min(work_area.height);
                let target_w = pref_w.clamp(min_width, work_area.width);
                let target_h = pref_h.clamp(min_height, work_area.height);
                let target_x = work_area.x.saturating_add((work_area.width - target_w) / 2);
                let target_y = work_area
                    .y
                    .saturating_add((work_area.height - target_h) / 2);
                WindowGeometry::new(target_x, target_y, target_w, target_h)
            } else {
                work_area
            }
        }
        WindowPresentationState::Tiled(placement) => {
            // A split needs two addressable columns/rows.  A degenerate
            // work area is therefore widened or heightened only for the
            // affected axis so no tiled configure has a zero/negative size.
            let (minimum_width, minimum_height) = match placement {
                TilePlacement::Left | TilePlacement::Right => (2, 1),
                TilePlacement::TopLeft
                | TilePlacement::TopRight
                | TilePlacement::BottomLeft
                | TilePlacement::BottomRight => (2, 2),
            };
            let work_area = normalize_area(work_area, minimum_width, minimum_height);
            let (left_width, right_width) = split_extent(work_area.width);
            let (top_height, bottom_height) = split_extent(work_area.height);
            let right_x = work_area.x.saturating_add(left_width);
            let bottom_y = work_area.y.saturating_add(top_height);

            match placement {
                TilePlacement::Left => {
                    WindowGeometry::new(work_area.x, work_area.y, left_width, work_area.height)
                }
                TilePlacement::Right => {
                    WindowGeometry::new(right_x, work_area.y, right_width, work_area.height)
                }
                TilePlacement::TopLeft => {
                    WindowGeometry::new(work_area.x, work_area.y, left_width, top_height)
                }
                TilePlacement::TopRight => {
                    WindowGeometry::new(right_x, work_area.y, right_width, top_height)
                }
                TilePlacement::BottomLeft => {
                    WindowGeometry::new(work_area.x, bottom_y, left_width, bottom_height)
                }
                TilePlacement::BottomRight => {
                    WindowGeometry::new(right_x, bottom_y, right_width, bottom_height)
                }
            }
        }
    };

    clamp_geometry_to_area(target, work_area)
}

fn normalize_area(area: WindowGeometry, minimum_width: i32, minimum_height: i32) -> WindowGeometry {
    WindowGeometry::new(
        area.x,
        area.y,
        area.width.max(minimum_width),
        area.height.max(minimum_height),
    )
}

fn split_extent(extent: i32) -> (i32, i32) {
    let first = extent / 2;
    (first, extent - first)
}

fn clamp_geometry_to_area(desired: WindowGeometry, area: WindowGeometry) -> WindowGeometry {
    let area = normalize_area(area, 1, 1);
    let width = desired.width.clamp(1, area.width);
    let height = desired.height.clamp(1, area.height);
    let max_x = area.x.saturating_add(area.width.saturating_sub(width));
    let max_y = area.y.saturating_add(area.height.saturating_sub(height));

    WindowGeometry::new(
        desired.x.clamp(area.x, max_x),
        desired.y.clamp(area.y, max_y),
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_action_parse() {
        assert_eq!(ZoomAction::parse("smart_zoom"), ZoomAction::SmartZoom);
        assert_eq!(ZoomAction::parse("fill"), ZoomAction::Fill);
        assert_eq!(ZoomAction::parse("fullscreen"), ZoomAction::FullScreen);
        assert_eq!(ZoomAction::parse("minimize"), ZoomAction::Minimize);
        assert_eq!(ZoomAction::parse("invalid"), ZoomAction::None);
    }

    #[test]
    fn test_zoom_policy_from_settings_map() {
        let mut map = HashMap::new();
        map.insert("zoom_button_action".to_string(), "fill".to_string());
        map.insert(
            "zoom_button_alternate_action".to_string(),
            "smart_zoom".to_string(),
        );
        map.insert(
            "titlebar_double_click_action".to_string(),
            "minimize".to_string(),
        );
        map.insert("edge_tiling".to_string(), "false".to_string());

        let policy = ZoomPolicyConfig::from_settings_map(&map);
        assert_eq!(policy.zoom_button_action, ZoomAction::Fill);
        assert_eq!(policy.zoom_button_alternate_action, ZoomAction::SmartZoom);
        assert_eq!(policy.titlebar_double_click_action, ZoomAction::Minimize);
        assert!(!policy.edge_tiling);
        assert!(policy.edge_fill);
    }

    #[test]
    fn test_calculate_presentation_geometry() {
        let work_area = WindowGeometry::new(0, 30, 1280, 770);
        let normal = WindowGeometry::new(100, 100, 600, 400);

        // Fill fills work area
        let fill_geom = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::Filled,
            None,
            normal,
        );
        assert_eq!(fill_geom, work_area);

        // Smart zoom uses preferred size if present
        let smart_geom = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::SmartZoomed,
            Some((800, 600)),
            normal,
        );
        assert_eq!(smart_geom.width, 800);
        assert_eq!(smart_geom.height, 600);

        // Tile left halves width
        let tile_left = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::Tiled(TilePlacement::Left),
            None,
            normal,
        );
        assert_eq!(tile_left.width, 640);
        assert_eq!(tile_left.height, 770);
        assert_eq!(tile_left.x, 0);
    }

    #[test]
    fn transition_fill_captures_normal_geometry_and_restores_it() {
        let work_area = WindowGeometry::new(0, 24, 1280, 712);
        let output = WindowGeometry::new(0, 0, 1280, 800);
        let normal = WindowGeometry::new(120, 88, 640, 420);

        let filled = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::Filled,
            work_area,
            output,
            None,
            "drm-0",
            2,
        );
        assert_eq!(filled.state, WindowPresentationState::Filled);
        assert_eq!(filled.geometry, work_area);
        assert_eq!(
            filled
                .restore_state
                .as_ref()
                .map(|restore| restore.normal_geometry),
            Some(normal)
        );
        assert_eq!(
            filled
                .restore_state
                .as_ref()
                .map(|restore| restore.output_id.as_str()),
            Some("drm-0")
        );

        let restored = transition_presentation_state(
            filled.state,
            filled.geometry,
            filled.restore_state.as_ref(),
            WindowPresentationState::Normal,
            work_area,
            output,
            None,
            "drm-0",
            2,
        );
        assert_eq!(restored.state, WindowPresentationState::Normal);
        assert_eq!(restored.geometry, normal);
        assert_eq!(restored.restore_state, None);
    }

    #[test]
    fn transition_fullscreen_uses_output_but_restores_same_normal_geometry() {
        let work_area = WindowGeometry::new(0, 24, 1280, 712);
        let output = WindowGeometry::new(0, 0, 1280, 800);
        let normal = WindowGeometry::new(40, 72, 720, 480);
        let filled = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::Filled,
            work_area,
            output,
            None,
            "nested-0",
            0,
        );
        let fullscreen = transition_presentation_state(
            filled.state,
            filled.geometry,
            filled.restore_state.as_ref(),
            WindowPresentationState::Fullscreen,
            work_area,
            output,
            None,
            "nested-0",
            0,
        );

        assert_eq!(fullscreen.geometry, output);
        assert_eq!(
            fullscreen
                .restore_state
                .as_ref()
                .map(|restore| restore.normal_geometry),
            Some(normal)
        );
    }

    #[test]
    fn transition_minimize_does_not_change_geometry() {
        let normal = WindowGeometry::new(30, 40, 640, 420);
        let transition = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::Minimized,
            WindowGeometry::new(0, 24, 1280, 712),
            WindowGeometry::new(0, 0, 1280, 800),
            None,
            "nested-0",
            0,
        );
        assert_eq!(transition.state, WindowPresentationState::Minimized);
        assert_eq!(transition.geometry, normal);
    }

    #[test]
    fn normal_transition_exposes_consumed_restore_metadata_without_active_record() {
        let normal = WindowGeometry::new(40, 60, 640, 420);
        let restore =
            WindowRestoreState::new(normal, WindowPresentationState::Normal, "output-a", 7)
                .with_stacking_intent(WindowStackingIntent::RestoreAt(3));

        let restored = transition_presentation_state(
            WindowPresentationState::Filled,
            WindowGeometry::new(0, 24, 1280, 712),
            Some(&restore),
            WindowPresentationState::Normal,
            WindowGeometry::new(0, 24, 1280, 712),
            WindowGeometry::new(0, 0, 1280, 800),
            None,
            "output-b",
            99,
        );

        assert_eq!(restored.state, WindowPresentationState::Normal);
        assert_eq!(restored.restore_state, None);
        let consumed = restored
            .restored_from
            .as_ref()
            .expect("consumed restore metadata");
        assert_eq!(consumed.normal_geometry, normal);
        assert_eq!(consumed.output_id, "output-a");
        assert_eq!(consumed.space_id, 7);
        assert_eq!(consumed.stacking_intent, WindowStackingIntent::RestoreAt(3));
    }

    #[test]
    fn minimized_preserves_current_geometry_without_work_area_clamping() {
        let current = WindowGeometry::new(-80, -10, 900, 700);
        let work_area = WindowGeometry::new(100, 200, 320, 240);
        let transition = transition_presentation_state(
            WindowPresentationState::Normal,
            current,
            None,
            WindowPresentationState::Minimized,
            work_area,
            WindowGeometry::new(100, 180, 320, 280),
            None,
            "output-a",
            7,
        );

        assert_eq!(transition.state, WindowPresentationState::Minimized);
        assert_eq!(transition.geometry, current);
        assert!(transition.geometry.x < work_area.x);
        assert!(transition.geometry.y < work_area.y);
        assert_eq!(transition.restored_from, None);
    }

    #[test]
    fn transition_minimize_captures_restore_metadata_and_restores_normal_geometry() {
        let normal = WindowGeometry::new(30, 40, 640, 420);
        let work_area = WindowGeometry::new(0, 24, 1280, 712);
        let output = WindowGeometry::new(0, 0, 1280, 800);

        let minimized = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::Minimized,
            work_area,
            output,
            None,
            "output-a",
            7,
        );
        let restore = minimized.restore_state.as_ref().expect("restore record");
        assert_eq!(restore.normal_geometry, normal);
        assert_eq!(restore.previous_state, WindowPresentationState::Normal);
        assert_eq!(restore.output_id, "output-a");
        assert_eq!(restore.space_id, 7);
        assert_eq!(restore.stacking_intent, WindowStackingIntent::Preserve);

        let minimized_again = transition_presentation_state(
            minimized.state,
            minimized.geometry,
            minimized.restore_state.as_ref(),
            WindowPresentationState::Minimized,
            WindowGeometry::new(100, 200, 320, 240),
            WindowGeometry::new(100, 180, 400, 300),
            None,
            "output-b",
            99,
        );
        assert_eq!(minimized_again.restore_state, minimized.restore_state);

        let restored = transition_presentation_state(
            minimized_again.state,
            minimized_again.geometry,
            minimized_again.restore_state.as_ref(),
            WindowPresentationState::Normal,
            WindowGeometry::new(100, 200, 320, 240),
            WindowGeometry::new(100, 180, 400, 300),
            None,
            "output-b",
            99,
        );
        assert_eq!(restored.state, WindowPresentationState::Normal);
        assert_eq!(restored.geometry, WindowGeometry::new(100, 200, 320, 240));
        assert_eq!(restored.restore_state, None);
    }

    #[test]
    fn restoring_after_work_area_change_clamps_saved_geometry() {
        let normal = WindowGeometry::new(-40, 150, 500, 300);
        let original_work_area = WindowGeometry::new(0, 24, 1280, 712);
        let original_output = WindowGeometry::new(0, 0, 1280, 800);
        let changed_work_area = WindowGeometry::new(100, 200, 800, 500);
        let changed_output = WindowGeometry::new(100, 180, 800, 540);

        let filled = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::Filled,
            original_work_area,
            original_output,
            None,
            "output-a",
            4,
        );
        let restored = transition_presentation_state(
            filled.state,
            filled.geometry,
            filled.restore_state.as_ref(),
            WindowPresentationState::Normal,
            changed_work_area,
            changed_output,
            None,
            "output-b",
            9,
        );

        assert_eq!(restored.state, WindowPresentationState::Normal);
        assert_eq!(restored.geometry, WindowGeometry::new(100, 200, 500, 300));
        assert_eq!(restored.restore_state, None);
    }

    #[test]
    fn repeated_toggle_uses_clamped_restore_as_new_normal_baseline() {
        let normal = WindowGeometry::new(-40, 150, 500, 300);
        let original_work_area = WindowGeometry::new(0, 24, 1280, 712);
        let original_output = WindowGeometry::new(0, 0, 1280, 800);
        let changed_work_area = WindowGeometry::new(100, 200, 800, 500);
        let changed_output = WindowGeometry::new(100, 180, 800, 540);
        let expected_normal = WindowGeometry::new(100, 200, 500, 300);

        let filled = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::Filled,
            original_work_area,
            original_output,
            None,
            "output-a",
            4,
        );
        let restored = transition_presentation_state(
            filled.state,
            filled.geometry,
            filled.restore_state.as_ref(),
            WindowPresentationState::Normal,
            changed_work_area,
            changed_output,
            None,
            "output-b",
            9,
        );
        let smart_zoomed = transition_presentation_state(
            restored.state,
            restored.geometry,
            restored.restore_state.as_ref(),
            WindowPresentationState::SmartZoomed,
            changed_work_area,
            changed_output,
            Some((600, 400)),
            "output-b",
            9,
        );
        let restored_again = transition_presentation_state(
            smart_zoomed.state,
            smart_zoomed.geometry,
            smart_zoomed.restore_state.as_ref(),
            WindowPresentationState::Normal,
            changed_work_area,
            changed_output,
            None,
            "output-b",
            9,
        );

        assert_eq!(restored.geometry, expected_normal);
        assert_eq!(
            smart_zoomed
                .restore_state
                .as_ref()
                .map(|restore| restore.normal_geometry),
            Some(expected_normal)
        );
        assert_eq!(restored_again.geometry, expected_normal);
    }

    #[test]
    fn transitions_preserve_explicit_stacking_restore_intent() {
        let normal = WindowGeometry::new(40, 60, 640, 420);
        let restore =
            WindowRestoreState::new(normal, WindowPresentationState::Normal, "output-a", 7)
                .with_stacking_intent(WindowStackingIntent::RestoreAt(3));

        let tiled = transition_presentation_state(
            WindowPresentationState::Filled,
            WindowGeometry::new(0, 24, 1280, 712),
            Some(&restore),
            WindowPresentationState::Tiled(TilePlacement::Right),
            WindowGeometry::new(0, 24, 1280, 712),
            WindowGeometry::new(0, 0, 1280, 800),
            None,
            "output-b",
            9,
        );

        assert_eq!(
            tiled
                .restore_state
                .as_ref()
                .map(|restore| restore.stacking_intent),
            Some(WindowStackingIntent::RestoreAt(3))
        );
    }

    #[test]
    fn transition_preserves_first_restore_record_through_all_presentation_states() {
        let normal = WindowGeometry::new(120, 88, 640, 420);
        let work_area = WindowGeometry::new(0, 24, 1280, 712);
        let output = WindowGeometry::new(0, 0, 1280, 800);

        let smart_zoomed = transition_presentation_state(
            WindowPresentationState::Normal,
            normal,
            None,
            WindowPresentationState::SmartZoomed,
            work_area,
            output,
            Some((800, 500)),
            "output-a",
            3,
        );
        let restore = smart_zoomed.restore_state.clone();

        let filled = transition_presentation_state(
            smart_zoomed.state,
            smart_zoomed.geometry,
            smart_zoomed.restore_state.as_ref(),
            WindowPresentationState::Filled,
            WindowGeometry::new(10, 40, 1100, 600),
            WindowGeometry::new(10, 0, 1100, 700),
            None,
            "output-b",
            8,
        );
        let tiled = transition_presentation_state(
            filled.state,
            filled.geometry,
            filled.restore_state.as_ref(),
            WindowPresentationState::Tiled(TilePlacement::BottomRight),
            work_area,
            output,
            None,
            "output-c",
            12,
        );
        let fullscreen = transition_presentation_state(
            tiled.state,
            tiled.geometry,
            tiled.restore_state.as_ref(),
            WindowPresentationState::Fullscreen,
            WindowGeometry::new(20, 30, 900, 500),
            WindowGeometry::new(20, 0, 900, 650),
            None,
            "output-d",
            16,
        );
        let minimized = transition_presentation_state(
            fullscreen.state,
            fullscreen.geometry,
            fullscreen.restore_state.as_ref(),
            WindowPresentationState::Minimized,
            work_area,
            output,
            None,
            "output-e",
            20,
        );

        assert_eq!(filled.restore_state, restore);
        assert_eq!(tiled.restore_state, restore);
        assert_eq!(fullscreen.restore_state, restore);
        assert_eq!(minimized.restore_state, restore);

        let restored = transition_presentation_state(
            minimized.state,
            minimized.geometry,
            minimized.restore_state.as_ref(),
            WindowPresentationState::Normal,
            work_area,
            output,
            None,
            "output-f",
            24,
        );
        assert_eq!(restored.geometry, normal);
        assert_eq!(restored.restore_state, None);
    }

    #[test]
    fn presentation_geometry_sanitizes_invalid_work_areas() {
        let invalid_area = WindowGeometry::new(-40, 18, -801, -601);
        let normal = WindowGeometry::new(20, 30, 640, 420);
        let states = [
            WindowPresentationState::SmartZoomed,
            WindowPresentationState::Filled,
            WindowPresentationState::Fullscreen,
            WindowPresentationState::Tiled(TilePlacement::Left),
            WindowPresentationState::Tiled(TilePlacement::BottomRight),
        ];

        for state in states {
            let geometry =
                calculate_presentation_geometry(invalid_area, state, Some((800, 500)), normal);
            assert!(geometry.width >= 1, "{state:?} has invalid width");
            assert!(geometry.height >= 1, "{state:?} has invalid height");
        }
    }

    #[test]
    fn every_presentation_target_is_contained_by_its_work_area() {
        let work_area = WindowGeometry::new(100, 200, 800, 500);
        let normal = WindowGeometry::new(-40, 150, 1200, 700);
        let states = [
            WindowPresentationState::Normal,
            WindowPresentationState::Minimized,
            WindowPresentationState::SmartZoomed,
            WindowPresentationState::Filled,
            WindowPresentationState::Fullscreen,
            WindowPresentationState::Tiled(TilePlacement::Left),
            WindowPresentationState::Tiled(TilePlacement::Right),
            WindowPresentationState::Tiled(TilePlacement::TopLeft),
            WindowPresentationState::Tiled(TilePlacement::TopRight),
            WindowPresentationState::Tiled(TilePlacement::BottomLeft),
            WindowPresentationState::Tiled(TilePlacement::BottomRight),
        ];

        for state in states {
            let geometry =
                calculate_presentation_geometry(work_area, state, Some((-1, 900)), normal);
            assert!(geometry.x >= work_area.x, "{state:?} escapes left edge");
            assert!(geometry.y >= work_area.y, "{state:?} escapes top edge");
            assert!(
                geometry.x + geometry.width <= work_area.x + work_area.width,
                "{state:?} escapes right edge"
            );
            assert!(
                geometry.y + geometry.height <= work_area.y + work_area.height,
                "{state:?} escapes bottom edge"
            );
        }
    }

    #[test]
    fn tiled_geometry_partitions_odd_work_area_without_gaps() {
        let work_area = WindowGeometry::new(11, 23, 801, 601);
        let normal = WindowGeometry::new(20, 30, 640, 420);

        let left = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::Tiled(TilePlacement::Left),
            None,
            normal,
        );
        let right = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::Tiled(TilePlacement::Right),
            None,
            normal,
        );
        let top_left = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::Tiled(TilePlacement::TopLeft),
            None,
            normal,
        );
        let bottom_right = calculate_presentation_geometry(
            work_area,
            WindowPresentationState::Tiled(TilePlacement::BottomRight),
            None,
            normal,
        );

        assert_eq!(left.width + right.width, work_area.width);
        assert_eq!(right.x, left.x + left.width);
        assert_eq!(top_left.height + bottom_right.height, work_area.height);
        assert_eq!(bottom_right.y, top_left.y + top_left.height);
        assert_eq!(right.width, 401);
        assert_eq!(bottom_right.height, 301);
    }
}
