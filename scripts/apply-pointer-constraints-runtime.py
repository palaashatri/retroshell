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
LIB = "crates/slopos-compositor/src/lib.rs"
RUNTIME = "scripts/verify-compositor-headless-runtime.sh"
POLICY = Path("crates/slopos-compositor/src/pointer_policy.rs")
CLIENT = Path("crates/slopos-compositor/examples/headless_pointer_constraints_client.rs")

POLICY.write_text(
    '''// Copyright (c) 2026 Palaash Atri
// SPDX-License-Identifier: MIT

//! Backend-independent pointer constraint motion policy.
//!
//! Smithay owns the Wayland object lifecycle. This module owns the small,
//! deterministic movement decision used by both SLOPOS compositor backends so
//! locked/confined semantics can be tested without a live input device.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerConstraintMotion {
    Free,
    Locked,
    Confined,
}

/// Resolve a proposed two-axis pointer delta.
///
/// `allow_x` and `allow_y` represent both surface-boundary and optional
/// confinement-region checks performed by the backend in surface-local space.
pub fn resolve_pointer_delta(
    mode: PointerConstraintMotion,
    delta: (f64, f64),
    allow_x: bool,
    allow_y: bool,
) -> (f64, f64) {
    match mode {
        PointerConstraintMotion::Free => delta,
        PointerConstraintMotion::Locked => (0.0, 0.0),
        PointerConstraintMotion::Confined => (
            if allow_x { delta.0 } else { 0.0 },
            if allow_y { delta.1 } else { 0.0 },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_pointer_keeps_both_axes() {
        assert_eq!(
            resolve_pointer_delta(PointerConstraintMotion::Free, (12.5, -4.0), false, false),
            (12.5, -4.0)
        );
    }

    #[test]
    fn locked_pointer_discards_motion() {
        assert_eq!(
            resolve_pointer_delta(PointerConstraintMotion::Locked, (12.5, -4.0), true, true),
            (0.0, 0.0)
        );
    }

    #[test]
    fn confined_pointer_keeps_allowed_axes_independently() {
        assert_eq!(
            resolve_pointer_delta(PointerConstraintMotion::Confined, (12.5, -4.0), false, true),
            (0.0, -4.0)
        );
        assert_eq!(
            resolve_pointer_delta(PointerConstraintMotion::Confined, (12.5, -4.0), true, false),
            (12.5, 0.0)
        );
    }

    #[test]
    fn confined_pointer_stops_when_neither_axis_is_valid() {
        assert_eq!(
            resolve_pointer_delta(PointerConstraintMotion::Confined, (12.5, -4.0), false, false),
            (0.0, 0.0)
        );
    }
}
''',
    encoding="utf-8",
)

CLIENT.write_text(
    '''// Copyright (c) 2026 Palaash Atri
// SPDX-License-Identifier: MIT

//! Headless protocol lifecycle client for pointer-constraints-unstable-v1.
//!
//! This deliberately does not claim pointer movement enforcement: the headless
//! compositor has no physical input device. It proves that the exact compositor
//! accepts, creates, commits and destroys both lock and confinement objects.

use std::error::Error;

use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_compositor, wl_pointer, wl_registry, wl_seat, wl_surface};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::wp::pointer_constraints::zv1::client::{
    zwp_confined_pointer_v1, zwp_locked_pointer_v1, zwp_pointer_constraints_v1,
};

#[derive(Default)]
struct State;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
    }
}

wayland_client::delegate_noop!(State: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(State: ignore wl_surface::WlSurface);
wayland_client::delegate_noop!(State: ignore wl_seat::WlSeat);
wayland_client::delegate_noop!(State: ignore wl_pointer::WlPointer);
wayland_client::delegate_noop!(State: ignore zwp_pointer_constraints_v1::ZwpPointerConstraintsV1);
wayland_client::delegate_noop!(State: ignore zwp_locked_pointer_v1::ZwpLockedPointerV1);
wayland_client::delegate_noop!(State: ignore zwp_confined_pointer_v1::ZwpConfinedPointerV1);

fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<State>(&connection)?;
    let qh = event_queue.handle();
    let mut state = State;

    let compositor = globals.bind::<wl_compositor::WlCompositor, _, _>(&qh, 1..=6, ())?;
    let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=9, ())?;
    let constraints = globals.bind::<zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, _, _>(
        &qh,
        1..=1,
        (),
    )?;
    let pointer = seat.get_pointer(&qh, ());
    let surface = compositor.create_surface(&qh, ());

    let locked = constraints.lock_pointer(
        &surface,
        &pointer,
        None,
        zwp_pointer_constraints_v1::Lifetime::Persistent,
        &qh,
        (),
    );
    surface.commit();
    event_queue.roundtrip(&mut state)?;
    println!("SLOPOS_POINTER_LOCK_REQUEST_ACCEPTED persistent=1");
    locked.destroy();
    event_queue.roundtrip(&mut state)?;

    let confined = constraints.confine_pointer(
        &surface,
        &pointer,
        None,
        zwp_pointer_constraints_v1::Lifetime::Persistent,
        &qh,
        (),
    );
    surface.commit();
    event_queue.roundtrip(&mut state)?;
    println!("SLOPOS_POINTER_CONFINE_REQUEST_ACCEPTED persistent=1");
    confined.destroy();
    event_queue.roundtrip(&mut state)?;

    constraints.destroy();
    pointer.release();
    surface.destroy();
    event_queue.roundtrip(&mut state)?;
    println!("SLOPOS_POINTER_CONSTRAINTS_OK");
    Ok(())
}
''',
    encoding="utf-8",
)

