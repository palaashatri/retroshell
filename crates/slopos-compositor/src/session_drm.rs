//! DRM/KMS + libseat session bootstrap for bare-metal / VT sessions.
//!
//! Selected when policy says [`CompositorBackendKind::SessionDrm`]. Docker-on-mac
//! will not exercise seat/DRM privileges; the code still ships, compiles into
//! `slopos-compositor`, and runs when `/dev/dri` + seatd/logind are available.
//!
//! Bootstrap:
//! - Open a libseat session
//! - Discover DRM primary nodes (pure helpers + seat open)
//! - Create `DrmDevice` + `GbmDevice` + EGL GLES renderer
//! - Expose a Wayland socket with xdg_shell, wlr-layer-shell, foreign-toplevel-list
//! - Drive calloop with udev hotplug + libinput + seat events
//!
//! Full multi-output scanout / pageflip is progressive: this path opens the
//! primary card, advertises an output, and runs a real protocol loop. Connectors
//! without modes fall back to env sizing (`SLOPOS_COMPOSITOR_WIDTH/HEIGHT`).

#![cfg(target_os = "linux")]

use std::collections::HashMap;
use std::os::unix::io::OwnedFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use smithay::backend::allocator::gbm::{GbmAllocator, GbmBufferFlags, GbmDevice};
use smithay::backend::allocator::Fourcc as DrmFourcc;
use smithay::backend::drm::compositor::{DrmCompositor, FrameFlags};
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::backend::drm::DrmDeviceFd;
use smithay::backend::egl::{EGLContext, EGLDisplay};
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::renderer::element::surface::{
    render_elements_from_surface_tree, WaylandSurfaceRenderElement,
};
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::session::{Event as SessionEvent, Session};
use smithay::backend::udev::{all_gpus, primary_gpu, UdevBackend, UdevEvent};
use smithay::input::keyboard::XkbConfig;
use smithay::input::pointer::{CursorImageStatus, CursorImageSurfaceData, Focus};
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Mode, Output, PhysicalProperties, Scale, Subpixel};
use smithay::reexports::calloop::{
    generic::Generic, EventLoop, Interest, Mode as CalloopMode, PostAction,
};
// Use smithay's rustix reexport so OFlags matches Session::open.
use smithay::desktop::utils::send_frames_surface_tree;
use smithay::desktop::{
    find_popup_root_surface, PopupGrab, PopupKeyboardGrab, PopupKind, PopupManager,
    PopupPointerGrab,
};
use smithay::reexports::rustix::fs::OFlags;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::reexports::wayland_server::protocol::{
    wl_buffer, wl_data_source::WlDataSource, wl_seat, wl_surface::WlSurface,
};
use smithay::reexports::wayland_server::{Client, Display, DisplayHandle, Resource};
use smithay::utils::{
    Clock, DeviceFd, Logical, Monotonic, Point, Rectangle, Serial, Size, Transform,
};
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{
    with_states, CompositorClientState, CompositorHandler, CompositorState,
};
use smithay::wayland::foreign_toplevel_list::{
    ForeignToplevelHandle, ForeignToplevelListHandler, ForeignToplevelListState,
};
use smithay::wayland::output::{OutputHandler, OutputManagerState};
use smithay::wayland::selection::data_device::{
    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
    ServerDndGrabHandler,
};
use smithay::wayland::selection::primary_selection::{
    set_primary_focus, PrimarySelectionHandler, PrimarySelectionState,
};
use smithay::wayland::selection::{SelectionHandler, SelectionSource, SelectionTarget};
use smithay::wayland::session_lock::{
    LockSurface, SessionLockHandler, SessionLockManagerState, SessionLocker,
};
use smithay::wayland::shell::wlr_layer::{
    Anchor, Layer, LayerSurface, LayerSurfaceCachedState, Margins, WlrLayerShellHandler,
    WlrLayerShellState,
};
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    XdgToplevelSurfaceData,
};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::socket::ListeningSocketSource;
use smithay::{
    delegate_compositor, delegate_data_device, delegate_foreign_toplevel_list,
    delegate_layer_shell, delegate_output, delegate_primary_selection, delegate_seat,
    delegate_session_lock, delegate_shm, delegate_xdg_shell,
};

use crate::frame_timing::{FrameScheduler, RefreshRate};
use crate::hdr::HdrCapabilities;
use crate::{
    assign_new_window_to_active, clamp_window_to_work_area, detect_output_scale_from_env,
    discover_drm_nodes, drm_presentation_pipeline, focus_window_after_workspace_switch,
    geometry_for_interactive_grab, output_scale_summary, plan_drm_modeset,
    pointer_grab_request_is_valid, preferred_primary_drm_node, register_wayland_display_source,
    session_mode_summary, transition_presentation_state, visible_paint_order,
    CompositorBackendKind, DisplayPolicy, DrmPresentationStage, InteractiveGrab,
    InteractiveGrabKind, OutputScale, ResizeEdges, WindowGeometry, WindowPresentationState,
    WorkspaceId, WorkspaceState, DEFAULT_OUTPUT_H, DEFAULT_OUTPUT_W, DEFAULT_WINDOW_H,
    DEFAULT_WINDOW_W,
};
use slopos_bus::{SessionControlListener, SessionControlRequest, WindowPresentationAction};
// Workspace cycle helpers (`cycle_workspace_*` / `activate_workspace_index`) request a
// full redraw and re-focus the topmost visible window. Super+key bindings can call them
// when seat keyboard filtering is wired (mirrors nested X11 main path).

/// Compositor-owned selection payload keyed by mime type.
type MimePayload = Arc<HashMap<String, Vec<u8>>>;

#[derive(Clone)]
struct PointerPress {
    serial: Serial,
    surface: WlSurface,
}

/// The concrete `DrmCompositor` this session uses: GBM-allocated buffers,
/// GBM framebuffer export, no per-frame user data, over a DRM device fd.
type RetroDrmCompositor =
    DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmFramebufferExporter<DrmDeviceFd>, (), DrmDeviceFd>;

/// Desktop background behind all client surfaces (classic retro gray).
const DRM_CLEAR_COLOR: [f32; 4] = [0.596, 0.596, 0.580, 1.0];
/// Solid black used while the session lock is active.
const DRM_LOCK_CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Probe whether a DRM session looks bootable (nodes exist under /dev/dri).
pub fn drm_session_available() -> bool {
    !discover_drm_nodes().is_empty() || Path::new("/dev/dri").exists()
}

/// Present one frame from an already-allocated scanout framebuffer via
/// `DrmSurface::commit`, falling back to `page_flip`.
///
/// The framebuffer is allocated once per session by [`arm_scanout_framebuffer`]
/// and reused. Allocating (and leaking) a full-screen dumb buffer per present —
/// which this used to do at ~1 Hz — is an unbounded kernel memory leak.
fn present_armed_frame(
    surface: &smithay::backend::drm::DrmSurface,
    fb_handle: smithay::reexports::drm::control::framebuffer::Handle,
    width: i32,
    height: i32,
) -> Result<()> {
    use smithay::backend::drm::{PlaneConfig, PlaneState};
    use smithay::utils::{Buffer as BufferCoords, Physical, Rectangle, Transform};

    let w = width.max(1) as u32;
    let h = height.max(1) as u32;

    let plane = surface.plane();
    let dst = Rectangle::<i32, Physical>::from_size((w as i32, h as i32).into());
    let src = Rectangle::<f64, BufferCoords>::from_size((f64::from(w), f64::from(h)).into());
    // First commit may modeset; on failure try non-blocking page_flip.
    let cfg = PlaneConfig {
        src,
        dst,
        transform: Transform::Normal,
        alpha: 1.0,
        damage_clips: None,
        fb: fb_handle,
        fence: None,
    };
    let states = [PlaneState {
        handle: plane,
        config: Some(cfg),
    }];
    match surface.commit(states.iter().cloned(), true) {
        Ok(()) => {
            tracing::debug!("DrmSurface::commit ok");
        }
        Err(err) => {
            tracing::debug!(?err, "commit failed, trying page_flip");
            let cfg2 = PlaneConfig {
                src,
                dst,
                transform: Transform::Normal,
                alpha: 1.0,
                damage_clips: None,
                fb: fb_handle,
                fence: None,
            };
            let states2 = [PlaneState {
                handle: plane,
                config: Some(cfg2),
            }];
            surface
                .page_flip(states2.iter().cloned(), true)
                .context("DrmSurface::page_flip")?;
        }
    }
    Ok(())
}

/// Build the render element list for one frame: layer-shell chrome plus every
/// window visible on the active workspace, bottom-to-top in stacking order.
///
/// `render_elements_from_surface_tree` returns elements front-to-back for a
/// single surface tree, and `DrmCompositor::render_frame` also wants
/// front-to-back overall — so surfaces are walked top-of-stack first.
fn collect_render_elements(
    renderer: &mut GlesRenderer,
    state: &DrmSessionState,
) -> Vec<WaylandSurfaceRenderElement<GlesRenderer>> {
    let mut elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = Vec::new();
    let output_scale = state
        .outputs
        .first()
        .map(|output| output.current_scale().fractional_scale())
        .unwrap_or(1.0);

    let physical_point = |x: f64, y: f64| {
        Point::<i32, smithay::utils::Physical>::from((
            (x * output_scale).round() as i32,
            (y * output_scale).round() as i32,
        ))
    };

    if state.locked {
        for (_, lock_surface) in &state.lock_surfaces {
            elements.extend(render_elements_from_surface_tree(
                renderer,
                lock_surface.wl_surface(),
                (0, 0),
                output_scale,
                1.0,
                Kind::Unspecified,
            ));
        }
        if let CursorImageStatus::Surface(ref surface) = state.cursor_status {
            let hotspot = with_states(surface, |states| {
                states
                    .data_map
                    .get::<CursorImageSurfaceData>()
                    .and_then(|attrs| attrs.lock().ok().map(|attrs| attrs.hotspot))
                    .unwrap_or_else(|| Point::from((0, 0)))
            });
            let loc = physical_point(
                state.pointer_location.x - f64::from(hotspot.x),
                state.pointer_location.y - f64::from(hotspot.y),
            );
            elements.extend(render_elements_from_surface_tree(
                renderer,
                surface,
                loc,
                output_scale,
                1.0,
                Kind::Cursor,
            ));
        }
        return elements;
    }

    // Cursor first: the element slice is front-to-back, so the pointer must
    // lead or it renders underneath the windows it is pointing at. Only a
    // client-provided surface can be drawn here; a named cursor needs a theme
    // (XCursor) which the DRM path does not load yet.
    if let CursorImageStatus::Surface(ref surface) = state.cursor_status {
        let hotspot = with_states(surface, |states| {
            states
                .data_map
                .get::<CursorImageSurfaceData>()
                .and_then(|attrs| attrs.lock().ok().map(|attrs| attrs.hotspot))
                .unwrap_or_else(|| Point::from((0, 0)))
        });
        let loc = (
            state.pointer_location.x.round() as i32 - hotspot.x,
            state.pointer_location.y.round() as i32 - hotspot.y,
        );
        elements.extend(render_elements_from_surface_tree(
            renderer,
            surface,
            loc,
            1.0,
            1.0,
            Kind::Cursor,
        ));
    }

    // Layer order: Overlay/Top above windows; Bottom/Background below (macOS/GNOME/KDE).
    for layer in state.layer_surfaces.iter().rev() {
        if matches!(layer.layer, Layer::Overlay | Layer::Top) {
            elements.extend(render_elements_from_surface_tree(
                renderer,
                layer.surface.wl_surface(),
                physical_point(layer.geo.loc.x as f64, layer.geo.loc.y as f64),
                output_scale,
                1.0,
                Kind::Unspecified,
            ));
        }
    }

    // Windows: last mapped is topmost, so iterate in reverse for front-to-back.
    for w in state
        .windows
        .iter()
        .rev()
        .filter(|w| !w.minimized && state.workspace_state.is_visible(&w.window_id))
    {
        let popup_elements = PopupManager::popups_for_surface(w.toplevel.wl_surface()).flat_map(
            |(popup, popup_offset)| {
                let geometry = popup.geometry();
                let popup_loc = (
                    w.position.x + popup_offset.x - geometry.loc.x,
                    w.position.y + popup_offset.y - geometry.loc.y,
                );
                render_elements_from_surface_tree(
                    renderer,
                    popup.wl_surface(),
                    physical_point(popup_loc.0 as f64, popup_loc.1 as f64),
                    output_scale,
                    1.0,
                    Kind::Unspecified,
                )
            },
        );
        elements.extend(popup_elements);
        elements.extend(render_elements_from_surface_tree(
            renderer,
            w.toplevel.wl_surface(),
            physical_point(w.position.x as f64, w.position.y as f64),
            output_scale,
            1.0,
            Kind::Unspecified,
        ));
    }

    for layer in state.layer_surfaces.iter().rev() {
        if matches!(layer.layer, Layer::Bottom | Layer::Background) {
            elements.extend(render_elements_from_surface_tree(
                renderer,
                layer.surface.wl_surface(),
                physical_point(layer.geo.loc.x as f64, layer.geo.loc.y as f64),
                output_scale,
                1.0,
                Kind::Unspecified,
            ));
        }
    }

    elements
}

