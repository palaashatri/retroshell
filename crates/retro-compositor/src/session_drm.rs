//! DRM/KMS + libseat session bootstrap for bare-metal / VT sessions.
//!
//! Selected when policy says [`CompositorBackendKind::SessionDrm`]. Docker-on-mac
//! will not exercise seat/DRM privileges; the code still ships, compiles into
//! `retro-compositor`, and runs when `/dev/dri` + seatd/logind are available.
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
//! without modes fall back to env sizing (`RETROSHELL_COMPOSITOR_WIDTH/HEIGHT`).

#![cfg(target_os = "linux")]

use std::collections::HashMap;
use std::os::unix::io::OwnedFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use smithay::backend::allocator::gbm::GbmDevice;
use smithay::backend::drm::DrmDeviceFd;
use smithay::backend::egl::{EGLContext, EGLDisplay};
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::session::{Event as SessionEvent, Session};
use smithay::backend::udev::{all_gpus, primary_gpu, UdevBackend, UdevEvent};
use smithay::input::keyboard::XkbConfig;
use smithay::input::pointer::CursorImageStatus;
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::calloop::{EventLoop, LoopSignal};
// Use smithay's rustix reexport so OFlags matches Session::open.
use smithay::reexports::rustix::fs::OFlags;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::backend::{
    ClientData, ClientId, DisconnectReason,
};
use smithay::reexports::wayland_server::protocol::{
    wl_buffer, wl_data_source::WlDataSource, wl_seat, wl_surface::WlSurface,
};
use smithay::reexports::wayland_server::{Client, Display, DisplayHandle, Resource};
use smithay::utils::{DeviceFd, Logical, Point, Serial, Size, Transform};
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
use smithay::wayland::shell::wlr_layer::{
    Layer, LayerSurface, WlrLayerShellHandler, WlrLayerShellState,
};
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    XdgToplevelSurfaceData,
};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::socket::ListeningSocketSource;
use smithay::{
    delegate_compositor, delegate_data_device, delegate_foreign_toplevel_list, delegate_layer_shell,
    delegate_output, delegate_primary_selection, delegate_seat, delegate_shm, delegate_xdg_shell,
};

use crate::frame_timing::{FrameScheduler, RefreshRate};
use crate::hdr::HdrCapabilities;
use crate::{
    assign_new_window_to_active, discover_drm_nodes, drm_presentation_pipeline,
    focus_window_after_workspace_switch, plan_drm_modeset, preferred_primary_drm_node,
    session_mode_summary, visible_paint_order, CompositorBackendKind, DisplayPolicy,
    DrmPresentationStage, WorkspaceId, WorkspaceState, DEFAULT_OUTPUT_H, DEFAULT_OUTPUT_W,
    DEFAULT_WINDOW_H, DEFAULT_WINDOW_W,
};

/// Compositor-owned selection payload keyed by mime type.
type MimePayload = Arc<HashMap<String, Vec<u8>>>;

/// Probe whether a DRM session looks bootable (nodes exist under /dev/dri).
pub fn drm_session_available() -> bool {
    !discover_drm_nodes().is_empty() || Path::new("/dev/dri").exists()
}

/// Present one solid dumb-buffer frame via `DrmSurface::commit` then `page_flip`.
///
/// This is the real scanout path (framebuffer + flip), not open-device only.
fn try_present_dumb_frame(
    surface: &smithay::backend::drm::DrmSurface,
    width: i32,
    height: i32,
) -> Result<()> {
    use smithay::backend::allocator::dumb::DumbAllocator;
    use smithay::backend::allocator::{Allocator, Fourcc, Modifier};
    use smithay::backend::drm::dumb::framebuffer_from_dumb_buffer;
    use smithay::backend::drm::{DrmDeviceFd, PlaneConfig, PlaneState};
    use smithay::utils::{Buffer as BufferCoords, Physical, Rectangle, Transform};

    let w = width.max(1) as u32;
    let h = height.max(1) as u32;
    let fd: DrmDeviceFd = surface.device_fd().clone();
    let mut dumb = DumbAllocator::new(fd.clone());
    let buffer = dumb
        .create_buffer(w, h, Fourcc::Xrgb8888, &[Modifier::Linear])
        .context("DumbAllocator::create_buffer for scanout")?;
    let fb = framebuffer_from_dumb_buffer(&fd, &buffer, true)
        .context("framebuffer_from_dumb_buffer")?;
    let fb_handle = *fb.as_ref();

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

    // Keep fb/buffer alive for the queued flip (process-lifetime leak is acceptable
    // for the single startup present; surface is retained by caller).
    std::mem::forget(fb);
    std::mem::forget(buffer);
    Ok(())
}

