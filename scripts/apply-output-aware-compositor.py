#!/usr/bin/env python3
"""Apply the first production-compositor implementation wave.

The migration is guarded and one-shot. It adds pure output-assignment policy,
normalises nested output layouts, and wires per-output presentation/work-area
selection into the live compositor. A second mode records the verified
implementation commit in TRUTH.md after build/runtime gates pass.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one exact match, found {count}")
    return text.replace(old, new, 1)


def replace_regex_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise RuntimeError(f"{label}: expected one regex match, found {count}")
    return updated


OUTPUT_ASSIGNMENT = r'''//! Pure multi-output assignment and geometry policy.
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

    let width = a_right.min(b_right).saturating_sub(a_left.max(b_left)).max(0);
    let height = a_bottom.min(b_bottom).saturating_sub(a_top.max(b_top)).max(0);
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
'''


def apply_code() -> None:
    module = ROOT / "crates/slopos-compositor/src/output_assignment.rs"
    if module.exists():
        raise RuntimeError(f"refusing to overwrite existing {module.relative_to(ROOT)}")
    module.write_text(OUTPUT_ASSIGNMENT, encoding="utf-8")

    lib_path = ROOT / "crates/slopos-compositor/src/lib.rs"
    lib = lib_path.read_text(encoding="utf-8")
    lib = replace_once(
        lib,
        "pub mod hdr;\npub mod perf_budget;\n",
        "pub mod hdr;\npub mod output_assignment;\npub mod perf_budget;\n",
        "lib module declaration",
    )
    lib = replace_once(
        lib,
        "pub use spaces::{\n",
        "pub use output_assignment::{\n"
        "    geometries_intersect, normalize_laid_out_outputs, output_geometry,\n"
        "    output_index_for_geometry, output_index_for_point, output_layout_bounds,\n"
        "};\n"
        "pub use spaces::{\n",
        "lib public exports",
    )
    lib = replace_once(
        lib,
        "            let laid_out = laid_out_from_layout_entries(&entries);\n",
        "            let laid_out =\n"
        "                normalize_laid_out_outputs(&laid_out_from_layout_entries(&entries));\n",
        "layout-spec normalization",
    )
    lib = replace_once(
        lib,
        "    let laid_out = layout_outputs(&configs, layout_mode);\n",
        "    let laid_out = normalize_laid_out_outputs(&layout_outputs(&configs, layout_mode));\n",
        "generated-layout normalization",
    )
    lib = replace_regex_once(
        lib,
        r"pub fn total_output_size\(laid_out: &\[LaidOutOutput\]\) -> OutputConfig \{.*?\n\}\n\n/// Resolve output list",
        "pub fn total_output_size(laid_out: &[LaidOutOutput]) -> OutputConfig {\n"
        "    let Some(bounds) = output_layout_bounds(laid_out) else {\n"
        "        return OutputConfig::default();\n"
        "    };\n"
        "    OutputConfig {\n"
        "        width: bounds.width.max(1),\n"
        "        height: bounds.height.max(1),\n"
        "    }\n"
        "}\n\n/// Resolve output list",
        "total output bounds",
    )
    lib_path.write_text(lib, encoding="utf-8")

    main_path = ROOT / "crates/slopos-compositor/src/main.rs"
    main = main_path.read_text(encoding="utf-8")
    main = replace_once(
        main,
        "        next_cascade_offset, output_scale_summary, pointer_grab_request_is_valid_for_window,\n",
        "        geometries_intersect, next_cascade_offset, output_geometry,\n"
        "        output_index_for_geometry, output_index_for_point, output_scale_summary,\n"
        "        pointer_grab_request_is_valid_for_window,\n",
        "main output-policy imports",
    )
    main = replace_once(
        main,
        "        outputs: Vec<Output>,\n        running: bool,\n",
        "        outputs: Vec<Output>,\n"
        "        /// Normalized logical output rectangles used for window assignment.\n"
        "        laid_out_outputs: Vec<LaidOutOutput>,\n"
        "        /// Connector or synthetic names parallel to `laid_out_outputs`.\n"
        "        output_names: Vec<String>,\n"
        "        running: bool,\n",
        "compositor output fields",
    )
    main = replace_once(
        main,
        "            let output = self.output_area();\n",
        "            let output = self.output_area_for_point(root_origin);\n",
        "popup output constraint",
    )
    main = replace_regex_once(
        main,
        r"        fn output_area\(&self\) -> WindowGeometry \{.*?\n        \}\n\n        fn work_area\(&self\) -> WindowGeometry \{.*?\n        \}\n\n        /// Keep normal windows",
        "        fn canvas_area(&self) -> WindowGeometry {\n"
        "            WindowGeometry::new(0, 0, self.output_size.w, self.output_size.h)\n"
        "        }\n\n"
        "        fn output_area_for_point(&self, point: Point<i32, Logical>) -> WindowGeometry {\n"
        "            output_index_for_point(&self.laid_out_outputs, point.x, point.y)\n"
        "                .and_then(|index| self.laid_out_outputs.get(index))\n"
        "                .map(output_geometry)\n"
        "                .unwrap_or_else(|| self.canvas_area())\n"
        "        }\n\n"
        "        fn work_area_for_output(&self, output: WindowGeometry) -> WindowGeometry {\n"
        "            let reservations = self.layer_surfaces.iter().filter_map(|layer| {\n"
        "                let layer_geometry = WindowGeometry::new(\n"
        "                    layer.geo.loc.x,\n"
        "                    layer.geo.loc.y,\n"
        "                    layer.geo.size.w,\n"
        "                    layer.geo.size.h,\n"
        "                );\n"
        "                if !geometries_intersect(output, layer_geometry) {\n"
        "                    return None;\n"
        "                }\n"
        "                let (_, anchor, margins, _) = layer_surface_request(&layer.surface);\n"
        "                Some(ExclusiveZoneReservation {\n"
        "                    exclusive_zone: layer.exclusive_zone,\n"
        "                    anchor_top: anchor.contains(Anchor::TOP),\n"
        "                    anchor_bottom: anchor.contains(Anchor::BOTTOM),\n"
        "                    anchor_left: anchor.contains(Anchor::LEFT),\n"
        "                    anchor_right: anchor.contains(Anchor::RIGHT),\n"
        "                    margin_top: margins.top,\n"
        "                    margin_bottom: margins.bottom,\n"
        "                    margin_left: margins.left,\n"
        "                    margin_right: margins.right,\n"
        "                })\n"
        "            });\n"
        "            compute_exclusive_work_area(output, reservations)\n"
        "        }\n\n"
        "        /// Keep normal windows",
        "per-output work-area methods",
    )
    main = replace_once(
        main,
        "        fn clamp_normal_windows_to_work_area(&mut self) {\n"
        "            let work_area = self.work_area();\n"
        "            let mut changed = false;\n",
        "        fn clamp_normal_windows_to_work_area(&mut self) {\n"
        "            let fallback_work_area = self.work_area_for_output(self.canvas_area());\n"
        "            let output_work_areas: Vec<WindowGeometry> = self\n"
        "                .laid_out_outputs\n"
        "                .iter()\n"
        "                .map(|output| self.work_area_for_output(output_geometry(output)))\n"
        "                .collect();\n"
        "            let mut changed = false;\n",
        "per-output normal-window clamp prelude",
    )
    main = replace_once(
        main,
        "                let current = window.geometry();\n"
        "                let next = clamp_window_to_work_area(current, work_area);\n",
        "                let current = window.geometry();\n"
        "                let work_area = output_index_for_geometry(&self.laid_out_outputs, current)\n"
        "                    .and_then(|index| output_work_areas.get(index).copied())\n"
        "                    .unwrap_or(fallback_work_area);\n"
        "                let next = clamp_window_to_work_area(current, work_area);\n",
        "per-output normal-window clamp selection",
    )
    main = replace_once(
        main,
        "            let transition = transition_presentation_state(\n"
        "                current_state,\n"
        "                old,\n"
        "                current_restore_state.as_ref(),\n"
        "                target_state,\n"
        "                self.work_area(),\n"
        "                self.output_area(),\n"
        "                None,\n"
        "                \"nested-0\",\n"
        "                self.workspace_state.active.as_usize(),\n"
        "            );\n",
        "            let output_index =\n"
        "                output_index_for_geometry(&self.laid_out_outputs, old).unwrap_or(0);\n"
        "            let output_area = self\n"
        "                .laid_out_outputs\n"
        "                .get(output_index)\n"
        "                .map(output_geometry)\n"
        "                .unwrap_or_else(|| self.canvas_area());\n"
        "            let work_area = self.work_area_for_output(output_area);\n"
        "            let output_id = self\n"
        "                .output_names\n"
        "                .get(output_index)\n"
        "                .cloned()\n"
        "                .unwrap_or_else(|| format!(\"output-{output_index}\"));\n"
        "            let transition = transition_presentation_state(\n"
        "                current_state,\n"
        "                old,\n"
        "                current_restore_state.as_ref(),\n"
        "                target_state,\n"
        "                work_area,\n"
        "                output_area,\n"
        "                None,\n"
        "                output_id,\n"
        "                self.workspace_state.active.as_usize(),\n"
        "            );\n",
        "per-output presentation transition",
    )
    main = replace_once(
        main,
        "        let (outputs, output_size) = create_outputs(\n"
        "            &display_handle,\n"
        "            &resolved.laid_out,\n"
        "            &resolved.names,\n"
        "            refresh_mhz,\n"
        "            output_scale,\n"
        "        );\n",
        "        let laid_out_outputs = resolved.laid_out.clone();\n"
        "        let output_names = resolved.names.clone();\n"
        "        let (outputs, output_size) = create_outputs(\n"
        "            &display_handle,\n"
        "            &laid_out_outputs,\n"
        "            &output_names,\n"
        "            refresh_mhz,\n"
        "            output_scale,\n"
        "        );\n",
        "resolved output state",
    )
    main = replace_once(
        main,
        "            seat,\n            outputs,\n            running: true,\n",
        "            seat,\n"
        "            outputs,\n"
        "            laid_out_outputs,\n"
        "            output_names,\n"
        "            running: true,\n",
        "compositor output initialization",
    )
    if "self.output_area()" in main or "self.work_area()" in main:
        raise RuntimeError("legacy whole-canvas presentation helper remains in main.rs")
    main_path.write_text(main, encoding="utf-8")


def update_truth(implementation_sha: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", implementation_sha):
        raise RuntimeError("truth mode requires a full lowercase commit SHA")
    truth_path = ROOT / "TRUTH.md"
    truth = truth_path.read_text(encoding="utf-8")
    truth = replace_regex_once(
        truth,
        r"\*\*Audited product implementation:\*\*\n`[0-9a-f]{40}`",
        f"**Audited product implementation:**\n`{implementation_sha}`",
        "audited implementation SHA",
    )
    marker = "\n---\n\n## 3. Production scoring model\n"
    wave = f'''\n### Current implementation wave — output-aware presentation\n\nImplementation commit `{implementation_sha}` is **BUILD VERIFIED**, **TEST\nVERIFIED** and covered by the existing **RUNTIME OBSERVED** headless compositor\ngate. The new multi-output geometry itself remains unverified on physical\nmulti-monitor hardware.\n\nThis wave:\n\n- normalises negative/offset nested output layouts while preserving relative\n  monitor placement;\n- computes true output-union bounds without assuming an origin of `(0, 0)`;\n- assigns windows deterministically by greatest output overlap and nearest-output\n  fallback;\n- constrains XDG popups to the output that owns their root surface;\n- applies Smart Zoom, Fill and fullscreen to one selected output instead of the\n  complete multi-monitor canvas;\n- stores the real connector/synthetic output name in restore state;\n- clamps normal windows against the selected output's exclusive work area;\n- adds pure tests for negative layouts, gaps, off-screen windows, overlap ties and\n  integer-boundary safety;\n- regenerates the workspace lockfile and passes compositor check, test, source\n  contract and headless runtime gates before commit.\n\nThe overall product score remains **63/100**. The compositor score advances from\n66 to **67/100**; physical hotplug, mixed-scale/refresh, per-output layer-shell\ntargeting, direct scanout and hardware evidence remain release blockers.\n'''
    if marker not in truth:
        raise RuntimeError("TRUTH.md production scoring marker missing")
    truth = truth.replace(marker, wave + marker, 1)
    truth = replace_once(
        truth,
        "| Compositor strict completion | **66** |",
        "| Compositor strict completion | **67** |",
        "executive compositor score",
    )
    truth = replace_once(
        truth,
        "| Displays and scaling | 5 | 12 |",
        "| Displays and scaling | 6 | 12 |",
        "display score",
    )
    truth = replace_once(
        truth,
        "| **Total** | **66** | **100** |",
        "| **Total** | **67** | **100** |",
        "compositor total",
    )
    truth_path.write_text(truth, encoding="utf-8")


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "apply"
    if mode == "apply":
        apply_code()
    elif mode == "truth" and len(sys.argv) == 3:
        update_truth(sys.argv[2])
    else:
        raise SystemExit("usage: apply-output-aware-compositor.py [apply | truth SHA]")


if __name__ == "__main__":
    main()