/// Allocate the session's single scanout dumb buffer and its framebuffer.
///
/// Returns both owners; the caller must keep them alive for as long as the
/// framebuffer handle is used in plane state, otherwise the kernel frees the
/// backing object out from under the flip.
fn arm_scanout_framebuffer(
    surface: &smithay::backend::drm::DrmSurface,
    width: i32,
    height: i32,
) -> Result<(
    smithay::backend::allocator::dumb::DumbBuffer,
    smithay::backend::drm::dumb::DumbFramebuffer,
)> {
    use smithay::backend::allocator::dumb::DumbAllocator;
    use smithay::backend::allocator::{Allocator, Fourcc, Modifier};
    use smithay::backend::drm::dumb::framebuffer_from_dumb_buffer;
    use smithay::backend::drm::DrmDeviceFd;

    let w = width.max(1) as u32;
    let h = height.max(1) as u32;
    let fd: DrmDeviceFd = surface.device_fd().clone();
    let mut dumb = DumbAllocator::new(fd.clone());
    let buffer = dumb
        .create_buffer(w, h, Fourcc::Xrgb8888, &[Modifier::Linear])
        .context("DumbAllocator::create_buffer for scanout")?;
    let fb =
        framebuffer_from_dumb_buffer(&fd, &buffer, true).context("framebuffer_from_dumb_buffer")?;
    Ok((buffer, fb))
}