fn w_from_env_or_default() -> i32 {
    std::env::var("RETROSHELL_COMPOSITOR_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_OUTPUT_W)
}

fn h_from_env_or_default() -> i32 {
    std::env::var("RETROSHELL_COMPOSITOR_HEIGHT")
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

/// Run the DRM/KMS session compositor path.
///
/// Returns `Err` with context if seat/DRM cannot be opened (no privileges,
/// nested container without `/dev/dri`). Callers may fall back to nested X11.
pub fn run_drm_session() -> Result<()> {
    tracing::info!("{}", session_mode_summary(CompositorBackendKind::SessionDrm));
    eprintln!(
        "[retro-compositor] starting DRM/KMS session path ({})",
        session_mode_summary(CompositorBackendKind::SessionDrm)
    );

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
        "[retro-compositor] display policy: {}",
        display_policy.summary_line(hdr_caps.hdr_supported)
    );

    // ---- Seat (VT / device ACLs) ----
    let (mut session, session_notifier) =
        LibSeatSession::new().context("LibSeatSession::new (need seatd/logind + privileges)")?;
    let seat_name = session.seat();
    eprintln!("[retro-compositor] libseat seat={seat_name}");

    // ---- Event loop + Wayland display ----
    let mut event_loop: EventLoop<'static, DrmSessionState> =
        EventLoop::try_new().context("EventLoop::try_new")?;
    let mut display: Display<DrmSessionState> = Display::new().context("Display::new")?;
    let dh = display.handle();
    let loop_handle = event_loop.handle();
    let loop_signal = event_loop.get_signal();

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

    let mut seat: Seat<DrmSessionState> = seat_state.new_wl_seat(&dh, "seat0");
    seat.add_keyboard(XkbConfig::default(), 200, 25)
        .context("add_keyboard")?;
    seat.add_pointer();

    // ---- Open primary GPU via seat ----
    let primary = resolve_primary_drm_path(&seat_name);
    eprintln!(
        "[retro-compositor] opening DRM node {}",
        primary.display()
    );

    let owned: OwnedFd = session
        .open(
            &primary,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
        )
        .with_context(|| format!("session.open({})", primary.display()))?;
    let device_fd = DrmDeviceFd::new(DeviceFd::from(owned));
    let (mut drm, _drm_notifier) =
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
    let mut connector_summaries: Vec<(String, bool, Option<(i32, i32, i32)>)> = Vec::new();
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
        "[retro-compositor] DRM modeset plan: connector={} {}x{}@{}mhz crtcs={} connectors={}",
        modeset_plan.connector_name,
        modeset_plan.mode_w,
        modeset_plan.mode_h,
        modeset_plan.refresh_mhz,
        drm.crtcs().len(),
        connector_summaries.len()
    );
    for stage in drm_presentation_pipeline() {
        tracing::debug!(stage = stage.as_str(), "drm presentation pipeline stage");
    }

    // Attempt real DrmSurface on first CRTC + connected connector (scanout path).
    let mut drm_surface: Option<DrmSurface> = None;
    if let Some((conn, mode, _idx)) = picked {
        if let Some(&crtc) = drm.crtcs().first() {
            match drm.create_surface(crtc, mode, &[conn]) {
                Ok(surface) => {
                    eprintln!(
                        "[retro-compositor] DRM scanout surface created (crtc+connector modeset)"
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
                    eprintln!("[retro-compositor] DRM create_surface failed: {err:?} (protocol-only fallback)");
                }
            }
        }
    } else {
        eprintln!(
            "[retro-compositor] no connected connector; virtual mode {}x{}",
            modeset_plan.mode_w, modeset_plan.mode_h
        );
    }
    let _renderer = renderer;
    // Keep DrmDevice alive for the session (ControlDevice for page_flip path).
    let _drm = drm;

    // ---- Pageflip / present attempt (not drop-the-surface) ----
    // Allocate a dumb XRGB8888 buffer, export as framebuffer, and issue a
    // modeset commit or page_flip so presentation is exercised when possible.
    let mut scanout_armed = false;
    if let Some(surface) = drm_surface.as_ref() {
        match try_present_dumb_frame(surface, modeset_plan.mode_w, modeset_plan.mode_h) {
            Ok(()) => {
                scanout_armed = true;
                eprintln!(
                    "[retro-compositor] DRM pageflip/commit present succeeded ({}x{})",
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
                eprintln!("[retro-compositor] DRM present failed: {err:#}");
            }
        }
    }
    // Retain surface for the process lifetime so create_surface is not a no-op.
    // Re-present periodically so scanout is continuous when armed (not one-shot).
    let mut drm_surface_keepalive = drm_surface;
    let scanout_armed = scanout_armed;
    let present_w = modeset_plan.mode_w;
    let present_h = modeset_plan.mode_h;

    // Wayland socket
    let socket = ListeningSocketSource::new_auto().context("ListeningSocketSource")?;
    let socket_name = socket.socket_name().to_string_lossy().into_owned();
    eprintln!("[retro-compositor] WAYLAND_DISPLAY={socket_name} (DRM session)");
    println!("WAYLAND_DISPLAY={socket_name}");
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let _ = std::fs::write(Path::new(&runtime).join("wayland-display"), &socket_name);
    }
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

    // Advertise connector mode when known; else env/default virtual size.
    let w = modeset_plan.mode_w;
    let h = modeset_plan.mode_h;
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
            make: "RetroShell".into(),
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
        None,
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
        loop_signal,
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
        outputs: vec![output],
        windows: Vec::new(),
        workspace_state: WorkspaceState::new(),
        layer_surfaces: Vec::new(),
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
    };

    eprintln!(
        "[retro-compositor] DRM session loop running (Wayland + seat + udev + libinput + layer-shell + foreign-toplevel; scanout_armed={scanout_armed})"
    );
    let mut frame_i: u64 = 0;
    while state.running {
        let _ = frame_scheduler.record_frame();
        // Keep workspace map honest if clients disconnect without destroy order.
        state.prune_dead_windows();
        // Continuous present: re-issue dumb pageflip ~1 Hz when scanout armed
        // so the path stays live (full damage-tracked GL scanout of client SHM is
        // follow-on; when added, only `window_ids_for_present()` should composite).
        if scanout_armed && frame_i % 60 == 0 {
            if let Some(surface) = drm_surface_keepalive.as_ref() {
                if let Err(err) = try_present_dumb_frame(surface, present_w, present_h) {
                    tracing::debug!(error = %err, "periodic DRM present failed");
                }
            }
        }
        frame_i = frame_i.wrapping_add(1);
        event_loop
            .dispatch(Some(Duration::from_millis(16)), &mut state)
            .context("event_loop.dispatch")?;
        let _ = display.dispatch_clients(&mut state);
        display.flush_clients().context("flush_clients")?;
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
        eprintln!("[retro-compositor/drm] client connected");
    }
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        eprintln!("[retro-compositor/drm] client disconnected");
    }
}

