#!/usr/bin/env bash
# Copyright (c) 2026 Palaash Atri
# SPDX-License-Identifier: MIT

# Runtime protocol smoke test for the SLOPOS-I compositor's own headless backend.
# It proves that the exact build owns a private socket, publishes authenticated
# readiness, serves registry clients, survives abrupt role disconnects, applies
# live xdg-toplevel presentation transitions, completes xdg-popup configure and
# reposition lifecycles, accepts a healthy client after stress, and terminates.
# It does not claim DRM/KMS, rendering, input, popup grabs, HDR, VRR or XWayland.

set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  printf 'verify-compositor-headless-runtime: Linux is required\n' >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

for tool in cargo git sed grep stat timeout wayland-info; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'verify-compositor-headless-runtime: missing required tool: %s\n' "$tool" >&2
    exit 2
  fi
done

commit_sha="$(git rev-parse HEAD)"
branch="$(git branch --show-current || true)"
timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
artifact_dir="${SLOPOS_QA_ARTIFACT_DIR:-artifacts/qa/compositor-headless-runtime}"
mkdir -p "$artifact_dir"
artifact="$artifact_dir/${commit_sha}.json"
compositor_log="$artifact_dir/${commit_sha}-compositor.log"
globals_log="$artifact_dir/${commit_sha}-wayland-info.log"
stress_log="$artifact_dir/${commit_sha}-disconnect-stress.log"
protocol_log="$artifact_dir/${commit_sha}-xdg-protocol.log"
pointer_constraints_log="$artifact_dir/${commit_sha}-pointer-constraints.log"
runtime_dir="$(mktemp -d "${TMPDIR:-/tmp}/slopos-headless-runtime.XXXXXX")"
chmod 700 "$runtime_dir"

compositor_pid=""
socket_name=""
shutdown_status="not_started"
socket_cleanup="not_observed"

has_protocol_marker() {
  local marker="$1"
  [[ -s "$protocol_log" ]] && grep -q "^${marker} " "$protocol_log"
}

stress_passed() {
  [[ -s "$stress_log" ]] && grep -q '^SLOPOS_ABRUPT_DISCONNECT_STRESS cycles=' "$stress_log"
}

write_artifact() {
  local status="$1"
  local failure="${2:-}"
  cat >"$artifact.tmp" <<JSON
{
  "schema": 5,
  "component": "slopos-compositor",
  "commit": "$commit_sha",
  "branch": "$branch",
  "started_at_utc": "$timestamp",
  "status": "$status",
  "failure": "$failure",
  "evidence_level": "headless_runtime_protocol_smoke",
  "backend": "headless",
  "runtime_verified": $([[ "$status" == "passed" ]] && printf true || printf false),
  "registry_client_verified": $([[ -s "$globals_log" ]] && printf true || printf false),
  "abrupt_disconnect_recovery_verified": $(stress_passed && printf true || printf false),
  "xdg_toplevel_configure_verified": $(has_protocol_marker SLOPOS_XDG_TOPLEVEL_CONFIGURED && printf true || printf false),
  "xdg_toplevel_maximize_verified": $(has_protocol_marker SLOPOS_XDG_TOPLEVEL_MAXIMIZED && printf true || printf false),
  "xdg_toplevel_fullscreen_verified": $(has_protocol_marker SLOPOS_XDG_TOPLEVEL_FULLSCREEN && printf true || printf false),
  "xdg_toplevel_restore_verified": $(has_protocol_marker SLOPOS_XDG_TOPLEVEL_RESTORED && printf true || printf false),
  "xdg_popup_configure_verified": $(has_protocol_marker SLOPOS_XDG_POPUP_CONFIGURED && printf true || printf false),
  "xdg_popup_reposition_verified": $(has_protocol_marker SLOPOS_XDG_POPUP_REPOSITIONED && printf true || printf false),
  "pointer_constraints_registry_verified": $([[ -s "$globals_log" ]] && grep -q "interface: 'zwp_pointer_constraints_v1'" "$globals_log" && printf true || printf false),
  "pointer_lock_request_verified": $([[ -s "$pointer_constraints_log" ]] && grep -q "^SLOPOS_POINTER_LOCK_REQUEST_ACCEPTED " "$pointer_constraints_log" && printf true || printf false),
  "pointer_confine_request_verified": $([[ -s "$pointer_constraints_log" ]] && grep -q "^SLOPOS_POINTER_CONFINE_REQUEST_ACCEPTED " "$pointer_constraints_log" && printf true || printf false),
  "hardware_verified": false,
  "drm_verified": false,
  "rendering_verified": false,
  "input_verified": false,
  "popup_grab_verified": false,
  "socket": "$socket_name",
  "compositor_pid": "${compositor_pid:-}",
  "shutdown_status": "$shutdown_status",
  "socket_cleanup": "$socket_cleanup",
  "runtime_directory_owner": "slopos-session",
  "compositor_log": "$(basename "$compositor_log")",
  "wayland_info_log": "$(basename "$globals_log")",
  "disconnect_stress_log": "$(basename "$stress_log")",
  "xdg_protocol_log": "$(basename "$protocol_log")",
  "pointer_constraints_log": "$(basename "$pointer_constraints_log")"
}
JSON
  mv "$artifact.tmp" "$artifact"
}