replace_once(LIB, "pub mod perf_budget;\n", "pub mod perf_budget;\npub mod pointer_policy;\n")

# Nested compositor protocol wiring.
replace_once(
    MAIN,
    """        delegate_compositor, delegate_foreign_toplevel_list, delegate_layer_shell, delegate_output,\n        delegate_relative_pointer, delegate_seat, delegate_shm, delegate_xdg_shell,\n""",
    """        delegate_compositor, delegate_foreign_toplevel_list, delegate_layer_shell, delegate_output,\n        delegate_pointer_constraints, delegate_relative_pointer, delegate_seat, delegate_shm,\n        delegate_xdg_shell,\n""",
)
replace_once(
    MAIN,
    """                PointerGrab, PointerInnerHandle, RelativeMotionEvent,\n""",
    """                PointerGrab, PointerHandle, PointerInnerHandle, RelativeMotionEvent,\n""",
)
replace_once(
    MAIN,
    """            output::{OutputHandler, OutputManagerState},\n            relative_pointer::RelativePointerManagerState,\n""",
    """            output::{OutputHandler, OutputManagerState},\n            pointer_constraints::{\n                with_pointer_constraint, PointerConstraint, PointerConstraintsHandler,\n                PointerConstraintsState,\n            },\n            relative_pointer::RelativePointerManagerState,\n""",
)
replace_once(
    MAIN,
    """        PlaceholderPresentStats, ResizeEdges, TextInputCapability, WindowGeometry,\n""",
    """        PlaceholderPresentStats, PointerConstraintMotion, ResizeEdges, TextInputCapability,\n        WindowGeometry,\n""",
)
replace_once(
    MAIN,
    """        seat_state: SeatState<SloposCompositor>,\n        _relative_pointer_state: RelativePointerManagerState,\n        xdg_shell_state: XdgShellState,\n""",
    """        seat_state: SeatState<SloposCompositor>,\n        _relative_pointer_state: RelativePointerManagerState,\n        _pointer_constraints_state: PointerConstraintsState,\n        xdg_shell_state: XdgShellState,\n""",
)
replace_once(
    MAIN,
    """        // Current pointer position (logical)\n        pointer_pos: Point<f64, Logical>,\n""",
    """        // Current compositor-visible pointer position (logical).\n        pointer_pos: Point<f64, Logical>,\n        /// Last raw absolute sample from the nested X11 backend. Kept separate\n        /// so relative-pointer deltas continue while an app locks the visible cursor.\n        last_backend_pointer_pos: Option<Point<f64, Logical>>,\n""",
)
replace_once(
    MAIN,
    """        let relative_pointer_state =\n            RelativePointerManagerState::new::<SloposCompositor>(&display_handle);\n        let xdg_shell_state = XdgShellState::new::<SloposCompositor>(&display_handle);\n""",
    """        let relative_pointer_state =\n            RelativePointerManagerState::new::<SloposCompositor>(&display_handle);\n        let pointer_constraints_state =\n            PointerConstraintsState::new::<SloposCompositor>(&display_handle);\n        let xdg_shell_state = XdgShellState::new::<SloposCompositor>(&display_handle);\n""",
)
replace_once(
    MAIN,
    """            seat_state,\n            _relative_pointer_state: relative_pointer_state,\n            xdg_shell_state,\n""",
    """            seat_state,\n            _relative_pointer_state: relative_pointer_state,\n            _pointer_constraints_state: pointer_constraints_state,\n            xdg_shell_state,\n""",
)
replace_once(
    MAIN,
    """            pointer_pos: Point::from((0.0_f64, 0.0_f64)),\n            cursor_status: CursorImageStatus::default_named(),\n""",
    """            pointer_pos: Point::from((0.0_f64, 0.0_f64)),\n            last_backend_pointer_pos: None,\n            cursor_status: CursorImageStatus::default_named(),\n""",
)
replace_once(
    MAIN,
    """    delegate_seat!(SloposCompositor);\n    delegate_relative_pointer!(SloposCompositor);\n""",
    """    delegate_seat!(SloposCompositor);\n    delegate_relative_pointer!(SloposCompositor);\n    delegate_pointer_constraints!(SloposCompositor);\n\n    impl PointerConstraintsHandler for SloposCompositor {\n        fn new_constraint(\n            &mut self,\n            _surface: &WlSurface,\n            pointer: &PointerHandle<Self>,\n        ) {\n            maybe_activate_pointer_constraint(self, pointer);\n        }\n\n        fn cursor_position_hint(\n            &mut self,\n            _surface: &WlSurface,\n            _pointer: &PointerHandle<Self>,\n            _location: Point<f64, Logical>,\n        ) {\n            // The unstable-v1 protocol defines this as an optional compositor\n            // warp hint. The nested X11 backend has no raw host-pointer warp\n            // primitive here, so retaining normal host cursor ownership is the\n            // least surprising and standards-compliant behaviour.\n        }\n    }\n""",
)

