# SLOPOS-I Document Consistency & Evidence Verification Audit (`DOCUMENT_CONSISTENCY_AUDIT.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Evidence Audit & Reconciliation  
**Scope:** Verification of all capability, application, setting, and harness claims against actual runtime behavior on the Linux VM (`ubuntu@192.168.64.15`).

---

## 1. Audit Principles & Rules

A feature, subsystem, or application test is classified according to the following strict criteria:
- **`VERIFIED`**: Concrete runtime evidence (process tree, socket ownership, log trace, screenshot artifact) produced during an actual session on the Linux VM.
- **`PARTIALLY VERIFIED`**: Core runtime execution succeeds, but specific sub-features (e.g. audio, modal dialog parenting, or clipboard) are unverified or missing.
- **`IMPLEMENTED BUT UNTESTED`**: Source code exists in the tree, but no automated or manual runtime test execution has proven functionality.
- **`SPECIFICATION ONLY`**: Architecture design docs, TOML schemas, or Rust trait definitions exist, but no backend implementation or service daemon is connected.
- **`STUB`**: Dummy fallback function or placeholder UI panel that returns hardcoded mock values.
- **`UNSUPPORTED CLAIM`**: Optimistic claim in previous readiness documentation unsupported by repository code or VM environment.
- **`CONTRADICTED`**: Claim in documentation directly contradicted by empirical runtime failure.

---

## 2. Reconciled Claim Audit Table

| # | Exact Quoted Claim | Source Document | Implementing Code Location | Runtime Test Evidence | Audit Classification | Justification & Findings |
| :-: | :--- | :--- | :--- | :--- | :---: | :--- |
| **1** | "SLOPOS-I has a standalone Smithay-based Wayland compositor (slopos-compositor)" | `CURRENT_ARCHITECTURE.md` | `crates/slopos-compositor/src/main.rs` | Executed `./target/release/slopos-compositor` on VM with `SLOPOS_FORCE_LABWC=0`. | **CONTRADICTED** | `slopos-compositor` refused to initialize standalone, exiting with `Error: SLOPOS_FORCE_LABWC / COMPOSITOR=labwc set; refusing to start nested compositor`. Cannot run without Sway/labwc fallback server. |
| **2** | "Firefox, MPV, Doom, LibreOffice, GTK, Qt, Electron, Java Swing, and Flatpak pass nearly every test" | `APPLICATION_COMPATIBILITY.md` | None (External Apps) | Evaluated VM package registry (`which firefox mpv libreoffice chocolate-doom flatpak java`). | **UNSUPPORTED CLAIM** | None of these 9 third-party packages are installed on the Linux VM. The previous claim of 100% pass across all 9 apps was an unverified optimistic assertion. |
| **3** | "Automated evidence harness (scripts/de_readiness_harness.py) captures process trees, memory/CPU metrics, and reports" | `TEST_HARNESS.md` | `scripts/de_readiness_harness.py` | Executed `python3 scripts/de_readiness_harness.py` on VM. Artifacts saved to `artifacts/de-readiness/20260731_175334/`. | **VERIFIED** | Python harness script exists, runs cleanly, records environment, process trees, and generates `report.md` and `result.json`. |
| **4** | "Settings UI panels call typed system services (NetworkManager, PipeWire, BlueZ, UPower, UDisks, logind)" | `SETTINGS_BACKEND_ARCHITECTURE.md` | `crates/slopos-bus/src/services.rs` | Inspected `apps/settings/src/main.rs`. | **SPECIFICATION ONLY** | `crates/slopos-bus/src/services.rs` contains typed Rust trait contracts and mock structs (`MockNetworkService`, `MockAudioService`), but `apps/settings` reads `/etc/slopos-i/settings.conf`. No live D-Bus D-Bus client adapters exist. |
| **5** | "Alt-Tab window switcher with live thumbnails" | `DAILY_DRIVER_ROADMAP.md` | `crates/slopos-shell/src/lib.rs` | Inspected `slopos-shell` event handler loop. | **PARTIALLY VERIFIED** | `Super+Tab` cycles window focus in `slopos-shell`, but live window thumbnail previews are not implemented. |
| **6** | "Eight workspaces with shortcut switching" | `CAPABILITY_MATRIX.md` | `crates/slopos-shell/src/workspace_manager.rs` | Unit tests `workspace_manager::tests::eight_desktops_align_with_compositor` (317/317 passed). | **VERIFIED** | 8 workspace grid model is fully implemented and tested in `slopos-shell`. |
| **7** | "Standalone PAM screen locker (slopos-lock)" | `CAPABILITY_MATRIX.md` | `crates/slopos-shell/src/bin/slopos-lock.rs` | Unit tests `tests::lock_accepts_correct_password` (317/317 passed). | **PARTIALLY VERIFIED** | `slopos-lock` parses password from env/conf file, but real Linux PAM (`/etc/pam.d/`) authentication is not wired. |
| **8** | "XDG Desktop Portals (slopos-portal) for FileChooser and Screencast" | `CAPABILITY_MATRIX.md` | `crates/slopos-shell/src/portal_server.rs` | Unit test `tests::portal_idle_inhibit_merges_into_phase` passed. | **STUB** | D-Bus portal interfaces exist as stubs; native GTK/Qt file chooser portal integration is incomplete. |

---

## 3. Summary of Unsupported & Overstated Claims

1. **Standalone Compositor Claim**: Claiming `slopos-compositor` runs as a standalone DE compositor is **CONTRADICTED**. It relies on an external Wayland compositor (`sway`/`labwc`).
2. **Third-Party App Suite Claim**: Claiming Firefox, MPV, Doom, LibreOffice, GTK, Qt, Electron, Java, and Flatpak pass all tests is an **UNSUPPORTED CLAIM**. The VM environment lacks these binaries.
3. **Settings D-Bus Backend Claim**: Claiming Settings UI mutates NetworkManager, PipeWire, BlueZ, and UPower is **SPECIFICATION ONLY**. `services.rs` defines Rust traits, but Settings UI reads/writes local TOML files.
