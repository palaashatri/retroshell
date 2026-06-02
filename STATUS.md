# RetroShell — Project Status

## Summary

Phase 1 scaffolding is complete: a multi-crate Rust workspace with all components defined, all core types in place, and 3 demo applications with full menu structures. The implementation is a structural skeleton — modules, types, and interfaces match the RFCs, but most components lack runtime integration.

## Workspace

| Crate | Path | Status |
|-------|------|--------|
| retro-render | `crates/retro-render/` | Scaffold — wgpu device init, texture, surface, shader, font, theme_renderer types defined |
| retro-kit | `crates/retro-kit/` | Scaffold — all 17 widget types, layout, event, accessibility, theme system defined |
| retro-shell | `crates/retro-shell/` | Scaffold — all 10 services defined with their interfaces |
| retro-bus | `crates/retro-bus/` | Scaffold — message types, service registry, transport trait |
| retro-sdk | `crates/retro-sdk/` | Scaffold — Application struct with menus, bus, window |

## Applications

| App | Path | Status |
|-----|------|--------|
| Finder | `apps/finder/` | Menu bar + sidebar tree + icon view + split view layout defined |
| Settings | `apps/settings/` | Menu bar + category tree + appearance panel defined |
| TextEdit | `apps/textedit/` | Menu bar + toolbar + text field + scroll view defined |
| Terminal | *missing* | Priority 1 per AGENTS.md — not implemented |

## Themes

| Theme | Files | Status |
|-------|-------|--------|
| Platinum | `themes/platinum/` | Theme.toml, Colors.toml, Metrics.toml, Typography.toml + in-code palette |
| Graphite | `themes/graphite/` | Same structure |
| OLED Graphite | `themes/oled-graphite/` | Same structure |
| High Contrast | `themes/high-contrast/` | Same structure |

All 4 themes have both TOML definitions and in-code `ThemeManager` palettes with light/dark/HDR token support.

## Re-verification Against Spec

### Implemented ✅

- Workspace structure with all 5 crates
- All widget types from RFC-0001 defined (Window, Button, Menu, ListView, TreeView, IconView, TextField, Toolbar, Dialog, ScrollView, SplitView, Label, ProgressBar, Slider)
- All RetroShell services from RFC-0002 defined (MenuServer, WindowManager, DesktopManager, Dock, NotificationCenter, WorkspaceManager, LaunchServices, SessionManager, ThemeManager, ApplicationRegistry)
- RetroRender with wgpu initialization and clear rendering
- RetroBus with message, service registry, and transport abstractions
- SDK with Application struct, AppDelegate trait
- 4 themes with token-based color systems (both TOML files and in-code)
- Menu definitions with keyboard shortcuts in all 3 apps
- Window lifecycle management (create, close, focus, minimize, maximize, fullscreen, move, workspace assign)
- Theme switching API
- Application scanning in LaunchServices
- build.sh for Ubuntu/Vulkan dependency setup
- .gitignore excluding target/

### Missing / Needs Work ❌

1. **Terminal app** — Priority 1, not implemented
2. **Tests** — Zero tests found. RFCs require unit + integration + visual regression tests for every feature
3. **App bundles** — No `.app` directory structure, no `App.toml` per RFC-0004
4. **Wayland integration** — No display protocol; `Renderer::new()` takes a `wgpu::Surface` but nothing creates it
5. **Event loop** — `retro-shell::run()` and `Application::run()` are no-ops (just set `running = true`)
6. **Widget rendering** — Widget `draw()` methods are defined but not wired to RetroRender; no render tree
7. **Menu bar rendering** — MenuServer manages menu state but no actual rendering
8. **Dock rendering** — Dock struct defined but no rendering
9. **Desktop background** — DesktopManager defined but no wallpaper/icon rendering
10. **Notification UI** — NotificationCenter defined but no UI
11. **Workspace switching** — WorkspaceManager defines state but no switching logic
12. **Session management** — SessionManager defines state but no login/logout
13. **File operations** — Finder app defines menus but no actual file operations
14. **Drag and drop** — Event types defined but no DnD implementation
15. **Clipboard** — Not implemented
16. **Search** — Not implemented
17. **HDR/VRR** — Theme types support HDR values but no actual HDR pipeline
18. **Accessibility** — Types defined but no screen reader integration
19. **IPC** — RetroBus defines message types but transport trait has no real implementation
20. **Theme.toml** — Missing `identifier` field per RFC-0005 spec
21. **Icons/Assets** — No icon directories in themes per RFC-0005

## Build

The project was last built successfully with `cargo build`. The `build.sh` script installs Ubuntu dependencies for Vulkan/Wayland.

```sh
# Linux (Ubuntu)
./build.sh
cargo build

# macOS (development only — RetroShell targets Linux)
cargo build
```

## Next Steps

1. Create Terminal app (Priority 1)
2. Add `.app` bundle structure with `App.toml` per RFC-0004
3. Wire up Wayland surface creation and event loop
4. Connect widget draw() to render tree and RetroRender
5. Implement menu bar rendering in MenuServer
6. Implement Dock rendering
7. Add tests across all crates
8. Implement file operations in Finder
9. Add icon assets to themes

## Recent Commits

```
095db3d Initial RetroShell implementation: multi-crate workspace (78 files, +8081)
67eae4f Add .gitignore
```