nested_helpers = r'''
    fn maybe_activate_pointer_constraint(
        state: &SloposCompositor,
        pointer: &PointerHandle<SloposCompositor>,
    ) {
        let location = state.pointer_pos;
        let Some((surface, surface_location)) = state.surface_under(location) else {
            return;
        };
        if pointer.current_focus().as_ref() != Some(&surface) {
            return;
        }
        with_pointer_constraint(&surface, pointer, |constraint| {
            let Some(constraint) = constraint else {
                return;
            };
            if constraint.is_active() {
                return;
            }
            let local = (location - surface_location).to_i32_round();
            if constraint.region().is_none_or(|region| region.contains(local)) {
                constraint.activate();
            }
        });
    }

    fn constrain_pointer_destination(
        state: &SloposCompositor,
        pointer: &PointerHandle<SloposCompositor>,
        current: Point<f64, Logical>,
        desired: Point<f64, Logical>,
    ) -> Point<f64, Logical> {
        let Some((surface, surface_location)) = state.surface_under(current) else {
            return desired;
        };

        let mut mode = PointerConstraintMotion::Free;
        let mut region = None;
        with_pointer_constraint(&surface, pointer, |constraint| {
            let Some(constraint) = constraint else {
                return;
            };
            if !constraint.is_active() {
                return;
            }
            let current_local = (current - surface_location).to_i32_round();
            if !constraint
                .region()
                .is_none_or(|candidate| candidate.contains(current_local))
            {
                return;
            }
            mode = match &*constraint {
                PointerConstraint::Locked(_) => PointerConstraintMotion::Locked,
                PointerConstraint::Confined(_) => PointerConstraintMotion::Confined,
            };
            region = constraint.region().cloned();
        });

        if mode == PointerConstraintMotion::Free {
            return desired;
        }
        if mode == PointerConstraintMotion::Locked {
            return current;
        }

        let delta = desired - current;
        let x_target = current + Point::from((delta.x, 0.0));
        let y_target = current + Point::from((0.0, delta.y));
        let same_surface = |target: Point<f64, Logical>| {
            state
                .surface_under(target)
                .is_some_and(|(candidate, _)| candidate == surface)
        };
        let inside_region = |target: Point<f64, Logical>| {
            region.as_ref().is_none_or(|candidate| {
                candidate.contains((target - surface_location).to_i32_round())
            })
        };
        let resolved = slopos_compositor::pointer_policy::resolve_pointer_delta(
            mode,
            (delta.x, delta.y),
            same_surface(x_target) && inside_region(x_target),
            same_surface(y_target) && inside_region(y_target),
        );
        let candidate = current + Point::from(resolved);
        if same_surface(candidate) && inside_region(candidate) {
            candidate
        } else {
            current
        }
    }

'''
replace_once(
    MAIN,
    """    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)\n""",
    nested_helpers + "    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)\n",
)

