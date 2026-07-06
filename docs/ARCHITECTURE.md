# RetroShell Architecture

---

## Crate Dependency Graph

```
retro-compositor (future Smithay compositor — Linux only, not yet integrated)

retro-shell  ──────────────────────────────────────────────────────────┐
  │  depends on:                                                       │
  ├── retro-sdk   ──────────┐                                          │
  │     │ depends on:       │                                          │
  │     ├── retro-kit ──────┤                                          │
  │     │     │ depends on: │                                          │
  │     │     └── retro-render                                         │
  │     ├── retro-bus       │                                          │
  │     └── retro-render    │                                          │
  ├── retro-kit             │                                          │
  ├── retro-render          │                                          │
  └── retro-bus             │                                          │
                            │                                          │
apps/finder                 │   (each app links retro-sdk)             │
apps/settings               │                                          │
apps/textedit               │                                          │
apps/terminal               │                                          │
apps/appstore  ─────────────┘                                          │
                                                                       │
External: winit → wgpu → Vulkan/Wayland backend ────────────────────── ┘
External: labwc (Wayland compositor, not part of this repo)
External: cosmic-text / ab_glyph (font rasterization)
External: zbus / D-Bus (retro-bus optional feature)
External: nix (PTY fork/exec in Terminal)
```

---

## Crate Responsibilities

### retro-render

Low-level graphics pipeline. Owns the wgpu device, surface, and render pass.

- Initializes the wgpu instance targeting Vulkan on Linux (falls back to whatever
  wgpu selects: DX12, Metal, GL).
- Provides a `Canvas` abstraction for immediate-mode drawing: filled rectangles,
  borders, text glyphs, icons.
- Text rasterization uses `cosmic-text` for font database/shaping and `ab_glyph`
  for per-glyph TrueType rasterization. A 5x7 pixel bitmap font is the fallback.
- NDC (Normalized Device Coordinates) translation happens here so that widget
  coordinates expressed in logical pixels are converted to GPU clip-space.
- Color format selection probes the surface for `Rgba16Float` (HDR) or
  `Rgb10a2Unorm` first; falls back to `Bgra8UnormSrgb`. No SDR-to-HDR
  tonemapping is applied yet.

### retro-kit

Native widget toolkit. No wgpu calls; it delegates all drawing to the active `Canvas`
via `retro-render`.

Widgets provided:
- `Button`, `Label`, `TextField` — basic controls
- `IconView` — spatial icon grid (Finder uses this for file browsing)
- `ListView` — scrollable list with selection state
- `MenuBar`, `Menu` — menu bar and pull-down menus
- `TabView` — tabbed content with clickable headers
- `Slider` — draggable slider with mouse-down/move handling
- `ProgressBar` — beveled fill track
- `PopupButton` — beveled drop-down control with arrow indicator
- `Dialog` — modal sheet with title, message, and action buttons
- `ScrollView` — scrollable container
- `SplitView` — resizable pane pair
- `TreeView` — hierarchical list
- `Toolbar`, `StatusBar` — chrome areas
- `DockView` — bottom dock bar with clickable item launch
- `WorkspaceGridView` — 2x2 workspace switcher grid
- `Window` — titled window chrome with close/minimize/zoom controls

The `Widget` trait defines `layout()`, `draw()`, and `handle_event()`. All widgets
implement this trait and can be composed freely.

Theme tokens (`ThemeToken` enum) are resolved through a `ThemeContext` passed into
every `draw()` call so that widgets never hard-code colors.

### retro-bus

IPC layer for cross-process communication between apps and the shell.

- `Message` — typed envelope with sender, recipient, and JSON payload.
- `ServiceRegistry` — service discovery and registration.
- `Transport` — async message delivery (tokio channels; D-Bus via the optional
  `dbus` feature using `zbus`).

The D-Bus feature is enabled by default (`features = ["dbus"]`).

### retro-sdk

Application framework. Every first-party app (`finder`, `settings`, etc.) links this
crate and calls `Application::new()` then `app.run()`.

- `Application` — owns the winit event loop, creates the wgpu surface, and drives
  the render-kit-event cycle.
- Menu manifest publishing: apps write a JSON `MenuManifest` to
  `$RETROSHELL_MENU_MANIFEST_DIR` or `$XDG_RUNTIME_DIR/retroshell/menus/<bundle_id>.json`
  so that `retro-shell` can load and display their menu bar.
- `RETROSHELL_GLOBAL_MENU=1` suppresses the app's own in-window menu bar; the shell
  then shows the app's menus in the system menu bar instead.
- Rendering helpers: `draw_progress_bar()`, `draw_tab_view()`, `draw_popup_button()`,
  `draw_dialog()`, `draw_dock_view()` — shared pixel-accurate implementations used
  by all apps.

### retro-shell

The shell process. Not a compositor; runs as a single Wayland client under labwc.

Internal subsystems (each in its own module, wrapped in `Arc<RwLock<_>>`):

