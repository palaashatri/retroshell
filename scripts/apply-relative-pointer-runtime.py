#!/usr/bin/env python3
# Copyright (c) 2026 Palaash Atri
# SPDX-License-Identifier: MIT

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one anchor, found {count}: {old[:100]!r}")
    file.write_text(text.replace(old, new, 1), encoding="utf-8")


MAIN = "crates/slopos-compositor/src/main.rs"
DRM = "crates/slopos-compositor/src/session_drm.rs"
RUNTIME = "scripts/verify-compositor-headless-runtime.sh"

# Nested/X11 compositor: advertise relative-pointer-v1 and keep the global alive.
replace_once(
    MAIN,
    """        delegate_compositor, delegate_foreign_toplevel_list, delegate_layer_shell, delegate_output,\n        delegate_seat, delegate_shm, delegate_xdg_shell,\n""",
    """        delegate_compositor, delegate_foreign_toplevel_list, delegate_layer_shell, delegate_output,\n        delegate_relative_pointer, delegate_seat, delegate_shm, delegate_xdg_shell,\n""",
)
replace_once(
    MAIN,
    """            output::{OutputHandler, OutputManagerState},\n""",
    """            output::{OutputHandler, OutputManagerState},\n            relative_pointer::RelativePointerManagerState,\n""",
)
replace_once(
    MAIN,
    """        seat_state: SeatState<SloposCompositor>,\n        xdg_shell_state: XdgShellState,\n""",
    """        seat_state: SeatState<SloposCompositor>,\n        _relative_pointer_state: RelativePointerManagerState,\n        xdg_shell_state: XdgShellState,\n""",
)
replace_once(
    MAIN,
    """    delegate_seat!(SloposCompositor);\n""",
    """    delegate_seat!(SloposCompositor);\n    delegate_relative_pointer!(SloposCompositor);\n""",
)
replace_once(
    MAIN,
    """        let mut seat_state = SeatState::new();\n        let xdg_shell_state = XdgShellState::new::<SloposCompositor>(&display_handle);\n""",
    """        let mut seat_state = SeatState::new();\n        let relative_pointer_state =\n            RelativePointerManagerState::new::<SloposCompositor>(&display_handle);\n        let xdg_shell_state = XdgShellState::new::<SloposCompositor>(&display_handle);\n""",
)
replace_once(
    MAIN,
    """            seat_state,\n            xdg_shell_state,\n""",
    """            seat_state,\n            _relative_pointer_state: relative_pointer_state,\n            xdg_shell_state,\n""",
)

old_nested_motion = """    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)\n    where\n        E: PointerMotionAbsoluteEvent<X11Input>,\n    {\n        let logical = Size::<i32, Logical>::from((state.output_size.w, state.output_size.h));\n        let pos = ev.position_transformed(logical);\n        state.pointer_pos = pos;\n        state.request_redraw();\n\n        // Hit-test layer chrome, popup trees, then ordinary toplevels.\n        let focus = state.surface_under(pos);\n\n        let serial = state.next_serial();\n        let time = ev.time_msec();\n\n        if let Some(ptr) = state.seat.get_pointer() {\n            ptr.motion(\n                state,\n                focus,\n                &MotionEvent {\n                    location: pos,\n                    serial,\n                    time,\n                },\n            );\n            ptr.frame(state);\n        }\n    }\n"""
new_nested_motion = """    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)\n    where\n        E: PointerMotionAbsoluteEvent<X11Input>,\n    {\n        let logical = Size::<i32, Logical>::from((state.output_size.w, state.output_size.h));\n        let previous = state.pointer_pos;\n        let pos = ev.position_transformed(logical);\n        state.pointer_pos = pos;\n        state.request_redraw();\n\n        // Hit-test layer chrome, popup trees, then ordinary toplevels.\n        let focus = state.surface_under(pos);\n\n        let serial = state.next_serial();\n        let time = ev.time_msec();\n        let delta = Point::from((pos.x - previous.x, pos.y - previous.y));\n\n        if let Some(ptr) = state.seat.get_pointer() {\n            // Smithay's X11 backend reports absolute pointer coordinates only.\n            // Derive a relative delta from consecutive samples; raw/unaccelerated\n            // motion is unavailable here, so the same delta is reported for both.\n            ptr.relative_motion(\n                state,\n                focus.clone(),\n                &RelativeMotionEvent {\n                    delta,\n                    delta_unaccel: delta,\n                    utime: u64::from(time) * 1_000,\n                },\n            );\n            ptr.motion(\n                state,\n                focus,\n                &MotionEvent {\n                    location: pos,\n                    serial,\n                    time,\n                },\n            );\n            ptr.frame(state);\n        }\n    }\n"""
replace_once(MAIN, old_nested_motion, new_nested_motion)