old_nested_motion = '''    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)
    where
        E: PointerMotionAbsoluteEvent<X11Input>,
    {
        let logical = Size::<i32, Logical>::from((state.output_size.w, state.output_size.h));
        let previous = state.pointer_pos;
        let pos = ev.position_transformed(logical);
        state.pointer_pos = pos;
        state.request_redraw();

        // Hit-test layer chrome, popup trees, then ordinary toplevels.
        let focus = state.surface_under(pos);

        let serial = state.next_serial();
        let time = ev.time_msec();
        let delta = Point::from((pos.x - previous.x, pos.y - previous.y));

        if let Some(ptr) = state.seat.get_pointer() {
            // Smithay's X11 backend reports absolute pointer coordinates only.
            // Derive a relative delta from consecutive samples; raw/unaccelerated
            // motion is unavailable here, so the same delta is reported for both.
            ptr.relative_motion(
                state,
                focus.clone(),
                &RelativeMotionEvent {
                    delta,
                    delta_unaccel: delta,
                    utime: u64::from(time) * 1_000,
                },
            );
            ptr.motion(
                state,
                focus,
                &MotionEvent {
                    location: pos,
                    serial,
                    time,
                },
            );
            ptr.frame(state);
        }
    }
'''
new_nested_motion = '''    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)
    where
        E: PointerMotionAbsoluteEvent<X11Input>,
    {
        let logical = Size::<i32, Logical>::from((state.output_size.w, state.output_size.h));
        let raw_pos = ev.position_transformed(logical);
        let previous_raw = state.last_backend_pointer_pos.replace(raw_pos).unwrap_or(raw_pos);
        let delta = Point::from((raw_pos.x - previous_raw.x, raw_pos.y - previous_raw.y));
        let current = state.pointer_pos;
        let serial = state.next_serial();
        let time = ev.time_msec();

        if let Some(ptr) = state.seat.get_pointer() {
            // Relative motion follows the raw host samples even while the visible
            // pointer is locked by zwp_pointer_constraints_v1.
            let relative_focus = state.surface_under(current);
            ptr.relative_motion(
                state,
                relative_focus,
                &RelativeMotionEvent {
                    delta,
                    delta_unaccel: delta,
                    utime: u64::from(time) * 1_000,
                },
            );

            let pos = constrain_pointer_destination(state, &ptr, current, raw_pos);
            state.pointer_pos = pos;
            state.request_redraw();
            let focus = state.surface_under(pos);
            ptr.motion(
                state,
                focus,
                &MotionEvent {
                    location: pos,
                    serial,
                    time,
                },
            );
            ptr.frame(state);
            maybe_activate_pointer_constraint(state, &ptr);
        } else {
            state.pointer_pos = raw_pos;
            state.request_redraw();
        }
    }
'''
replace_once(MAIN, old_nested_motion, new_nested_motion)

