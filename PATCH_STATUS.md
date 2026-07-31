# SLOPOS-I Patch Status — 2026-08-01

This archive contains a source-level implementation pass focused on the current
foundational defects and architecture. It is intentionally marked **QA pending**:
the editing environment did not have a Rust toolchain or Linux Wayland/DRM
runtime, so the workspace was not compiled or exercised here.

## Code included

- New MIT-licensed `slopos-session` Rust supervisor.
  - Starts `slopos-compositor` first.
  - Waits for the compositor-owned private Wayland socket.
  - Launches `slopos-shell` with the private socket explicitly selected.
  - Separates host and client Wayland-display variables.
  - Terminates the other critical process group when shell or compositor exits.
  - Removes only the known readiness file; it does not glob-delete host Wayland sockets.
- `scripts/start-slopos-i` now delegates to `slopos-session` and no longer silently
  falls back to labwc or Sway.
- Explicit compositor backend selection (`drm`, `nested`/`x11`, `headless`).
- Compositor-owned xdg-toplevel state and interaction paths for move, resize,
  focus/raise, minimize, maximize, fullscreen and close handling.
- Nested compositor software cursor fallback plus client cursor-surface hotspot handling.
- DRM cursor-surface hotspot handling and compositor interaction state plumbing.
- Damage/dirty-driven nested compositor event-loop work to avoid unconditional redraw.
- SDK client-decoration hit testing that sends proper xdg move/resize requests through winit.
- Scale-aware canvas/text rasterization changes and measured-width ellipsis.
- Finder toolbar/button sizing cleanup.
- Evidence harness changed to report only launch/liveness facts; interaction fields remain
  `UNTESTED` instead of being fabricated.
- MIT ownership/license scaffolding (`COPYRIGHT`, `THIRD_PARTY_LICENSES.md`, `deny.toml`).
- Packaging updated to include `slopos-session` and remove labwc as a required production component.

## Static validation completed here

- `bash -n` passed for modified shell/packaging scripts.
- `python -m py_compile scripts/de_readiness_harness.py` passed.
- All Cargo/TOML files parsed successfully with Python `tomllib`.
- `git diff --check` passed.

## Required QA on the Linux VM

1. `cargo fmt --all -- --check`
2. `cargo check --workspace --all-targets`
3. `cargo test --workspace`
4. Run nested session and verify private socket ownership.
5. Verify visible cursor over shell and application surfaces.
6. Verify real pointer-driven move and resize.
7. Profile idle CPU and memory for at least 60 seconds.
8. Verify DRM session separately on hardware/VM support.
9. Verify HDR, VRR and hardware cursor planes only on suitable physical hardware.
10. Capture rendering regressions at scale 1.0, 1.25, 1.5 and 2.0.

## Known caveats

- The nested compositor path uses Smithay's X11 backend; a true Wayland-hosted winit
  nested backend is not implemented in this patch.
- The DRM path has client-provided cursor-surface support, but the procedural fallback
  cursor is not yet integrated as a DRM render element/hardware cursor plane.
- Text rendering is improved but remains a per-glyph rectangle renderer rather than a
  production glyph-atlas/shaping pipeline.
- Compilation and runtime behavior are not claimed until the VM QA steps pass.
