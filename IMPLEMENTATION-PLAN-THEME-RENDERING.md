# Implementation Plan: Connect Theme System to Rendering

## Problem Statement

RetroShell has:
- ✅ Complete rendering system with beveled 3D effects (in retro-sdk)
- ✅ Complete theme token system (in retro-shell)
- ❌ **No connection between them** - rendering uses hardcoded colors

Current state:
- `draw_widget()` in retro-sdk/src/lib.rs renders all UI with hardcoded RGB
- `ThemeContext` in retro-shell/src/theme_manager.rs has all color tokens
- `apply_theme()` sets dark-mode + accent, but no other colors propagate
- Result: Changing theme colors has no visual effect

## Root Cause Analysis

1. **Rendering code** uses hardcoded colors like `rgb(222, 222, 218)` throughout
2. **Theme system** exists separately with tokens like `WindowBackground`, `ButtonBackground`, etc.
3. **No bridge** between the two systems
4. **Architecture is sound** - no refactoring needed, just wiring

## Solution Approach

### Phase 1: Create Color Mapping System
Wire theme tokens into rendering using a lookup system.

**Strategy:**
1. Store `ThemeContext` in a thread-local or global during `apply_theme()`
2. Create lookup functions that map rendering concepts to theme tokens
3. Replace hardcoded `rgb()` with `theme_color()` calls

**Implementation:**
- Add thread-local `CURRENT_THEME` (set by `apply_theme()`)
- Create mapping layer: "button_background" → `ThemeToken::ButtonBackground`
- Modify draw functions to use `get_ui_color(token)` instead of `rgb(x,y,z)`

### Phase 2: Update Drawing Code
Replace hardcoded colors in draw functions with theme-aware lookups.

**Functions to update:**
- `draw_widget()` - buttons, text fields, sliders, menus
- `draw_window()` - window background, borders, shadows
- `draw_classic_titlebar()` - title bar colors
- `draw_dialog()` - dialog backgrounds
- `draw_menu_bar()` - menu styling
- `draw_dock_view()` - dock appearance
- All supporting draw functions

**Pattern:**
```rust
// Before:
let bg = rgb(222, 222, 218);  // hardcoded

// After:
let bg = get_theme_color(ThemeToken::ButtonBackground);  // from theme
```

### Phase 3: Test & Verify

**Testing:**
1. Verify colors change when theme changes
2. Test light vs dark modes
3. Test different theme variants (Classic, Graphite, etc.)
4. Verify beveling still works with new colors
5. Check contrast ratios for accessibility

## Detailed Design

### Color Lookup Architecture

```
ThemeContext (in retro-shell) 
    ↓ apply_theme()
CURRENT_THEME (global state in retro-sdk)
    ↓ get_theme_color(token)
[f32; 4] RGB value
    ↓ ui() helper
Correct light/dark variant
    ↓ canvas.rect()
Pixel on screen
```

### Implementation Steps

#### Step 1: Create theme color access layer
File: `crates/retro-sdk/src/lib.rs`

Add at the top level:
```rust
// Global theme state (set by apply_theme from retro-shell)
static THEME_TOKEN_COLORS: Mutex<HashMap<String, [f32; 4]>> = /* ... */;

fn set_theme_color(token_name: &str, light: [f32; 4], dark: [f32; 4]) {
    let color = if render_dark_mode() { dark } else { light };
    THEME_TOKEN_COLORS.lock().insert(token_name.to_string(), color);
}

fn get_theme_color(token_name: &str) -> [f32; 4] {
    THEME_TOKEN_COLORS.lock()
        .get(token_name)
        .copied()
        .unwrap_or_else(|| [0.5, 0.5, 0.5, 1.0])  // fallback gray
}
```

#### Step 2: Update apply_theme to populate colors
Modify `apply_theme()` to map theme tokens to rendering colors:
```rust
pub fn apply_theme(is_dark: bool, accent: [f32; 4]) {
    set_render_dark_mode(is_dark);
    set_render_accent(accent);
    
    // Map theme tokens to colors
    // (These would normally come from retro-shell, 
    //  for now use current hardcoded defaults as fallback)
    set_theme_color("window_bg", /* light */, /* dark */);
    set_theme_color("button_bg", /* light */, /* dark */);
    // ... etc
}
```