# DRM/libinput compositor protocol wiring and enforcement.
replace_once(
    DRM,
    """    GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent,\n""",
    """    GrabStartData, MotionEvent, PointerGrab, PointerHandle, PointerInnerHandle,\n    RelativeMotionEvent,\n""",
)
replace_once(
    DRM,
    """use smithay::wayland::output::{OutputHandler, OutputManagerState};\nuse smithay::wayland::relative_pointer::RelativePointerManagerState;\n""",
    """use smithay::wayland::output::{OutputHandler, OutputManagerState};\nuse smithay::wayland::pointer_constraints::{\n    with_pointer_constraint, PointerConstraint, PointerConstraintsHandler, PointerConstraintsState,\n};\nuse smithay::wayland::relative_pointer::RelativePointerManagerState;\n""",
)
replace_once(
    DRM,
    """    delegate_layer_shell, delegate_output, delegate_primary_selection, delegate_relative_pointer,\n    delegate_seat, delegate_session_lock, delegate_shm, delegate_xdg_shell,\n""",
    """    delegate_layer_shell, delegate_output, delegate_pointer_constraints,\n    delegate_primary_selection, delegate_relative_pointer, delegate_seat, delegate_session_lock,\n    delegate_shm, delegate_xdg_shell,\n""",
)
replace_once(
    DRM,
    """    DrmPresentationStage, InteractiveGrab, InteractiveGrabKind, OutputScale, ResizeEdges,\n""",
    """    DrmPresentationStage, InteractiveGrab, InteractiveGrabKind, OutputScale,\n    PointerConstraintMotion, ResizeEdges,\n""",
)
replace_once(
    DRM,
    """    seat_state: SeatState<Self>,\n    _relative_pointer_state: RelativePointerManagerState,\n    seat: Seat<Self>,\n""",
    """    seat_state: SeatState<Self>,\n    _relative_pointer_state: RelativePointerManagerState,\n    _pointer_constraints_state: PointerConstraintsState,\n    seat: Seat<Self>,\n""",
)
replace_once(
    DRM,
    """    let relative_pointer_state = RelativePointerManagerState::new::<DrmSessionState>(&dh);\n    let xdg_shell_state = XdgShellState::new::<DrmSessionState>(&dh);\n""",
    """    let relative_pointer_state = RelativePointerManagerState::new::<DrmSessionState>(&dh);\n    let pointer_constraints_state = PointerConstraintsState::new::<DrmSessionState>(&dh);\n    let xdg_shell_state = XdgShellState::new::<DrmSessionState>(&dh);\n""",
)
replace_once(
    DRM,
    """        seat_state,\n        _relative_pointer_state: relative_pointer_state,\n        seat,\n""",
    """        seat_state,\n        _relative_pointer_state: relative_pointer_state,\n        _pointer_constraints_state: pointer_constraints_state,\n        seat,\n""",
)
replace_once(
    DRM,
    """delegate_seat!(DrmSessionState);\ndelegate_relative_pointer!(DrmSessionState);\n""",
    """delegate_seat!(DrmSessionState);\ndelegate_relative_pointer!(DrmSessionState);\ndelegate_pointer_constraints!(DrmSessionState);\n\nimpl PointerConstraintsHandler for DrmSessionState {\n    fn new_constraint(&mut self, _surface: &WlSurface, pointer: &PointerHandle<Self>) {\n        if !self.locked {\n            maybe_activate_drm_pointer_constraint(self, pointer);\n        }\n    }\n\n    fn cursor_position_hint(\n        &mut self,\n        _surface: &WlSurface,\n        _pointer: &PointerHandle<Self>,\n        _location: Point<f64, Logical>,\n    ) {\n        // This protocol field is a hint: clients must not depend on a warp.\n        // SLOPOS deliberately keeps physical libinput ownership authoritative.\n    }\n}\n""",
)

drm_helpers = r'''
fn maybe_activate_drm_pointer_constraint(
    state: &DrmSessionState,
    pointer: &PointerHandle<DrmSessionState>,
) {
    if state.locked {
        return;
    }
    let location = state.pointer_location;
    let Some((surface, surface_location)) = state.surface_under(location) else {
        return;
    };
    if pointer.current_focus().as_ref() != Some(&surface) {
        return;
    }
    with_pointer_constraint(&surface, pointer, |constraint| {
        let Some(constraint) = constraint else {
            return;
        };
        if constraint.is_active() {
            return;
        }
        let local = (location - surface_location).to_i32_round();
        if constraint.region().is_none_or(|region| region.contains(local)) {
            constraint.activate();
        }
    });
}

fn constrain_drm_pointer_destination(
    state: &DrmSessionState,
    pointer: &PointerHandle<DrmSessionState>,
    current: Point<f64, Logical>,
    desired: Point<f64, Logical>,
) -> Point<f64, Logical> {
    // Session lock owns the whole input surface and takes precedence over
    // application pointer constraints.
    if state.locked {
        return desired;
    }
    let Some((surface, surface_location)) = state.surface_under(current) else {
        return desired;
    };

    let mut mode = PointerConstraintMotion::Free;
    let mut region = None;
    with_pointer_constraint(&surface, pointer, |constraint| {
        let Some(constraint) = constraint else {
            return;
        };
        if !constraint.is_active() {
            return;
        }
        let current_local = (current - surface_location).to_i32_round();
        if !constraint
            .region()
            .is_none_or(|candidate| candidate.contains(current_local))
        {
            return;
        }
        mode = match &*constraint {
            PointerConstraint::Locked(_) => PointerConstraintMotion::Locked,
            PointerConstraint::Confined(_) => PointerConstraintMotion::Confined,
        };
        region = constraint.region().cloned();
    });

    if mode == PointerConstraintMotion::Free {
        return desired;
    }
    if mode == PointerConstraintMotion::Locked {
        return current;
    }

    let delta = desired - current;
    let x_target = current + Point::from((delta.x, 0.0));
    let y_target = current + Point::from((0.0, delta.y));
    let same_surface = |target: Point<f64, Logical>| {
        state
            .surface_under(target)
            .is_some_and(|(candidate, _)| candidate == surface)
    };
    let inside_region = |target: Point<f64, Logical>| {
        region.as_ref().is_none_or(|candidate| {
            candidate.contains((target - surface_location).to_i32_round())
        })
    };
    let resolved = crate::pointer_policy::resolve_pointer_delta(
        mode,
        (delta.x, delta.y),
        same_surface(x_target) && inside_region(x_target),
        same_surface(y_target) && inside_region(y_target),
    );
    let candidate = current + Point::from(resolved);
    if same_surface(candidate) && inside_region(candidate) {
        candidate
    } else {
        current
    }
}

'''
replace_once(DRM, "impl DrmSessionState {\n", drm_helpers + "impl DrmSessionState {\n")