fn w_from_env_or_default() -> i32 {
    std::env::var("SLOPOS_COMPOSITOR_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_OUTPUT_W)
}

fn h_from_env_or_default() -> i32 {
    std::env::var("SLOPOS_COMPOSITOR_HEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_OUTPUT_H)
}

/// Resolve the primary DRM node path for seat open.
fn resolve_primary_drm_path(seat_name: &str) -> PathBuf {
    if let Some(n) = preferred_primary_drm_node(&discover_drm_nodes()) {
        return n.path.clone();
    }
    if let Ok(Some(p)) = primary_gpu(seat_name) {
        return p;
    }
    if let Ok(gpus) = all_gpus(seat_name) {
        if let Some(p) = gpus.into_iter().next() {
            return p;
        }
    }
    PathBuf::from("/dev/dri/card0")
}

/// Release frame callbacks only after a frame has actually been rendered.
///
/// Clients such as winit/wgpu use `wl_surface.frame` as their render
/// throttle. Sending callbacks from an idle loop both wakes every client and
/// makes the compositor repaint continuously. A callback belongs to the
/// frame that just made the corresponding surface visible.
fn release_frame_callbacks(state: &DrmSessionState, clock: &Clock<Monotonic>) {
    let now = clock.now();
    let Some(output) = state.outputs.first().cloned() else {
        return;
    };

    if state.locked {
        for (_, lock_surface) in &state.lock_surfaces {
            send_frames_surface_tree(
                lock_surface.wl_surface(),
                &output,
                now,
                Some(Duration::ZERO),
                |_, _| None,
            );
        }
        return;
    }

    let visible: Vec<WlSurface> = state
        .windows
        .iter()
        .filter(|w| !w.minimized && state.workspace_state.is_visible(&w.window_id))
        .map(|w| w.toplevel.wl_surface().clone())
        .collect();
    for surface in visible {
        send_frames_surface_tree(&surface, &output, now, Some(Duration::ZERO), |_, _| None);
    }

    let layers: Vec<WlSurface> = state
        .layer_surfaces
        .iter()
        .map(|layer| layer.surface.wl_surface().clone())
        .collect();
    for surface in layers {
        send_frames_surface_tree(&surface, &output, now, Some(Duration::ZERO), |_, _| None);
    }
}

/// Run the DRM/KMS session compositor path.
///
/// Returns `Err` with context if seat/DRM cannot be opened (no privileges,
/// nested container without `/dev/dri`). Callers may fall back to nested X11.
pub fn run_drm_session() -> Result<()> {
    tracing::info!(
        "{}",
        session_mode_summary(CompositorBackendKind::SessionDrm)
    );
    eprintln!(
        "[slopos-compositor] starting DRM/KMS session path ({})",
        session_mode_summary(CompositorBackendKind::SessionDrm)
    );
    // QA: SIGUSR1 → write a PNG of the next composited frame (see screenshot.rs).
    crate::screenshot::install_signal_handler();

    let display_policy = DisplayPolicy::resolve();
    let mut hdr_caps = HdrCapabilities::detect();
    let _ = hdr_caps.apply_request(display_policy.hdr_requested, display_policy.color_space);
    let effective_refresh = display_policy.effective_refresh_rate();
    let mut frame_scheduler = FrameScheduler::new(effective_refresh);
    let refresh_mhz: i32 = match effective_refresh {
        RefreshRate::Adaptive => 60_000,
        r => (r.as_hz() as i32) * 1000,
    };
    eprintln!(
        "[slopos-compositor] display policy: {}",
        display_policy.summary_line(hdr_caps.hdr_supported)
    );

    // ---- Seat (VT / device ACLs) ----
    let (mut session, session_notifier) =
        LibSeatSession::new().context("LibSeatSession::new (need seatd/logind + privileges)")?;
    let seat_name = session.seat();
    eprintln!("[slopos-compositor] libseat seat={seat_name}");

    // ---- Event loop + Wayland display ----
    let mut event_loop: EventLoop<'static, DrmSessionState> =
        EventLoop::try_new().context("EventLoop::try_new")?;
    let display: Display<DrmSessionState> = Display::new().context("Display::new")?;
    let dh = display.handle();
    let loop_handle = event_loop.handle();

    // Protocol globals
    let compositor_state = CompositorState::new::<DrmSessionState>(&dh);
    let shm_state = ShmState::new::<DrmSessionState>(&dh, vec![]);
    let mut seat_state = SeatState::new();
    let xdg_shell_state = XdgShellState::new::<DrmSessionState>(&dh);
    let data_device_state = DataDeviceState::new::<DrmSessionState>(&dh);
    let primary_selection_state = PrimarySelectionState::new::<DrmSessionState>(&dh);
    let output_manager_state = OutputManagerState::new_with_xdg_output::<DrmSessionState>(&dh);
    // XWayland is available on the nested X11 path; DRM path wires XWM in a follow-up
    // once XWayland spawn is attached to this seat/session loop.
    let layer_shell_state = WlrLayerShellState::new::<DrmSessionState>(&dh);
    let foreign_toplevel_list = ForeignToplevelListState::new::<DrmSessionState>(&dh);
    let session_lock_state =
        SessionLockManagerState::new::<DrmSessionState, _>(&dh, |_client| true);

    let mut seat: Seat<DrmSessionState> = seat_state.new_wl_seat(&dh, "seat0");
    seat.add_keyboard(XkbConfig::default(), 200, 25)
        .context("add_keyboard")?;
    seat.add_pointer();

    // ---- Open primary GPU via seat ----
    let primary = resolve_primary_drm_path(&seat_name);
    eprintln!("[slopos-compositor] opening DRM node {}", primary.display());

    let owned: OwnedFd = session
        .open(
            &primary,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
        )
        .with_context(|| format!("session.open({})", primary.display()))?;
    let device_fd = DrmDeviceFd::new(DeviceFd::from(owned));
    let (mut drm, drm_notifier) =
        smithay::backend::drm::DrmDevice::new(device_fd.clone(), true).context("DrmDevice::new")?;
    let gbm = GbmDevice::new(device_fd.clone()).context("GbmDevice::new")?;

    // EGL + GLES on GBM — used for presentation when a scanout surface is available
    let egl_display = unsafe { EGLDisplay::new(gbm.clone()) }.context("EGLDisplay::new(gbm)")?;
    let egl_context = EGLContext::new(&egl_display).context("EGLContext::new")?;
    let renderer = unsafe { GlesRenderer::new(egl_context) }.context("GlesRenderer::new")?;

    // ---- Connector enumeration + modeset / DrmSurface (presentation leap) ----
    use smithay::backend::drm::DrmSurface;
    use smithay::reexports::drm::control::{
        connector, Device as ControlDevice, Mode as DrmMode, ModeTypeFlags,
    };

    let resources = drm
        .resource_handles()
        .context("drm.resource_handles for connector scan")?;
    let mut connector_summaries = Vec::new();
    let mut picked: Option<(connector::Handle, DrmMode, usize)> = None;

    for (conn_i, conn) in resources.connectors().iter().enumerate() {
        let info = match drm.get_connector(*conn, true) {
            Ok(i) => i,
            Err(err) => {
                tracing::debug!(?err, "get_connector failed");
                continue;
            }
        };
        let name = format!("{:?}-{}", info.interface(), info.interface_id());
        let connected = info.state() == connector::State::Connected;
        let modes = info.modes();
        let preferred = modes
            .iter()
            .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
            .or_else(|| modes.first());
        let mode_summary = preferred.map(|m| {
            let sz = m.size();
            (sz.0 as i32, sz.1 as i32, m.vrefresh() as i32 * 1000)
        });
        connector_summaries.push((name.clone(), connected, mode_summary));
        if connected && picked.is_none() {
            if let Some(m) = preferred.copied() {
                picked = Some((*conn, m, conn_i.min(drm.crtcs().len().saturating_sub(1))));
            }
        }
    }

    let modeset_plan = plan_drm_modeset(
        &connector_summaries,
        w_from_env_or_default(),
        h_from_env_or_default(),
        refresh_mhz,
    );
    eprintln!(
        "[slopos-compositor] DRM modeset plan: connector={} {}x{}@{}mhz crtcs={} connectors={}",
        modeset_plan.connector_name,
        modeset_plan.mode_w,
        modeset_plan.mode_h,
        modeset_plan.refresh_mhz,
        drm.crtcs().len(),
        connector_summaries.len()
    );
    let output_scale = detect_output_scale_from_env().unwrap_or(OutputScale::IDENTITY);
    let output_scale_i32 = output_scale.integer_buffer_scale();
    let effective_scale =
        OutputScale::new(output_scale_i32 as u32, 1).unwrap_or(OutputScale::IDENTITY);
    let logical_output_size = crate::scale_physical_to_logical(
        (modeset_plan.mode_w, modeset_plan.mode_h),
        effective_scale,
    );
    eprintln!(
        "[slopos-compositor] {} (DRM wl_output buffer scale={} effective={} logical canvas={}x{})",
        output_scale_summary(output_scale),
        output_scale_i32,
        output_scale_summary(effective_scale),
        logical_output_size.0,
        logical_output_size.1,
    );
    for stage in drm_presentation_pipeline() {
        tracing::debug!(stage = stage.as_str(), "drm presentation pipeline stage");
    }

    // ---- Real HDR / VRR capability probe on the chosen connector ----
    // Replaces the old hardcoded `hdr_supported = false`: these read the actual
    // kernel properties, so a capable display reports true and a VM reports
    // false for the honest reason (vmwgfx exposes neither property).
    if let Some((conn, _mode, _idx)) = picked {
        match crate::drm_props::PropertyIndex::read(&drm, conn) {
            Ok(conn_props) => {
                let caps = crate::drm_props::probe_hdr(&conn_props);
                eprintln!("[slopos-compositor] connector HDR: {}", caps.summary());
                tracing::info!(
                    hdr_metadata = caps.has_hdr_metadata,
                    bt2020 = caps.has_bt2020_colorspace,
                    max_bpc = ?caps.max_bpc,
                    hdr10_capable = caps.hdr10_capable(),
                    "connector HDR capability probed from DRM properties"
                );
                hdr_caps.hdr_supported = caps.hdr10_capable();
                if caps.hdr10_capable() {
                    hdr_caps
                        .supported_color_spaces
                        .push(crate::hdr::ColorSpace::Rec2020);
                }

                let crtc_props = drm
                    .crtcs()
                    .first()
                    .and_then(|&c| crate::drm_props::PropertyIndex::read(&drm, c).ok())
                    .unwrap_or_default();
                let vrr = crate::drm_props::probe_vrr(&conn_props, &crtc_props);
                eprintln!(
                    "[slopos-compositor] connector VRR: capable={} controllable={} enabled={}",
                    vrr.capable, vrr.controllable, vrr.enabled
                );

                // Apply what the user asked for, but only what the hardware allows.
                if display_policy.hdr_requested {
                    let md = crate::drm_props::HdrOutputMetadata::hdr10(1000, 0.005, 1000, 400);
                    match crate::drm_props::apply_hdr10(&drm, conn, &conn_props, &md) {
                        Ok(Some(_blob)) => {
                            eprintln!("[slopos-compositor] HDR10 metadata applied to connector")
                        }
                        Ok(None) => eprintln!(
                            "[slopos-compositor] HDR requested but connector is not HDR10-capable; staying SDR"
                        ),
                        Err(err) => {
                            eprintln!("[slopos-compositor] HDR apply failed: {err}")
                        }
                    }
                }
                if display_policy.vrr_adaptive {
                    if let Some(&crtc) = drm.crtcs().first() {
                        match crate::drm_props::set_vrr_enabled(
                            &drm, crtc, &crtc_props, vrr, true,
                        ) {
                            Ok(true) => eprintln!("[slopos-compositor] VRR_ENABLED set on CRTC"),
                            Ok(false) => eprintln!(
                                "[slopos-compositor] VRR requested but connector is not vrr_capable; fixed refresh"
                            ),
                            Err(err) => eprintln!("[slopos-compositor] VRR apply failed: {err}"),
                        }
                    }
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "could not read connector properties");
            }
        }
    }

    // Attempt real DrmSurface on first CRTC + connected connector (scanout path).
    let mut drm_surface: Option<DrmSurface> = None;
    if let Some((conn, mode, _idx)) = picked {
        if let Some(&crtc) = drm.crtcs().first() {
            match drm.create_surface(crtc, mode, &[conn]) {
                Ok(surface) => {
                    eprintln!(
                        "[slopos-compositor] DRM scanout surface created (crtc+connector modeset)"
                    );
                    tracing::info!(
                        stage = DrmPresentationStage::CreateDrmSurface.as_str(),
                        "DrmSurface ready for pageflip presentation"
                    );
                    drm_surface = Some(surface);
                }
                Err(err) => {
                    tracing::warn!(
                        ?err,
                        "create_surface failed — continuing protocol loop without scanout"
                    );
                    eprintln!("[slopos-compositor] DRM create_surface failed: {err:?} (protocol-only fallback)");
                }
            }
        }
    } else {
        eprintln!(
            "[slopos-compositor] no connected connector; virtual mode {}x{}",
            modeset_plan.mode_w, modeset_plan.mode_h
        );
    }
    // Renderer and device stay alive: the renderer composites client surfaces
    // into the DrmCompositor's GBM swapchain, the device owns cursor sizing.
    let mut renderer = renderer;

    // ---- GL composition (ROADMAP 1.2) ----
    // Build a DrmCompositor over the scanout surface so client buffers reach
    // the screen. The dumb-buffer path below stays only as a fallback for when
    // this cannot be constructed (no GBM formats, inactive surface, …), because
    // a solid flip at least proves the modeset works.
    let mut drm_compositor: Option<RetroDrmCompositor> = None;
    if let Some(surface) = drm_surface.take() {
        let output_for_comp = Output::new(
            modeset_plan.connector_name.clone(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "SLOPOS-I".into(),
                model: "DRM".into(),
            },
        );
        output_for_comp.change_current_state(
            Some(Mode {
                size: (modeset_plan.mode_w, modeset_plan.mode_h).into(),
                refresh: if modeset_plan.refresh_mhz > 0 {
                    modeset_plan.refresh_mhz
                } else {
                    refresh_mhz
                },
            }),
            Some(Transform::Normal),
            Some(Scale::Integer(output_scale_i32)),
            None,
        );
        let allocator = GbmAllocator::new(
            gbm.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        );
        let exporter = GbmFramebufferExporter::new(gbm.clone(), None);
        let renderer_formats = renderer.egl_context().dmabuf_render_formats().clone();
        match DrmCompositor::new(
            &output_for_comp,
            surface,
            None,
            allocator,
            exporter,
            [DrmFourcc::Xrgb8888, DrmFourcc::Argb8888],
            renderer_formats,
            drm.cursor_size(),
            Some(gbm.clone()),
        ) {
            Ok(comp) => {
                eprintln!(
                    "[slopos-compositor] DRM GL compositor ready ({}x{}) — client surfaces will be composited",
                    modeset_plan.mode_w, modeset_plan.mode_h
                );
                tracing::info!(
                    stage = DrmPresentationStage::PageFlipOrPresent.as_str(),
                    "DrmCompositor initialized; GL composition active"
                );
                drm_compositor = Some(comp);
            }
            Err(err) => {
                eprintln!(
                    "[slopos-compositor] DrmCompositor init failed ({err}); falling back to solid dumb-buffer present"
                );
                tracing::warn!(error = %err, "DrmCompositor init failed");
            }
        }
    }
    let composition_active = drm_compositor.is_some();

    // Keep DrmDevice alive for the session (ControlDevice for page_flip path).
    let _drm = drm;

    // ---- Pageflip / present attempt (not drop-the-surface) ----
    // Allocate ONE dumb XRGB8888 buffer + framebuffer for the whole session and
    // issue a modeset commit or page_flip with it. `_scanout_owners` must stay
    // in scope for as long as `armed_fb` is flipped, or the kernel frees the
    // backing object mid-flight.
    let mut scanout_armed = false;
    let mut armed_fb: Option<smithay::reexports::drm::control::framebuffer::Handle> = None;
    let mut _scanout_owners = None;
    if let Some(surface) = drm_surface.as_ref().filter(|_| !composition_active) {
        match arm_scanout_framebuffer(surface, modeset_plan.mode_w, modeset_plan.mode_h) {
            Ok((buffer, fb)) => {
                let handle = *fb.as_ref();
                match present_armed_frame(surface, handle, modeset_plan.mode_w, modeset_plan.mode_h)
                {
                    Ok(()) => {
                        scanout_armed = true;
                        armed_fb = Some(handle);
                        _scanout_owners = Some((buffer, fb));
                        eprintln!(
                            "[slopos-compositor] DRM pageflip/commit present succeeded ({}x{})",
                            modeset_plan.mode_w, modeset_plan.mode_h
                        );
                        tracing::info!(
                            stage = DrmPresentationStage::PageFlipOrPresent.as_str(),
                            "dumb-buffer pageflip/commit path armed"
                        );
                    }
                    Err(err) => {
                        tracing::warn!(
                            error = %err,
                            "DRM present path failed; surface kept for session, protocol continues"
                        );
                        eprintln!("[slopos-compositor] DRM present failed: {err:#}");
                    }
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "could not allocate scanout framebuffer");
                eprintln!("[slopos-compositor] scanout framebuffer alloc failed: {err:#}");
            }
        }
    }
    // Retain surface for the process lifetime so create_surface is not a no-op.
    // Re-present periodically so scanout is continuous when armed (not one-shot).
    let drm_surface_keepalive = drm_surface;
    let scanout_armed = scanout_armed;
    let armed_fb = armed_fb;
    let present_w = modeset_plan.mode_w;
    let present_h = modeset_plan.mode_h;

    // Wayland socket
    let socket = ListeningSocketSource::new_auto().context("ListeningSocketSource")?;
    let socket_name = socket.socket_name().to_string_lossy().into_owned();
    eprintln!("[slopos-compositor] WAYLAND_DISPLAY={socket_name} (DRM session)");
    println!("WAYLAND_DISPLAY={socket_name}");
    let control_listener = std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR")
        .map(PathBuf::from)
        .map(|runtime| {
            SessionControlListener::bind(&runtime)
                .map_err(|error| anyhow!("bind session control socket: {error}"))
        })
        .transpose()?;
    crate::publish_session_readiness(&socket_name, logical_output_size.0, logical_output_size.1)
        .context("publish private session readiness")?;
    std::env::set_var("SLOPOS_CLIENT_WAYLAND_DISPLAY", &socket_name);
    std::env::set_var("WAYLAND_DISPLAY", &socket_name);

    loop_handle
        .insert_source(socket, |stream, _, state| {
            if let Err(err) = state
                .display_handle
                .insert_client(stream, Arc::new(ClientState::default()))
            {
                tracing::error!("insert_client: {err}");
            }
        })
        .map_err(|e| anyhow!("insert wayland socket: {e}"))?;
    register_wayland_display_source(&loop_handle, display)
        .context("insert Wayland display source")?;

    // Advertise connector mode when known; else env/default virtual size.
    let w = logical_output_size.0;
    let h = logical_output_size.1;
    std::env::set_var("SLOPOS_COMPOSITOR_WIDTH", w.to_string());
    std::env::set_var("SLOPOS_COMPOSITOR_HEIGHT", h.to_string());
    let out_refresh = if modeset_plan.refresh_mhz > 0 {
        modeset_plan.refresh_mhz
    } else {
        refresh_mhz
    };
    let output = Output::new(
        modeset_plan.connector_name.clone(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "SLOPOS-I".into(),
            model: "DRM Output".into(),
        },
    );
    let mode = Mode {
        size: (w, h).into(),
        refresh: out_refresh,
    };
    output.change_current_state(
        Some(mode),
        Some(Transform::Normal),
        Some(Scale::Integer(output_scale_i32)),
        Some((0, 0).into()),
    );
    output.set_preferred(mode);
    output.create_global::<DrmSessionState>(&dh);

    // Udev hotplug
    let udev = UdevBackend::new(&seat_name).context("UdevBackend::new")?;
    loop_handle
        .insert_source(udev, |event, _, state| match event {
            UdevEvent::Added { device_id, path } => {
                tracing::info!("udev added device_id={device_id:?} path={}", path.display());
                state.note_udev_event(format!("added:{}", path.display()));
            }
            UdevEvent::Changed { device_id } => {
                tracing::debug!("udev changed {device_id:?}");
            }
            UdevEvent::Removed { device_id } => {
                tracing::info!("udev removed {device_id:?}");
                state.note_udev_event(format!("removed:{device_id:?}"));
            }
        })
        .map_err(|e| anyhow!("insert udev: {e}"))?;

    // Libinput via seat interface
    let mut libinput_context = input::Libinput::new_with_udev::<
        LibinputSessionInterface<LibSeatSession>,
    >(session.clone().into());
    libinput_context
        .udev_assign_seat(&seat_name)
        .map_err(|_| anyhow!("libinput udev_assign_seat failed"))?;
    let libinput_backend = LibinputInputBackend::new(libinput_context);
    loop_handle
        .insert_source(libinput_backend, |event, _, state| {
            state.handle_libinput(event);
        })
        .map_err(|e| anyhow!("insert libinput: {e}"))?;

    // DRM vblank: frame_submitted() MUST follow each queued flip or the
    // swapchain runs out of buffers and rendering stalls after a few frames.
    loop_handle
        .insert_source(drm_notifier, |event, _meta, state| match event {
            smithay::backend::drm::DrmEvent::VBlank(_crtc) => {
                if let Some(comp) = state.drm_compositor.as_mut() {
                    if let Err(err) = comp.frame_submitted() {
                        tracing::warn!(error = %err, "frame_submitted failed");
                    }
                }
            }
            smithay::backend::drm::DrmEvent::Error(err) => {
                tracing::error!(error = %err, "DRM device error");
            }
        })
        .map_err(|e| anyhow!("insert drm notifier: {e}"))?;

    // Session notifier (VT switch)
    loop_handle
        .insert_source(session_notifier, |event, _, state| match event {
            SessionEvent::PauseSession => {
                tracing::info!("session paused");
                state.active.store(false, Ordering::SeqCst);
            }
            SessionEvent::ActivateSession => {
                tracing::info!("session activated");
                state.active.store(true, Ordering::SeqCst);
            }
        })
        .map_err(|e| anyhow!("insert session notifier: {e}"))?;

    // Keep GPU objects alive for the session lifetime
    let _gbm = gbm;
    // Presentation: when `_drm_surface` is Some, pageflip path is armed for follow-on
    // frame queueing; protocol loop always runs.
    tracing::info!(
        stage = DrmPresentationStage::ProtocolLoop.as_str(),
        "DRM session entering protocol + seat event loop"
    );

    let mut state = DrmSessionState {
        display_handle: dh,
        compositor_state,
        shm_state,
        seat_state,
        seat,
        xdg_shell_state,
        data_device_state,
        primary_selection_state,
        output_manager_state,
        layer_shell_state,
        foreign_toplevel_list,
        session_lock_state,
        locked: false,
        lock_surfaces: Vec::new(),
        wayland_socket_name: socket_name,
        outputs: vec![output],
        windows: Vec::new(),
        workspace_state: WorkspaceState::new(),
        layer_surfaces: Vec::new(),
        popup_manager: PopupManager::default(),
        popup_grab: None,
        activated_window_id: None,
        last_minimized_window_id: None,
        active: Arc::new(AtomicBool::new(true)),
        udev_events: Vec::new(),
        pointer_location: Point::from((w as f64 / 2.0, h as f64 / 2.0)),
        output_size: (w, h),
        serial: 0,
        clipboard_source: None,
        primary_source: None,
        clipboard_data: HashMap::new(),
        primary_data: HashMap::new(),
        server_dnd_data: HashMap::new(),
        dnd_icon: None,
        running: true,
        frame_dirty: true,
        need_full_redraw: true,
        drm_compositor,
        physical_output_size: (modeset_plan.mode_w, modeset_plan.mode_h),
        cursor_status: CursorImageStatus::default_named(),
        interactive_grab: None,
        left_button_down: false,
        last_pointer_press: None,
    };

    // The session control socket is part of the event loop, not a polled
    // side-channel. This keeps the compositor asleep when idle while still
    // waking immediately for shell requests such as Minimize or Fill.
    if let Some(listener) = control_listener {
        loop_handle
            .insert_source(
                Generic::new(listener, Interest::READ, CalloopMode::Level),
                |_, listener, state| {
                    for request in listener.drain() {
                        state.apply_session_control_request(request);
                    }
                    Ok(PostAction::Continue)
                },
            )
            .map_err(|error| anyhow!("insert session control socket: {error}"))?;
    }

    eprintln!(
        "[slopos-compositor] DRM session loop running (Wayland + seat + udev + libinput + layer-shell + foreign-toplevel; scanout_armed={scanout_armed})"
    );
    // The session supervisor owns shell startup.  The compositor must not
    // launch a second shell here: doing so races the supervisor, creates a
    // duplicate desktop client, and breaks the single-shell/private-socket
    // topology.  Explicit user actions below may still launch first-party
    // clients such as Finder or the lock screen.
    let clock = Clock::<Monotonic>::new();
    while state.running {
        // Keep workspace map honest if clients disconnect without destroy order.
        state.prune_dead_windows();
        state.cleanup_popup_state();
        let should_render = state.frame_dirty || state.need_full_redraw;
        if should_render {
            let mut presented = false;
            let force_present = state.need_full_redraw;

            if state.drm_compositor.is_some() {
                // Composite every visible client surface plus layer-shell chrome
                // into the GBM swapchain and page-flip it. This is what puts client
                // pixels on a real screen; the dumb-buffer path below only ever
                // showed a solid colour.
                //
                // Elements are collected before taking the &mut on the compositor:
                // both live on `state`, and the borrow checker cannot split fields
                // across the helper call.
                let elements = collect_render_elements(&mut renderer, &state);
                let clear = if state.locked {
                    DRM_LOCK_CLEAR_COLOR
                } else {
                    DRM_CLEAR_COLOR
                };
                // QA: honour a pending SIGUSR1 screenshot request before the real
                // scanout render (offscreen readback; see screenshot.rs).
                crate::screenshot::capture_if_requested(
                    &mut renderer,
                    &elements,
                    state.physical_output_size,
                    clear,
                );
                if let Some(comp) = state.drm_compositor.as_mut() {
                    match comp.render_frame::<_, _>(
                        &mut renderer,
                        &elements,
                        clear,
                        FrameFlags::DEFAULT,
                    ) {
                        Ok(result) => {
                            presented = true;
                            if !result.is_empty {
                                // Drop the borrow of `result` before queueing.
                                drop(result);
                                if let Err(err) = comp.queue_frame(()) {
                                    tracing::debug!(error = %err, "queue_frame failed");
                                    // A failed queue leaves no pending flip, so the
                                    // vblank handler will not fire; recover on the
                                    // next event-loop wake.
                                    let _ = comp.frame_submitted();
                                }
                            }
                        }
                        Err(err) => {
                            tracing::warn!(error = %err, "render_frame failed");
                        }
                    }
                }
            } else if scanout_armed && force_present {
                if let (Some(surface), Some(fb)) = (drm_surface_keepalive.as_ref(), armed_fb) {
                    match present_armed_frame(surface, fb, present_w, present_h) {
                        Ok(()) => presented = true,
                        Err(err) => {
                            tracing::debug!(error = %err, "DRM present failed");
                        }
                    }
                }
            }

            if presented {
                state.frame_dirty = false;
                state.need_full_redraw = false;
                let _ = frame_scheduler.record_frame();
                release_frame_callbacks(&state, &clock);
            }
        }

        event_loop
            .dispatch(None, &mut state)
            .context("event_loop.dispatch")?;
    }
    let _ = drm_surface_keepalive;

    Ok(())
}

// ---------------------------------------------------------------------------
// Per-client data
// ---------------------------------------------------------------------------

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        eprintln!("[slopos-compositor/drm] client connected");
    }
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        eprintln!("[slopos-compositor/drm] client disconnected");
    }
}

