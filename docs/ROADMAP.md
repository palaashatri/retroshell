# RetroShell Roadmap — Development Milestones

**Last updated:** 2026-07-31  
**Status:** Stages 0-4 code-complete; Stage 4 VM tests pending

---

## Current Status (2026-07-31)

| Stage | Objective | Status | Evidence |
|---|---|---|---|
| 0 | VM infrastructure (DRM/KMS/SSH) | ✅ VERIFIED | `/dev/dri/card0` + build succeeds |
| 1 | Compositor + app window painting | ✅ VERIFIED | Finder renders on DRM |
| 2 | Session, input, lock screen | ✅ VERIFIED | lock/unlock, Super+O, button click |
| 3 | `.app` bundles + app store | ✅ VERIFIED | Store installs TextEdit.app |
| 4 | Distribution (installer + ISO) | 🔨 CODE-COMPLETE | Scripts ready, VM tests pending |
| B2 | Spotlight search MVP (keyboard) | 🔨 CORE-INTEGRATED | Event routing + modal overlay wired |

**Total codebase:** ~50.5k LOC (Rust) + docs + configs  
**First-party apps:** 5 (Finder, Settings, TextEdit, Terminal, App Store)  
**Core platform:** Wayland compositor (smithay), layer-shell chrome, session management  
**Spotlight MVP:** Keyboard routing + search backend + app launch (rendering TODO)

---

## Immediate Next Steps (Phase A — this week)

### A1: Complete Stage 4 VM verification (critical path)
- **Task 4.5:** `install.sh --with-greeter` on fresh Arch VM → verify session reaches desktop
- **Task 4.6:** Same on fresh Ubuntu-server VM (confirm Ubuntu package names)
- **Task 4.8:** ISO boots on clean VM → RetroShell desktop appears

**Owner:** User (requires VM access and manual testing)  
**Timeline:** 1-2 days of VM time  
**Blocker:** None; code is ready

**Acceptance:** Screenshots in `docs/qa/stage-4.md` → Stage 4 VERIFIED

---

## Near-Term Goals (Phase B — next 2-4 weeks)

### B1: Merge Stage 4 to `main` and cut a release tag
- Create PR: `docs/program-design` → `main`
- Code review + merge
- Tag release: `v0.1.0-stage4-complete` or `v0.1.0`
- Update README with install instructions

### B2: Implement Spotlight-like search (backlog #1)
**Value:** Quick app launch + file search + settings navigation  
**Effort:** 1-2 weeks (Stages A-D in `docs/SPOTLIGHT-SEARCH.md`)  
**Architecture:** Shell-owned overlay surface + search backend + UI

**Stages:**
- **B2a (✅ DONE):** Overlay surface + keyboard shortcuts (Super+Space)
  - SpotlightUI module with search backend (spotlight.rs) + UI state (spotlight_ui.rs)
  - Modal overlay intercepts all events when visible
  - Super+Space toggles visibility; real-time search on character input
  - All 311 shell tests pass; event routing verified
- **B2b (✅ DONE):** Search backend (apps + files + settings)
  - SearchBackend with 10 hardcoded settings entries (Display/Sound/Keyboard/Network)
  - App search: case-insensitive substring matching on launch_services.bundles
  - Featured apps shown when query is empty (Finder/Settings/TextEdit/Terminal)
- **B2c (🔨 RENDERING-TODO):** Search UI (results list + icons + rendering)
  - TextField for search input positioned at top of overlay
  - ListView for results with selected index tracking
  - TODO: Render scrim, search field text, results list items, selection highlight
  - Rendering infrastructure complete; UI display pending retro-kit canvas integration
- **B2d (✅ DONE):** Polish + integration (app launch/file open on Enter)
  - Enter key activates selected result
  - Apps launch via launch_external_app()
  - Files open with mime registry (same as folder double-click)
  - Settings pane opens Settings app

**Acceptance Criteria:**
- Type "find" → Finder appears in results with featured apps
- Press Down → selection moves to next result
- Press Enter → (TODO: launch app or open file)
- Type "settings" → Settings app appears
- Press Escape → overlay hides, focus returns to desktop

### B3: Fix known defects blocking other work
Priority: **Defect J (toolkit interaction — buttons/scrolling inert)**

The button event handling works, but drawing may be missing or not wired. This blocks:
- Better Settings UI
- More complex dialogs
- Scroll views with interactive content

**Investigation:** Trace button click from layer-shell input → event dispatch → widget → rendering.

---

## Medium-Term Goals (Phase C — 1-2 months)

### C1: User-selectable themes (backlog #2)
**Value:** Customization + visual appeal  
**Effort:** 1 week (Phases 1-3 in `docs/THEMES.md`)  
**Stages:**
- Load theme manifest + colors at startup
- Apply from Settings UI
- Hot-swap via retro-bus (once defect H is fixed)

