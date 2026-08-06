//! Pure multi-output assignment and geometry policy.
//!
//! Live backends use these helpers to keep fullscreen, Fill, Smart Zoom,
//! popups and restore geometry on one real output rather than treating the
//! entire multi-monitor canvas as a single monitor.

use crate::{LaidOutOutput, WindowGeometry};

/// Convert a logical output description into compositor geometry.
pub fn output_geometry(output: &LaidOutOutput) -> WindowGeometry {
    WindowGeometry::new(
        output.x,
        output.y,
        output.config.width.max(1),
        output.config.height.max(1),
    )
}

/// Bounding rectangle for every output, including negative origins.
pub fn output_layout_bounds(outputs: &[LaidOutOutput]) -> Option<WindowGeometry> {
    let first = outputs.first()?;
    let mut min_x = i64::from(first.x);
    let mut min_y = i64::from(first.y);
    let mut max_x = i64::from(first.x) + i64::from(first.config.width.max(1));
    let mut max_y = i64::from(first.y) + i64::from(first.config.height.max(1));

    for output in &outputs[1..] {
        let width = i64::from(output.config.width.max(1));
        let height = i64::from(output.config.height.max(1));
        let x = i64::from(output.x);
        let y = i64::from(output.y);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x.saturating_add(width));
        max_y = max_y.max(y.saturating_add(height));
    }

    Some(WindowGeometry::new(
        clamp_i64_to_i32(min_x),
        clamp_i64_to_i32(min_y),
        clamp_positive_i64_to_i32(max_x.saturating_sub(min_x)),
        clamp_positive_i64_to_i32(max_y.saturating_sub(min_y)),
    ))
}

/// Shift a layout so its union starts at compositor coordinate `(0, 0)`.
///
/// Wayland output positions remain relative to each other, while the nested
/// framebuffer no longer clips outputs placed left or above the nominal origin.
pub fn normalize_laid_out_outputs(outputs: &[LaidOutOutput]) -> Vec<LaidOutOutput> {
    let Some(bounds) = output_layout_bounds(outputs) else {
        return Vec::new();
    };
    outputs
        .iter()
        .map(|output| LaidOutOutput {
            config: output.config,
            x: output.x.saturating_sub(bounds.x),
            y: output.y.saturating_sub(bounds.y),
        })
        .collect()
}

/// Select the output containing a point, or the nearest output when the point
/// is outside every output or inside a layout gap.
pub fn output_index_for_point(
    outputs: &[LaidOutOutput],
    point_x: i32,
    point_y: i32,
) -> Option<usize> {
    let point_x = i64::from(point_x);
    let point_y = i64::from(point_y);
    let mut nearest: Option<(usize, i128)> = None;

    for (index, output) in outputs.iter().enumerate() {
        let geometry = output_geometry(output);
        let left = i64::from(geometry.x);
        let top = i64::from(geometry.y);
        let right = left.saturating_add(i64::from(geometry.width));
        let bottom = top.saturating_add(i64::from(geometry.height));

        if point_x >= left && point_x < right && point_y >= top && point_y < bottom {
            return Some(index);
        }

        let dx = axis_distance_to_half_open_rect(point_x, left, right);
        let dy = axis_distance_to_half_open_rect(point_y, top, bottom);
        let distance = i128::from(dx) * i128::from(dx) + i128::from(dy) * i128::from(dy);
        if nearest.is_none_or(|(_, best)| distance < best) {
            nearest = Some((index, distance));
        }
    }

    nearest.map(|(index, _)| index)
}

/// Select the output owning a window.
///
/// The output with the greatest intersection area wins. Ties prefer the output
/// containing the window centre, then retain stable layout order. Completely
/// off-screen windows use the nearest output to their centre.
pub fn output_index_for_geometry(
    outputs: &[LaidOutOutput],
    geometry: WindowGeometry,
) -> Option<usize> {
    if outputs.is_empty() {
        return None;
    }

    let geometry = WindowGeometry::new(
        geometry.x,
        geometry.y,
        geometry.width.max(1),
        geometry.height.max(1),
    );
    let centre_x = i64::from(geometry.x) + i64::from(geometry.width) / 2;
    let centre_y = i64::from(geometry.y) + i64::from(geometry.height) / 2;
    let mut best: Option<(usize, i64, bool)> = None;

    for (index, output) in outputs.iter().enumerate() {
        let output_geometry = output_geometry(output);
        let area = intersection_area(geometry, output_geometry);
        let centre_inside = point_inside_i64(centre_x, centre_y, output_geometry);
        let replace = best.is_none_or(|(_, best_area, best_contains)| {
            area > best_area || (area == best_area && centre_inside && !best_contains)
        });
        if replace {
            best = Some((index, area, centre_inside));
        }
    }

    let (best_index, best_area, _) = best.expect("non-empty outputs always produce a candidate");
    if best_area > 0 {
        Some(best_index)
    } else {
        output_index_for_point(
            outputs,
            clamp_i64_to_i32(centre_x),
            clamp_i64_to_i32(centre_y),
        )
    }
}

