# Stage 2b — Root-level chrome via layer-shell (winit → sctk + wgpu)

> Supersedes the compositor-side "size the shell toplevel to fill output" hack.
> Goal: the desktop wallpaper + global menu bar + dock become **real
> compositor-owned layer-shell surfaces**, not a normal client window — like
> macOS / GNOME / KDE. See the audit in this session and `docs/qa/stage-2.md`.

## Why (audit findings)
- retro-shell renders its whole desktop through **one winit+wgpu xdg-toplevel**
  (`crates/retro-sdk` `Application`/`WgpuPresenter`). The compositor forces every
  toplevel to 640×480 (`session_drm.rs` `new_toplevel`), so the desktop was a
  small box in the corner.
- The existing `layer_shell_client.rs` is a throwaway PoC (binds, commits gray
  placeholder buffers, drops the connection). `should_paint_kit_chrome()` is
  hardcoded `true`; real layer-shell never carried pixels.
- The legacy in-window menu bar was already removed (retro-sdk global-menu-only).
- Font baseline + descenders already fixed (retro-render/retro-sdk).

## Approach — reuse wgpu, replace windowing/input
Keep the entire wgpu `Canvas` + widget draw path. Replace **only** winit with
smithay-client-toolkit (sctk, already a dep) for surface creation + input:

1. Create wl_surface + `zwlr_layer_surface_v1` via sctk (layer/anchor/exclusive).
2. Do the configure/ack handshake; on configure, (re)size.
3. Build a `wgpu::Surface` from the raw Wayland handles
   (`RawDisplayHandle::Wayland{ display }`, `RawWindowHandle::Wayland{ surface }`)
   via `instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle{..})`.
   (Current `WgpuPresenter::new(Arc<winit::Window>)` must gain a raw-handle ctor.)
4. Render the existing `ShellDesktop` into it (unchanged draw code).
5. Route sctk `wl_pointer`/`wl_keyboard` events into the shell's existing input
   handling (the same logic the winit `AppHandler` feeds today).

## Phases
- **Phase 1 — `LayerSurfacePresenter` in retro-sdk (or retro-shell):** a reusable
  sctk-backed presenter that owns the connection, one layer surface (params:
  layer, anchors, exclusive_zone, size), the configure loop, a wgpu surface built
  from raw handles, and a per-frame render callback + input event stream. Unit of
  reuse for all chrome surfaces.
- **Phase 2a — DONE (committed `1bd616c`):** `retro_sdk::RawSurfaceRenderer`
  (+ pub `Canvas`) wraps `WgpuPresenter::new_raw` so retro-shell can render onto a
  surface it created. Compiles on the VM.
- **Phase 2b — shell uses one full-output BACKGROUND layer.** retro-shell maps a
  single background layer surface (anchor all 4 edges, exclusive_zone -1) and
  renders the full `ShellDesktop` into it via `RawSurfaceRenderer`. **This is the
  milestone deliverable** but it is a real refactor, not just "add a loop":
  - `ShellDesktop` is driven entirely by retro-sdk's **winit `AppHandler`**
    (`Application::run`), which owns event translation, dirty-tracking, scale, and
    the presenter render loop. There is no reusable "render this Window + dispatch
    input" core today.
  - There are two widget layers: retro-kit `Widget::draw(&self, &ThemeContext)`
    and the retro-sdk Canvas draw path (`draw_window`/`draw_desktop_backdrop`).
  - **Plan:** extract the AppHandler render+input core in retro-sdk into a
    backend-agnostic unit (input event enum in → `render(|canvas| ...)` out), then
    add an sctk-driven driver in retro-shell that (1) creates the layer surface +
    configure/ack, (2) builds `RawSurfaceRenderer` from the raw handles, (3) feeds
    wl_pointer/wl_keyboard into the core, (4) renders on frame callbacks. Gate the
    whole path behind `RETROSHELL_LAYER_SHELL_CHROME` so the winit default is
    untouched until the layer path is proven by a VM screenshot.
- **Phase 3 — split exclusive chrome:** menu bar → `top` layer (exclusive=menu_h),
  dock → `bottom` layer (exclusive=dock_h), background keeps wallpaper+icons. Now
  maximized apps respect reserved zones and can't cover the menu/dock. Un-stub
  `should_paint_kit_chrome`; delete the throwaway `layer_shell_client.rs` PoC.

## QA (this environment)
- VM: UTM `Ubuntu` (aarch64), 192.168.64.15, user ubuntu. `/dev/dri/card0` KMS ok.
- Build: `cargo build --release --workspace` on the VM (native aarch64).
- Run headless DRM over SSH: user must be in `video`,`render`,`input` groups;
  `LIBSEAT_BACKEND=seatd`; `LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe`
  (virtio hw GL fails for wgpu clients; llvmpipe works).
- **Screenshot capture:** compositor SIGUSR1 offscreen readback (`screenshot.rs`)
  is blocked — this GLES context rejects `glReadPixels` for all formats. Working
  method: run on **Xvfb** + `import -window root` (winit/sctk both fine on Xvfb).
  Revisit compositor readback with a GLES3 PBO path later.

## API references (smithay-client-toolkit 0.19 / wgpu / wayland-protocols-wlr 0.3)
- Layer shell client: `wayland_protocols_wlr::layer_shell::v1::client` (see the
  existing `layer_shell_client.rs` for the bind/anchor/exclusive/configure calls).
- wgpu raw surface: `wgpu::SurfaceTargetUnsafe::RawHandle` +
  `raw_window_handle::{RawDisplayHandle::Wayland, RawWindowHandle::Wayland}`.