// ---------------------------------------------------------------------------
// Tracked windows / layers
// ---------------------------------------------------------------------------

struct MappedWindow {
    toplevel: ToplevelSurface,
    foreign: ForeignToplevelHandle,
    window_id: String,
    /// Wayland app_id captured at map time for compositor-authoritative menu
    /// activation publication.
    app_id: String,
    position: Point<i32, Logical>,
    size: Size<i32, Logical>,
    presentation_state: WindowPresentationState,
    restore_state: Option<crate::WindowRestoreState>,
    minimized: bool,
}

impl MappedWindow {
    fn geometry(&self) -> WindowGeometry {
        WindowGeometry {
            x: self.position.x,
            y: self.position.y,
            width: self.size.w,
            height: self.size.h,
        }
    }
}

struct MappedLayer {
    surface: LayerSurface,
    layer: Layer,
    #[allow(dead_code)]
    namespace: String,
    /// Output-local placement of this layer surface (menu strip, dock, …).
    geo: Rectangle<i32, Logical>,
    /// Exclusive work-area reservation requested by the layer client.
    exclusive_zone: i32,
}

fn layer_geometry_for(
    namespace: &str,
    layer: Layer,
    output: (i32, i32),
    requested: (i32, i32),
    anchor: Anchor,
    margins: Margins,
) -> Rectangle<i32, Logical> {
    let (ow, oh) = output;
    let (fallback_w, fallback_h, fallback_anchor) = match namespace {
        "slopos-i-menu" | "menu-bar" => (ow, 24, Anchor::TOP | Anchor::LEFT | Anchor::RIGHT),
        "slopos-i-dock" | "dock" => (ow, 64, Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT),
        "slopos-i-menu-popup" => (1, 1, Anchor::TOP | Anchor::LEFT),
        _ => (
            ow,
            oh,
            Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
        ),
    };
    let anchor = if anchor.is_empty() {
        fallback_anchor
    } else {
        anchor
    };
    let left = margins.left.max(0);
    let right = margins.right.max(0);
    let top = margins.top.max(0);
    let bottom = margins.bottom.max(0);
    let w = if requested.0 == 0 {
        if anchor.anchored_horizontally() {
            (ow - left - right).max(1)
        } else {
            fallback_w
        }
    } else {
        requested.0
    }
    .clamp(1, ow.max(1));
    let h = if requested.1 == 0 {
        if anchor.anchored_vertically() {
            (oh - top - bottom).max(1)
        } else {
            fallback_h
        }
    } else {
        requested.1
    }
    .clamp(1, oh.max(1));
    let x = if anchor.contains(Anchor::LEFT) && anchor.contains(Anchor::RIGHT) {
        left
    } else if anchor.contains(Anchor::RIGHT) {
        (ow - w - right).max(0)
    } else if anchor.contains(Anchor::LEFT) {
        left
    } else {
        (ow - w) / 2
    };
    let y = if anchor.contains(Anchor::TOP) && anchor.contains(Anchor::BOTTOM) {
        top
    } else if anchor.contains(Anchor::BOTTOM) {
        (oh - h - bottom).max(0)
    } else if anchor.contains(Anchor::TOP) {
        top
    } else {
        (oh - h) / 2
    };
    let _ = layer;
    Rectangle::new((x, y).into(), (w, h).into())
}

fn layer_surface_request(surface: &LayerSurface) -> ((i32, i32), Anchor, Margins, i32) {
    with_states(surface.wl_surface(), |states| {
        let mut cached = states.cached_state.get::<LayerSurfaceCachedState>();
        let current = *cached.current();
        (
            (current.size.w, current.size.h),
            current.anchor,
            current.margin,
            current.exclusive_zone.into(),
        )
    })
}