cleanup() {
  local exit_code=$?
  trap - EXIT INT TERM
  if [[ -n "$compositor_pid" ]] && kill -0 "$compositor_pid" 2>/dev/null; then
    kill -TERM "$compositor_pid" 2>/dev/null || true
    for _ in $(seq 1 50); do
      if ! kill -0 "$compositor_pid" 2>/dev/null; then
        break
      fi
      sleep 0.1
    done
    if kill -0 "$compositor_pid" 2>/dev/null; then
      kill -KILL "$compositor_pid" 2>/dev/null || true
    fi
    wait "$compositor_pid" 2>/dev/null || true
  fi
  rm -rf "$runtime_dir"
  if [[ $exit_code -ne 0 && ! -f "$artifact" ]]; then
    write_artifact failed "unexpected_exit_$exit_code"
  fi
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then
  write_artifact failed "tracked_worktree_dirty"
  printf 'verify-compositor-headless-runtime: tracked working tree is dirty\n' >&2
  exit 2
fi

printf 'Building exact-commit compositor %s\n' "$commit_sha"
cargo build -p slopos-compositor --examples --locked

export XDG_RUNTIME_DIR="$runtime_dir"
export SLOPOS_SESSION_RUNTIME_DIR="$runtime_dir"
export SLOPOS_SESSION_TOKEN="headless-smoke-${commit_sha}-$$"
unset DISPLAY WAYLAND_DISPLAY SLOPOS_CLIENT_WAYLAND_DISPLAY

printf 'Starting SLOPOS-owned headless compositor\n'
target/debug/slopos-compositor --backend headless >"$compositor_log" 2>&1 &
compositor_pid=$!

readiness="$runtime_dir/readiness"
for _ in $(seq 1 100); do
  if [[ -s "$readiness" ]]; then
    break
  fi
  if ! kill -0 "$compositor_pid" 2>/dev/null; then
    wait "$compositor_pid" || true
    write_artifact failed "compositor_exited_before_readiness"
    cat "$compositor_log" >&2
    exit 1
  fi
  sleep 0.1
done

if [[ ! -s "$readiness" ]]; then
  write_artifact failed "readiness_timeout"
  cat "$compositor_log" >&2
  exit 1
fi

socket_name="$(sed -n '1p' "$readiness")"
ready_pid="$(sed -n 's/^pid=//p' "$readiness")"
ready_token="$(sed -n 's/^token=//p' "$readiness")"
ready_width="$(sed -n 's/^width=//p' "$readiness")"
ready_height="$(sed -n 's/^height=//p' "$readiness")"

if [[ ! "$socket_name" =~ ^wayland-[0-9]+$ ]]; then
  write_artifact failed "invalid_socket_name"
  exit 1
fi
if [[ "$ready_pid" != "$compositor_pid" ]]; then
  write_artifact failed "readiness_pid_mismatch"
  exit 1
fi
if [[ "$ready_token" != "$SLOPOS_SESSION_TOKEN" ]]; then
  write_artifact failed "readiness_token_mismatch"
  exit 1
fi
if [[ ! "$ready_width" =~ ^[1-9][0-9]*$ || ! "$ready_height" =~ ^[1-9][0-9]*$ ]]; then
  write_artifact failed "invalid_output_dimensions"
  exit 1
fi
if [[ ! -S "$runtime_dir/$socket_name" ]]; then
  write_artifact failed "wayland_socket_missing"
  exit 1
fi

runtime_mode="$(stat -c '%a' "$runtime_dir")"
if [[ "$runtime_mode" != "700" ]]; then
  write_artifact failed "runtime_directory_not_private"
  exit 1
fi

printf 'Connecting registry client to %s\n' "$socket_name"
WAYLAND_DISPLAY="$socket_name" timeout 10s wayland-info >"$globals_log" 2>&1
for required_global in wl_compositor wl_shm wl_seat xdg_wm_base zwp_relative_pointer_manager_v1 zwp_pointer_constraints_v1; do
  if ! grep -q "interface: '${required_global}'" "$globals_log"; then
    write_artifact failed "missing_global_${required_global}"
    cat "$globals_log" >&2
    exit 1
  fi
done

printf 'Exercising pointer-constraint request/destroy lifecycle\n'
WAYLAND_DISPLAY="$socket_name" timeout 20s \
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

printf 'Stressing abrupt toplevel and popup disconnect cleanup\n'
WAYLAND_DISPLAY="$socket_name" SLOPOS_DISCONNECT_STRESS_CYCLES=64 timeout 45s \
  target/debug/examples/headless_disconnect_stress >"$stress_log" 2>&1
if ! stress_passed; then
  write_artifact failed "abrupt_disconnect_stress_failed"
  cat "$stress_log" >&2
  cat "$compositor_log" >&2
  exit 1
fi
if ! kill -0 "$compositor_pid" 2>/dev/null; then
  write_artifact failed "compositor_died_after_disconnect_stress"
  cat "$compositor_log" >&2
  exit 1
fi

printf 'Completing healthy presentation and popup lifecycles after stress\n'
WAYLAND_DISPLAY="$socket_name" timeout 30s \
  target/debug/examples/headless_toplevel_client >"$protocol_log" 2>&1
for marker in \
  SLOPOS_XDG_TOPLEVEL_CONFIGURED \
  SLOPOS_XDG_TOPLEVEL_MAXIMIZED \
  SLOPOS_XDG_TOPLEVEL_FULLSCREEN \
  SLOPOS_XDG_TOPLEVEL_RESTORED \
  SLOPOS_XDG_POPUP_CONFIGURED \
  SLOPOS_XDG_POPUP_REPOSITIONED; do
  if ! has_protocol_marker "$marker"; then
    write_artifact failed "missing_${marker}"
    cat "$protocol_log" >&2
    exit 1
  fi
done

kill -TERM "$compositor_pid"
for _ in $(seq 1 50); do
  if ! kill -0 "$compositor_pid" 2>/dev/null; then
    break
  fi
  sleep 0.1
done
if kill -0 "$compositor_pid" 2>/dev/null; then
  shutdown_status="timeout"
  write_artifact failed "shutdown_timeout"
  exit 1
fi
wait "$compositor_pid" 2>/dev/null || true
shutdown_status="terminated"

if [[ -e "$runtime_dir/$socket_name" ]]; then
  socket_cleanup="supervisor_required"
else
  socket_cleanup="removed_by_compositor"
fi

write_artifact passed
printf 'Headless runtime protocol smoke passed for %s\n' "$commit_sha"
printf 'Evidence: %s\n' "$artifact"
printf 'This does not prove DRM/KMS, rendering, input, popup grabs, XWayland, HDR, VRR, or hardware compatibility.\n'
