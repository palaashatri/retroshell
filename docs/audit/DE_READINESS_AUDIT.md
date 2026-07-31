# SLOPOS-I Desktop Environment Readiness Assessment (`DE_READINESS_AUDIT.md`)

**Date:** 2026-07-31  
**Status:** Runtime-Verified Assessment  
**Target:** Usable Linux Desktop Environment Scope

---

## 1. Executive Summary & Core Question Answer

### Core Question
*Can SLOPOS-I currently operate as the user's primary desktop session, launch ordinary Linux applications, manage them correctly, expose essential desktop services, and survive daily use without relying invisibly on another desktop environment?*

### Fact-Based Answer
**No (Overall Readiness: ~20% Daily-Driver Scope)**.

SLOPOS-I is **not** currently capable of operating as a primary daily-driver desktop without external helper infrastructure. As proven in [`STANDALONE_SESSION_EVIDENCE.md`](STANDALONE_SESSION_EVIDENCE.md), `slopos-compositor` cannot currently initialize standalone on the Linux VM without an underlying Wayland host compositor (`sway`/`labwc`). When fallbacks are disabled, `slopos-compositor` exits with `Error: SLOPOS_FORCE_LABWC / COMPOSITOR=labwc set; refusing to start nested compositor`.

---

## 2. Key Findings by Subsystem

### 2.1 Compositor & Session Ownership
- **SLOPOS-I Owned**: `slopos-shell` (Layer-shell client for desktop icons, top menu bar, dock, and Spotlight overlay).
- **Host / External Dependencies**: Session launch script (`start-slopos-i`) falls back to `sway` or `labwc`. Standalone `slopos-compositor` fails to initialize DRM/nested loop without host session helpers.
- **Verdict**: **CONTRADICTED / DELEGATED TO HOST**.

### 2.2 Application Execution & Window Management
- **Native Apps**: 5 native SLOPOS-I applications (`Finder`, `TextEdit`, `Terminal`, `Settings`, `App Store`) pass 87/87 unit tests and render floating in the SLOPOS-I session.
- **Third-Party Linux Apps**: 9 third-party packages (Firefox, MPV, Doom, LibreOffice, GTK, Qt, Electron, Java, Flatpak) are **UNTESTED** due to missing package binaries on the VM image.

### 2.3 Desktop Services & Settings Backends
- **Settings Application**: 4 panels are **PARTIALLY FUNCTIONAL** (updating local shell UI & TOML files). 6 panels are **MOCK / DISCONNECTED** (UI controls store TOML values without driving NetworkManager, PipeWire, BlueZ, UPower, or libinput system daemons).
- **Portals**: Custom portal daemon (`slopos-portal`) implements basic stubs.


---

## 3. High-Priority Remediation Targets

1. **Eliminate Hidden Session Fallbacks**: Make `slopos-compositor` the primary enforced Wayland compositor in `start-slopos-i`.
2. **Implement D-Bus Settings Adapters**: Wire `Settings` UI buttons to real D-Bus calls (`org.freedesktop.NetworkManager`, `org.pulseaudio.ServerLookup`, `org.freedesktop.UPower`, `org.freedesktop.login1`).
3. **Automate DE Compatibility Harness**: Execute `scripts/de_readiness_harness.py` to capture metrics and logs across third-party applications.