fn layer_geo_contains(geo: &Rectangle<i32, Logical>, pos: Point<f64, Logical>) -> bool {
    let x = pos.x as i32;
    let y = pos.y as i32;
    x >= geo.loc.x && y >= geo.loc.y && x < geo.loc.x + geo.size.w && y < geo.loc.y + geo.size.h
}

// ---------------------------------------------------------------------------
// Main session state
// ---------------------------------------------------------------------------

struct DrmSessionState {
    /// Latest client-set cursor image; drawn topmost each frame.
    cursor_status: CursorImageStatus,
    interactive_grab: Option<InteractiveGrab>,
    left_button_down: bool,
    /// The most recent left-button press delivered to an application surface.
    /// xdg_toplevel.move/resize must consume this exact serial while held.
    last_pointer_press: Option<PointerPress>,
    /// GL compositor over the scanout surface. `None` when it could not be
    /// built, in which case the session falls back to a solid dumb-buffer flip.
    /// Lives in the state so the vblank handler can call `frame_submitted()`.
    drm_compositor: Option<RetroDrmCompositor>,
    display_handle: DisplayHandle,
    compositor_state: CompositorState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,
    seat: Seat<Self>,
    xdg_shell_state: XdgShellState,
    data_device_state: DataDeviceState,
    primary_selection_state: PrimarySelectionState,
    #[allow(dead_code)]
    output_manager_state: OutputManagerState,
    layer_shell_state: WlrLayerShellState,
    foreign_toplevel_list: ForeignToplevelListState,
    session_lock_state: SessionLockManagerState,
    locked: bool,
    lock_surfaces: Vec<(Output, LockSurface)>,
    wayland_socket_name: String,
    #[allow(dead_code)]
    outputs: Vec<Output>,
    windows: Vec<MappedWindow>,
    workspace_state: WorkspaceState,
    layer_surfaces: Vec<MappedLayer>,
    popup_manager: PopupManager,
    popup_grab: Option<PopupGrab<DrmSessionState>>,
    activated_window_id: Option<String>,
    /// Generic Restore targets the most recently minimized client. Focus
    /// moves to another visible window after minimize, so the active id
    /// alone cannot identify the Dock restore target.
    last_minimized_window_id: Option<String>,
    active: Arc<AtomicBool>,
    udev_events: Vec<String>,
    pointer_location: Point<f64, Logical>,
    output_size: (i32, i32),
    /// Physical scanout size; `output_size` is the logical compositor space.
    physical_output_size: (i32, i32),
    serial: u32,
    clipboard_source: Option<SelectionSource>,
    primary_source: Option<SelectionSource>,
    clipboard_data: HashMap<String, Vec<u8>>,
    primary_data: HashMap<String, Vec<u8>>,
    server_dnd_data: HashMap<String, Vec<u8>>,
    dnd_icon: Option<WlSurface>,
    running: bool,
    /// A frame is produced only after damage, input, a configure, or a frame
    /// event. This prevents the DRM path from repainting an idle desktop.
    frame_dirty: bool,
    /// Set on workspace switch so the next present/composite pass redraws fully.
    need_full_redraw: bool,
}

impl DrmSessionState {
    fn next_serial(&mut self) -> Serial {
        self.serial = self.serial.wrapping_add(1);
        Serial::from(self.serial)
    }

    fn note_udev_event(&mut self, msg: String) {
        self.udev_events.push(msg);
        if self.udev_events.len() > 64 {
            self.udev_events.remove(0);
        }
    }

    /// Drop dead xdg windows and keep `workspace_state` in sync.
    fn prune_dead_windows(&mut self) {
        let before: Vec<String> = self.windows.iter().map(|w| w.window_id.clone()).collect();
        self.windows.retain(|w| w.toplevel.alive());
        let alive: std::collections::HashSet<String> =
            self.windows.iter().map(|w| w.window_id.clone()).collect();
        let mut removed = false;
        for id in before {
            if !alive.contains(&id) {
                self.workspace_state.remove_window(&id);
                removed = true;
            }
        }
        if self
            .last_minimized_window_id
            .as_ref()
            .is_some_and(|id| !alive.contains(id))
        {
            self.last_minimized_window_id = None;
        }
        if removed {
            self.request_full_redraw();
        }
    }

    /// Window ids that should present / list on the active workspace (bottom→top order).
    ///
    /// Client GL scanout of SHM trees is not yet wired on the DRM path (dumb-buffer
    /// pageflip only); this filter is the live listing contract for focus and any
    /// future composite path.
    fn window_ids_for_present(&self) -> Vec<&str> {
        let order: Vec<&str> = self
            .windows
            .iter()
            .filter(|w| !w.minimized)
            .map(|w| w.window_id.as_str())
            .collect();
        visible_paint_order(&self.workspace_state, &order)
    }

    /// Focus topmost visible window after map/destroy/workspace change; clear if none.
    fn apply_focus_after_workspace_switch(&mut self) {
        let order: Vec<&str> = self
            .windows
            .iter()
            .filter(|w| !w.minimized)
            .map(|w| w.window_id.as_str())
            .collect();
        let target =
            focus_window_after_workspace_switch(&self.workspace_state, &order).map(str::to_owned);
        if let Some(id) = target {
            if let Some(w) = self.windows.iter().find(|w| w.window_id == id) {
                let surf = w.toplevel.wl_surface().clone();
                self.focus_surface(Some(surf));
                return;
            }
        }
        self.focus_surface(None);
    }

    fn request_full_redraw(&mut self) {
        self.frame_dirty = true;
        self.need_full_redraw = true;
    }

    fn request_redraw(&mut self) {
        self.frame_dirty = true;
    }

    fn apply_session_control_request(&mut self, request: SessionControlRequest) {
        match request {
            SessionControlRequest::FocusedWindow { action } => {
                self.apply_focused_window_action(action);
            }
            SessionControlRequest::ActivateApplication { bundle_id } => {
                self.activate_application(&bundle_id);
            }
            SessionControlRequest::FocusedApplicationMenu {
                bundle_id,
                action_id,
            } => {
                tracing::warn!(
                    %bundle_id,
                    %action_id,
                    "application menu request reached compositor without an app endpoint"
                );
            }
        }
    }

    /// Activate a matching mapped client on behalf of shell chrome.
    ///
    /// The shell sends only a semantic application id; this backend owns the
    /// actual restore, stacking, focus, and active-toplevel update.
    fn activate_application(&mut self, bundle_id: &str) {
        let Some(idx) = self
            .windows
            .iter()
            .rposition(|window| window.app_id == bundle_id)
        else {
            tracing::debug!(%bundle_id, "application activation found no mapped client");
            return;
        };

        let window_id = self.windows[idx].window_id.clone();
        if self.windows[idx].minimized {
            let surface = self.windows[idx].toplevel.clone();
            self.set_window_presentation_state(&surface, WindowPresentationState::Normal);
            self.windows[idx].minimized = false;
            if self.last_minimized_window_id.as_deref() == Some(window_id.as_str()) {
                self.last_minimized_window_id = None;
            }
        }
        self.focus_window_at_index(idx);
        tracing::info!(%bundle_id, %window_id, "activated existing application client");
    }

    fn apply_focused_window_action(&mut self, action: WindowPresentationAction) {
        let window_id = if action == WindowPresentationAction::Restore {
            self.last_minimized_window_id
                .as_ref()
                .and_then(|id| {
                    self.windows
                        .iter()
                        .find(|window| window.window_id == *id && window.minimized)
                        .map(|window| window.window_id.clone())
                })
                .or_else(|| self.activated_window_id.clone())
        } else {
            self.activated_window_id.clone()
        };
        let Some(window_id) = window_id else {
            tracing::debug!(
                ?action,
                "ignored focused-window action with no focused toplevel"
            );
            return;
        };
        let Some(idx) = self
            .windows
            .iter()
            .position(|window| window.window_id == window_id)
        else {
            tracing::debug!(%window_id, "focused-window action targeted a stale toplevel");
            return;
        };

        let current = self.windows[idx].presentation_state;
        let target = match action {
            WindowPresentationAction::ToggleZoom => {
                if matches!(current, WindowPresentationState::Normal) {
                    WindowPresentationState::SmartZoomed
                } else {
                    WindowPresentationState::Normal
                }
            }
            WindowPresentationAction::SmartZoom => WindowPresentationState::SmartZoomed,
            WindowPresentationAction::Fill => WindowPresentationState::Filled,
            WindowPresentationAction::ToggleFullscreen => {
                if current == WindowPresentationState::Fullscreen {
                    WindowPresentationState::Normal
                } else {
                    WindowPresentationState::Fullscreen
                }
            }
            WindowPresentationAction::Fullscreen => WindowPresentationState::Fullscreen,
            WindowPresentationAction::Minimize => WindowPresentationState::Minimized,
            WindowPresentationAction::Restore => WindowPresentationState::Normal,
            WindowPresentationAction::Close => {
                self.windows[idx].toplevel.send_close();
                tracing::info!(%window_id, "sent close request to focused toplevel");
                return;
            }
        };

        let surface = self.windows[idx].toplevel.clone();
        self.set_window_presentation_state(&surface, target);
        self.windows[idx].minimized = target == WindowPresentationState::Minimized;
        if target == WindowPresentationState::Minimized {
            self.last_minimized_window_id = Some(window_id.clone());
        } else if target == WindowPresentationState::Normal
            && self.last_minimized_window_id.as_deref() == Some(window_id.as_str())
        {
            self.last_minimized_window_id = None;
        }
        tracing::info!(
            %window_id,
            ?action,
            state = ?target,
            "applied compositor presentation request"
        );
        if self.windows[idx].minimized {
            self.apply_focus_after_workspace_switch();
        } else if action == WindowPresentationAction::Restore {
            self.focus_window_at_index(idx);
        } else {
            self.request_full_redraw();
        }
    }

    fn active_lock_surface(&self) -> Option<WlSurface> {
        self.lock_surfaces
            .first()
            .map(|(_, lock)| lock.wl_surface().clone())
    }

    /// Spawn a first-party binary as a Wayland client of this compositor.
    fn spawn_client(&self, bin: &str) {
        crate::client_spawn::spawn_client(&self.wayland_socket_name, bin);
    }

    /// Super+workspace (or other key) entry points — full redraw + focus rebind.
    #[allow(dead_code)] // seat Super+key filter will call these when wired
    fn cycle_workspace_next(&mut self) {
        self.workspace_state.cycle_next();
        self.request_full_redraw();
        eprintln!(
            "[slopos-compositor/drm] {}",
            self.workspace_state.summary_line()
        );
        self.apply_focus_after_workspace_switch();
    }

    #[allow(dead_code)]
    fn cycle_workspace_prev(&mut self) {
        self.workspace_state.cycle_prev();
        self.request_full_redraw();
        eprintln!(
            "[slopos-compositor/drm] {}",
            self.workspace_state.summary_line()
        );
        self.apply_focus_after_workspace_switch();
    }

    #[allow(dead_code)]
    fn activate_workspace_index(&mut self, index: u8) {
        if let Some(ws) = WorkspaceId::new(index) {
            if self.workspace_state.activate(ws) {
                self.request_full_redraw();
                eprintln!(
                    "[slopos-compositor/drm] {}",
                    self.workspace_state.summary_line()
                );
                self.apply_focus_after_workspace_switch();
            }
        }
    }