# Bare-metal DRM/libinput compositor: advertise the same protocol and forward
# both accelerated and raw libinput deltas to clients.
replace_once(
    DRM,
    """use smithay::wayland::output::{OutputHandler, OutputManagerState};\n""",
    """use smithay::wayland::output::{OutputHandler, OutputManagerState};\nuse smithay::wayland::relative_pointer::RelativePointerManagerState;\n""",
)
replace_once(
    DRM,
    """    delegate_layer_shell, delegate_output, delegate_primary_selection, delegate_seat,\n    delegate_session_lock, delegate_shm, delegate_xdg_shell,\n""",
    """    delegate_layer_shell, delegate_output, delegate_primary_selection, delegate_relative_pointer,\n    delegate_seat, delegate_session_lock, delegate_shm, delegate_xdg_shell,\n""",
)
replace_once(
    DRM,
    """    seat_state: SeatState<Self>,\n    seat: Seat<Self>,\n""",
    """    seat_state: SeatState<Self>,\n    _relative_pointer_state: RelativePointerManagerState,\n    seat: Seat<Self>,\n""",
)
replace_once(
    DRM,
    """delegate_seat!(DrmSessionState);\n""",
    """delegate_seat!(DrmSessionState);\ndelegate_relative_pointer!(DrmSessionState);\n""",
)
replace_once(
    DRM,
    """    let mut seat_state = SeatState::new();\n    let xdg_shell_state = XdgShellState::new::<DrmSessionState>(&dh);\n""",
    """    let mut seat_state = SeatState::new();\n    let relative_pointer_state = RelativePointerManagerState::new::<DrmSessionState>(&dh);\n    let xdg_shell_state = XdgShellState::new::<DrmSessionState>(&dh);\n""",
)
replace_once(
    DRM,
    """        seat_state,\n        seat,\n""",
    """        seat_state,\n        _relative_pointer_state: relative_pointer_state,\n        seat,\n""",
)

old_drm_motion = """            InputEvent::PointerMotion { event } => {\n                // Relative motion (real mice): accumulate and clamp to output.\n                let (dx, dy) = (event.delta_x(), event.delta_y());\n                let x = (self.pointer_location.x + dx).clamp(0.0, self.output_size.0 as f64 - 1.0);\n                let y = (self.pointer_location.y + dy).clamp(0.0, self.output_size.1 as f64 - 1.0);\n                self.pointer_location = Point::from((x, y));\n                self.forward_pointer_motion(event.time_msec());\n                self.request_redraw();\n            }\n"""
new_drm_motion = """            InputEvent::PointerMotion { event } => {\n                // Relative motion (real mice): preserve both accelerated and raw\n                // libinput deltas for zwp_relative_pointer_v1, then update the\n                // compositor-space cursor position for ordinary wl_pointer.\n                let (dx, dy) = (event.delta_x(), event.delta_y());\n                let delta = Point::from((dx, dy));\n                let delta_unaccel =\n                    Point::from((event.delta_x_unaccel(), event.delta_y_unaccel()));\n                let x = (self.pointer_location.x + dx).clamp(0.0, self.output_size.0 as f64 - 1.0);\n                let y = (self.pointer_location.y + dy).clamp(0.0, self.output_size.1 as f64 - 1.0);\n                self.pointer_location = Point::from((x, y));\n\n                let focus = if self.locked {\n                    self.active_lock_surface()\n                } else {\n                    self.surface_under(self.pointer_location)\n                };\n                if let Some(pointer) = self.seat.get_pointer() {\n                    pointer.relative_motion(\n                        self,\n                        focus,\n                        &RelativeMotionEvent {\n                            delta,\n                            delta_unaccel,\n                            utime: event.time(),\n                        },\n                    );\n                }\n                self.forward_pointer_motion(event.time_msec());\n                self.request_redraw();\n            }\n"""
replace_once(DRM, old_drm_motion, new_drm_motion)

# The permanent headless runtime gate must fail if the protocol disappears.
replace_once(
    RUNTIME,
    """for required_global in wl_compositor wl_shm wl_seat xdg_wm_base; do\n""",
    """for required_global in wl_compositor wl_shm wl_seat xdg_wm_base zwp_relative_pointer_manager_v1; do\n""",
)

print("relative pointer runtime migration applied")
