# Spotlight Search — Global launcher & search (planned)

> Feature backlog item #1. Design & implementation plan for a system-wide
> launcher/search interface inspired by macOS Spotlight.

## Goals

1. **Quick app launch** — type an app name, hit Enter
2. **File search** — search `~/` for files by name
3. **Settings navigation** — search settings and jump to them
4. **Efficient** — incremental search (no lag), show top results
5. **Integrated** — uses shell-owned overlay surface (layer-shell), no app window

## Design

### Invocation
- **Keyboard shortcut:** `Super+Space` (command-space on classic Mac)
- **Fallback:** accessible from menu bar or dock if keyboard disabled
- **Behavior:** overlay appears centered, search field focused; full-height or large modal

### Search scope (priority order)
1. **Installed apps** — from `launch_services::scan_applications()`
2. **Files** — `$HOME` and `~/Desktop` (incremental, ~0.5s)
3. **Settings** — hardcoded entries (Settings app pages, display, sound, etc.)

### UI layout
```
┌─────────────────────────────────────┐
│         RetroShell Search           │
├─────────────────────────────────────┤
│ [text input field]                  │
├─────────────────────────────────────┤
│  🔎 Finder          Open File...    │
│  ⚙️ Settings        Display...      │
│  🎹 Terminal        Open Terminal   │
│                                     │
│     (no results for "xyz")          │
├─────────────────────────────────────┤
│  Type to search... (?) Cmd+Space again to hide
└─────────────────────────────────────┘
```

### Result types
- **Apps:** icon + name + (optional description)
- **Files:** folder icon + path + file type
- **Settings:** gear icon + setting name + category

### Behavior
- **Live search:** results update as you type (debounced ~100ms for fs)
- **Arrow keys:** navigate results
- **Enter:** launch/open selected result
- **Escape:** close search
- **Mouse:** click to select

## Implementation stages

### Stage A: Shell infrastructure (retro-shell)
- Add a `spotlight` module to manage the search UI state
- Define keyboard shortcut handling (Super+Space)
- Create overlay surface via layer-shell (z-order: above normal, below exclusive)
- Emit input events (typing, arrow keys, Enter, Esc)

### Stage B: Search backend (retro-shell)
- Query `launch_services::scan_applications()` for app results
- Implement file search in `$HOME` (non-blocking via a background thread)
- Hardcode settings entries

### Stage C: Search UI (retro-sdk + retro-kit)
- Build a results list widget in `retro-kit::list_view` or custom
- Display app icons (from the theme)
- Show file previews / highlights
- Theme-aware styling (fonts, colors, focus behavior)

### Stage D: Integration
- Polish (animations, responsiveness, keyboard focus)
- Add result caching to avoid repeated disk searches
- Optional: history / recently searched

## Known constraints

- Files search must **not** block the UI (use async/rayon)
- Must respect `$HOME` and not scan system directories  
- Icons must be available from the installed theme
- Keyboard must work reliably (modifier key handling via sctk)
- Overlay layer must not exceed output bounds (layer-shell constraints)

## Defect risk

The retro-kit interaction layer (defect J) is partly dead. If buttons/list clicks
are inert, the mouse interaction in results won't work until **that** is fixed.
Start with keyboard-only if needed.

## Acceptance criteria (per-stage)

**Stage A:** Overlay appears on Super+Space, input captured, Esc closes it.
```bash
# On the VM: press Super+Space, type "finder", press Esc.
# Verify: overlay appears/disappears without crashing shell or dropping clients.
```

**Stage B:** Search returns apps + files.
```bash
# Type "find" → Finder app appears in results
# Type "texte" → TextEdit app appears
# Type "doc" → files matching ~/*/doc* appear
```

**Stage C:** Results render with icons and styling.
```bash
# Visual verification: results show app icons + colors + fonts match theme
```

**Stage D:** Full flow works (launch app or open file).
```bash
# Type "finder", press Enter → Finder launches
# Type a file path, press Enter → file opens (if handler exists)
```

## Timeline estimate
- **Stage A:** 1-2 days (layer-shell surface + keyboard)
- **Stage B:** 2-3 days (search impl + threading)
- **Stage C:** 2-3 days (UI layout + rendering)
- **Stage D:** 1 day (integration + polish)

**Total:** 1–2 weeks for a MVP (keyboard-only, no file icons initially).