    fn begin_interactive_grab(
        &mut self,
        surface: &ToplevelSurface,
        kind: InteractiveGrabKind,
        seat: &wl_seat::WlSeat,
        serial: Serial,
    ) {
        let requested_surface = surface.wl_surface();
        let same_surface = self
            .last_pointer_press
            .as_ref()
            .is_some_and(|press| press.surface == *requested_surface);
        let pressed_serial = self
            .last_pointer_press
            .as_ref()
            .map(|press| u32::from(press.serial));
        let authorized = pointer_grab_request_is_valid(
            u32::from(serial),
            pressed_serial,
            same_surface,
            self.left_button_down,
            self.seat.owns(seat),
        );
        if !authorized {
            tracing::debug!(
                request_serial = u32::from(serial),
                ?kind,
                same_surface,
                "rejecting unauthorized xdg move/resize request"
            );
            return;
        }
        let Some(window) = self
            .windows
            .iter()
            .find(|w| w.toplevel.wl_surface() == requested_surface)
        else {
            return;
        };
        self.interactive_grab = Some(InteractiveGrab {
            window_id: window.window_id.clone(),
            kind,
            start_pointer_x: self.pointer_location.x.round() as i32,
            start_pointer_y: self.pointer_location.y.round() as i32,
            start_geometry: window.geometry(),
        });
    }

    fn update_interactive_grab(&mut self) {
        let Some(grab) = self.interactive_grab.clone() else {
            return;
        };
        let Some(idx) = self
            .windows
            .iter()
            .position(|w| w.window_id == grab.window_id)
        else {
            self.interactive_grab = None;
            return;
        };
        let next = geometry_for_interactive_grab(
            &grab,
            self.pointer_location.x.round() as i32,
            self.pointer_location.y.round() as i32,
            160,
            96,
            self.output_size.0,
            self.output_size.1,
        );
        if self.windows[idx].geometry() == next {
            return;
        }
        self.windows[idx].position = Point::from((next.x, next.y));
        self.windows[idx].size = Size::from((next.width, next.height));
        if matches!(grab.kind, InteractiveGrabKind::Resize(_)) {
            let toplevel = self.windows[idx].toplevel.clone();
            toplevel.with_pending_state(|state| {
                state.size = Some(Size::from((next.width, next.height)));
                state.states.set(xdg_toplevel::State::Resizing);
            });
            toplevel.send_configure();
        }
        self.request_full_redraw();
    }

    fn finish_interactive_grab(&mut self) {
        let Some(grab) = self.interactive_grab.take() else {
            return;
        };
        if matches!(grab.kind, InteractiveGrabKind::Resize(_)) {
            if let Some(window) = self.windows.iter().find(|w| w.window_id == grab.window_id) {
                let toplevel = window.toplevel.clone();
                let size = window.size;
                toplevel.with_pending_state(|state| {
                    state.states.unset(xdg_toplevel::State::Resizing);
                    state.size = Some(size);
                });
                toplevel.send_configure();
            }
        }
        self.request_full_redraw();
    }

    fn output_area(&self) -> WindowGeometry {
        WindowGeometry::new(0, 0, self.output_size.0, self.output_size.1)
    }

    fn work_area(&self) -> WindowGeometry {
        let output = self.output_area();
        let top = self
            .layer_surfaces
            .iter()
            .filter(|layer| layer.layer == Layer::Top)
            .map(|layer| layer.exclusive_zone.max(0))
            .max()
            .unwrap_or(0)
            .min(output.height);
        let bottom = self
            .layer_surfaces
            .iter()
            .filter(|layer| layer.layer == Layer::Bottom)
            .map(|layer| layer.exclusive_zone.max(0))
            .max()
            .unwrap_or(0)
            .min(output.height.saturating_sub(top));
        WindowGeometry::new(
            0,
            top,
            output.width,
            output.height.saturating_sub(top + bottom).max(1),
        )
    }

    /// Keep normal windows inside the current compositor-owned work area after
    /// a layer-shell surface changes its exclusive reservation.
    fn clamp_normal_windows_to_work_area(&mut self) {
        let work_area = self.work_area();
        let mut changed = false;
        for window in &mut self.windows {
            if window.minimized
                || window.presentation_state != WindowPresentationState::Normal
                || window.app_id.starts_with("com.slopos.shell")
            {
                continue;
            }
            let current = window.geometry();
            let next = clamp_window_to_work_area(current, work_area);
            if current == next {
                continue;
            }
            window.position = Point::from((next.x, next.y));
            window.size = Size::from((next.width, next.height));
            let toplevel = window.toplevel.clone();
            toplevel.with_pending_state(|state| {
                state.size = Some(Size::from((next.width, next.height)));
            });
            toplevel.send_configure();
            changed = true;
        }
        if changed {
            self.request_full_redraw();
        }
    }

    fn set_window_presentation_state(
        &mut self,
        surface: &ToplevelSurface,
        target_state: WindowPresentationState,
    ) {
        let Some(idx) = self
            .windows
            .iter()
            .position(|w| w.toplevel.wl_surface() == surface.wl_surface())
        else {
            return;
        };
        let old = self.windows[idx].geometry();
        let current_state = self.windows[idx].presentation_state;
        let current_restore_state = self.windows[idx].restore_state.clone();
        let transition = transition_presentation_state(
            current_state,
            old,
            current_restore_state.as_ref(),
            target_state,
            self.work_area(),
            self.output_area(),
            None,
            "drm-0",
            self.workspace_state.active.as_usize(),
        );
        self.windows[idx].presentation_state = transition.state;
        self.windows[idx].restore_state = transition.restore_state;
        self.windows[idx].position = Point::from((transition.geometry.x, transition.geometry.y));
        self.windows[idx].size =
            Size::from((transition.geometry.width, transition.geometry.height));
        let toplevel = self.windows[idx].toplevel.clone();
        let size = self.windows[idx].size;
        toplevel.with_pending_state(|state| {
            state.states.unset(xdg_toplevel::State::Maximized);
            state.states.unset(xdg_toplevel::State::Fullscreen);
            match target_state {
                WindowPresentationState::Filled => {
                    state.states.set(xdg_toplevel::State::Maximized);
                }
                WindowPresentationState::Fullscreen => {
                    state.states.set(xdg_toplevel::State::Fullscreen);
                }
                _ => {}
            }
            state.size = Some(size);
        });
        toplevel.send_configure();
        self.request_full_redraw();
    }

    fn handle_libinput(
        &mut self,
        event: smithay::backend::input::InputEvent<LibinputInputBackend>,
    ) {
        use smithay::backend::input::{
            AbsolutePositionEvent, ButtonState, Event as _, InputEvent, KeyState, KeyboardKeyEvent,
            PointerButtonEvent, PointerMotionEvent,
        };
        use smithay::input::keyboard::{FilterResult, Keysym};
        use smithay::input::pointer::ButtonEvent;

        match event {
            InputEvent::Keyboard { event } => {
                let serial = self.next_serial();
                let time = event.time_msec();
                let keycode = event.key_code();
                let key_state = event.state();
                let Some(kb) = self.seat.get_keyboard() else {
                    return;
                };
                if self.locked {
                    if let Some(surf) = self.active_lock_surface() {
                        self.focus_surface(Some(surf));
                    }
                }
                kb.input::<(), _>(
                    self,
                    keycode,
                    key_state,
                    serial,
                    time,
                    |data, mods, keysym| {
                        if data.locked {
                            if key_state == KeyState::Pressed && mods.logo {
                                return FilterResult::Intercept(());
                            }
                            return FilterResult::Forward;
                        }
                        // Super+Left/Right/PageUp/PageDown and Super+1..8 switch
                        // workspaces, matching the nested X11 bindings.
                        if key_state == KeyState::Pressed && mods.logo {
                            let sym = keysym.modified_sym();
                            if sym == Keysym::o || sym == Keysym::O {
                                data.spawn_client("finder");
                                return FilterResult::Intercept(());
                            }
                            if sym == Keysym::l || sym == Keysym::L {
                                data.spawn_client("slopos-lock");
                                return FilterResult::Intercept(());
                            }
                            if sym == Keysym::Right || sym == Keysym::Page_Down {
                                data.cycle_workspace_next();
                                return FilterResult::Intercept(());
                            }
                            if sym == Keysym::Left || sym == Keysym::Page_Up {
                                data.cycle_workspace_prev();
                                return FilterResult::Intercept(());
                            }
                            let digit = match sym {
                                Keysym::_1 => Some(0u8),
                                Keysym::_2 => Some(1),
                                Keysym::_3 => Some(2),
                                Keysym::_4 => Some(3),
                                Keysym::_5 => Some(4),
                                Keysym::_6 => Some(5),
                                Keysym::_7 => Some(6),
                                Keysym::_8 => Some(7),
                                _ => None,
                            };
                            if let Some(i) = digit {
                                data.activate_workspace_index(i);
                                return FilterResult::Intercept(());
                            }
                        }
                        FilterResult::Forward
                    },
                );
            }
            InputEvent::PointerMotionAbsolute { event } => {
                let x = event.x_transformed(self.output_size.0);
                let y = event.y_transformed(self.output_size.1);
                self.pointer_location = Point::from((x, y));
                self.update_interactive_grab();
                self.forward_pointer_motion(event.time_msec());
                // The DRM cursor is compositor-rendered, so pointer motion is
                // damage even when no client surface changed.
                self.request_redraw();
            }
            InputEvent::PointerMotion { event } => {
                // Relative motion (real mice): accumulate and clamp to output.
                let (dx, dy) = (event.delta_x(), event.delta_y());
                let x = (self.pointer_location.x + dx).clamp(0.0, self.output_size.0 as f64 - 1.0);
                let y = (self.pointer_location.y + dy).clamp(0.0, self.output_size.1 as f64 - 1.0);
                self.pointer_location = Point::from((x, y));
                self.update_interactive_grab();
                self.forward_pointer_motion(event.time_msec());
                self.request_redraw();
            }
            InputEvent::PointerButton { event } => {
                let serial = self.next_serial();
                let time = event.time_msec();
                let button = event.button_code();
                let btn_state = event.state();
                if button == 0x110 || button == 1 {
                    self.left_button_down = btn_state == ButtonState::Pressed;
                    if btn_state == ButtonState::Released {
                        self.finish_interactive_grab();
                        self.last_pointer_press = None;
                    }
                }

                if btn_state == ButtonState::Pressed && !self.locked {
                    let pos = self.pointer_location;
                    let hit = self.surface_under(pos);
                    let target_surface = self.surface_under(pos).map(|(surface, _)| surface);
                    if button == 0x110 || button == 1 {
                        self.last_pointer_press = target_surface
                            .clone()
                            .map(|surface| PointerPress { serial, surface });
                    }
                    match hit {
                        Some((surface, _)) => {
                            if let Some(idx) = self
                                .windows
                                .iter()
                                .position(|window| window.toplevel.wl_surface() == &surface)
                            {
                                self.focus_window_at_index(idx);
                            } else {
                                self.focus_surface(Some(surface));
                            }
                        }
                        None => self.focus_surface(None),
                    }
                    // Retarget pointer so the focused surface gets Enter/Motion
                    // at the true click coordinates before the button event.
                    self.forward_pointer_motion(time);
                } else if btn_state == ButtonState::Pressed && (button == 0x110 || button == 1) {
                    self.last_pointer_press = None;
                }

                if let Some(ptr) = self.seat.get_pointer() {
                    ptr.button(
                        self,
                        &ButtonEvent {
                            serial,
                            time,
                            button,
                            state: btn_state,
                        },
                    );
                    ptr.frame(self);
                }
            }
            _ => {}
        }
    }

