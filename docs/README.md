# SLOPOS-I Documentation Directory Index (`docs/README.md`)

This directory contains all architecture, audit, readiness, and capability specification documents for SLOPOS-I.

---

## 📁 Audit & Verification Documents (`docs/audit/`)
- 📄 **[`EVIDENCE_LEVELS.md`](audit/EVIDENCE_LEVELS.md)** — Mandatory 7-tier evidence taxonomy specification.
- 📄 **[`RUNTIME_TOPOLOGY.md`](audit/RUNTIME_TOPOLOGY.md)** — Display server sockets, process trees, and protocol graph.
- 📄 **[`CURSOR_RUNTIME_EVIDENCE.md`](audit/CURSOR_RUNTIME_EVIDENCE.md)** — Topmost diagnostic magenta cursor pass & pixel verification.
- 📄 **[`WINDOW_MOVE_RUNTIME_EVIDENCE.md`](audit/WINDOW_MOVE_RUNTIME_EVIDENCE.md)** — Titlebar drag & compositor surface relocation log.
- 📄 **[`DE_READINESS_AUDIT.md`](audit/DE_READINESS_AUDIT.md)** — Desktop Environment Readiness Assessment.
- 📄 **[`DOCUMENT_CONSISTENCY_AUDIT.md`](audit/DOCUMENT_CONSISTENCY_AUDIT.md)** — Claim reconciliation matrix.
- 📄 **[`RENDERING_AUDIT.md`](audit/RENDERING_AUDIT.md)** — Comprehensive rendering pipeline bug hunt.
- 📄 **[`STANDALONE_SESSION_EVIDENCE.md`](audit/STANDALONE_SESSION_EVIDENCE.md)** — Standalone compositor backend failure trace.

---

## 📁 Capability & Compatibility Matrices (`docs/matrices/`)
- 📄 **[`VERIFIED_CAPABILITY_MATRIX_V2.md`](matrices/VERIFIED_CAPABILITY_MATRIX_V2.md)** — Runtime evidence-level capability matrix (v2).
- 📄 **[`VERIFIED_APPLICATION_COMPATIBILITY_V2.md`](matrices/VERIFIED_APPLICATION_COMPATIBILITY_V2.md)** — Runtime evidence-level application compatibility matrix (v2).
- 📄 **[`VERIFIED_SETTINGS_MATRIX.md`](matrices/VERIFIED_SETTINGS_MATRIX.md)** — Honest Settings control classification matrix.
- 📄 **[`CAPABILITY_MATRIX.md`](matrices/CAPABILITY_MATRIX.md)** — Historical capability specification.
- 📄 **[`APPLICATION_COMPATIBILITY.md`](matrices/APPLICATION_COMPATIBILITY.md)** — Historical application compatibility matrix.

---

## 📁 System Architecture Specifications (`docs/architecture/`)
- 📄 **[`CURRENT_ARCHITECTURE.md`](architecture/CURRENT_ARCHITECTURE.md)** — Component ownership & login process hierarchy.
- 📄 **[`COORDINATE_SYSTEMS.md`](architecture/COORDINATE_SYSTEMS.md)** — Formal 10-coordinate-space matrix & mathematical conversion rules.
- 📄 **[`TEXT_RENDERING.md`](architecture/TEXT_RENDERING.md)** — Typography model, glyph bearing, and baseline specification.
- 📄 **[`COMPOSITOR_ARCHITECTURE.md`](architecture/COMPOSITOR_ARCHITECTURE.md)** — 10-tier z-index layer ordering & window frame boundaries.
- 📄 **[`SETTINGS_BACKEND_ARCHITECTURE.md`](architecture/SETTINGS_BACKEND_ARCHITECTURE.md)** — Typed Rust service interfaces for Settings panels.
- 📄 **[`DAILY_DRIVER_ROADMAP.md`](architecture/DAILY_DRIVER_ROADMAP.md)** — Milestones M0 through M5.
- 📄 **[`TEST_HARNESS.md`](architecture/TEST_HARNESS.md)** — Automated Evidence Harness specification.
