#!/usr/bin/env python3
# Copyright (c) 2026 Palaash Atri
# SPDX-License-Identifier: MIT

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one anchor, found {count}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1), encoding="utf-8")


LIB = "crates/slopos-compositor/src/lib.rs"
CARGO = "crates/slopos-compositor/Cargo.toml"
DRM = "crates/slopos-compositor/src/session_drm.rs"
RUNTIME = "scripts/verify-compositor-headless-runtime.sh"

# The first migration intentionally stops at the first DRM input block on the
# current tree. Everything before that point has already been applied in the
# workflow worktree. Finish the migration using anchors copied from the exact
# branch head, and make the unstable client protocol feature explicit.
replace_once(
    LIB,
    "pub mod pointer_policy;\n",
    "pub mod pointer_policy;\npub use pointer_policy::PointerConstraintMotion;\n",
)
replace_once(
    CARGO,
    'wayland-protocols = { workspace = true, features = ["client"] }\n',
    'wayland-protocols = { workspace = true, features = ["client", "unstable"] }\n',
)

old_absolute = '''            InputEvent::PointerMotionAbsolute { event } => {
                let x = event.x_transformed(self.output_size.0);
                let y = event.y_transformed(self.output_size.1);
                self.pointer_location = Point::from((x, y));
                self.forward_pointer_motion(event.time_msec());
                // The DRM cursor is compositor-rendered, so pointer motion is
                // damage even when no client surface changed.
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
                // The DRM cursor is compositor-rendered, so pointer motion is
                // damage even when no client surface changed.
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
                let delta_unaccel = Point::from((event.delta_x_unaccel(), event.delta_y_unaccel()));
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
                let delta_unaccel = Point::from((event.delta_x_unaccel(), event.delta_y_unaccel()));
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

# Permanent protocol lifecycle evidence. The headless compositor has no input
# device, so these fields prove registry + request lifecycle only, not physical
# lock/confine movement enforcement.
replace_once(
    RUNTIME,
    'protocol_log="$artifact_dir/${commit_sha}-xdg-protocol.log"\n',
    'protocol_log="$artifact_dir/${commit_sha}-xdg-protocol.log"\npointer_constraints_log="$artifact_dir/${commit_sha}-pointer-constraints.log"\n',
)
replace_once(RUNTIME, '  "schema": 4,\n', '  "schema": 5,\n')
replace_once(
    RUNTIME,
    '  "xdg_popup_reposition_verified": $(has_protocol_marker SLOPOS_XDG_POPUP_REPOSITIONED && printf true || printf false),\n',
    '  "xdg_popup_reposition_verified": $(has_protocol_marker SLOPOS_XDG_POPUP_REPOSITIONED && printf true || printf false),\n'
    '  "pointer_constraints_registry_verified": $([[ -s "$globals_log" ]] && grep -q "interface: \'zwp_pointer_constraints_v1\'" "$globals_log" && printf true || printf false),\n'
    '  "pointer_lock_request_verified": $([[ -s "$pointer_constraints_log" ]] && grep -q "^SLOPOS_POINTER_LOCK_REQUEST_ACCEPTED " "$pointer_constraints_log" && printf true || printf false),\n'
    '  "pointer_confine_request_verified": $([[ -s "$pointer_constraints_log" ]] && grep -q "^SLOPOS_POINTER_CONFINE_REQUEST_ACCEPTED " "$pointer_constraints_log" && printf true || printf false),\n',
)
replace_once(
    RUNTIME,
    '  "xdg_protocol_log": "$(basename "$protocol_log")"\n',
    '  "xdg_protocol_log": "$(basename "$protocol_log")",\n'
    '  "pointer_constraints_log": "$(basename "$pointer_constraints_log")"\n',
)
replace_once(
    RUNTIME,
    'for required_global in wl_compositor wl_shm wl_seat xdg_wm_base zwp_relative_pointer_manager_v1; do\n',
    'for required_global in wl_compositor wl_shm wl_seat xdg_wm_base zwp_relative_pointer_manager_v1 zwp_pointer_constraints_v1; do\n',
)
replace_once(
    RUNTIME,
    "printf 'Stressing abrupt toplevel and popup disconnect cleanup\\n'\n",
    '''printf 'Exercising pointer-constraint request/destroy lifecycle\\n'
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

printf 'Stressing abrupt toplevel and popup disconnect cleanup\\n'
''',
)

print("pointer constraints migration finished")
