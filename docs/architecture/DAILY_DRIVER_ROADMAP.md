# SLOPOS-I Daily-Driver Maturity Roadmap (`DAILY_DRIVER_ROADMAP.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Roadmap  
**Target:** Genuinely usable, production-ready Linux Desktop Environment

---

## 1. Maturity Milestones & Roadmap Phases

### Milestone 0: Architecture Truth & Standalone Foundation (CURRENT)
- **Objectives**: Establish fact-based DE readiness audit; verify standalone `slopos-compositor` operation; eliminate hidden host DE fallbacks.
- **Deliverables**: `DE_READINESS_AUDIT.md`, `CURRENT_ARCHITECTURE.md`, `CAPABILITY_MATRIX.md`, `APPLICATION_COMPATIBILITY.md`.
- **Status**: **100% COMPLETE**.

### Milestone 1: Daily-Driver Desktop Shell
- **Objectives**: Stable session startup, robust window management (move, resize, focus, Alt-Tab), system notifications, standalone PAM lock screen (`slopos-lock`), and core settings integration.
- **Target Features**:
  - Alt-Tab window switcher with live thumbnails.
  - Standalone PAM session lock & login greeter.
  - Multi-client process auto-reaping & crash recovery.
- **Target Date**: Q3 2026.

### Milestone 2: Broad Application Compatibility
- **Objectives**: Seamless execution of Firefox, MPV, Doom (SDL2), LibreOffice, GTK3/4, Qt6, Electron, Java Swing, and Flatpak applications.
- **Target Features**:
  - XWayland clipboard synchronization (`wlr-data-control`).
  - Seamless child dialog parent frame anchoring.
  - GPU acceleration passthrough (OpenGL / Vulkan / VAAPI).
- **Target Date**: Q4 2026.

### Milestone 3: Hardware & Hardware Integration
- **Objectives**: Native multi-monitor management, per-monitor display scaling, Wi-Fi configuration via NetworkManager, PipeWire audio routing, UPower battery management, and removable media automount.
- **Target Features**:
  - Settings UI NetworkManager & PipeWire D-Bus adapters.
  - RANDR / multi-monitor arrangement tool.
- **Target Date**: Q1 2027.

### Milestone 4: Full Desktop Integration
- **Objectives**: XDG desktop portals (`slopos-portal`), file chooser, MIME type associations, default apps, system tray status items, secrets keyring integration, and screenshot/screencast tools.
- **Target Date**: Q2 2027.

### Milestone 5: Production Reliability & QA Verification
- **Objectives**: Automated evidence harness integration (`scripts/de_readiness_harness.py`), 24-hour stability testing, memory leak profiling, and full accessibility audit.
- **Target Date**: Q3 2027.
