#!/usr/bin/env python3
"""Apply the final shared compositor fixes once, with guarded source anchors."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates" / "slopos-compositor" / "src"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


def replace_between(text: str, start: str, end: str, replacement: str, label: str) -> str:
    start_index = text.find(start)
    if start_index < 0:
        raise RuntimeError(f"{label}: start anchor not found")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise RuntimeError(f"{label}: end anchor not found")
    if text.find(start, start_index + len(start)) >= 0:
        raise RuntimeError(f"{label}: start anchor is not unique")
    return text[:start_index] + replacement + text[end_index:]


def update_lib() -> None:
    path = SRC / "lib.rs"
    text = path.read_text()
    text = replace_once(
        text,
        "pub mod workspace_focus;\n",
        "pub mod work_area;\npub mod workspace_focus;\n",
        "register work_area module",
    )
    path.write_text(text)


def update_nested() -> None:
    path = SRC / "main.rs"
    text = path.read_text()
    text = replace_once(
        text,
        "    use slopos_compositor::hdr::HdrCapabilities;\n",
        "    use slopos_compositor::hdr::HdrCapabilities;\n"
        "    use slopos_compositor::work_area::{\n"
        "        compute_exclusive_work_area, ExclusiveZoneReservation,\n"
        "    };\n",
        "nested work-area import",
    )

    text = replace_between(
        text,
        "        fn prune_dead_windows(&mut self) {",
        "        /// After Super+workspace switch",
        '''        fn prune_dead_windows(&mut self) {
            let dead_ids: HashSet<String> = self
                .windows
                .iter()
                .filter(|window| !window.toplevel.alive())
                .map(|window| window.window_id.clone())
                .collect();
            if dead_ids.is_empty() {
                return;
            }

            if self
                .interactive_grab
                .as_ref()
                .is_some_and(|grab| dead_ids.contains(&grab.window_id))
            {
                self.cancel_interactive_grab();
            }
            if self
                .last_pointer_press
                .as_ref()
                .is_some_and(|press| dead_ids.contains(&press.window_id))
            {
                self.last_pointer_press = None;
                self.left_button_down = false;
            }

            let mut retained = Vec::with_capacity(self.windows.len().saturating_sub(dead_ids.len()));
            for window in self.windows.drain(..) {
                if dead_ids.contains(&window.window_id) {
                    self.workspace_state.remove_window(&window.window_id);
                    window.foreign.send_closed();
                } else {
                    retained.push(window);
                }
            }
            self.windows = retained;

            if self
                .last_minimized_window_id
                .as_ref()
                .is_some_and(|id| dead_ids.contains(id))
            {
                self.last_minimized_window_id = None;
            }

            self.request_full_redraw();
            self.apply_focus_after_workspace_switch();
        }

''',
        "nested abrupt-disconnect cleanup",
    )

    text = replace_between(
        text,
        "        fn work_area(&self) -> WindowGeometry {",
        "        /// Keep normal windows",
        '''        fn work_area(&self) -> WindowGeometry {
            let reservations = self.layer_surfaces.iter().map(|layer| {
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
            compute_exclusive_work_area(self.output_area(), reservations)
        }

''',
        "nested four-edge work area",
    )

    text = replace_between(
        text,
        "        fn minimize_request(&mut self, surface: ToplevelSurface) {",
        "        fn toplevel_destroyed",
        '''        fn minimize_request(&mut self, surface: ToplevelSurface) {
            let Some(idx) = self
                .windows
                .iter()
                .position(|window| window.toplevel.wl_surface() == surface.wl_surface())
            else {
                return;
            };
            let window_id = self.windows[idx].window_id.clone();
            self.set_window_presentation_state(&surface, WindowPresentationState::Minimized);
            self.windows[idx].minimized = true;
            self.last_minimized_window_id = Some(window_id);
            self.request_full_redraw();
            self.apply_focus_after_workspace_switch();
        }

''',
        "nested minimize transition",
    )
    path.write_text(text)


def update_drm() -> None:
    path = SRC / "session_drm.rs"
    text = path.read_text()
    text = replace_once(
        text,
        "use crate::hdr::HdrCapabilities;\n",
        "use crate::hdr::HdrCapabilities;\n"
        "use crate::work_area::{compute_exclusive_work_area, ExclusiveZoneReservation};\n",
        "DRM work-area import",
    )

    text = replace_between(
        text,
        "    fn prune_dead_windows(&mut self) {",
        "    /// Window ids that should present",
        '''    fn prune_dead_windows(&mut self) {
        let dead_ids: std::collections::HashSet<String> = self
            .windows
            .iter()
            .filter(|window| !window.toplevel.alive())
            .map(|window| window.window_id.clone())
            .collect();
        if dead_ids.is_empty() {
            return;
        }

        if self
            .interactive_grab
            .as_ref()
            .is_some_and(|grab| dead_ids.contains(&grab.window_id))
        {
            self.cancel_interactive_grab();
        }
        if self
            .last_pointer_press
            .as_ref()
            .is_some_and(|press| dead_ids.contains(&press.window_id))
        {
            self.last_pointer_press = None;
            self.left_button_down = false;
        }

        let mut retained = Vec::with_capacity(self.windows.len().saturating_sub(dead_ids.len()));
        for window in self.windows.drain(..) {
            if dead_ids.contains(&window.window_id) {
                self.workspace_state.remove_window(&window.window_id);
                window.foreign.send_closed();
            } else {
                retained.push(window);
            }
        }
        self.windows = retained;

        if self
            .last_minimized_window_id
            .as_ref()
            .is_some_and(|id| dead_ids.contains(id))
        {
            self.last_minimized_window_id = None;
        }

        self.request_full_redraw();
        self.apply_focus_after_workspace_switch();
    }

''',
        "DRM abrupt-disconnect cleanup",
    )

    text = replace_between(
        text,
        "    fn work_area(&self) -> WindowGeometry {",
        "    /// Keep normal windows",
        '''    fn work_area(&self) -> WindowGeometry {
        let reservations = self.layer_surfaces.iter().map(|layer| {
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
        compute_exclusive_work_area(self.output_area(), reservations)
    }

''',
        "DRM four-edge work area",
    )

    text = replace_between(
        text,
        "    fn minimize_request(&mut self, surface: ToplevelSurface) {",
        "    fn toplevel_destroyed",
        '''    fn minimize_request(&mut self, surface: ToplevelSurface) {
        let Some(idx) = self
            .windows
            .iter()
            .position(|window| window.toplevel.wl_surface() == surface.wl_surface())
        else {
            return;
        };
        let window_id = self.windows[idx].window_id.clone();
        self.set_window_presentation_state(&surface, WindowPresentationState::Minimized);
        self.windows[idx].minimized = true;
        self.last_minimized_window_id = Some(window_id);
        self.request_full_redraw();
        self.apply_focus_after_workspace_switch();
    }

''',
        "DRM minimize transition",
    )
    path.write_text(text)


if __name__ == "__main__":
    update_lib()
    update_nested()
    update_drm()
    print("Applied compositor backend finalization patch")
