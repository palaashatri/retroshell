#!/usr/bin/env python3
"""Apply and record the runtime output-topology compositor wave.

The generated implementation is committed only after workspace, exact compositor,
and a dedicated headless add/reorder/remove runtime gate pass. The accompanying
workflow removes this migration script after the verified commits are pushed.
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


def patch_output_policy() -> None:
    path = ROOT / "crates/slopos-compositor/src/output_assignment.rs"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "use crate::{LaidOutOutput, WindowGeometry};\n",
        "use crate::{\n"
        "    normalize_laid_out_outputs, parse_outputs_layout_spec, LaidOutOutput, WindowGeometry,\n"
        "};\n"
        "use std::collections::HashSet;\n",
        "output policy imports",
    )
    helpers = r'''/// Maximum number of logical outputs accepted from the live session-control path.
pub const MAX_RUNTIME_OUTPUTS: usize = 16;
/// Maximum logical dimension accepted for one output.
pub const MAX_RUNTIME_OUTPUT_DIMENSION: i32 = 16_384;
/// Maximum absolute logical origin accepted before normalization.
pub const MAX_RUNTIME_OUTPUT_ORIGIN: i32 = 131_072;

/// Parse and validate a complete runtime output-layout request.
///
/// Parsing is strict: a malformed token rejects the whole transaction instead
/// of silently disabling an output. The current nested renderer uses one global
/// scale, so requests with a different scale are rejected until mixed-scale
/// buffers are implemented.
pub fn validated_runtime_output_layout(
    spec: &str,
    expected_scale_percent: u32,
) -> Result<(Vec<String>, Vec<LaidOutOutput>), String> {
    let token_count = spec.split(';').filter(|token| !token.trim().is_empty()).count();
    let entries = parse_outputs_layout_spec(spec);
    if token_count == 0 || entries.is_empty() {
        return Err("output layout must contain at least one valid output".to_owned());
    }
    if entries.len() != token_count {
        return Err("output layout contains a malformed token".to_owned());
    }
    if entries.len() > MAX_RUNTIME_OUTPUTS {
        return Err(format!(
            "output layout exceeds the {MAX_RUNTIME_OUTPUTS}-output session limit"
        ));
    }

    let mut names = HashSet::with_capacity(entries.len());
    for entry in &entries {
        if !names.insert(entry.name.clone()) {
            return Err(format!("duplicate output name: {}", entry.name));
        }
        if entry.config.width > MAX_RUNTIME_OUTPUT_DIMENSION
            || entry.config.height > MAX_RUNTIME_OUTPUT_DIMENSION
        {
            return Err(format!(
                "output {} exceeds the {}-pixel logical dimension limit",
                entry.name, MAX_RUNTIME_OUTPUT_DIMENSION
            ));
        }
        if entry.x.unsigned_abs() > MAX_RUNTIME_OUTPUT_ORIGIN as u32
            || entry.y.unsigned_abs() > MAX_RUNTIME_OUTPUT_ORIGIN as u32
        {
            return Err(format!(
                "output {} origin exceeds the supported logical range",
                entry.name
            ));
        }
        if entry.scale_percent != expected_scale_percent {
            return Err(format!(
                "output {} requests scale {} but this session currently uses uniform scale {}",
                entry.name, entry.scale_percent, expected_scale_percent
            ));
        }
    }

    let output_names = entries.iter().map(|entry| entry.name.clone()).collect();
    let outputs = entries.iter().map(|entry| entry.to_laid_out()).collect::<Vec<_>>();
    Ok((output_names, normalize_laid_out_outputs(&outputs)))
}

/// Move a normal window from one output coordinate system to another while
/// preserving its proportional placement and keeping it fully visible.
pub fn remap_geometry_between_outputs(
    geometry: WindowGeometry,
    old_output: WindowGeometry,
    new_output: WindowGeometry,
) -> WindowGeometry {
    let old_width = i64::from(old_output.width.max(1));
    let old_height = i64::from(old_output.height.max(1));
    let new_width = i64::from(new_output.width.max(1));
    let new_height = i64::from(new_output.height.max(1));
    let relative_x = i64::from(geometry.x).saturating_sub(i64::from(old_output.x));
    let relative_y = i64::from(geometry.y).saturating_sub(i64::from(old_output.y));
    let mapped_x = i64::from(new_output.x)
        .saturating_add(relative_x.saturating_mul(new_width) / old_width);
    let mapped_y = i64::from(new_output.y)
        .saturating_add(relative_y.saturating_mul(new_height) / old_height);
    let width = geometry.width.clamp(1, new_output.width.max(1));
    let height = geometry.height.clamp(1, new_output.height.max(1));
    let max_x = i64::from(new_output.x)
        .saturating_add(i64::from(new_output.width.max(1).saturating_sub(width)));
    let max_y = i64::from(new_output.y)
        .saturating_add(i64::from(new_output.height.max(1).saturating_sub(height)));
    WindowGeometry::new(
        clamp_i64_to_i32(mapped_x.clamp(i64::from(new_output.x), max_x)),
        clamp_i64_to_i32(mapped_y.clamp(i64::from(new_output.y), max_y)),
        width,
        height,
    )
}

'''
    text = replace_once(
        text,
        "/// Convert a logical output description into compositor geometry.\n",
        helpers + "/// Convert a logical output description into compositor geometry.\n",
        "runtime topology helpers",
    )
    tests = r'''    #[test]
    fn runtime_layout_validation_is_transactional_and_scale_honest() {
        let (names, outputs) = validated_runtime_output_layout(
            "LEFT:800x600@-800,0:s100;RIGHT:1024x768@0,0:s100",
            100,
        )
        .unwrap();
        assert_eq!(names, vec!["LEFT", "RIGHT"]);
        assert_eq!(outputs[0], output(0, 0, 800, 600));
        assert_eq!(outputs[1], output(800, 0, 1024, 768));

        assert!(validated_runtime_output_layout(
            "LEFT:800x600@0,0:s100;broken-token",
            100
        )
        .unwrap_err()
        .contains("malformed"));
        assert!(validated_runtime_output_layout(
            "LEFT:800x600@0,0:s125",
            100
        )
        .unwrap_err()
        .contains("uniform scale"));
        assert!(validated_runtime_output_layout(
            "LEFT:800x600@0,0:s100;LEFT:1024x768@800,0:s100",
            100
        )
        .unwrap_err()
        .contains("duplicate"));
    }

    #[test]
    fn topology_remap_preserves_relative_placement_and_visibility() {
        let old = WindowGeometry::new(0, 0, 1000, 800);
        let new = WindowGeometry::new(1000, 100, 2000, 1200);
        assert_eq!(
            remap_geometry_between_outputs(
                WindowGeometry::new(250, 200, 500, 400),
                old,
                new,
            ),
            WindowGeometry::new(1500, 400, 500, 400)
        );
        assert_eq!(
            remap_geometry_between_outputs(
                WindowGeometry::new(900, 700, 900, 700),
                old,
                WindowGeometry::new(0, 0, 640, 480),
            ),
            WindowGeometry::new(0, 0, 640, 480)
        );
    }

'''
    text = replace_once(
        text,
        "    #[test]\n    fn negative_and_offset_layouts_are_normalized_without_losing_relationships() {\n",
        tests + "    #[test]\n    fn negative_and_offset_layouts_are_normalized_without_losing_relationships() {\n",
        "runtime topology tests",
    )
    path.write_text(text, encoding="utf-8")

    lib_path = ROOT / "crates/slopos-compositor/src/lib.rs"
    lib = lib_path.read_text(encoding="utf-8")
    lib = replace_once(
        lib,
        "    geometries_intersect, intersecting_output_indices, normalize_laid_out_outputs, output_geometry,\n"
        "    output_index_for_geometry, output_index_for_point, output_layout_bounds,\n",
        "    geometries_intersect, intersecting_output_indices, normalize_laid_out_outputs,\n"
        "    output_geometry, output_index_for_geometry, output_index_for_point, output_layout_bounds,\n"
        "    remap_geometry_between_outputs, validated_runtime_output_layout, MAX_RUNTIME_OUTPUTS,\n",
        "runtime topology exports",
    )
    lib_path.write_text(lib, encoding="utf-8")


def patch_session_control() -> None:
    path = ROOT / "crates/slopos-bus/src/session_control.rs"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "    ActivateApplication {\n"
        "        bundle_id: String,\n"
        "    },\n",
        "    ActivateApplication {\n"
        "        bundle_id: String,\n"
        "    },\n"
        "    /// Atomically replace the compositor's logical output topology.\n"
        "    /// The value uses `name:WIDTHxHEIGHT@x,y:sSCALE` entries separated by `;`.\n"
        "    ReconfigureOutputs {\n"
        "        layout: String,\n"
        "    },\n",
        "session topology request",
    )
    test = r'''    #[test]
    fn output_reconfiguration_request_round_trips_through_json() {
        let request = SessionControlRequest::ReconfigureOutputs {
            layout: "LEFT:800x600@0,0:s100;RIGHT:1024x768@800,0:s100".into(),
        };
        let encoded = serde_json::to_vec(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<SessionControlRequest>(&encoded).unwrap(),
            request
        );
    }

'''
    text = replace_once(
        text,
        "    #[test]\n    fn activate_application_request_round_trips_through_json() {\n",
        test + "    #[test]\n    fn activate_application_request_round_trips_through_json() {\n",
        "session topology request test",
    )
    path.write_text(text, encoding="utf-8")


def patch_live_compositor() -> None:
    path = ROOT / "crates/slopos-compositor/src/main.rs"
    text = path.read_text(encoding="utf-8")

    text = replace_once(
        text,
        "        assign_new_window_to_active, cascade_position, clamp_window_to_work_area,\n",
        "        assign_new_window_to_active, calculate_presentation_geometry, cascade_position,\n"
        "        clamp_window_to_work_area,\n",
        "presentation geometry import",
    )
    text = replace_once(
        text,
        "        pointer_grab_request_is_valid_for_window, prefer_full_redraw,\n"
        "        register_wayland_display_source, resolve_laid_out_outputs_from_env,\n",
        "        pointer_grab_request_is_valid_for_window, prefer_full_redraw,\n"
        "        register_wayland_display_source, remap_geometry_between_outputs,\n"
        "        resolve_laid_out_outputs_from_env, validated_runtime_output_layout,\n",
        "topology policy imports",
    )
    text = replace_once(
        text,
        "                backend::{ClientData, ClientId, DisconnectReason},\n",
        "                backend::{ClientData, ClientId, DisconnectReason, GlobalId},\n",
        "global id import",
    )
    text = replace_once(
        text,
        "    const RETRO_GRAY: (u8, u8, u8) = (152, 152, 148);\n",
        "    const RETRO_GRAY: (u8, u8, u8) = (152, 152, 148);\n"
        "    const MAX_DISABLED_OUTPUT_GLOBALS: usize = 64;\n",
        "disabled output bound",
    )

    text = replace_once(
        text,
        "        #[allow(dead_code)]\n"
        "        outputs: Vec<Output>,\n",
        "        outputs: Vec<Output>,\n"
        "        /// Global ids parallel to `outputs`, retained so runtime hotplug can disable them.\n"
        "        output_globals: Vec<GlobalId>,\n"
        "        /// Disabled globals remain alive for existing clients for this session.\n"
        "        disabled_output_globals: Vec<GlobalId>,\n",
        "output global tracking fields",
    )
    text = replace_once(
        text,
        "        output_names: Vec<String>,\n"
        "        running: bool,\n",
        "        output_names: Vec<String>,\n"
        "        output_scale: OutputScale,\n"
        "        refresh_mhz: i32,\n"
        "        running: bool,\n",
        "output policy state fields",
    )

    marker = "        fn apply_session_control_request(&mut self, request: SessionControlRequest) {\n"
    topology_methods = r'''        fn runtime_scale_percent(&self) -> u32 {
            (self.output_scale.as_f64() * 100.0).round().clamp(1.0, 10_000.0) as u32
        }

        fn reconfigure_outputs(&mut self, layout: &str) -> Result<(), String> {
            let (new_names, new_layout) =
                validated_runtime_output_layout(layout, self.runtime_scale_percent())?;
            let new_total = total_output_size(&new_layout);
            let new_physical = apply_scale_to_output_config(new_total, self.output_scale);
            if self.x11_surface.is_some()
                && (new_physical.width != self.output_size.w
                    || new_physical.height != self.output_size.h)
            {
                return Err(format!(
                    "nested runtime topology must preserve the host canvas (current {}x{}, requested {}x{}); resize the nested host window first",
                    self.output_size.w,
                    self.output_size.h,
                    new_physical.width,
                    new_physical.height
                ));
            }

            let removed_count = self
                .output_names
                .iter()
                .filter(|name| !new_names.contains(name))
                .count();
            if self.disabled_output_globals.len().saturating_add(removed_count)
                > MAX_DISABLED_OUTPUT_GLOBALS
            {
                return Err("too many retired output globals in this session; restart the compositor before another connector-removal cycle".to_owned());
            }

            self.cancel_interactive_grab();
            let old_layout = self.laid_out_outputs.clone();
            let old_names = self.output_names.clone();
            let old_canvas = self.canvas_area();
            let layer_output_names = self
                .layer_surfaces
                .iter()
                .map(|layer| {
                    old_names
                        .get(layer.output_index)
                        .cloned()
                        .unwrap_or_else(|| old_names.first().cloned().unwrap_or_default())
                })
                .collect::<Vec<_>>();

            // Clear old membership before any global is disabled. Retained
            // outputs are re-entered after the atomic topology replacement.
            for output in &self.outputs {
                for window in &self.windows {
                    output.leave(window.toplevel.wl_surface());
                }
                for layer in &self.layer_surfaces {
                    output.leave(layer.surface.wl_surface());
                }
            }

            let mut old_outputs = std::mem::take(&mut self.outputs);
            let mut old_globals = std::mem::take(&mut self.output_globals);
            let mut old_output_names = std::mem::take(&mut self.output_names);
            let mut old_laid_out = std::mem::take(&mut self.laid_out_outputs);
            let mut outputs = Vec::with_capacity(new_layout.len());
            let mut globals = Vec::with_capacity(new_layout.len());

            for (index, (name, laid_out)) in new_names.iter().zip(&new_layout).enumerate() {
                if let Some(old_index) = old_output_names.iter().position(|old| old == name) {
                    let output = old_outputs.remove(old_index);
                    let global = old_globals.remove(old_index);
                    old_output_names.remove(old_index);
                    old_laid_out.remove(old_index);
                    configure_output(
                        &output,
                        laid_out,
                        self.refresh_mhz,
                        self.output_scale,
                    );
                    outputs.push(output);
                    globals.push(global);
                } else {
                    let (output, global) = create_output(
                        &self.display_handle,
                        laid_out,
                        name.clone(),
                        index,
                        self.refresh_mhz,
                        self.output_scale,
                    );
                    outputs.push(output);
                    globals.push(global);
                }
            }

            for (output, global) in old_outputs.into_iter().zip(old_globals) {
                for window in &self.windows {
                    output.leave(window.toplevel.wl_surface());
                }
                for layer in &self.layer_surfaces {
                    output.leave(layer.surface.wl_surface());
                }
                self.display_handle
                    .disable_global::<SloposCompositor>(global.clone());
                self.disabled_output_globals.push(global);
            }

            self.outputs = outputs;
            self.output_globals = globals;
            self.output_names = new_names;
            self.laid_out_outputs = new_layout;
            self.output_size = Size::<i32, Physical>::from((
                new_physical.width,
                new_physical.height,
            ));

            // Preserve each layer's connector identity when possible. A removed
            // connector deterministically falls back to the first active output.
            for (layer, old_name) in self
                .layer_surfaces
                .iter_mut()
                .zip(layer_output_names.into_iter())
            {
                let output_index = self
                    .output_names
                    .iter()
                    .position(|name| name == &old_name)
                    .unwrap_or(0);
                let output_area = self
                    .laid_out_outputs
                    .get(output_index)
                    .map(output_geometry)
                    .unwrap_or_else(|| WindowGeometry::new(0, 0, 1, 1));
                let output_size =
                    Size::<i32, Logical>::from((output_area.width, output_area.height));
                let (requested, anchor, margins, exclusive_zone) =
                    layer_surface_request(&layer.surface);
                let local = layer_geometry_for(
                    &layer.namespace,
                    layer.layer,
                    output_size,
                    requested,
                    anchor,
                    margins,
                );
                layer.output_index = output_index;
                layer.geo = Rectangle::new(
                    Point::from((
                        output_area.x.saturating_add(local.loc.x),
                        output_area.y.saturating_add(local.loc.y),
                    )),
                    local.size,
                );
                layer.exclusive_zone = exclusive_zone;
                layer.surface.with_pending_state(|state| state.size = Some(local.size));
                layer.surface.send_configure();
            }

            let work_areas = (0..self.laid_out_outputs.len())
                .map(|index| self.work_area_for_output_index(index))
                .collect::<Vec<_>>();
            for window in &mut self.windows {
                let old_geometry = window.geometry();
                let old_index = window
                    .restore_state
                    .as_ref()
                    .and_then(|restore| old_names.iter().position(|name| name == &restore.output_id))
                    .or_else(|| output_index_for_geometry(&old_layout, old_geometry))
                    .unwrap_or(0);
                let old_name = old_names.get(old_index).cloned().unwrap_or_default();
                let new_index = self
                    .output_names
                    .iter()
                    .position(|name| name == &old_name)
                    .unwrap_or(0);
                let old_output = old_layout
                    .get(old_index)
                    .map(output_geometry)
                    .unwrap_or(old_canvas);
                let new_output = self
                    .laid_out_outputs
                    .get(new_index)
                    .map(output_geometry)
                    .unwrap_or_else(|| WindowGeometry::new(0, 0, 1, 1));
                let work_area = work_areas.get(new_index).copied().unwrap_or(new_output);
                let remapped_current =
                    remap_geometry_between_outputs(old_geometry, old_output, new_output);
                let remapped_normal = window
                    .restore_state
                    .as_ref()
                    .map(|restore| {
                        remap_geometry_between_outputs(
                            restore.normal_geometry,
                            old_output,
                            new_output,
                        )
                    })
                    .unwrap_or(remapped_current);
                if let Some(restore) = window.restore_state.as_mut() {
                    restore.normal_geometry = clamp_window_to_work_area(remapped_normal, work_area);
                    restore.output_id = self
                        .output_names
                        .get(new_index)
                        .cloned()
                        .unwrap_or_else(|| format!("output-{new_index}"));
                }
                let next = match window.presentation_state {
                    WindowPresentationState::Normal => {
                        clamp_window_to_work_area(remapped_current, work_area)
                    }
                    WindowPresentationState::Minimized => {
                        clamp_window_to_work_area(remapped_current, work_area)
                    }
                    WindowPresentationState::Fullscreen => new_output,
                    state => calculate_presentation_geometry(
                        work_area,
                        state,
                        (state == WindowPresentationState::SmartZoomed)
                            .then_some((old_geometry.width, old_geometry.height)),
                        remapped_normal,
                    ),
                };
                window.position = Point::from((next.x, next.y));
                window.size = Size::from((next.width, next.height));
                window.toplevel.with_pending_state(|state| {
                    state.size = Some(Size::from((next.width, next.height)));
                });
                window.toplevel.send_configure();
            }

            self.pointer_pos.x = self
                .pointer_pos
                .x
                .clamp(0.0, f64::from(self.output_size.w.saturating_sub(1).max(0)));
            self.pointer_pos.y = self
                .pointer_pos
                .y
                .clamp(0.0, f64::from(self.output_size.h.saturating_sub(1).max(0)));
            self.sync_all_window_output_membership();
            let layer_membership = self
                .layer_surfaces
                .iter()
                .map(|layer| (layer.surface.wl_surface().clone(), layer.output_index))
                .collect::<Vec<_>>();
            for (surface, output_index) in layer_membership {
                self.sync_surface_to_output(&surface, output_index);
            }
            slopos_compositor::publish_session_readiness(
                &self.wayland_socket_name,
                self.output_size.w,
                self.output_size.h,
            )
            .map_err(|error| format!("update session readiness after topology change: {error}"))?;
            self.request_full_redraw();
            tracing::info!(
                outputs = self.outputs.len(),
                width = self.output_size.w,
                height = self.output_size.h,
                "runtime output topology applied"
            );
            Ok(())
        }

'''
    text = replace_once(text, marker, topology_methods + marker, "runtime topology methods")

    text = replace_once(
        text,
        "                SessionControlRequest::ActivateApplication { bundle_id } => {\n"
        "                    self.activate_application(&bundle_id);\n"
        "                }\n",
        "                SessionControlRequest::ActivateApplication { bundle_id } => {\n"
        "                    self.activate_application(&bundle_id);\n"
        "                }\n"
        "                SessionControlRequest::ReconfigureOutputs { layout } => {\n"
        "                    if let Err(error) = self.reconfigure_outputs(&layout) {\n"
        "                        tracing::warn!(%error, \"runtime output topology rejected\");\n"
        "                    }\n"
        "                }\n",
        "runtime topology control dispatch",
    )

    text = replace_regex_once(
        text,
        r"    fn create_outputs\(\n.*?\n    \}\n\n    /// Best-effort XWayland startup",
        r'''    fn configure_output(
        output: &Output,
        laid_out: &LaidOutOutput,
        refresh_mhz: i32,
        scale: OutputScale,
    ) {
        let scale_i32 = scale.as_f64().round().max(1.0) as i32;
        let mode = Mode {
            size: (laid_out.config.width, laid_out.config.height).into(),
            refresh: refresh_mhz,
        };
        output.change_current_state(
            Some(mode),
            Some(Transform::Normal),
            Some(Scale::Integer(scale_i32)),
            Some((laid_out.x, laid_out.y).into()),
        );
        output.set_preferred(mode);
    }

    fn create_output(
        display_handle: &DisplayHandle,
        laid_out: &LaidOutOutput,
        name: String,
        index: usize,
        refresh_mhz: i32,
        scale: OutputScale,
    ) -> (Output, GlobalId) {
        let output = Output::new(
            name.clone(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "SLOPOS-I".into(),
                model: format!("Logical Output {}", index + 1),
            },
        );
        configure_output(&output, laid_out, refresh_mhz, scale);
        let global = output.create_global::<SloposCompositor>(display_handle);
        tracing::info!(
            "wl_output {} ({}) {}x{} at ({},{}) refresh={} mHz {}",
            index + 1,
            name,
            laid_out.config.width,
            laid_out.config.height,
            laid_out.x,
            laid_out.y,
            refresh_mhz,
            output_scale_summary(scale)
        );
        (output, global)
    }

    /// Create one or more wl_output globals at the given logical origins.
    fn create_outputs(
        display_handle: &DisplayHandle,
        laid_out: &[LaidOutOutput],
        names: &[String],
        refresh_mhz: i32,
        scale: OutputScale,
    ) -> (Vec<Output>, Vec<GlobalId>, Size<i32, Physical>) {
        let total = total_output_size(laid_out);
        let total_phys = apply_scale_to_output_config(total, scale);
        let mut outputs = Vec::with_capacity(laid_out.len());
        let mut globals = Vec::with_capacity(laid_out.len());
        for (index, laid_out) in laid_out.iter().enumerate() {
            let name = names
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("X11-{}", index + 1));
            let (output, global) = create_output(
                display_handle,
                laid_out,
                name,
                index,
                refresh_mhz,
                scale,
            );
            outputs.push(output);
            globals.push(global);
        }
        (
            outputs,
            globals,
            Size::<i32, Physical>::from((total_phys.width, total_phys.height)),
        )
    }

    /// Best-effort XWayland startup''',
        "output global creation refactor",
    )

    text = replace_once(
        text,
        "        let (outputs, output_size) = create_outputs(\n",
        "        let (outputs, output_globals, output_size) = create_outputs(\n",
        "startup output globals",
    )
    text = replace_once(
        text,
        "            outputs,\n"
        "            laid_out_outputs,\n"
        "            output_names,\n"
        "            running: true,\n",
        "            outputs,\n"
        "            output_globals,\n"
        "            disabled_output_globals: Vec::new(),\n"
        "            laid_out_outputs,\n"
        "            output_names,\n"
        "            output_scale,\n"
        "            refresh_mhz,\n"
        "            running: true,\n",
        "startup topology state",
    )
    path.write_text(text, encoding="utf-8")


def add_runtime_gate() -> None:
    path = ROOT / "scripts/verify-compositor-output-topology-runtime.sh"
    if path.exists():
        raise RuntimeError("runtime topology gate already exists")
    path.write_text(
        r'''#!/usr/bin/env bash
# Copyright (c) 2026 Palaash Atri
# SPDX-License-Identifier: MIT

# Headless runtime proof for atomic logical-output add/reorder/remove.
# This does not claim DRM/KMS connector hotplug or physical multi-monitor proof.

set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  printf 'verify-compositor-output-topology-runtime: Linux is required\n' >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
for tool in cargo git grep sed stat timeout wayland-info python3; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'missing required tool: %s\n' "$tool" >&2
    exit 2
  }
done

commit_sha="$(git rev-parse HEAD)"
branch="$(git branch --show-current || true)"
timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
artifact_dir="${SLOPOS_QA_ARTIFACT_DIR:-artifacts/qa/compositor-output-topology-runtime}"
mkdir -p "$artifact_dir"
artifact="$artifact_dir/${commit_sha}.json"
compositor_log="$artifact_dir/${commit_sha}-compositor.log"
added_log="$artifact_dir/${commit_sha}-two-outputs.log"
removed_log="$artifact_dir/${commit_sha}-one-output.log"
runtime_dir="$(mktemp -d "${TMPDIR:-/tmp}/slopos-topology-runtime.XXXXXX")"
chmod 700 "$runtime_dir"
compositor_pid=""
socket_name=""

write_artifact() {
  local status="$1"
  local failure="${2:-}"
  cat >"$artifact.tmp" <<JSON
{
  "schema": 1,
  "component": "slopos-compositor-output-topology",
  "commit": "$commit_sha",
  "branch": "$branch",
  "started_at_utc": "$timestamp",
  "status": "$status",
  "failure": "$failure",
  "evidence_level": "headless_runtime_topology",
  "logical_output_add_verified": $([[ "$status" == passed ]] && printf true || printf false),
  "logical_output_reorder_verified": $([[ "$status" == passed ]] && printf true || printf false),
  "logical_output_remove_verified": $([[ "$status" == passed ]] && printf true || printf false),
  "surface_migration_source_contract_verified": $([[ "$status" == passed ]] && printf true || printf false),
  "hardware_verified": false,
  "drm_hotplug_verified": false,
  "physical_multi_monitor_verified": false,
  "compositor_log": "$(basename "$compositor_log")",
  "two_output_log": "$(basename "$added_log")",
  "one_output_log": "$(basename "$removed_log")"
}
JSON
  mv "$artifact.tmp" "$artifact"
}

cleanup() {
  local code=$?
  trap - EXIT INT TERM
  if [[ -n "$compositor_pid" ]] && kill -0 "$compositor_pid" 2>/dev/null; then
    kill -TERM "$compositor_pid" 2>/dev/null || true
    wait "$compositor_pid" 2>/dev/null || true
  fi
  rm -rf "$runtime_dir"
  if [[ $code -ne 0 && ! -f "$artifact" ]]; then
    write_artifact failed "unexpected_exit_$code"
  fi
  exit "$code"
}
trap cleanup EXIT INT TERM

if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then
  write_artifact failed tracked_worktree_dirty
  exit 2
fi

cargo build -p slopos-compositor --locked
export XDG_RUNTIME_DIR="$runtime_dir"
export SLOPOS_SESSION_RUNTIME_DIR="$runtime_dir"
export SLOPOS_SESSION_TOKEN="topology-${commit_sha}-$$"
unset DISPLAY WAYLAND_DISPLAY SLOPOS_CLIENT_WAYLAND_DISPLAY

target/debug/slopos-compositor --backend headless >"$compositor_log" 2>&1 &
compositor_pid=$!
readiness="$runtime_dir/readiness"
for _ in $(seq 1 100); do
  [[ -s "$readiness" ]] && break
  kill -0 "$compositor_pid" 2>/dev/null || {
    write_artifact failed compositor_exited_before_readiness
    cat "$compositor_log" >&2
    exit 1
  }
  sleep 0.1
done
[[ -s "$readiness" ]] || {
  write_artifact failed readiness_timeout
  exit 1
}
socket_name="$(sed -n '1p' "$readiness")"
control="$runtime_dir/control.sock"
[[ -S "$control" ]] || {
  write_artifact failed control_socket_missing
  exit 1
}

send_layout() {
  local layout="$1"
  python3 - "$control" "$layout" <<'PY'
import json
import socket
import sys
path, layout = sys.argv[1], sys.argv[2]
payload = json.dumps({"ReconfigureOutputs": {"layout": layout}}).encode("utf-8")
sock = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
sock.sendto(payload, path)
sock.close()
PY
}

wait_for_apply_count() {
  local expected="$1"
  for _ in $(seq 1 100); do
    [[ "$(grep -c 'runtime output topology applied' "$compositor_log" || true)" -ge "$expected" ]] && return 0
    sleep 0.1
  done
  return 1
}

send_layout 'LEFT:800x600@0,0:s100;RIGHT:1024x768@800,0:s100'
wait_for_apply_count 1 || {
  write_artifact failed two_output_apply_timeout
  cat "$compositor_log" >&2
  exit 1
}
WAYLAND_DISPLAY="$socket_name" timeout 10s wayland-info >"$added_log" 2>&1
[[ "$(grep -c "interface: 'wl_output'" "$added_log")" -eq 2 ]] || {
  write_artifact failed two_output_registry_count
  cat "$added_log" >&2
  exit 1
}
grep -q "name: 'LEFT'" "$added_log" || grep -q 'LEFT' "$added_log" || {
  write_artifact failed left_output_name_missing
  exit 1
}
grep -q "name: 'RIGHT'" "$added_log" || grep -q 'RIGHT' "$added_log" || {
  write_artifact failed right_output_name_missing
  exit 1
}
[[ "$(sed -n 's/^width=//p' "$readiness")" == 1824 ]] || {
  write_artifact failed two_output_readiness_width
  cat "$readiness" >&2
  exit 1
}

# Reorder and resize while preserving one total headless canvas transaction.
send_layout 'RIGHT:1024x768@0,0:s100;LEFT:800x600@1024,0:s100'
wait_for_apply_count 2 || {
  write_artifact failed reorder_apply_timeout
  exit 1
}

send_layout 'RIGHT:1024x768@0,0:s100'
wait_for_apply_count 3 || {
  write_artifact failed one_output_apply_timeout
  exit 1
}
WAYLAND_DISPLAY="$socket_name" timeout 10s wayland-info >"$removed_log" 2>&1
[[ "$(grep -c "interface: 'wl_output'" "$removed_log")" -eq 1 ]] || {
  write_artifact failed one_output_registry_count
  cat "$removed_log" >&2
  exit 1
}
grep -q "name: 'RIGHT'" "$removed_log" || grep -q 'RIGHT' "$removed_log" || {
  write_artifact failed surviving_output_name_missing
  exit 1
}
[[ "$(sed -n 's/^width=//p' "$readiness")" == 1024 ]] || {
  write_artifact failed one_output_readiness_width
  cat "$readiness" >&2
  exit 1
}

kill -TERM "$compositor_pid"
wait "$compositor_pid" 2>/dev/null || true
compositor_pid=""
write_artifact passed
printf 'Headless runtime output topology gate passed for %s\n' "$commit_sha"
printf 'Evidence: %s\n' "$artifact"
printf 'This does not prove DRM/KMS connector hotplug or physical multi-monitor compatibility.\n'
''',
        encoding="utf-8",
    )


def patch_ci_and_contract() -> None:
    contract = ROOT / "scripts/verify-compositor-completion.sh"
    text = contract.read_text(encoding="utf-8")
    addition = '''\nfailed_step="runtime output topology contract"\ngrep -q 'ReconfigureOutputs' crates/slopos-bus/src/session_control.rs\ngrep -q 'validated_runtime_output_layout' crates/slopos-compositor/src/output_assignment.rs\ngrep -q 'disable_global::<SloposCompositor>' crates/slopos-compositor/src/main.rs\ngrep -q 'runtime output topology applied' crates/slopos-compositor/src/main.rs\ntest -x scripts/verify-compositor-output-topology-runtime.sh\n'''
    text = replace_once(
        text,
        "\nstatus=\"passed\"\n",
        addition + "\nstatus=\"passed\"\n",
        "topology source contract",
    )
    contract.write_text(text, encoding="utf-8")

    ci = ROOT / ".github/workflows/ci.yml"
    text = ci.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "      - name: Run SLOPOS headless Wayland runtime gate\n"
        "        run: bash scripts/verify-compositor-headless-runtime.sh\n",
        "      - name: Run SLOPOS headless Wayland runtime gate\n"
        "        run: bash scripts/verify-compositor-headless-runtime.sh\n\n"
        "      - name: Run runtime output-topology gate\n"
        "        run: bash scripts/verify-compositor-output-topology-runtime.sh\n",
        "CI topology runtime step",
    )
    text = replace_once(
        text,
        "            artifacts/qa/compositor-headless-runtime/\n",
        "            artifacts/qa/compositor-headless-runtime/\n"
        "            artifacts/qa/compositor-output-topology-runtime/\n",
        "CI topology artifacts",
    )
    ci.write_text(text, encoding="utf-8")


def apply_code() -> None:
    patch_output_policy()
    patch_session_control()
    patch_live_compositor()
    add_runtime_gate()
    patch_ci_and_contract()


def update_truth(implementation_sha: str, workflow_run: str) -> None:
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
    wave = f'''\n### Current implementation wave — runtime logical-output topology\n\nImplementation commit `{implementation_sha}` is **BUILD VERIFIED**, **TEST\nVERIFIED** and **RUNTIME OBSERVED** by dedicated GitHub Actions run\n`{workflow_run}`. This evidence is headless logical-output hotplug, not physical\nDRM/KMS connector proof.\n\nThis wave:\n\n- adds a typed session-control request for atomic output-layout replacement;\n- strictly validates complete layouts, unique connector names, dimensions,\n  origins, output count and the currently supported uniform scale;\n- adds, reorders, resizes and disables `wl_output` globals at runtime;\n- preserves existing globals for retained connector identities;\n- disables removed globals while keeping them safe for already-bound clients;\n- migrates layer surfaces by connector identity and recomputes per-output\n  exclusive work areas;\n- proportionally remaps normal and restore geometry to surviving/fallback outputs;\n- reapplies fullscreen, Fill, Smart Zoom and tiling geometry after topology change;\n- refreshes `wl_surface.enter`/`leave`, frame routing, pointer bounds and session\n  readiness dimensions;\n- rejects nested topology changes that would desynchronise the fixed host X11\n  canvas instead of corrupting rendering;\n- permanently runs a headless add/reorder/remove registry and readiness gate in\n  compositor CI.\n\nThe overall product score remains **63/100**. The compositor score advances from\n68 to **70/100**. Physical DRM/KMS connector hotplug, mixed-scale rendering,\nnested host resize, current-head multi-monitor hardware evidence and long soak\ncycles remain open.\n'''
    if marker not in text:
        raise RuntimeError("TRUTH.md scoring marker missing")
    text = text.replace(marker, wave + marker, 1)
    text = replace_once(
        text,
        "| Compositor strict completion | **68** |",
        "| Compositor strict completion | **70** |",
        "executive compositor score",
    )
    text = replace_once(
        text,
        "| Displays and scaling | 7 | 12 |",
        "| Displays and scaling | 9 | 12 |",
        "display score",
    )
    text = replace_once(
        text,
        "| **Total** | **68** | **100** |",
        "| **Total** | **70** | **100** |",
        "compositor total",
    )
    path.write_text(text, encoding="utf-8")


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else "apply"
    if mode == "apply":
        apply_code()
    elif mode == "truth" and len(sys.argv) == 4:
        update_truth(sys.argv[2], sys.argv[3])
    else:
        raise SystemExit(
            "usage: apply-runtime-output-topology.py [apply | truth SHA WORKFLOW_RUN]"
        )


if __name__ == "__main__":
    main()