/// True when two logical rectangles overlap by at least one pixel.
pub fn geometries_intersect(a: WindowGeometry, b: WindowGeometry) -> bool {
    intersection_area(a, b) > 0
}

fn intersection_area(a: WindowGeometry, b: WindowGeometry) -> i64 {
    let a_left = i64::from(a.x);
    let a_top = i64::from(a.y);
    let a_right = a_left.saturating_add(i64::from(a.width.max(1)));
    let a_bottom = a_top.saturating_add(i64::from(a.height.max(1)));
    let b_left = i64::from(b.x);
    let b_top = i64::from(b.y);
    let b_right = b_left.saturating_add(i64::from(b.width.max(1)));
    let b_bottom = b_top.saturating_add(i64::from(b.height.max(1)));

    let width = a_right
        .min(b_right)
        .saturating_sub(a_left.max(b_left))
        .max(0);
    let height = a_bottom
        .min(b_bottom)
        .saturating_sub(a_top.max(b_top))
        .max(0);
    width.saturating_mul(height)
}

fn point_inside_i64(x: i64, y: i64, geometry: WindowGeometry) -> bool {
    let left = i64::from(geometry.x);
    let top = i64::from(geometry.y);
    let right = left.saturating_add(i64::from(geometry.width.max(1)));
    let bottom = top.saturating_add(i64::from(geometry.height.max(1)));
    x >= left && x < right && y >= top && y < bottom
}

fn axis_distance_to_half_open_rect(point: i64, start: i64, end: i64) -> i64 {
    if point < start {
        start.saturating_sub(point)
    } else if point >= end {
        point.saturating_sub(end.saturating_sub(1))
    } else {
        0
    }
}

fn clamp_i64_to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn clamp_positive_i64_to_i32(value: i64) -> i32 {
    value.clamp(1, i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutputConfig;

    fn output(x: i32, y: i32, width: i32, height: i32) -> LaidOutOutput {
        LaidOutOutput {
            config: OutputConfig { width, height },
            x,
            y,
        }
    }

    #[test]
    fn negative_and_offset_layouts_are_normalized_without_losing_relationships() {
        let normalized = normalize_laid_out_outputs(&[
            output(-1920, 120, 1920, 1080),
            output(0, -200, 2560, 1440),
        ]);
        assert_eq!(normalized[0], output(0, 320, 1920, 1080));
        assert_eq!(normalized[1], output(1920, 0, 2560, 1440));
        assert_eq!(
            output_layout_bounds(&normalized),
            Some(WindowGeometry::new(0, 0, 4480, 1440))
        );
    }

    #[test]
    fn greatest_window_overlap_selects_the_owning_output() {
        let outputs = [output(0, 0, 1000, 800), output(1000, 0, 1600, 900)];
        assert_eq!(
            output_index_for_geometry(&outputs, WindowGeometry::new(850, 100, 600, 500)),
            Some(1)
        );
        assert_eq!(
            output_index_for_geometry(&outputs, WindowGeometry::new(700, 100, 500, 500)),
            Some(0)
        );
    }

    #[test]
    fn equal_overlap_prefers_the_output_containing_the_window_centre() {
        let outputs = [output(0, 0, 1000, 800), output(1000, 0, 1000, 800)];
        assert_eq!(
            output_index_for_geometry(&outputs, WindowGeometry::new(750, 100, 500, 500)),
            Some(1)
        );
    }

    #[test]
    fn layout_gaps_and_offscreen_windows_choose_the_nearest_output() {
        let outputs = [output(0, 0, 800, 600), output(1200, 0, 800, 600)];
        assert_eq!(output_index_for_point(&outputs, 900, 300), Some(0));
        assert_eq!(output_index_for_point(&outputs, 1100, 300), Some(1));
        assert_eq!(
            output_index_for_geometry(&outputs, WindowGeometry::new(3000, 100, 400, 300)),
            Some(1)
        );
    }

    #[test]
    fn intersection_is_half_open_and_overflow_safe() {
        assert!(!geometries_intersect(
            WindowGeometry::new(0, 0, 100, 100),
            WindowGeometry::new(100, 0, 100, 100)
        ));
        assert!(geometries_intersect(
            WindowGeometry::new(i32::MAX - 20, 0, 100, 10),
            WindowGeometry::new(i32::MAX - 10, 0, 10, 10)
        ));
    }
}