    /// Hit-test: Overlay/Top layers → windows → Bottom/Background layers.
    ///
    /// The Point in the returned pair is the **surface origin in the same
    /// coordinate space as `MotionEvent.location`** (smithay sends
    /// `location - origin` to the client as surface-local coords).
    /// Layer strips only hit when the pointer is inside their geo.
    fn surface_under(&self, pos: Point<f64, Logical>) -> Option<(WlSurface, Point<f64, Logical>)> {
        for layer in self.layer_surfaces.iter().rev() {
            if matches!(layer.layer, Layer::Overlay | Layer::Top)
                && layer_geo_contains(&layer.geo, pos)
            {
                return Some((
                    layer.surface.wl_surface().clone(),
                    Point::from((layer.geo.loc.x as f64, layer.geo.loc.y as f64)),
                ));
            }
        }
        for w in self
            .windows
            .iter()
            .rev()
            .filter(|w| !w.minimized && self.workspace_state.is_visible(&w.window_id))
        {
            let popups: Vec<_> =
                PopupManager::popups_for_surface(w.toplevel.wl_surface()).collect();
            for (popup, popup_offset) in popups.into_iter().rev() {
                let geometry = popup.geometry();
                let origin: Point<i32, Logical> = Point::from((
                    w.position.x + popup_offset.x - geometry.loc.x,
                    w.position.y + popup_offset.y - geometry.loc.y,
                ));
                if geometry.size.w > 0
                    && geometry.size.h > 0
                    && (pos.x as i32) >= origin.x
                    && (pos.y as i32) >= origin.y
                    && (pos.x as i32) < origin.x + geometry.size.w
                    && (pos.y as i32) < origin.y + geometry.size.h
                {
                    return Some((
                        popup.wl_surface().clone(),
                        Point::from((origin.x as f64, origin.y as f64)),
                    ));
                }
            }
            if w.geometry().contains_f64(pos.x, pos.y) {
                return Some((
                    w.toplevel.wl_surface().clone(),
                    Point::from((w.position.x as f64, w.position.y as f64)),
                ));
            }
        }
        for layer in self.layer_surfaces.iter().rev() {
            if matches!(layer.layer, Layer::Bottom | Layer::Background)
                && layer_geo_contains(&layer.geo, pos)
            {
                return Some((
                    layer.surface.wl_surface().clone(),
                    Point::from((layer.geo.loc.x as f64, layer.geo.loc.y as f64)),
                ));
            }
        }
        None
    }

    fn activated_window_for_surface(&self, surface: &WlSurface) -> Option<String> {
        if let Some(window) = self
            .windows
            .iter()
            .find(|w| w.toplevel.wl_surface() == surface)
        {
            return Some(window.window_id.clone());
        }
        let popup = self.popup_manager.find_popup(surface)?;
        let root = find_popup_root_surface(&popup).ok()?;
        self.windows
            .iter()
            .find(|w| w.toplevel.wl_surface() == &root)
            .map(|w| w.window_id.clone())
    }

    fn sync_activated_for_surface(&mut self, surface: Option<&WlSurface>) {
        let next = surface.and_then(|surface| self.activated_window_for_surface(surface));
        if self.activated_window_id == next {
            return;
        }
        let previous = self.activated_window_id.take();
        self.activated_window_id = next.clone();
        for window in &self.windows {
            let was_active = previous.as_ref() == Some(&window.window_id);
            let is_active = next.as_ref() == Some(&window.window_id);
            if !was_active && !is_active {
                continue;
            }
            window.toplevel.with_pending_state(|state| {
                if is_active {
                    state.states.set(xdg_toplevel::State::Activated);
                } else {
                    state.states.unset(xdg_toplevel::State::Activated);
                }
            });
            window.toplevel.send_configure();
        }
    }

    fn cleanup_popup_state(&mut self) {
        self.popup_manager.cleanup();
        if self.popup_grab.as_ref().is_some_and(PopupGrab::has_ended) {
            self.popup_grab = None;
            self.request_full_redraw();
        }
    }

    fn begin_popup_grab(&mut self, surface: PopupSurface, seat: wl_seat::WlSeat, serial: Serial) {
        let popup = PopupKind::from(surface);
        if !self.seat.owns(&seat) {
            tracing::debug!("rejecting xdg popup grab from an unknown seat");
            return;
        }
        let Some(root) = find_popup_root_surface(&popup).ok() else {
            tracing::debug!("rejecting xdg popup grab without a live root surface");
            return;
        };
        let popup_surface = popup.wl_surface().clone();
        let Ok(grab) = self
            .popup_manager
            .grab_popup(root, popup, &self.seat, serial)
        else {
            tracing::debug!("rejecting invalid xdg popup grab");
            return;
        };
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_grab(self, PopupKeyboardGrab::new(&grab), serial);
        }
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.set_grab(self, PopupPointerGrab::new(&grab), serial, Focus::Keep);
        }
        self.popup_grab = Some(grab);
        self.focus_surface(Some(popup_surface));
        self.request_full_redraw();
    }

    /// Send the current pointer location to the seat, retargeting focus to
    /// whatever surface is under it.
    fn forward_pointer_motion(&mut self, time: u32) {
        use smithay::input::pointer::MotionEvent;

        let pos = self.pointer_location;
        let focus = if self.locked {
            self.active_lock_surface()
                .map(|surf| (surf, Point::from((0.0, 0.0))))
        } else {
            self.surface_under(pos)
        };
        let serial = self.next_serial();
        if let Some(ptr) = self.seat.get_pointer() {
            ptr.motion(
                self,
                focus,
                &MotionEvent {
                    location: pos,
                    serial,
                    time,
                },
            );
            ptr.frame(self);
        }
    }

    /// Raise the window at `idx` to the top of the stack and give it focus.
    fn focus_window_at_index(&mut self, idx: usize) {
        if idx >= self.windows.len() {
            return;
        }
        let app_id = self.windows[idx].app_id.clone();
        let w = self.windows.remove(idx);
        let surface = w.toplevel.wl_surface().clone();
        self.windows.push(w);
        self.focus_surface(Some(surface));
        if let Err(err) = crate::publish_active_toplevel(Some(&app_id)) {
            tracing::debug!(error = %err, app_id = %app_id, "could not publish active application");
        }
        self.request_full_redraw();
    }

    fn focus_surface(&mut self, surface: Option<WlSurface>) {
        self.sync_activated_for_surface(surface.as_ref());
        let active_app_id = surface.as_ref().and_then(|surface| {
            self.activated_window_for_surface(surface)
                .and_then(|window_id| {
                    self.windows
                        .iter()
                        .find(|window| window.window_id == window_id)
                        .map(|window| window.app_id.clone())
                })
        });
        if let Err(err) = crate::publish_active_toplevel(active_app_id.as_deref()) {
            tracing::debug!(error = %err, app_id = ?active_app_id, "could not publish active application");
        }
        let serial = self.next_serial();
        if let Some(kb) = self.seat.get_keyboard() {
            kb.set_focus(self, surface.clone(), serial);
        }
        let client = surface.and_then(|s| s.client());
        set_data_device_focus(&self.display_handle, &self.seat, client.clone());
        set_primary_focus(&self.display_handle, &self.seat, client);
        self.request_redraw();
    }
}

// ---------------------------------------------------------------------------
// Protocol handlers
// ---------------------------------------------------------------------------

impl BufferHandler for DrmSessionState {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl CompositorHandler for DrmSessionState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("client must carry ClientState")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // Promote the newly attached buffer into renderable state. Without
        // this, render_elements_from_surface_tree finds no texture and the
        // compositor paints only its clear colour — clients map and render but
        // never appear on screen.
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);
        self.popup_manager.commit(surface);
        for w in self.windows.iter_mut() {
            if w.toplevel.wl_surface() == surface {
                let st = w.toplevel.current_state();
                let sw = st
                    .size
                    .map(|s| s.w)
                    .filter(|v| *v > 0)
                    .unwrap_or(DEFAULT_WINDOW_W);
                let sh = st
                    .size
                    .map(|s| s.h)
                    .filter(|v| *v > 0)
                    .unwrap_or(DEFAULT_WINDOW_H);
                w.size = Size::from((sw, sh));
                break;
            }
        }
        // Refresh layer geometry from client-requested size/margins after commit.
        for layer in self.layer_surfaces.iter_mut() {
            if layer.surface.wl_surface() == surface {
                let (ow, oh) = self.output_size;
                let (requested, anchor, margins, exclusive_zone) =
                    layer_surface_request(&layer.surface);
                let geo = layer_geometry_for(
                    &layer.namespace,
                    layer.layer,
                    (ow, oh),
                    requested,
                    anchor,
                    margins,
                );
                let cur = layer.surface.current_state();
                let needs_configure = cur.size != Some(geo.size);
                if needs_configure {
                    layer.surface.with_pending_state(|state| {
                        state.size = Some(geo.size);
                    });
                    layer.surface.send_configure();
                }
                layer.geo = geo;
                layer.exclusive_zone = exclusive_zone;
                break;
            }
        }
        self.clamp_normal_windows_to_work_area();
        self.request_redraw();
    }
}
delegate_compositor!(DrmSessionState);

impl ShmHandler for DrmSessionState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
delegate_shm!(DrmSessionState);

impl SeatHandler for DrmSessionState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    /// Remember the client-set cursor so the compositor can draw it. Ignoring
    /// this is why the DRM session had no visible pointer at all.
    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_status = image;
        self.request_full_redraw();
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let client = focused.and_then(|s| s.client());
        set_data_device_focus(&self.display_handle, seat, client.clone());
        set_primary_focus(&self.display_handle, seat, client);
    }
}
delegate_seat!(DrmSessionState);

fn write_selection_fd(_mime_type: String, fd: OwnedFd, data: Option<Vec<u8>>) {
    use std::io::Write;
    if let Err(err) = std::thread::Builder::new()
        .name("drm-selection-send".into())
        .spawn(move || {
            let mut file = std::fs::File::from(fd);
            if let Some(bytes) = data {
                let _ = file.write_all(&bytes);
            }
            let _ = file.flush();
        })
    {
        tracing::warn!(error = %err, "failed to spawn selection-send thread");
    }
}

impl SelectionHandler for DrmSessionState {
    type SelectionUserData = MimePayload;

    fn new_selection(
        &mut self,
        ty: SelectionTarget,
        source: Option<SelectionSource>,
        _seat: Seat<Self>,
    ) {
        match ty {
            SelectionTarget::Clipboard => {
                self.clipboard_source = source;
                if self.clipboard_source.is_none() {
                    self.clipboard_data.clear();
                }
            }
            SelectionTarget::Primary => {
                self.primary_source = source;
                if self.primary_source.is_none() {
                    self.primary_data.clear();
                }
            }
        }
    }

    fn send_selection(
        &mut self,
        ty: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        _seat: Seat<Self>,
        user_data: &Self::SelectionUserData,
    ) {
        let from_user = user_data.get(&mime_type).cloned();
        let from_store = match ty {
            SelectionTarget::Clipboard => self.clipboard_data.get(&mime_type).cloned(),
            SelectionTarget::Primary => self.primary_data.get(&mime_type).cloned(),
        };
        write_selection_fd(mime_type, fd, from_user.or(from_store));
    }
}

impl DataDeviceHandler for DrmSessionState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for DrmSessionState {
    fn started(
        &mut self,
        _source: Option<WlDataSource>,
        icon: Option<WlSurface>,
        _seat: Seat<Self>,
    ) {
        self.dnd_icon = icon;
    }

    fn dropped(&mut self, _target: Option<WlSurface>, _validated: bool, _seat: Seat<Self>) {
        self.dnd_icon = None;
    }
}

impl ServerDndGrabHandler for DrmSessionState {
    fn send(&mut self, mime_type: String, fd: OwnedFd, _seat: Seat<Self>) {
        let data = self.server_dnd_data.get(&mime_type).cloned();
        write_selection_fd(mime_type, fd, data);
    }