| Module                   | Purpose                                                         |
|--------------------------|-----------------------------------------------------------------|
| `WindowManager`          | Tracks internal shell windows: focus, minimize, zoom, workspace |
| `WorkspaceManager`       | 4 virtual desktops; window assignment and switching             |
| `MenuServer`             | Shell menu bar, shortcut bindings, app menu manifest loading    |
| `LaunchServices`         | Scans `.app` bundles, maps bundle IDs to binaries               |
| `DesktopManager`         | Desktop icon layout and state                                   |
| `Dock`                   | Dock item list and app-launch dispatch                          |
| `NotificationCenter`     | In-process notification post/query/clear                        |
| `SessionManager`         | Lock/unlock screen state                                        |
| `ThemeManager`           | Theme palette loading and dark-mode toggle                      |
| `ApplicationRegistry`    | Running application tracking                                    |

`ShellDesktop` is the root widget that composes the menu bar, desktop icon view,
shell-managed windows, dock, and notification popups into a single render tree.

### retro-compositor

A future Smithay-based Wayland compositor (Linux only). Currently a skeleton binary
that initializes Smithay with `backend_x11`, `wayland_frontend`, `renderer_gl`, and
`desktop` features. It is not launched or integrated by the shell today.

---

## Wayland Protocol Stack

Current state (prototype):

```
RetroShell process
  └── winit (xdg_surface / wl_surface client)
        └── wgpu Vulkan/Wayland backend
              └── labwc (acts as the real compositor)
                    └── Wayland display server
                          └── Linux kernel DRM/KMS
```

Apps (Finder, Terminal, etc.) each open their own `xdg_toplevel` surface under labwc.
labwc manages surface stacking, focus, and decoration for external windows. RetroShell
only manages windows that it draws inside its own wgpu canvas.

Target state (future):

```
retro-compositor (Smithay)
  ├── xdg_shell clients (Finder, Terminal, third-party apps)
  │     └── wl_surface surfaces composited by retro-compositor
  ├── RetroShell (shell UI drawn by compositor)
  └── Linux kernel DRM/KMS (direct output ownership)
```

The compositor path requires implementing Smithay's `CompositorHandler`, `XdgShell`,
`SeatHandler`, and output management. This is tracked as a long-term milestone.

---

## Rendering Pipeline

Each frame:

```
1. winit RedrawRequested event
2. retro-render: acquire wgpu surface texture
3. retro-render: begin render pass (clear with desktop background color)
4. ShellDesktop::draw(theme)
   a. IconView::draw()       — desktop icons
   b. For each ShellWindow (back to front):
        Window::draw()       — chrome + content widget
   c. DockView::draw()       — bottom dock
   d. notification popups
   e. MenuBar::draw()        — always on top
5. retro-render: submit command buffer
6. retro-render: present surface texture
```

Drawing is immediate-mode. There is no retained scene graph; every widget redraws
every frame. Dirty-rect optimization is not implemented.

Text rendering per glyph:
1. `cosmic-text` resolves the font family and performs Unicode shaping.
2. `ab_glyph` rasterizes each glyph to a coverage bitmap.
3. The bitmap is uploaded as a wgpu texture and composited into the frame.
4. Glyphs are cached in a texture atlas to avoid re-rasterization.

---

## How to Add a New App

### 1. Create the crate

```bash
cargo new --bin apps/myapp
```

Add it to the workspace `members` in `/Cargo.toml`:

```toml
members = [
    ...
    "apps/myapp",
]
```

### 2. Declare dependencies in `apps/myapp/Cargo.toml`

```toml
[dependencies]
retro-sdk = { path = "../../crates/retro-sdk" }
retro-kit = { path = "../../crates/retro-kit" }
```

### 3. Write the entry point

```rust
use retro_sdk::{Application, build_menu};
use retro_kit::window::Window;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut app = Application::new("My App", "com.retro.myapp");

    // Build menus
    let mut file_menu = build_menu("File");
    file_menu.add_action("Open...");
    app.set_menus(vec![file_menu]);

    // Build main window
    let mut window = Window::new("My App");
    window.set_content(Box::new(MyContentWidget::new()));
    app.set_main_window(window);

    app.run();
}
```

### 4. Create an App.toml bundle descriptor

```toml
name = "My App"
bundle_id = "com.retro.myapp"
version = "1.0.0"
author = "Your Name"
minimum_platform = "1.0"
entrypoint = "Executable/myapp"
file_types = []
permissions = []
```

Place it at `apps/myapp/App.toml`. The build script or Docker workflow copies it into
the bundle directory.

### 5. Register the binary in `retro-shell`

Add a line to the `launch_app_binary` match in
`crates/retro-shell/src/lib.rs` so the shell can launch it:

```rust
"com.retro.myapp" => "myapp",
```

### 6. Add to the dock (optional)

Edit `crates/retro-shell/src/dock.rs` to include the app's bundle ID in the default
dock items list.

### 7. Build and test

```bash
cargo build
WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/tmp/runtime-root ./target/debug/myapp
```

Menu manifest publishing is automatic when `RETROSHELL_GLOBAL_MENU=1` is set. The
shell will load the manifest from `$XDG_RUNTIME_DIR/retroshell/menus/com.retro.myapp.json`
and display the app's menus in the system menu bar.