#### Step 3: Replace hardcoded colors in draw_widget
Pattern in `draw_widget()`:

**Before:**
```rust
let bg = if button.widget_state().hovered {
    ui(rgb(226, 235, 246), rgb(70, 76, 84))
} else {
    ui(rgb(222, 222, 218), rgb(58, 60, 64))
};
canvas.rect(rect, bg);
```

**After:**
```rust
let base_bg = get_theme_color("button_bg");
let hover_bg = get_theme_color("button_bg_hover");
let bg = if button.widget_state().hovered { hover_bg } else { base_bg };
canvas.rect(rect, bg);
```

#### Step 4: Create mapping table
Map human-readable names to theme concepts:
```rust
const COLOR_MAP: &[(&str, &str)] = &[
    // Widget backgrounds
    ("button_bg", "ButtonBackground"),
    ("button_bg_hover", "ButtonHighlight"),
    ("window_bg", "WindowBackground"),
    ("text_primary", "TextPrimary"),
    // ... etc
];
```

This allows `draw_widget()` to use simple names while mapping to actual theme tokens.

#### Step 5: Wire theme system end-to-end
In retro-shell (layer_desktop.rs):
- When `apply_theme()` is called, pass full ThemeContext or color map
- Populate SDK's theme color cache with actual token values
- Trigger full redraw so new colors take effect

## Files to Modify

1. **crates/retro-sdk/src/lib.rs** (main work ~200-300 lines)
   - Add color lookup system
   - Update apply_theme()
   - Replace hardcoded colors in draw_* functions (~40-50 locations)
   - Add COLOR_MAP constant

2. **crates/retro-shell/src/layer_desktop.rs** (~20 lines)
   - Pass theme colors to SDK when apply_theme is called
   - Or create API that SDK can query

3. **crates/retro-shell/src/lib.rs** (~10 lines)
   - Export theme token values
   - Create function to get colors from ThemeContext

## Benefits of This Approach

1. **Non-breaking** - Existing rendering code structure unchanged
2. **Incremental** - Can migrate function by function
3. **Maintainable** - All color definitions in one place (theme_manager.rs)
4. **Extensible** - New themes automatically work
5. **Debuggable** - Color map is explicit and visible
6. **Fallback-safe** - Has sensible defaults for missing colors

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Colors don't update | Use global state that gets set before every paint |
| Lookup performance | Cache colors after theme change, not per-draw |
| Accessibility broken | Maintain same contrast as hardcoded originals |
| Backward compat | Keep fallback to hardcoded if lookup fails |

## Success Criteria

- [ ] Theme colors visually change when theme is switched
- [ ] All 8 theme variants work (Classic, Dark, Grape, Blueberry, Strawberry, Solarized, Dracula, HighContrast)
- [ ] Light and dark modes both work
- [ ] Beveling still appears correctly
- [ ] No regression in other UI rendering
- [ ] Accessibility standards maintained
- [ ] All 316 tests still pass

## Estimated Effort

- **Research**: 30 min (this document)
- **Implementation**: 2-3 hours
- **Testing**: 1 hour
- **Debugging/refinement**: 1-2 hours
- **Total**: 4-7 hours

## Why This Is the Right Approach

1. **Respects existing architecture** - No refactoring of rendering
2. **Minimal code changes** - Just color lookups
3. **Maximum impact** - Makes theme system actually work
4. **Foundation for future** - Enables:
   - Custom themes in Settings
   - Theme hot-swapping
   - Per-app color schemes
   - Color customization UI

## Next Steps

1. ✅ Complete Phase 1: Root cause analysis (DONE)
2. ✅ Complete Phase 2: Design (DONE - this document)
3. → Phase 3: Implementation
   - Start with core color lookup system
   - Migrate draw_widget() button rendering
   - Test on real desktop
   - Expand to other draw functions
   - Full integration testing
4. → Phase 4: Verification
   - Test all theme variants
   - Verify contrast ratios
   - Test keyboard focus indicators
   - Test disabled states

This plan connects the existing, working rendering system to the existing, working theme system. No architecture changes needed - just wiring.