    fn cancelled(&mut self, _seat: Seat<Self>) {
        self.server_dnd_data.clear();
    }

    fn finished(&mut self, _seat: Seat<Self>) {
        self.server_dnd_data.clear();
    }
}
delegate_data_device!(DrmSessionState);

impl PrimarySelectionHandler for DrmSessionState {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection_state
    }
}
delegate_primary_selection!(DrmSessionState);

impl XdgShellHandler for DrmSessionState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // Read app_id BEFORE the first configure so we can size the shell to fill
        // the output. The SLOPOS-I desktop (app_id "com.slopos.shell") is the
        // root session surface: it must span the whole output, anchored at (0,0),
        // not the cascaded 640×480 default used for ordinary app windows.
        let (title, app_id) = with_states(surface.wl_surface(), |states| {
            let data = states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .map(|d| d.lock().unwrap());
            let title = data
                .as_ref()
                .and_then(|d| d.title.clone())
                .unwrap_or_else(|| "Untitled".into());
            let app_id = data
                .as_ref()
                .and_then(|d| d.app_id.clone())
                .unwrap_or_else(|| "slopos-i.app".into());
            (title, app_id)
        });
        let is_shell = app_id == "com.slopos.shell" || app_id.starts_with("com.slopos.shell");

        let desired = if is_shell {
            self.output_area()
        } else {
            let offset = (self.windows.len() as i32) * 32;
            let (x, y) = (64 + offset, 64 + offset);
            WindowGeometry::new(x, y, DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)
        };
        let geometry = if is_shell {
            desired
        } else {
            clamp_window_to_work_area(desired, self.work_area())
        };
        let (win_w, win_h) = (geometry.width, geometry.height);
        surface.with_pending_state(|state| {
            state.size = Some(Size::from((win_w, win_h)));
            state.states.set(xdg_toplevel::State::Activated);
            if is_shell {
                // Fill the output like a maximized/fullscreen surface.
                state.states.set(xdg_toplevel::State::Maximized);
                state.states.set(xdg_toplevel::State::Fullscreen);
            }
        });
        surface.send_configure();

        let foreign = self
            .foreign_toplevel_list
            .new_toplevel::<DrmSessionState>(&title, &app_id);

        let position = Point::from((geometry.x, geometry.y));
        eprintln!(
            "[slopos-compositor/drm] toplevel mapped at ({},{}) size={win_w}x{win_h} title={title} app_id={app_id} shell={is_shell}",
            position.x, position.y
        );

        let window_id = foreign.identifier();
        // Map → active workspace; remove is paired in destroy/prune.
        if !assign_new_window_to_active(&mut self.workspace_state, window_id.clone()) {
            let _ = self
                .workspace_state
                .assign_window(window_id.clone(), WorkspaceId::FIRST);
        }
        self.windows.push(MappedWindow {
            toplevel: surface.clone(),
            foreign,
            window_id: window_id.clone(),
            app_id: app_id.clone(),
            position,
            size: Size::from((win_w, win_h)),
            presentation_state: WindowPresentationState::Normal,
            restore_state: None,
            minimized: false,
        });
        // Listing/present filter: only active-workspace ids (client SHM composite TBD).
        eprintln!(
            "[slopos-compositor/drm] {} window_id={window_id} present={:?}",
            self.workspace_state.summary_line(),
            self.window_ids_for_present()
        );
        self.focus_surface(Some(surface.wl_surface().clone()));
        if let Err(err) = crate::publish_active_toplevel(Some(&app_id)) {
            tracing::debug!(error = %err, app_id = %app_id, "could not publish active application");
        }
        self.request_full_redraw();
    }

    fn move_request(&mut self, surface: ToplevelSurface, seat: wl_seat::WlSeat, serial: Serial) {
        self.begin_interactive_grab(&surface, InteractiveGrabKind::Move, &seat, serial);
    }

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        seat: wl_seat::WlSeat,
        serial: Serial,
        edges: xdg_toplevel::ResizeEdge,
    ) {
        let edges = match edges {
            xdg_toplevel::ResizeEdge::Top => ResizeEdges::TOP,
            xdg_toplevel::ResizeEdge::Bottom => ResizeEdges::BOTTOM,
            xdg_toplevel::ResizeEdge::Left => ResizeEdges::LEFT,
            xdg_toplevel::ResizeEdge::Right => ResizeEdges::RIGHT,
            xdg_toplevel::ResizeEdge::TopLeft => ResizeEdges::TOP_LEFT,
            xdg_toplevel::ResizeEdge::TopRight => ResizeEdges::TOP_RIGHT,
            xdg_toplevel::ResizeEdge::BottomLeft => ResizeEdges::BOTTOM_LEFT,
            xdg_toplevel::ResizeEdge::BottomRight => ResizeEdges::BOTTOM_RIGHT,
            _ => return,
        };
        self.begin_interactive_grab(&surface, InteractiveGrabKind::Resize(edges), &seat, serial);
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        self.set_window_presentation_state(&surface, WindowPresentationState::Filled);
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        self.set_window_presentation_state(&surface, WindowPresentationState::Normal);
    }

    fn fullscreen_request(
        &mut self,
        surface: ToplevelSurface,
        _output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
    ) {
        self.set_window_presentation_state(&surface, WindowPresentationState::Fullscreen);
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        self.set_window_presentation_state(&surface, WindowPresentationState::Normal);
    }

    fn minimize_request(&mut self, surface: ToplevelSurface) {
        if let Some(idx) = self
            .windows
            .iter()
            .position(|w| w.toplevel.wl_surface() == surface.wl_surface())
        {
            self.last_minimized_window_id = Some(self.windows[idx].window_id.clone());
            self.windows[idx].minimized = true;
            self.request_full_redraw();
            self.apply_focus_after_workspace_switch();
        }
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        if let Some(idx) = self
            .windows
            .iter()
            .position(|w| w.toplevel.wl_surface() == surface.wl_surface())
        {
            let win = self.windows.remove(idx);
            self.workspace_state.remove_window(&win.window_id);
            if self.last_minimized_window_id.as_deref() == Some(win.window_id.as_str()) {
                self.last_minimized_window_id = None;
            }
            win.foreign.send_closed();
        }
        // Prefer topmost **visible** window; clear focus if none on active workspace.
        self.apply_focus_after_workspace_switch();
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        let popup = PopupKind::from(surface.clone());
        if let Err(err) = self.popup_manager.track_popup(popup) {
            tracing::debug!(?err, "failed to track xdg popup");
            return;
        }
        surface.with_pending_state(|state| {
            state.positioner = positioner;
        });
        if let Err(err) = surface.send_configure() {
            tracing::debug!(?err, "failed to configure xdg popup");
        }
        self.request_full_redraw();
    }

    fn grab(&mut self, surface: PopupSurface, seat: wl_seat::WlSeat, serial: Serial) {
        self.begin_popup_grab(surface, seat, serial);
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            state.positioner = positioner;
        });
        let _serial = surface.send_repositioned(token);
        self.request_full_redraw();
    }

    fn title_changed(&mut self, surface: ToplevelSurface) {
        let title = with_states(surface.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|d| d.lock().unwrap().title.clone())
                .unwrap_or_default()
        });
        if let Some(w) = self
            .windows
            .iter()
            .find(|w| w.toplevel.wl_surface() == surface.wl_surface())
        {
            w.foreign.send_title(&title);
            w.foreign.send_done();
        }
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        let app_id = with_states(surface.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|d| d.lock().unwrap().app_id.clone())
                .unwrap_or_default()
        });
        let active_window_id = self.activated_window_id.clone();
        if let Some(w) = self
            .windows
            .iter_mut()
            .find(|w| w.toplevel.wl_surface() == surface.wl_surface())
        {
            let is_active = active_window_id.as_ref() == Some(&w.window_id);
            w.app_id = app_id.clone();
            w.foreign.send_app_id(&app_id);
            w.foreign.send_done();
            if is_active {
                if let Err(err) = crate::publish_active_toplevel(Some(&app_id)) {
                    tracing::debug!(
                        error = %err,
                        app_id = %app_id,
                        "could not refresh active application"
                    );
                }
            }
        }
    }
}
delegate_xdg_shell!(DrmSessionState);

impl OutputHandler for DrmSessionState {}
delegate_output!(DrmSessionState);

impl WlrLayerShellHandler for DrmSessionState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        _output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
        layer: Layer,
        namespace: String,
    ) {
        eprintln!(
            "[slopos-compositor/drm] layer-shell surface namespace={namespace} layer={layer:?}"
        );
        let (ow, oh) = self.output_size;
        let (requested, anchor, margins, exclusive_zone) = layer_surface_request(&surface);
        let geo = layer_geometry_for(&namespace, layer, (ow, oh), requested, anchor, margins);
        surface.with_pending_state(|state| {
            state.size = Some(geo.size);
        });
        surface.send_configure();
        self.layer_surfaces.push(MappedLayer {
            surface,
            layer,
            namespace,
            geo,
            exclusive_zone,
        });
        self.request_full_redraw();
    }

    fn layer_destroyed(&mut self, surface: LayerSurface) {
        let before = self.layer_surfaces.len();
        self.layer_surfaces
            .retain(|l| l.surface.wl_surface() != surface.wl_surface());
        if self.layer_surfaces.len() != before {
            self.request_full_redraw();
        }
    }
}
delegate_layer_shell!(DrmSessionState);

impl SessionLockHandler for DrmSessionState {
    fn lock_state(&mut self) -> &mut SessionLockManagerState {
        &mut self.session_lock_state
    }

    fn lock(&mut self, confirmation: SessionLocker) {
        self.locked = true;
        confirmation.lock();
        if let Some(surf) = self.active_lock_surface() {
            self.focus_surface(Some(surf));
        }
        self.request_full_redraw();
        tracing::info!("session locked");
        eprintln!("[slopos-compositor] session locked");
    }

    fn unlock(&mut self) {
        self.locked = false;
        self.lock_surfaces.clear();
        self.apply_focus_after_workspace_switch();
        self.request_full_redraw();
        tracing::info!("session unlocked");
        eprintln!("[slopos-compositor] session unlocked");
    }

    fn new_surface(
        &mut self,
        surface: LockSurface,
        output: smithay::reexports::wayland_server::protocol::wl_output::WlOutput,
    ) {
        if let Some(out) = Output::from_resource(&output) {
            let out = out.clone();
            let size = out
                .current_mode()
                .map(|m| m.size)
                .unwrap_or_else(|| Size::from(self.output_size));
            surface.with_pending_state(|states| {
                states.size = Some(Size::from((size.w as u32, size.h as u32)));
            });
            surface.send_configure();
            self.lock_surfaces.push((out, surface));
            if let Some(surf) = self.active_lock_surface() {
                self.focus_surface(Some(surf));
            }
        }
        self.request_full_redraw();
    }
}
delegate_session_lock!(DrmSessionState);

impl ForeignToplevelListHandler for DrmSessionState {
    fn foreign_toplevel_list_state(&mut self) -> &mut ForeignToplevelListState {
        &mut self.foreign_toplevel_list
    }
}
delegate_foreign_toplevel_list!(DrmSessionState);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drm_session_available_is_bool() {
        // Pure: just ensure the probe does not panic on this host.
        let _ = drm_session_available();
    }

    #[test]
    fn resolve_primary_prefers_discover_or_default() {
        // Without a real seat, path is either discovered or /dev/dri/card0.
        let p = resolve_primary_drm_path("seat0");
        assert!(
            p.to_string_lossy().contains("dri") || p.ends_with("card0"),
            "unexpected path {p:?}"
        );
    }
}