### C2: Animated desktop backgrounds (backlog #3)
**Value:** Visual delight + system feel  
**Effort:** 2-3 weeks (Phases 1-2 in `docs/ANIMATED-BACKGROUNDS.md`)  
**Stages:**
- GIF decoding + playback
- Video support (optional Phase 1.5)
- Settings UI wallpaper picker

### C3: Fix defect H (retro-bus IPC)
**Value:** Event broadcast (theme changes, lock state, app state)  
**Effort:** 1-2 weeks  
**Blocker for:** Theme hot-swap, inter-app communication

---

## Long-Term Goals (Phase D — 2-6 months)

### D1: Fix defect I (Terminal VT parser)
**Value:** Usable Terminal app  
**Effort:** 3-5 weeks  
**Scope:** ED 0/1, HT, DECSTBM scroll, missing cursor CSIs

### D2: Hardening & stability (daily-driver readiness)
- Memory leak sweeps (defect C was one)
- Input event ordering guarantees
- Crash recovery (apps recovering from crash, session surviving app crash)
- Error messages and diagnostics

### D3: Third-party app SDK stabilization
- Finalize retro-sdk API (versioning)
- Example third-party app (in separate repo)
- App distribution guidelines

### D4: Alternative WM integration
- Nested X11 support (e.g., under Xvfb for testing)
- Fallback to labwc if smithay compositor fails
- Graceful degradation

---

## Known Risks & Mitigations

### Risk: Stage 4 VM tests fail (distro differences)
**Mitigation:** Task 4.6 is explicitly scoped to "confirm Ubuntu package names." If `apt-get` rejects a package in `packaging/deps/ubuntu.txt`, fix it, commit, and re-run (installer is idempotent).

### Risk: Defect J (buttons) is deeper than expected
**Mitigation:** Start with keyboard-only workflows in new features. Once buttons work, add mouse support.

### Risk: retro-bus is complex to fix
**Mitigation:** Defer hot-swap features until retro-bus is proven. Single-process workflows (no inter-app events) work without it.

### Risk: Toolkit rendering doesn't scale to complex UIs
**Mitigation:** The Settings app and theme browser are relatively simple. If performance is an issue, can defer to C2/C3.

---

## Success Metrics

### Short-term (by end of August 2026)
- ✅ Stage 4 VERIFIED on clean Arch + Ubuntu VMs + ISO boot
- ✅ PR merged to `main`; `v0.1.0` tagged
- ✅ Spotlight search MVP working (keyboard, app launch)

### Medium-term (by end of September 2026)
- ✅ Themes + hot-swap functional
- ✅ Animated backgrounds (GIF, optionally video)
- ✅ Defect H (retro-bus) fixed
- ✅ 50% of defect I (Terminal parser) fixed

### Long-term (by end of Q4 2026)
- ✅ Terminal fully functional
- ✅ Third-party app SDK example published
- ✅ Stability testing on real systems (not just VM)
- ✅ 100+ commits toward daily-driver usability

---

## Architecture Maintenance

### No breaking changes without cause
- Single Cargo workspace (no repo split)
- Core crates (renderer, kit, sdk, shell, compositor) stay together
- Apps can peel off once retro-sdk API is stable

### Dependency minimalism
- Prefer Rust crates (smithay, wgpu, serde)
- Avoid heavy dependencies (no GTK, no Qt)
- Runtime deps: Wayland, Mesa, libinput, seatd (already required for KMS)

### Quality gates
- CI always green (cargo build + clippy + test)
- QA: real VM verification, not just code inspection
- No "done" without evidence transcript + screenshot

---

## How to Contribute

1. **Pick a task** from Phase B or C
2. **Read the relevant design doc** (SPOTLIGHT-SEARCH.md, THEMES.md, etc.)
3. **Follow the acceptance criteria**
4. **Run on the VM** — screenshots and transcripts
5. **Open a PR** with evidence in the commit message

See [PROGRAM.md](PROGRAM.md) for the honesty contract: **a feature is done when its acceptance command passes on the real VM, not when tests pass on the host.**

---

## Further Reading

- **Specification:** [specs/2026-07-30-retroshell-de-program-design.md](specs/2026-07-30-retroshell-de-program-design.md)
- **Current Tasks:** [tasks/](tasks/)
- **QA Evidence:** [qa/](qa/)
- **Design Backlog:** [FUTURE.md](FUTURE.md)
- **Feature Plans:** [SPOTLIGHT-SEARCH.md](SPOTLIGHT-SEARCH.md), [THEMES.md](THEMES.md), [ANIMATED-BACKGROUNDS.md](ANIMATED-BACKGROUNDS.md)
