# SLOPOS-I DE Readiness Test Harness Specification (`TEST_HARNESS.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Specification  
**Scope:** Automated evidence collection harness script and JSON schema for DE compatibility verification.

---

## 1. Automated Test Harness Overview

The Desktop Environment Readiness Test Harness (`scripts/de_readiness_harness.py`) provides reproducible, automated testing of third-party applications inside a clean SLOPOS-I session on the Linux VM.

### Output Artifact Directory Structure
```
artifacts/de-readiness/<timestamp>/
    environment.json
    process-tree.txt
    compositor.log
    report.md
    applications/
        firefox/
            result.json
            launch.png
            fullscreen.png
            logs.txt
        mpv/
            result.json
            launch.png
            fullscreen.png
            logs.txt
        doom/
            result.json
            launch.png
            logs.txt
        libreoffice/
            result.json
            launch.png
            logs.txt
```

---

## 2. Evidence Collection Protocol

For each test application:
1. **Launch**: Harness executes application command line within the Wayland session.
2. **Metadata Capture**: Records process PID, parent PID, CPU usage, RSS memory, Wayland vs XWayland protocol.
3. **Geometry Verification**: Queries window bounds via Wayland foreign-toplevel API.
4. **Visual Capture**: Takes screenshot via `grim`.
5. **Action Verification**: Simulates window move, resize, minimize, maximize, and close actions.
6. **Report Generation**: Emits `result.json` and compiles master `report.md`.