old_absolute = '''            InputEvent::PointerMotionAbsolute { event } => {
                let x = event.x_transformed(self.output_size.0);
                let y = event.y_transformed(self.output_size.1);
                self.pointer_location = Point::from((x, y));
                self.forward_pointer_motion(event.time_msec());
                self.request_redraw();
            }
'''
new_absolute = '''            InputEvent::PointerMotionAbsolute { event } => {
                let x = event.x_transformed(self.output_size.0);
                let y = event.y_transformed(self.output_size.1);
                let desired = Point::from((x, y));
                if let Some(pointer) = self.seat.get_pointer() {
                    self.pointer_location = constrain_drm_pointer_destination(
                        self,
                        &pointer,
                        self.pointer_location,
                        desired,
                    );
                    self.forward_pointer_motion(event.time_msec());
                    maybe_activate_drm_pointer_constraint(self, &pointer);
                } else {
                    self.pointer_location = desired;
                }
                self.request_redraw();
            }
'''
replace_once(DRM, old_absolute, new_absolute)

old_relative = '''            InputEvent::PointerMotion { event } => {
                // Relative motion (real mice): preserve both accelerated and raw
                // libinput deltas for zwp_relative_pointer_v1, then update the
                // compositor-space cursor position for ordinary wl_pointer.
                let (dx, dy) = (event.delta_x(), event.delta_y());
                let delta = Point::from((dx, dy));
                let delta_unaccel =
                    Point::from((event.delta_x_unaccel(), event.delta_y_unaccel()));
                let x = (self.pointer_location.x + dx).clamp(0.0, self.output_size.0 as f64 - 1.0);
                let y = (self.pointer_location.y + dy).clamp(0.0, self.output_size.1 as f64 - 1.0);
                self.pointer_location = Point::from((x, y));

                let focus = if self.locked {
                    self.active_lock_surface()
                        .map(|surface| (surface, Point::from((0.0, 0.0))))
                } else {
                    self.surface_under(self.pointer_location)
                };
                if let Some(pointer) = self.seat.get_pointer() {
                    pointer.relative_motion(
                        self,
                        focus,
                        &RelativeMotionEvent {
                            delta,
                            delta_unaccel,
                            utime: event.time(),
                        },
                    );
                }
                self.forward_pointer_motion(event.time_msec());
                self.request_redraw();
            }
'''
new_relative = '''            InputEvent::PointerMotion { event } => {
                // Preserve both accelerated and raw libinput deltas for
                // zwp_relative_pointer_v1 even when pointer-constraints keeps
                // the compositor-visible cursor stationary.
                let (dx, dy) = (event.delta_x(), event.delta_y());
                let delta = Point::from((dx, dy));
                let delta_unaccel =
                    Point::from((event.delta_x_unaccel(), event.delta_y_unaccel()));
                let current = self.pointer_location;
                let x = (current.x + dx).clamp(0.0, self.output_size.0 as f64 - 1.0);
                let y = (current.y + dy).clamp(0.0, self.output_size.1 as f64 - 1.0);
                let desired = Point::from((x, y));

                if let Some(pointer) = self.seat.get_pointer() {
                    let relative_focus = if self.locked {
                        self.active_lock_surface()
                            .map(|surface| (surface, Point::from((0.0, 0.0))))
                    } else {
                        self.surface_under(current)
                    };
                    pointer.relative_motion(
                        self,
                        relative_focus,
                        &RelativeMotionEvent {
                            delta,
                            delta_unaccel,
                            utime: event.time(),
                        },
                    );
                    self.pointer_location =
                        constrain_drm_pointer_destination(self, &pointer, current, desired);
                    self.forward_pointer_motion(event.time_msec());
                    maybe_activate_drm_pointer_constraint(self, &pointer);
                } else {
                    self.pointer_location = desired;
                }
                self.request_redraw();
            }
'''
replace_once(DRM, old_relative, new_relative)

