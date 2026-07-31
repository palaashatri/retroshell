# Mandatory 7-Tier Evidence Levels Specification (`EVIDENCE_LEVELS.md`)

**Date:** 2026-08-01  
**Status:** Authoritative Evidence Taxonomy  
**Scope:** Defines the mandatory 7 evidence classes required to justify capability and application statuses.

---

## 1. Evidence Hierarchy

| Level | Evidence Class | Description | Acceptable Status Assignment |
| :-: | :--- | :--- | :--- |
| **1** | `SOURCE PRESENT` | Source code files, functions, or structs exist in the repository tree. | `SPECIFICATION ONLY` / `PLANNED` |
| **2** | `UNIT TEST PASSED` | Isolated Rust unit tests passed (`cargo test`). Does NOT prove runtime desktop execution. | `UNIT-TESTED` |
| **3** | `APP PROCESS LAUNCHED` | Executable process spawned and visible in process tree (`ps aux`). | `PROCESS LAUNCHED` |
| **4** | `WINDOW MAPPED` | Surface/window mapped to compositor layer or display server. | `WINDOW MAPPED` |
| **5** | `SCREENSHOT OBSERVED` | Static screenshot image captured (`grim` / `wlr-screencopy`). | `SCREENSHOT OBSERVED` |
| **6** | `INTERACTIVE RUNTIME TEST PASSED` | Live interactive user input, pointer motion, dragging, typing, or audio tested cleanly. | **`VERIFIED`** |
| **7** | `END-TO-END SESSION TEST PASSED` | Full session startup, multi-app workflow, screen lock, logout, and process tree teardown verified. | **`VERIFIED`** |

---

## 2. Strict Status Rules

- **Only Levels 6 and 7** may justify `VERIFIED` for interactive desktop capabilities or application functionality.
- Unit tests (Level 2) justify `UNIT-TESTED`, never runtime `VERIFIED`.
- Screenshots showing a window (Level 5) justify `WINDOW MAPPED`, never application `PASS`.
