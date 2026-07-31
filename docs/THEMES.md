# Themes System — User-selectable appearance (planned)

> Feature backlog item #2. Design & implementation for a runtime theme system.

## Current state

- **Hardcoded themes:** `themes/{graphite,platinum,oled-graphite,high-contrast}/`
- **Selection:** `retro_sdk::render_dark_mode()` returns a bool; no UI to change it
- **Scope:** colors, typography, icons; not window decorations or layer chrome

## Goals

1. **User-selectable** — Settings app shows theme list, user picks one
2. **Hot-swappable** — theme changes without restart
3. **Per-theme assets** — icons, colors, fonts tied to each theme
4. **Persistent** — choice is remembered across sessions

## Design

### Theme structure (existing, no change needed)
```
themes/
├── graphite/
│   ├── colors.toml       # RGB palette
│   ├── icons/            # icon .png files
│   └── fonts/            # (optional) font overrides
├── platinum/
├── oled-graphite/
└── high-contrast/
```

### Theme metadata
Add `themes/manifest.toml`:
```toml
[[theme]]
id = "graphite"
name = "Graphite"
description = "Classic Mac gray"
category = "light"
author = "RetroShell Contributors"

[[theme]]
id = "platinum"
name = "Platinum"
description = "Platinum blue"
category = "light"
```

### Settings UI
- **Location:** Settings app → Appearance → Themes
- **Display:** grid of theme previews (name + small preview swatch)
- **Behavior:** click to select, immediately apply

### Implementation

#### Phase 1: Load & enumerate themes
- Scan `themes/` directory
- Parse manifest + color files
- Store active theme in `~/.config/retroshell/theme.toml`

#### Phase 2: Apply at startup
- Read `theme.toml` on shell startup
- Load colors/icons from the selected theme
- Fallback to "graphite" if config missing

#### Phase 3: Hot-swap
- Settings app calls shell via IPC (retro-bus)
- Shell updates all rendering contexts
- Clients (apps) receive a signal to re-render

#### Phase 4: Per-app theme overrides (future)
- Apps can opt-in to light/dark regardless of system theme
- Useful for apps with strong visual identity

## Technical notes

- Colors loaded from TOML (or JSON) at startup
- Icons loaded into texture atlas
- Hot-swap requires re-uploading wgpu textures + broadcasting event
- retro-bus may need to work first (defect H) for app notification

## Acceptance criteria

**Phase 1 (load):**
```bash
# Settings → Appearance shows list of available themes
# Each theme displays its name + a preview swatch
```

**Phase 2 (apply):**
```bash
# Change theme in Settings
# UI colors, fonts, icons immediately update
# Close + reopen Settings → theme persists
```

**Phase 3 (hot-swap):**
```bash
# Change theme while Finder and another app are running
# Both apps re-render with new colors instantly
# No crashes or black flashes
```

## Dependencies

- **retro-bus** (defect H) needs to work for client notification
- **retro-kit** color/styling system needs flexibility for theme swaps
- **Settings app** needs a preferences UI panel

## Timeline estimate
- **Phase 1:** 1-2 days (manifest + loader)
- **Phase 2:** 1 day (startup integration)
- **Phase 3:** 2-3 days (hot-swap via retro-bus)

**Total:** 1 week for basic functionality.
