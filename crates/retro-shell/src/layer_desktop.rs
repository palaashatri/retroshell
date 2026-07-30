//! wlr-layer-shell background surface driver for RetroShell.
//!
//! Renders the desktop as a real root-level layer surface (fullscreen)
//! via `zwlr_layer_shell_v1` instead of a winit xdg-toplevel. Gated behind
//! `RETROSHELL_LAYER_SHELL_CHROME` environment variable.
//!
//! Phase 2b uses `Layer::Top` so the surface receives pointer/keyboard focus
//! under our compositor. Phase 3 will split menu→Top exclusive, dock→Bottom,
//! wallpaper→Background.
//!
//! Linux only; unavailable on macOS/Windows.

#![cfg(target_os = "linux")]

use anyhow::anyhow;
use retro_kit::event::{KeyCode, MouseButton};
use retro_kit::{Event, Widget};
use retro_sdk::{RawSurfaceRenderer, UiRuntime};
use std::ffi::c_void;
use std::time::{SystemTime, UNIX_EPOCH};
use wayland_client::{
    protocol::{
        wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat, wl_shm, wl_surface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, ZwlrLayerSurfaceV1},
};

/// Linux BTN_* codes from `<linux/input-event-codes.h>`.
fn mouse_button_from_linux(code: u32) -> Option<MouseButton> {
    match code {
        0x110 => Some(MouseButton::Left),
        0x111 => Some(MouseButton::Right),
        0x112 => Some(MouseButton::Middle),
        _ => None,
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Map a subset of Linux KEY_* codes to kit KeyCode (enough for shell shortcuts).
fn keycode_from_linux(code: u32) -> Option<KeyCode> {
    Some(match code {
        1 => KeyCode::Escape,
        14 => KeyCode::Backspace,
        15 => KeyCode::Tab,
        28 => KeyCode::Enter,
        57 => KeyCode::Space,
        105 => KeyCode::ArrowLeft,
        106 => KeyCode::ArrowRight,
        103 => KeyCode::ArrowUp,
        108 => KeyCode::ArrowDown,
        102 => KeyCode::Home,
        107 => KeyCode::End,
        111 => KeyCode::Delete,
        _ => return None,
    })
}

/// Main entry point: run the layer-shell desktop with the given content widget.
pub fn run_layer_desktop(content: Box<dyn Widget>, width: u32, height: u32) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env().map_err(|e| anyhow!("wayland connect: {}", e))?;

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let display_obj = conn.display();
    let _registry = display_obj.get_registry(&qh, ());

    let mut state = LayerDesktopState {
        compositor: None,
        shm: None,
        layer_shell: None,
        seat: None,
        wl_surface: None,
        wl_pointer: None,
        wl_keyboard: None,
        layer_surface: None,
        configured_size: None,
        runtime: None,
        renderer: None,
        running: true,
        last_pointer: (0.0, 0.0),
    };

    // Roundtrip to collect globals
    event_queue
        .roundtrip(&mut state)
        .map_err(|e| anyhow!("registry roundtrip: {}", e))?;

    let compositor = state
        .compositor
        .clone()
        .ok_or_else(|| anyhow!("wl_compositor not found"))?;
    let _shm = state
        .shm
        .clone()
        .ok_or_else(|| anyhow!("wl_shm not found"))?;
    let layer_shell = state
        .layer_shell
        .clone()
        .ok_or_else(|| anyhow!("zwlr_layer_shell_v1 not found"))?;

    // Create wl_surface
    let surface = compositor.create_surface(&qh, ());

    // Create layer surface as Top (interactive) for Phase 2b — receives pointer.
    // Phase 3 splits menu→Top exclusive, dock→Bottom exclusive, wallpaper→Background.
    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        Layer::Top,
        "retroshell-desktop".into(),
        &qh,
        (),
    );
    let anchor = Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right;
    layer_surface.set_anchor(anchor);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_size(width, height);

    // Initial commit (protocol required before we can get a configure)
    surface.commit();

    // Store the surface and layer_surface for later use
    state.wl_surface = Some(surface.clone());
    state.layer_surface = Some(layer_surface.clone());

    // Roundtrip to let the compositor send the configure event
    event_queue
        .roundtrip(&mut state)
        .map_err(|e| anyhow!("configure roundtrip: {}", e))?;

    // Determine actual size to use (configured or requested)
    let (actual_width, actual_height) = state.configured_size.unwrap_or((width, height));

    // Extract raw pointers for wgpu integration
    // UNCERTAINTY: wayland-client API for getting *mut wl_display and *mut wl_surface
    // Using .backend().display_ptr() for display and surface.id().as_ptr() for surface.
    let display_ptr = conn.backend().display_ptr() as *mut c_void;
    let surface_ptr = surface.id().as_ptr() as *mut c_void;

    // Create RawSurfaceRenderer (async, uses block_on)
    let mut renderer = futures::executor::block_on(unsafe {
        RawSurfaceRenderer::new(display_ptr, surface_ptr, actual_width, actual_height)
    })
    .map_err(|e| anyhow!("RawSurfaceRenderer init: {}", e))?;

    // Create UiRuntime
    let mut runtime = UiRuntime::new(content, actual_width, actual_height, 1.0);

    // Initial paint
    runtime
        .paint(&mut renderer)
        .map_err(|e| anyhow!("initial paint: {}", e))?;
    surface.commit();

    // Store in state for the event loop
    state.renderer = Some(renderer);
    state.runtime = Some(runtime);

    // Main event loop: dispatch events, repaint if dirty
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(|e| anyhow!("dispatch: {}", e))?;

        if let (Some(renderer), Some(runtime)) =
            (state.renderer.as_mut(), state.runtime.as_mut())
        {
            // Per-frame tick drives ShellDesktop::update() (rebuilds the dock,
            // notifications, clock). Without this the dock never populates.
            runtime.tick();
            if runtime.is_dirty() {
                runtime
                    .paint(renderer)
                    .map_err(|e| anyhow!("repaint: {}", e))?;
                if let Some(surface) = &state.wl_surface {
                    surface.commit();
                }
            }
        }
    }

    Ok(())
}