// ---------------------------------------------------------------------------
// Tracked windows / layers
// ---------------------------------------------------------------------------

struct MappedWindow {
    toplevel: ToplevelSurface,
    foreign: ForeignToplevelHandle,
    window_id: String,
    position: Point<i32, Logical>,
    size: Size<i32, Logical>,
}

struct MappedLayer {
    surface: LayerSurface,
    #[allow(dead_code)]
    layer: Layer,
    #[allow(dead_code)]
    namespace: String,
}

// ---------------------------------------------------------------------------
// Main session state
// ---------------------------------------------------------------------------

struct DrmSessionState {
    display_handle: DisplayHandle,
    loop_signal: LoopSignal,
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
    #[allow(dead_code)]
    outputs: Vec<Output>,
    windows: Vec<MappedWindow>,
    workspace_state: WorkspaceState,
    layer_surfaces: Vec<MappedLayer>,
    active: Arc<AtomicBool>,
    udev_events: Vec<String>,
    pointer_location: Point<f64, Logical>,
    output_size: (i32, i32),
    serial: u32,
    clipboard_source: Option<SelectionSource>,
    primary_source: Option<SelectionSource>,
    clipboard_data: HashMap<String, Vec<u8>>,
    primary_data: HashMap<String, Vec<u8>>,
    server_dnd_data: HashMap<String, Vec<u8>>,
    dnd_icon: Option<WlSurface>,
    running: bool,
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
        let alive: std::collections::HashSet<&str> =
            self.windows.iter().map(|w| w.window_id.as_str()).collect();
        for id in before {
            if !alive.contains(id.as_str()) {
                self.workspace_state.remove_window(&id);
            }
        }
    }

    /// Window ids that should present / list on the active workspace (bottom→top order).
    ///
    /// Client GL scanout of SHM trees is not yet wired on the DRM path (dumb-buffer
    /// pageflip only); this filter is the live listing contract for focus and any
    /// future composite path.
    fn window_ids_for_present(&self) -> Vec<&str> {
        let order: Vec<&str> = self.windows.iter().map(|w| w.window_id.as_str()).collect();
        visible_paint_order(&self.workspace_state, &order)
    }

    /// Focus topmost visible window after map/destroy/workspace change; clear if none.
    fn apply_focus_after_workspace_switch(&mut self) {
        let order: Vec<&str> = self.windows.iter().map(|w| w.window_id.as_str()).collect();
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

    fn handle_libinput(
        &mut self,
        event: smithay::backend::input::InputEvent<LibinputInputBackend>,
    ) {
        use smithay::backend::input::{
            AbsolutePositionEvent, Event as _, InputEvent, KeyboardKeyEvent, PointerButtonEvent,
        };
        match event {
            InputEvent::Keyboard { event } => {
                let _ = event.key_code();
                let _ = event.time_msec();
            }
            InputEvent::PointerMotionAbsolute { event } => {
                let x = event.x_transformed(self.output_size.0);
                let y = event.y_transformed(self.output_size.1);
                self.pointer_location = Point::from((x, y));
            }
            InputEvent::PointerButton { event } => {
                let _ = event.button_code();
            }
            _ => {}
        }
    }

    fn focus_surface(&mut self, surface: Option<WlSurface>) {
        let serial = self.next_serial();
        if let Some(kb) = self.seat.get_keyboard() {
            kb.set_focus(self, surface.clone(), serial);
        }
        let client = surface.and_then(|s| s.client());
        set_data_device_focus(&self.display_handle, &self.seat, client.clone());
        set_primary_focus(&self.display_handle, &self.seat, client);
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
        for w in self.windows.iter_mut() {
            if w.toplevel.wl_surface() == surface {
                let st = w.toplevel.current_state();
                let sw = st.size.map(|s| s.w).filter(|v| *v > 0).unwrap_or(DEFAULT_WINDOW_W);
                let sh = st.size.map(|s| s.h).filter(|v| *v > 0).unwrap_or(DEFAULT_WINDOW_H);
                w.size = Size::from((sw, sh));
                break;
            }
        }
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

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}

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
        surface.with_pending_state(|state| {
            state.size = Some(Size::from((DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)));
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();

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
                .unwrap_or_else(|| "retroshell.app".into());
            (title, app_id)
        });
        let foreign = self
            .foreign_toplevel_list
            .new_toplevel::<DrmSessionState>(&title, &app_id);

        let offset = (self.windows.len() as i32) * 32;
        let position = Point::from((64 + offset, 64 + offset));
        eprintln!(
            "[retro-compositor/drm] toplevel mapped at ({},{}) title={title}",
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
            position,
            size: Size::from((DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)),
        });
        // Listing/present filter: only active-workspace ids (client SHM composite TBD).
        eprintln!(
            "[retro-compositor/drm] {} window_id={window_id} present={:?}",
            self.workspace_state.summary_line(),
            self.window_ids_for_present()
        );
        self.focus_surface(Some(surface.wl_surface().clone()));
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        if let Some(idx) = self
            .windows
            .iter()
            .position(|w| w.toplevel.wl_surface() == surface.wl_surface())
        {
            let win = self.windows.remove(idx);
            self.workspace_state.remove_window(&win.window_id);
            win.foreign.send_closed();
        }
        // Prefer topmost **visible** window; clear focus if none on active workspace.
        self.apply_focus_after_workspace_switch();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
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
        if let Some(w) = self
            .windows
            .iter()
            .find(|w| w.toplevel.wl_surface() == surface.wl_surface())
        {
            w.foreign.send_app_id(&app_id);
            w.foreign.send_done();
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
            "[retro-compositor/drm] layer-shell surface namespace={namespace} layer={layer:?}"
        );
        // Size hint: full-output; clients still set exclusive zone / anchors.
        let (w, h) = self.output_size;
        surface.with_pending_state(|state| {
            state.size = Some(Size::from((w, h)));
        });
        surface.send_configure();
        self.layer_surfaces.push(MappedLayer {
            surface,
            layer,
            namespace,
        });
    }

    fn layer_destroyed(&mut self, surface: LayerSurface) {
        self.layer_surfaces
            .retain(|l| l.surface.wl_surface() != surface.wl_surface());
    }
}
delegate_layer_shell!(DrmSessionState);

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
