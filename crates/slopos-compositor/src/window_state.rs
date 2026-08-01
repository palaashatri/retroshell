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
                let target_w = pref_w.clamp(200, work_area.width);
                let target_h = pref_h.clamp(150, work_area.height);
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
}