# Permanent headless protocol lifecycle evidence.
replace_once(
    RUNTIME,
    """protocol_log=\"$artifact_dir/${commit_sha}-xdg-protocol.log\"\n""",
    """protocol_log=\"$artifact_dir/${commit_sha}-xdg-protocol.log\"\npointer_constraints_log=\"$artifact_dir/${commit_sha}-pointer-constraints.log\"\n""",
)
replace_once(
    RUNTIME,
    '"schema": 4,\n',
    '"schema": 5,\n',
)
replace_once(
    RUNTIME,
    '  "xdg_popup_reposition_verified": $(has_protocol_marker SLOPOS_XDG_POPUP_REPOSITIONED && printf true || printf false),\n',
    '  "xdg_popup_reposition_verified": $(has_protocol_marker SLOPOS_XDG_POPUP_REPOSITIONED && printf true || printf false),\n  "pointer_constraints_registry_verified": $([[ -s "$globals_log" ]] && grep -q "interface: \'zwp_pointer_constraints_v1\'" "$globals_log" && printf true || printf false),\n  "pointer_lock_request_verified": $([[ -s "$pointer_constraints_log" ]] && grep -q "^SLOPOS_POINTER_LOCK_REQUEST_ACCEPTED " "$pointer_constraints_log" && printf true || printf false),\n  "pointer_confine_request_verified": $([[ -s "$pointer_constraints_log" ]] && grep -q "^SLOPOS_POINTER_CONFINE_REQUEST_ACCEPTED " "$pointer_constraints_log" && printf true || printf false),\n',
)
replace_once(
    RUNTIME,
    '  "xdg_protocol_log": "$(basename "$protocol_log")"\n',
    '  "xdg_protocol_log": "$(basename "$protocol_log")",\n  "pointer_constraints_log": "$(basename "$pointer_constraints_log")"\n',
)
replace_once(
    RUNTIME,
    """for required_global in wl_compositor wl_shm wl_seat xdg_wm_base zwp_relative_pointer_manager_v1; do\n""",
    """for required_global in wl_compositor wl_shm wl_seat xdg_wm_base zwp_relative_pointer_manager_v1 zwp_pointer_constraints_v1; do\n""",
)
marker = '''printf 'Stressing abrupt toplevel and popup disconnect cleanup\\n'
'''
pointer_gate = '''printf 'Exercising pointer-constraint request/destroy lifecycle\\n'
WAYLAND_DISPLAY="$socket_name" timeout 20s \\
  target/debug/examples/headless_pointer_constraints_client >"$pointer_constraints_log" 2>&1
for marker in SLOPOS_POINTER_LOCK_REQUEST_ACCEPTED SLOPOS_POINTER_CONFINE_REQUEST_ACCEPTED SLOPOS_POINTER_CONSTRAINTS_OK; do
  if ! grep -q "^${marker}" "$pointer_constraints_log"; then
    write_artifact failed "missing_${marker}"
    cat "$pointer_constraints_log" >&2
    exit 1
  fi
done
if ! kill -0 "$compositor_pid" 2>/dev/null; then
  write_artifact failed "compositor_died_after_pointer_constraints"
  cat "$compositor_log" >&2
  exit 1
fi

'''
replace_once(RUNTIME, marker, pointer_gate + marker)

print("pointer constraints runtime migration applied")
