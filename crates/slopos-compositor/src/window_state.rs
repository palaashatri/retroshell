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

/// Geometry and location recorded prior to a presentation state transition
/// (zoom, fill, fullscreen, tiling).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowRestoreState {
    pub normal_geometry: WindowGeometry,
    pub previous_state: WindowPresentationState,
    pub output_id: String,
    pub space_id: usize,
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
    let mut restore_state = current_restore_state.cloned();

    if !matches!(
        target_state,
        WindowPresentationState::Normal | WindowPresentationState::Minimized
    ) && restore_state.is_none()
    {
        restore_state = Some(WindowRestoreState::new(
            current_geometry,
            current_state,
            output_id,
            space_id,
        ));
    }

    if target_state == WindowPresentationState::Normal {
        let geometry = restore_state
            .as_ref()
            .map(|restore| restore.normal_geometry)
            .unwrap_or(current_geometry);
        return PresentationTransition {
            state: WindowPresentationState::Normal,
            geometry,
            restore_state: None,
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
        }
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
    match state {
        WindowPresentationState::Normal => normal_geometry,
        WindowPresentationState::Minimized => normal_geometry,
        WindowPresentationState::Filled | WindowPresentationState::Fullscreen => work_area,
        WindowPresentationState::SmartZoomed => {
            if let Some((pref_w, pref_h)) = preferred_size {
                let area_width = work_area.width.max(1);
                let area_height = work_area.height.max(1);
                let min_width = 200.min(area_width);
                let min_height = 150.min(area_height);
                let target_w = pref_w.clamp(min_width, area_width);
                let target_h = pref_h.clamp(min_height, area_height);
                let target_x = work_area.x + (work_area.width - target_w) / 2;
                let target_y = work_area.y + (work_area.height - target_h) / 2;
                WindowGeometry::new(target_x, target_y, target_w, target_h)
            } else {
                work_area
            }
        }
        WindowPresentationState::Tiled(placement) => match placement {
            TilePlacement::Left => WindowGeometry::new(
                work_area.x,
                work_area.y,
                work_area.width / 2,
                work_area.height,
            ),
            TilePlacement::Right => WindowGeometry::new(
                work_area.x + work_area.width / 2,
                work_area.y,
                work_area.width / 2,
                work_area.height,
            ),
            TilePlacement::TopLeft => WindowGeometry::new(
                work_area.x,
                work_area.y,
                work_area.width / 2,
                work_area.height / 2,
            ),
            TilePlacement::TopRight => WindowGeometry::new(
                work_area.x + work_area.width / 2,
                work_area.y,
                work_area.width / 2,
                work_area.height / 2,
            ),
            TilePlacement::BottomLeft => WindowGeometry::new(
                work_area.x,
                work_area.y + work_area.height / 2,
                work_area.width / 2,
                work_area.height / 2,
            ),
            TilePlacement::BottomRight => WindowGeometry::new(
                work_area.x + work_area.width / 2,
                work_area.y + work_area.height / 2,
                work_area.width / 2,
                work_area.height / 2,
            ),
        },
    }
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
}
