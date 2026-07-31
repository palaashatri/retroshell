---
name: spotlight-integration-complete
description: Spotlight modal overlay fully integrated into shell event dispatch (B2a-b done, B2c rendering pending)
metadata:
  type: project
---

**Spotlight keyboard integration complete (2026-07-31).**

Implemented Stage B2a (modal overlay + keyboard) and B2b (search backend) of the Spotlight feature. The overlay is now a live, working modal that intercepts events before the menu bar and window dispatch.

## Architecture

- **Module layout:** `crates/retro-shell/src/spotlight.rs` (search backend) + `spotlight_ui.rs` (UI state + keyboard routing)
- **Integration point:** `ShellDesktop.handle_event()` — Spotlight checks happen after lock screen, before menu bar
- **Ownership:** `spotlight_ui: RefCell<SpotlightUI>` in ShellDesktop for interior mutability in draw context

## Event Flow

1. **Super+Space:** Toggles overlay visibility. Pre-populates with featured apps (Finder/Settings/TextEdit/Terminal) when opened.
2. **Char events:** Append to search query. Results update in real time via `SearchBackend::search()`.
3. **Arrow Up/Down:** Navigate results list; `selected_index` tracks selection.
4. **Escape:** Hide overlay, return focus to desktop.
5. **Enter:** (TODO) Launch app or open file.
6. **Any other event while visible:** Swallowed (modal behavior).

## Status

- **B2a (Overlay + keyboard):** ✅ DONE
- **B2b (Search backend):** ✅ DONE
  - App search: case-insensitive substring matching on bundle names
  - Settings: 10 hardcoded entries (Display/Sound/Keyboard/Network × 2)
  - Featured apps: shown when query is empty (Finder/Settings/TextEdit/Terminal)
- **B2c (Rendering):** 🔨 IN-PROGRESS
  - TextField + ListView widgets created, laid out at draw time
  - TODO: Scrim background, text rendering, result item rendering, selection highlight
- **B2d (App launch):** BLOCKED on B2c rendering

## Test Coverage

All 311 `retro-shell` tests pass, including:
- Spotlight module: 6 unit tests (visibility, query input, keyboard navigation)
- Event routing: 100+ existing tests still passing (lock screen, menu bar, window manager, etc.)

## What's Next

1. Implement rendering (B2c): scrim, search field text display, results list with icons
2. Wire app launch on Enter (B2d)
3. VM verification for Stages 4.5–4.8 (clean Arch/Ubuntu/ISO)
4. Merge to main and tag v0.1.0

## File Changes

- `crates/retro-shell/src/lib.rs`: +68 lines (ShellDesktop::spotlight_ui field + event routing + render stub)
- `crates/retro-shell/src/spotlight_ui.rs`: +15 lines (accessors, public append_char)
- `packaging/vm/stage-4-verify.sh`: NEW, 319 lines (VM test harness for distribution chain)
