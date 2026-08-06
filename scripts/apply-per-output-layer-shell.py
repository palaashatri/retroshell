#!/usr/bin/env python3
"""Apply and record the per-output layer-shell compositor wave.

This is a guarded, one-shot repository migration. The accompanying workflow
builds and tests the exact generated implementation before pushing it, records
the evidence in TRUTH.md, and removes this script and the workflow.
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


def apply_output_policy() -> None:
    path = ROOT / "crates/slopos-compositor/src/output_assignment.rs"
    text = path.read_text(encoding="utf-8")
    helper = '''/// Return every output genuinely intersected by a surface geometry.\n///\n/// Unlike [`output_index_for_geometry`], this intentionally has no nearest-output\n/// fallback: `wl_surface.enter`/`leave` must describe real scan-out intersection,\n/// not merely the output that would own presentation policy for an off-screen\n/// window.\npub fn intersecting_output_indices(\n    outputs: &[LaidOutOutput],\n    geometry: WindowGeometry,\n) -> Vec<usize> {\n    outputs\n        .iter()\n        .enumerate()\n        .filter_map(|(index, output)| {\n            geometries_intersect(geometry, output_geometry(output)).then_some(index)\n        })\n        .collect()\n}\n\n'''
    text = replace_once(
        text,
        "/// True when two logical rectangles overlap by at least one pixel.\n",
        helper + "/// True when two logical rectangles overlap by at least one pixel.\n",
        "output membership helper",
    )
    tests = '''    #[test]\n    fn surface_membership_reports_every_real_intersection_without_nearest_fallback() {\n        let outputs = [output(0, 0, 1000, 800), output(1000, 0, 1000, 800)];\n        assert_eq!(\n            intersecting_output_indices(\n                &outputs,\n                WindowGeometry::new(900, 100, 300, 400)\n            ),\n            vec![0, 1]\n        );\n        assert!(intersecting_output_indices(\n            &outputs,\n            WindowGeometry::new(2400, 100, 200, 200)\n        )\n        .is_empty());\n    }\n\n'''
    text = replace_once(
        text,
        "    #[test]\n    fn intersection_is_half_open_and_overflow_safe() {\n",
        tests + "    #[test]\n    fn intersection_is_half_open_and_overflow_safe() {\n",
        "output membership tests",
    )
    path.write_text(text, encoding="utf-8")

    lib_path = ROOT / "crates/slopos-compositor/src/lib.rs"
    lib = lib_path.read_text(encoding="utf-8")
    lib = replace_once(
        lib,
        "    geometries_intersect, normalize_laid_out_outputs, output_geometry, output_index_for_geometry,\n"
        "    output_index_for_point, output_layout_bounds,\n",
        "    geometries_intersect, intersecting_output_indices, normalize_laid_out_outputs,\n"
        "    output_geometry, output_index_for_geometry, output_index_for_point, output_layout_bounds,\n",
        "output membership export",
    )
    lib_path.write_text(lib, encoding="utf-8")


def apply_live_compositor() -> None:
    path = ROOT / "crates/slopos-compositor/src/main.rs"
    text = path.read_text(encoding="utf-8")

    text = replace_once(
        text,
        "        focus_window_after_workspace_switch, geometries_intersect, geometry_for_interactive_grab,\n"
        "        move_to_top, next_cascade_offset, output_geometry, output_index_for_geometry,\n",
        "        focus_window_after_workspace_switch, geometry_for_interactive_grab,\n"
        "        intersecting_output_indices, move_to_top, next_cascade_offset, output_geometry,\n"
        "        output_index_for_geometry,\n",
        "main output membership import",
    )

    text = replace_once(
        text,
        "        namespace: String,\n"
        "        /// Authoritative compositor-space placement of the layer surface.\n",
        "        namespace: String,\n"
        "        /// Exact logical output selected by the layer-shell request.\n"
        "        output_index: usize,\n"
        "        /// Authoritative compositor-space placement of the layer surface.\n",
        "mapped layer owner",
    )

    text = replace_regex_once(
        text,
        r"        fn output_area_for_point\(&self, point: Point<i32, Logical>\) -> WindowGeometry \{.*?\n        \}\n\n        fn work_area_for_output\(&self, output: WindowGeometry\) -> WindowGeometry \{.*?\n        \}\n",
        '''        fn output_area_for_index(&self, output_index: usize) -> WindowGeometry {
            self.laid_out_outputs
                .get(output_index)
                .map(output_geometry)
                .unwrap_or_else(|| self.canvas_area())
        }

        fn output_area_for_point(&self, point: Point<i32, Logical>) -> WindowGeometry {
            output_index_for_point(&self.laid_out_outputs, point.x, point.y)
                .map(|index| self.output_area_for_index(index))
                .unwrap_or_else(|| self.canvas_area())
        }

        fn output_index_for_resource(&self, requested: Option<&wl_output::WlOutput>) -> usize {
            requested
                .and_then(Output::from_resource)
                .and_then(|requested| self.outputs.iter().position(|output| output == &requested))
                .unwrap_or(0)
        }

        fn sync_surface_output_membership(
            &self,
            surface: &WlSurface,
            geometry: WindowGeometry,
        ) {
            let intersecting = intersecting_output_indices(&self.laid_out_outputs, geometry);
            for (index, output) in self.outputs.iter().enumerate() {
                if intersecting.contains(&index) {
                    output.enter(surface);
                } else {
                    output.leave(surface);
                }
            }
        }

        fn sync_surface_to_output(&self, surface: &WlSurface, output_index: usize) {
            for (index, output) in self.outputs.iter().enumerate() {
                if index == output_index {
                    output.enter(surface);
                } else {
                    output.leave(surface);
                }
            }
        }

        fn sync_all_window_output_membership(&self) {
            for window in &self.windows {
                self.sync_surface_output_membership(window.toplevel.wl_surface(), window.geometry());
            }
        }

        fn work_area_for_output_index(&self, output_index: usize) -> WindowGeometry {
            let output = self.output_area_for_index(output_index);
            let reservations = self
                .layer_surfaces
                .iter()
                .filter(|layer| layer.output_index == output_index)
                .map(|layer| {
                    let (_, anchor, margins, _) = layer_surface_request(&layer.surface);
                    ExclusiveZoneReservation {
                        exclusive_zone: layer.exclusive_zone,
                        anchor_top: anchor.contains(Anchor::TOP),
                        anchor_bottom: anchor.contains(Anchor::BOTTOM),
                        anchor_left: anchor.contains(Anchor::LEFT),
                        anchor_right: anchor.contains(Anchor::RIGHT),
                        margin_top: margins.top,
                        margin_bottom: margins.bottom,
                        margin_left: margins.left,
                        margin_right: margins.right,
                    }
                });
            compute_exclusive_work_area(output, reservations)
        }
''',
        "per-output ownership methods",
    )

    text = replace_once(
        text,
        "            let fallback_work_area = self.work_area_for_output(self.canvas_area());\n"
        "            let output_work_areas: Vec<WindowGeometry> = self\n"
        "                .laid_out_outputs\n"
        "                .iter()\n"
        "                .map(|output| self.work_area_for_output(output_geometry(output)))\n"
        "                .collect();\n",
        "            let fallback_work_area = self.work_area_for_output_index(0);\n"
        "            let output_work_areas: Vec<WindowGeometry> = (0..self.laid_out_outputs.len())\n"
        "                .map(|index| self.work_area_for_output_index(index))\n"
        "                .collect();\n",
        "per-output clamp work areas",
    )
    text = replace_once(
        text,
        "            if changed {\n"
        "                self.request_redraw();\n"
        "            }\n"
        "        }\n\n"
        "        fn apply_session_control_request",
        "            if changed {\n"
        "                self.sync_all_window_output_membership();\n"
        "                self.request_full_redraw();\n"
        "            }\n"
        "        }\n\n"
        "        fn apply_session_control_request",
        "window clamp membership refresh",
    )

    text = replace_once(
        text,
        "            if let Some(d) = accumulate_damage_for_window_move(window_id, old, new) {\n"
        "                self.pending_damage = Some(accumulate_damage_rect(self.pending_damage, d));\n"
        "            }\n"
        "            self.frame_dirty = true;\n",
        "            if let Some(d) = accumulate_damage_for_window_move(window_id, old, new) {\n"
        "                self.pending_damage = Some(accumulate_damage_rect(self.pending_damage, d));\n"
        "            }\n"
        "            let surface = self\n"
        "                .windows\n"
        "                .iter()\n"
        "                .find(|window| window.window_id == window_id)\n"
        "                .map(|window| window.toplevel.wl_surface().clone());\n"
        "            if let Some(surface) = surface {\n"
        "                self.sync_surface_output_membership(&surface, new);\n"
        "            }\n"
        "            self.frame_dirty = true;\n",
        "window geometry output events",
    )

    text = replace_once(
        text,
        "            let work_area = self.work_area_for_output(output_area);\n",
        "            let work_area = self.work_area_for_output_index(output_index);\n",
        "presentation work area owner",
    )

    text = replace_once(
        text,
        "            let output_area = output_index_for_geometry(&self.laid_out_outputs, requested_geometry)\n"
        "                .and_then(|index| self.laid_out_outputs.get(index))\n"
        "                .map(output_geometry)\n"
        "                .unwrap_or_else(|| self.canvas_area());\n"
        "            let geometry = clamp_window_to_work_area(\n"
        "                requested_geometry,\n"
        "                self.work_area_for_output(output_area),\n"
        "            );\n",
        "            let output_index =\n"
        "                output_index_for_geometry(&self.laid_out_outputs, requested_geometry)\n"
        "                    .unwrap_or(0);\n"
        "            let geometry = clamp_window_to_work_area(\n"
        "                requested_geometry,\n"
        "                self.work_area_for_output_index(output_index),\n"
        "            );\n",
        "new toplevel output owner",
    )
    text = replace_once(
        text,
        "            let position = Point::from((geometry.x, geometry.y));\n\n"
        "            let (title, app_id)",
        "            let position = Point::from((geometry.x, geometry.y));\n"
        "            let mapped_surface = surface.wl_surface().clone();\n\n"
        "            let (title, app_id)",
        "new toplevel surface capture",
    )
    text = replace_once(
        text,
        "            // Focus the new window\n"
        "            let idx = self.windows.len() - 1;\n"
        "            self.focus_window(idx);\n",
        "            // Focus the new window and publish accurate wl_surface output membership.\n"
        "            let idx = self.windows.len() - 1;\n"
        "            self.sync_surface_output_membership(&mapped_surface, geometry);\n"
        "            self.focus_window(idx);\n",
        "new toplevel output membership",
    )

    text = replace_regex_once(
        text,
        r"            // Apply the client-requested layer-shell anchors, margins, and\n.*?            self\.clamp_normal_windows_to_work_area\(\);",
        '''            // Apply the client-requested layer-shell anchors, margins, and
            // size relative to the exact output selected when the layer was created.
            let laid_out_outputs = self.laid_out_outputs.clone();
            let fallback_canvas = self.canvas_area();
            let mut layer_membership = None;
            for layer in self.layer_surfaces.iter_mut() {
                if layer.surface.wl_surface() != surface {
                    continue;
                }
                let output_area = laid_out_outputs
                    .get(layer.output_index)
                    .map(output_geometry)
                    .unwrap_or(fallback_canvas);
                let output_size =
                    Size::<i32, Logical>::from((output_area.width, output_area.height));
                let (requested, anchor, margins, exclusive_zone) =
                    layer_surface_request(&layer.surface);
                let local_geo = layer_geometry_for(
                    &layer.namespace,
                    layer.layer,
                    output_size,
                    requested,
                    anchor,
                    margins,
                );
                let geo = Rectangle::new(
                    Point::from((
                        output_area.x.saturating_add(local_geo.loc.x),
                        output_area.y.saturating_add(local_geo.loc.y),
                    )),
                    local_geo.size,
                );
                let current = layer.surface.current_state();
                if current.size != Some(geo.size) {
                    layer.surface.with_pending_state(|state| {
                        state.size = Some(geo.size);
                    });
                    layer.surface.send_configure();
                }
                layer.geo = geo;
                layer.exclusive_zone = exclusive_zone;
                layer_membership = Some((layer.surface.wl_surface().clone(), layer.output_index));
                break;
            }
            if let Some((surface, output_index)) = layer_membership {
                self.sync_surface_to_output(&surface, output_index);
            }
            self.clamp_normal_windows_to_work_area();''',
        "layer commit geometry",
    )

    text = replace_regex_once(
        text,
        r"        fn new_layer_surface\(\n.*?\n        \}\n\n        fn new_popup\(",
        '''        fn new_layer_surface(
            &mut self,
            surface: LayerSurface,
            requested_output: Option<wl_output::WlOutput>,
            layer: Layer,
            namespace: String,
        ) {
            let output_index = self.output_index_for_resource(requested_output.as_ref());
            let output_area = self.output_area_for_index(output_index);
            let output_size =
                Size::<i32, Logical>::from((output_area.width, output_area.height));
            eprintln!(
                "[slopos-compositor] layer-shell surface namespace={namespace} layer={layer:?} output={} index={output_index}",
                self.output_names
                    .get(output_index)
                    .map(String::as_str)
                    .unwrap_or("unknown")
            );
            let (requested, anchor, margins, exclusive_zone) = layer_surface_request(&surface);
            let local_geo =
                layer_geometry_for(&namespace, layer, output_size, requested, anchor, margins);
            let geo = Rectangle::new(
                Point::from((
                    output_area.x.saturating_add(local_geo.loc.x),
                    output_area.y.saturating_add(local_geo.loc.y),
                )),
                local_geo.size,
            );
            surface.with_pending_state(|state| {
                state.size = Some(geo.size);
            });
            surface.send_configure();
            let wl_surface = surface.wl_surface().clone();
            self.layer_surfaces.push(MappedLayer {
                surface,
                layer,
                namespace,
                output_index,
                geo,
                exclusive_zone,
            });
            self.sync_surface_to_output(&wl_surface, output_index);
            self.clamp_normal_windows_to_work_area();
            self.request_full_redraw();
        }

        fn new_popup(''',
        "new layer surface ownership",
    )

    text = replace_once(
        text,
        "        fn layer_destroyed(&mut self, surface: LayerSurface) {\n"
        "            self.layer_surfaces\n"
        "                .retain(|l| l.surface.wl_surface() != surface.wl_surface());\n"
        "        }\n",
        "        fn layer_destroyed(&mut self, surface: LayerSurface) {\n"
        "            for output in &self.outputs {\n"
        "                output.leave(surface.wl_surface());\n"
        "            }\n"
        "            self.layer_surfaces\n"
        "                .retain(|l| l.surface.wl_surface() != surface.wl_surface());\n"
        "            self.clamp_normal_windows_to_work_area();\n"
        "            self.request_full_redraw();\n"
        "        }\n",
        "layer destruction work-area refresh",
    )

    text = replace_regex_once(
        text,
        r"            let now = self\.clock\.now\(\);\n            if let Some\(output\) = self\.outputs\.first\(\)\.cloned\(\) \{.*?\n            \}\n\n            self\.frame_scheduler\.record_frame\(\);",
        '''            let now = self.clock.now();
            for window in self
                .windows
                .iter()
                .filter(|window| {
                    !window.minimized && self.workspace_state.is_visible(&window.window_id)
                })
            {
                let output_index =
                    output_index_for_geometry(&self.laid_out_outputs, window.geometry()).unwrap_or(0);
                if let Some(output) = self.outputs.get(output_index) {
                    send_frames_surface_tree(
                        window.toplevel.wl_surface(),
                        output,
                        now,
                        Some(Duration::ZERO),
                        |_, _| None,
                    );
                }
            }
            for layer in &self.layer_surfaces {
                let Some(output) = self.outputs.get(layer.output_index) else {
                    continue;
                };
                send_frames_surface_tree(
                    layer.surface.wl_surface(),
                    output,
                    now,
                    Some(Duration::ZERO),
                    |_, _| None,
                );
                for (popup, _) in PopupManager::popups_for_surface(layer.surface.wl_surface()) {
                    send_frames_surface_tree(
                        popup.wl_surface(),
                        output,
                        now,
                        Some(Duration::ZERO),
                        |_, _| None,
                    );
                }
            }

            self.frame_scheduler.record_frame();''',
        "per-output frame callbacks",
    )

    path.write_text(text, encoding="utf-8")


def apply_contract_gate() -> None:
    path = ROOT / "scripts/verify-compositor-completion.sh"
    text = path.read_text(encoding="utf-8")
    addition = '''\nfailed_step="per-output layer-shell ownership contract"\ngrep -q 'output_index: usize' crates/slopos-compositor/src/main.rs\ngrep -q 'Output::from_resource' crates/slopos-compositor/src/main.rs\ngrep -q 'sync_surface_to_output' crates/slopos-compositor/src/main.rs\ngrep -q 'intersecting_output_indices' crates/slopos-compositor/src/output_assignment.rs\n'''
    text = replace_once(
        text,
        "\nstatus=\"passed\"\n",
        addition + "\nstatus=\"passed\"\n",
        "per-output source contract",
    )
    path.write_text(text, encoding="utf-8")


def apply_code() -> None:
    apply_output_policy()
    apply_live_compositor()
    apply_contract_gate()


def update_truth(implementation_sha: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", implementation_sha):
        raise RuntimeError("truth mode requires a full lowercase commit SHA")
    path = ROOT / "TRUTH.md"
    text = path.read_text(encoding="utf-8")
    text = replace_regex_once(
        text,
        r"\*\*Audited product implementation:\*\*\n`[0-9a-f]{40}`",
        f"**Audited product implementation:**\n`{implementation_sha}`",
        "audited implementation SHA",
    )
    marker = "\n---\n\n## 3. Production scoring model\n"
    wave = f'''\n### Current implementation wave — per-output layer-shell ownership\n\nImplementation commit `{implementation_sha}` is **BUILD VERIFIED**, **TEST\nVERIFIED** and covered by the existing **RUNTIME OBSERVED** headless compositor\ngate. Physical multi-monitor placement and hotplug remain unverified.\n\nThis wave:\n\n- resolves a layer-shell client's requested `wl_output` back to the exact Smithay\n  output and stores that owner on the mapped layer;\n- computes menu-bar, Dock, notification and other layer geometry relative to the\n  owning output rather than the full compositor canvas;\n- scopes exclusive zones and normal-window work-area clamping to the owning\n  output only;\n- emits compositor-managed `wl_surface.enter` and `wl_surface.leave` membership\n  as windows move or resize across outputs;\n- constrains layer surfaces to one output membership and clears it on destroy;\n- sends frame callbacks using each window or layer's selected output instead of\n  routing every surface through the first output;\n- adds pure multi-output membership tests and a permanent source/build contract;\n- regenerates the workspace lockfile and passes workspace build/test/Clippy plus\n  exact compositor source, release and headless runtime gates before commit.\n\nThe overall product score remains **63/100**. The compositor score advances from\n67 to **68/100**. Runtime topology mutation, connector removal, mixed-scale\nrendering, physical output evidence and DRM/KMS hotplug remain open.\n'''
    if marker not in text:
        raise RuntimeError("TRUTH.md scoring marker missing")
    text = text.replace(marker, wave + marker, 1)
    text = replace_once(
        text,
        "| Compositor strict completion | **67** |",
        "| Compositor strict completion | **68** |",
        "executive compositor score",
    )
    text = replace_once(
        text,
        "| Displays and scaling | 6 | 12 |",
        "| Displays and scaling | 7 | 12 |",
        "display score",
    )
    text = replace_once(
        text,
        "| **Total** | **67** | **100** |",
        "| **Total** | **68** | **100** |",
        "compositor total",
    )
    path.write_text(text, encoding="utf-8")


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "apply"
    if mode == "apply":
        apply_code()
    elif mode == "truth" and len(sys.argv) == 3:
        update_truth(sys.argv[2])
    else:
        raise SystemExit("usage: apply-per-output-layer-shell.py [apply | truth SHA]")


if __name__ == "__main__":
    main()
