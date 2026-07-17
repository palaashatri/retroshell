# HDR/VRR Implementation Plan

## Overview
Full HDR (High Dynamic Range) and VRR (Variable Refresh Rate) support for RetroShell compositor and shell.

## Architecture

### 1. Compositor (retro-compositor) Changes

**File:** `crates/retro-compositor/src/main.rs`

#### 1a. Output Capabilities
- Detect GPU HDR support via wgpu
- Query DRM device for supported color spaces and refresh rates
- Expose via Wayland `wl_output` protocol extensions

#### 1b. Color Space Negotiation
- Implement `wp_color_representation_v1` protocol (color space negotiation)
- Per-surface color space tracking
- Tone-mapping when client sends sRGB content to HDR output

#### 1c. Frame Timing
- Implement `wp_presentation` protocol for frame scheduling
- Support dynamic refresh rate switching
- Synchronize to monitor VSync

**New code locations:**
- `crates/retro-compositor/src/hdr.rs` — HDR capability detection and negotiation
- `crates/retro-compositor/src/frame_timing.rs` — VRR and presentation feedback

### 2. Rendering Pipeline (retro-render + wgpu) Changes

**File:** `crates/retro-render/src/lib.rs`, new module `color_space.rs`

#### 2a. Texture Formats
- sRGB: RGBA8 (SDR, backward compatible)
- Rec2020: RGBA16F (wide color gamut)
- scRGB: RGBA16F (signed, allows > 1.0 for HDR)

#### 2b. Tone-Mapping
- Reinhard tone-mapper for SDR→HDR
- ACES tone-mapping for professional color
- Per-surface color space conversion

#### 2c. GPU Detection
- Query wgpu adapter for HDR texture format support
- Fall back to SDR (sRGB) if unavailable

**New code locations:**
- `crates/retro-render/src/color_space.rs` — color space definitions and conversion
- Shader updates in `crates/retro-render/shaders/` for tone-mapping

### 3. Shell (retro-shell) Changes

**File:** `crates/retro-shell/src/lib.rs`

#### 3a. Settings UI
- New "Display" settings pane (or extend existing one)
- Toggles: "Enable HDR", "Enable VRR"
- Selector: Refresh rate (60Hz, 120Hz, 144Hz, 165Hz, adaptive)
- Selector: Color space (sRGB, Rec2020, scRGB)
- Display: Current monitor capabilities (detected from compositor)

#### 3b. Configuration
- Persist to `~/.config/retroshell/settings.conf`:
  ```
  [display]
  hdr_enabled=true
  vrr_enabled=true
  refresh_rate=120
  color_space=rec2020
  ```

#### 3c. Runtime
- Query compositor capabilities on startup
- Apply settings to surfaces via Wayland protocol
- Fallback to SDR if compositor doesn't support HDR

**New code locations:**
- `crates/retro-shell/src/display_settings.rs` — HDR/VRR settings pane
- `crates/retro-shell/src/config.rs` — persist display settings

### 4. Testing & Verification

#### Host Tests (cargo test)
- Color space conversion math (sRGB ↔ rec2020 ↔ scRGB)
- Tone-mapping correctness
- Settings serialization/deserialization
- Protocol negotiation logic

#### Ubuntu Server Verification Script
- Query DRM device: `lspci | grep -i vga` + `ls /dev/dri/`
- Query Wayland socket and capabilities
- Start compositor; check output protocol versions
- Query shell settings; verify HDR/VRR config read/write
- Render a test pattern (gradient) and verify color space

## Implementation Order

1. **Phase 1:** Compositor HDR detection + Wayland protocol wiring
2. **Phase 2:** Rendering pipeline color space support
3. **Phase 3:** Shell Settings UI and config persistence
4. **Phase 4:** Integration tests + Ubuntu build script

## Known Limitations (document in code/README)

- Nested X11/Xvfb: HDR unavailable (no DRM), falls back to SDR silently
- Tone-mapping on client SDR→server HDR: placeholder (Reinhard; ACES is future)
- wl_data_device: full implementation deferred (uses placeholder from T4)
- XWayland clients: HDR passthrough not implemented (future)
- Multi-monitor HDR sync: single output only (future)

## Success Criteria

- ✅ Compositor compiles and runs on Ubuntu server with native Wayland/DRM
- ✅ `wl_output` protocol exposes HDR/VRR capabilities
- ✅ Shell Settings UI shows detected capabilities
- ✅ Settings persist and apply on restart
- ✅ Test gradient renders in configured color space
- ✅ Refresh rate switching works (verify via `wl_output` protocol events)
- ✅ All host tests pass
- ✅ Ubuntu build script runs end-to-end
