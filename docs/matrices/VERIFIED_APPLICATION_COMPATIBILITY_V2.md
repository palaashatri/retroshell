# SLOPOS-I Verified Application Compatibility Audit V2 (`VERIFIED_APPLICATION_COMPATIBILITY_V2.md`)

**Date:** 2026-08-01  
**Status:** Mandatory Evidence-Level Application Audit  
**Scope:** Honest classification of native apps vs third-party Linux desktop packages.

---

## 1. Native Application Audit Matrix

| Application | Evidence Class | Unit Tests | Window Mapped | Interactive Test | Data Persistence | Audit Classification | Justification & Artifact Path |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| **Finder** | Level 4 | 21 / 21 | ✅ Yes | 🟡 Partial | ✅ Yes | **WINDOW MAPPED** | Spatial folder windows render floating (`02-finder-window.png`). Full interactive drag un-verified. |
| **TextEdit** | Level 4 | 20 / 20 | ✅ Yes | 🟡 Partial | ✅ Yes | **WINDOW MAPPED** | TextEdit editor window renders floating (`03-textedit-app.png`). Typing & save actions unit-tested. |
| **Terminal** | Level 4 | 17 / 17 | ✅ Yes | 🟡 Partial | N/A | **WINDOW MAPPED** | PTY terminal window renders floating (`04-terminal-app.png`). 2px top padding verified. |
| **Settings** | Level 4 | 14 / 14 | ✅ Yes | 🟡 Partial | ✅ Yes | **WINDOW MAPPED** | Settings theme panel renders floating (`05-settings-app.png`). Local TOML persistence verified. |
| **App Store** | Level 4 | 15 / 15 | ✅ Yes | 🟡 Partial | N/A | **WINDOW MAPPED** | App Store window renders floating (`06-appstore-app.png`). Catalog search unit-tested. |

---

## 2. Third-Party Linux Application Suite Matrix

| Application Package | Required Binary | VM Package Status | Highest Evidence Class | Audit Classification | Justification / Findings |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **GTK Demo** | `gtk3-demo` | ✅ **INSTALLED** | Level 6 | **PASS** | Version `3.24.52-0ubuntu1`, Wayland-1 protocol (`artifacts/de-readiness/20260731_190013/applications/gtk3-demo/`). |
| **MPV** | `mpv` | ✅ **INSTALLED** | Level 6 | **PASS** | Version `0.41.0-2ubuntu4`, Wayland-1 protocol (`artifacts/de-readiness/20260731_190016/applications/mpv/`). |
| **Firefox** | `firefox` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |
| **Doom (SDL2)** | `chocolate-doom` / `prboom-plus` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |
| **LibreOffice** | `libreoffice` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |
| **Qt Demo** | `qtmopen` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |
| **Electron** | `electron` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |
| **Java Swing** | `java` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |
| **Flatpak** | `flatpak` | ❌ NOT INSTALLED | None | **UNTESTED** | Package missing on VM image. |

---

## 3. Mandatory Classification Rules Applied

1. No native application is labelled `PASS` without Level 6/7 interactive execution evidence. All 5 native apps are accurately classified as `WINDOW MAPPED` (Level 4).
2. All 9 third-party packages are explicitly classified as `UNTESTED` due to package absence on the VM image.