struct LayerDesktopState {
    compositor: Option<wayland_client::protocol::wl_compositor::WlCompositor>,
    shm: Option<wayland_client::protocol::wl_shm::WlShm>,
    layer_shell: Option<ZwlrLayerShellV1>,
    seat: Option<wayland_client::protocol::wl_seat::WlSeat>,
    wl_surface: Option<wayland_client::protocol::wl_surface::WlSurface>,
    wl_pointer: Option<wayland_client::protocol::wl_pointer::WlPointer>,
    wl_keyboard: Option<wl_keyboard::WlKeyboard>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    /// Configured (w, h) from the layer surface Configure event
    configured_size: Option<(u32, u32)>,
    /// UI runtime (initialized after configure)
    runtime: Option<UiRuntime>,
    /// Raw surface renderer (initialized after configure)
    renderer: Option<RawSurfaceRenderer>,
    /// Keep running until false
    running: bool,
    /// Last pointer position in surface coordinates (from Motion/Enter).
    last_pointer: (f64, f64),
}

impl Dispatch<wl_registry::WlRegistry, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let v = version.min(4);
                    state.compositor = Some(registry.bind(name, v, qh, ()));
                }
                "wl_shm" => {
                    state.shm = Some(registry.bind(name, 1, qh, ()));
                }
                "zwlr_layer_shell_v1" => {
                    let v = version.min(4);
                    state.layer_shell = Some(registry.bind(name, v, qh, ()));
                }
                "wl_seat" => {
                    let v = version.min(9);
                    state.seat = Some(registry.bind(name, v, qh, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wayland_client::protocol::wl_compositor::WlCompositor, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_compositor::WlCompositor,
        _: wayland_client::protocol::wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wayland_client::protocol::wl_shm::WlShm, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_shm::WlShm,
        _: wayland_client::protocol::wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wayland_client::protocol::wl_surface::WlSurface, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_surface::WlSurface,
        _: wayland_client::protocol::wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wayland_client::protocol::wl_seat::WlSeat, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        seat: &wayland_client::protocol::wl_seat::WlSeat,
        event: wayland_client::protocol::wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_seat::Event::Capabilities { capabilities } = event {
            if let WEnum::Value(caps) = capabilities {
                if caps.contains(wayland_client::protocol::wl_seat::Capability::Pointer)
                    && state.wl_pointer.is_none()
                {
                    state.wl_pointer = Some(seat.get_pointer(qh, ()));
                }
                if caps.contains(wayland_client::protocol::wl_seat::Capability::Keyboard)
                    && state.wl_keyboard.is_none()
                {
                    state.wl_keyboard = Some(seat.get_keyboard(qh, ()));
                }
            }
        }
    }
}

impl Dispatch<wayland_client::protocol::wl_pointer::WlPointer, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        _: &wayland_client::protocol::wl_pointer::WlPointer,
        event: wayland_client::protocol::wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.last_pointer = (surface_x, surface_y);
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.pointer_moved(surface_x as f32, surface_y as f32);
                }
            }
            wl_pointer::Event::Button {
                button,
                state: btn_state,
                ..
            } => {
                let Some(mouse) = mouse_button_from_linux(button) else {
                    return;
                };
                let pressed = matches!(
                    btn_state,
                    WEnum::Value(wl_pointer::ButtonState::Pressed)
                );
                let (px, py) = state.last_pointer;
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.pointer_moved(px as f32, py as f32);
                    let _ = runtime.pointer_button(mouse, pressed, now_ms());
                }
            }
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            } => {
                state.last_pointer = (surface_x, surface_y);
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.pointer_moved(surface_x as f32, surface_y as f32);
                    runtime.set_focus(true);
                }
            }
            wl_pointer::Event::Leave { .. } => {
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.set_focus(false);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(runtime) = state.runtime.as_mut() else {
            return;
        };
        match event {
            wl_keyboard::Event::Key {
                key,
                state: key_state,
                ..
            } => {
                let Some(code) = keycode_from_linux(key) else {
                    return;
                };
                let pressed = matches!(
                    key_state,
                    WEnum::Value(wl_keyboard::KeyState::Pressed)
                );
                let ev = if pressed {
                    Event::KeyDown {
                        key: code,
                        modifiers: retro_kit::event::Modifiers::NONE,
                    }
                } else {
                    Event::KeyUp {
                        key: code,
                        modifiers: retro_kit::event::Modifiers::NONE,
                    }
                };
                runtime.key(ev);
            }
            wl_keyboard::Event::Enter { .. } => {
                runtime.set_focus(true);
            }
            wl_keyboard::Event::Leave { .. } => {
                runtime.set_focus(false);
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            state.configured_size = Some((width as u32, height as u32));
            surface.ack_configure(serial);
        }
    }
}
