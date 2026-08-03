#!/usr/bin/env bash
# Copyright (c) 2026 Palaash Atri
# SPDX-License-Identifier: MIT

# Runtime protocol smoke test for the SLOPOS-I compositor's own headless backend.
# This is intentionally narrower than hardware QA: it proves that the exact
# built compositor can own a private Wayland socket, publish authenticated
# readiness, answer a real Wayland client, and terminate on request. The
# slopos-session supervisor, not a standalone compositor process, owns removal
# of the per-session runtime directory. This test does not claim DRM/KMS,
# rendering, input, HDR, VRR, or XWayland.

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
runtime_dir="$(mktemp -d "${TMPDIR:-/tmp}/slopos-headless-runtime.XXXXXX")"
chmod 700 "$runtime_dir"

compositor_pid=""
socket_name=""
shutdown_status="not_started"
socket_cleanup="not_observed"

write_artifact() {
  local status="$1"
  local failure="${2:-}"
  cat >"$artifact.tmp" <<JSON
{
  "schema": 1,
  "component": "slopos-compositor",
  "commit": "$commit_sha",
  "branch": "$branch",
  "started_at_utc": "$timestamp",
  "status": "$status",
  "failure": "$failure",
  "evidence_level": "headless_runtime_protocol_smoke",
  "backend": "headless",
  "runtime_verified": $([[ "$status" == "passed" ]] && printf true || printf false),
  "hardware_verified": false,
  "drm_verified": false,
  "rendering_verified": false,
  "input_verified": false,
  "socket": "$socket_name",
  "compositor_pid": "${compositor_pid:-}",
  "shutdown_status": "$shutdown_status",
  "socket_cleanup": "$socket_cleanup",
  "runtime_directory_owner": "slopos-session",
  "compositor_log": "$(basename "$compositor_log")",
  "wayland_info_log": "$(basename "$globals_log")"
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
cargo build -p slopos-compositor --locked

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

printf 'Connecting a real Wayland protocol client to %s\n' "$socket_name"
WAYLAND_DISPLAY="$socket_name" timeout 10s wayland-info >"$globals_log" 2>&1

for required_global in wl_compositor wl_shm wl_seat xdg_wm_base; do
  if ! grep -q "interface: '${required_global}'" "$globals_log"; then
    write_artifact failed "missing_global_${required_global}"
    cat "$globals_log" >&2
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

# A standalone compositor may leave its socket pathname after a signal because
# the session supervisor normally owns and removes the entire private runtime
# directory. Record this distinction instead of fabricating graceful cleanup.
if [[ -e "$runtime_dir/$socket_name" ]]; then
  socket_cleanup="supervisor_required"
else
  socket_cleanup="removed_by_compositor"
fi

write_artifact passed
printf 'Headless runtime protocol smoke passed for %s\n' "$commit_sha"
printf 'Evidence: %s\n' "$artifact"
printf 'This does not prove DRM/KMS, rendering, input, XWayland, HDR, VRR, or hardware compatibility.\n'
