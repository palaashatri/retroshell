//! wlr-layer-shell background surface driver for RetroShell.
//!
//! Renders the desktop as a real root-level BACKGROUND layer surface (fullscreen)
//! via `zwlr_layer_shell_v1` instead of a winit xdg-toplevel. Gated behind
//! `RETROSHELL_LAYER_SHELL_CHROME` environment variable.
//!
//! Linux only; unavailable on macOS/Windows.

#![cfg(target_os = "linux")]

use anyhow::anyhow;
use retro_kit::Widget;
use retro_sdk::{RawSurfaceRenderer, UiRuntime};
use std::ffi::c_void;
use std::os::unix::io::AsFd;
use wayland_client::{
    protocol::{wl_compositor, wl_pointer, wl_registry, wl_seat, wl_shm, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, ZwlrLayerSurfaceV1},
};

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
        layer_surface: None,
        configured_size: None,
        runtime: None,
        renderer: None,
        running: true,
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

    // Create layer surface as BACKGROUND, anchored on all edges, exclusive_zone = -1
    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        Layer::Background,
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

        if let Some(runtime) = &state.runtime {
            if runtime.is_dirty() {
                if let (Some(renderer), Some(runtime)) =
                    (state.renderer.as_mut(), state.runtime.as_mut())
                {
                    runtime
                        .paint(renderer)
                        .map_err(|e| anyhow!("repaint: {}", e))?;
                    if let Some(surface) = &state.wl_surface {
                        surface.commit();
                    }
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
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    /// Configured (w, h) from the layer surface Configure event
    configured_size: Option<(u32, u32)>,
    /// UI runtime (initialized after configure)
    runtime: Option<UiRuntime>,
    /// Raw surface renderer (initialized after configure)
    renderer: Option<RawSurfaceRenderer>,
    /// Keep running until false
    running: bool,
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
        // Rendering-first: pointer events are accepted but not yet routed into
        // the UiRuntime. `surface_x/surface_y` (Motion) and `button`/`state`
        // (Button) will be wired to runtime.pointer_moved/pointer_button in a
        // follow-up (Phase 2b-iii, input). For now this keeps the seat alive.
        let _ = (state, event);
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
