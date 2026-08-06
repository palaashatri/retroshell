//! slopos-compositor — minimal Wayland compositor using Smithay.
//!
//! This compositor replaces labwc in the SLOPOS-I stack. It:
//!   - Opens an X11 window (running nested under Xvfb on DISPLAY=:99)
//!   - Exposes a Wayland socket so slopos-shell (winit/wgpu) can connect
//!   - Implements xdg_shell, wl_shm, wl_seat for basic window management
//!   - Implements wl_data_device selection send (clipboard + primary store)
//!   - Optionally multi-output via SLOPOS_OUTPUTS=WxH,WxH or
//!     SLOPOS_OUTPUTS_LAYOUT (shell display arrange: name:WxH@x,y:sNN;...)
//!   - Optionally starts XWayland (best-effort under nested X11)
//!
//! Linux-only: requires libgbm, libdrm, libEGL, libxcb and libwayland-server.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("slopos-compositor is Linux-only (requires Wayland/DRM/GBM system libraries).");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() -> anyhow::Result<()> {
    linux::run()
}

#[cfg(target_os = "linux")]
mod linux {
    use anyhow::Context;
    use std::collections::{HashMap, HashSet};
    use std::io::Write;
    use std::os::unix::io::OwnedFd;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;

    use slopos_bus::{SessionControlListener, SessionControlRequest, WindowPresentationAction};
    use slopos_compositor::frame_timing::{FrameScheduler, RefreshRate};
    use slopos_compositor::hdr::HdrCapabilities;
    use slopos_compositor::work_area::{compute_exclusive_work_area, ExclusiveZoneReservation};
    use slopos_compositor::{
        accumulate_damage_for_window_move, accumulate_damage_rect, apply_scale_to_output_config,
        assign_new_window_to_active, cascade_position, clamp_window_to_work_area,
        clear_interactive_grab_state, detect_output_scale_from_env,
        focus_window_after_workspace_switch, geometries_intersect, geometry_for_interactive_grab,
        move_to_top, next_cascade_offset, output_geometry, output_index_for_geometry,
        output_index_for_point, output_scale_summary, pointer_grab_request_is_valid_for_window,
        prefer_full_redraw, register_wayland_display_source, resolve_laid_out_outputs_from_env,
        selection_bytes_for_mime_with_text_fallback, session_mode_note, surface_tree_root,
        text_input_capability_from_env, text_input_capability_summary, total_output_size,
        transition_presentation_state, window_paint_source, CompositorBackendKind, DamageRect,
        DisplayPolicy, InteractiveGrab, InteractiveGrabKind, LaidOutOutput, OutputScale,
        PlaceholderPresentStats, ResizeEdges, TextInputCapability, WindowGeometry,
        WindowPaintSource, WindowPresentationState, WindowRestoreState, WorkspaceId,
        WorkspaceState, DEFAULT_WINDOW_H, DEFAULT_WINDOW_W,
    };
    use smithay::desktop::{
        find_popup_root_surface, get_popup_toplevel_coords, utils::under_from_surface_tree,
        PopupGrab, PopupKeyboardGrab, PopupKind, PopupManager, PopupPointerGrab, WindowSurfaceType,
    };
    use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
    use smithay::reexports::wayland_server::protocol::{
        wl_buffer, wl_data_source::WlDataSource, wl_seat,
    };
    use smithay::utils::Serial as WlSerial;
    use smithay::wayland::xwayland_shell::{XWaylandShellHandler, XWaylandShellState};
    use smithay::xwayland::xwm::{Reorder, ResizeEdge, X11Window, XwmId};
    use smithay::xwayland::{
        X11Surface as X11WmSurface, X11Wm, XWayland, XWaylandEvent, XwmHandler,
    };
    use smithay::{
        backend::{
            allocator::{
                dmabuf::DmabufAllocator,
                gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            },
            egl::{EGLContext, EGLDisplay},
            input::{
                ButtonState, InputEvent as BackendInputEvent, KeyboardKeyEvent, PointerButtonEvent,
                PointerMotionAbsoluteEvent,
            },
            renderer::{
                element::{
                    surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                    Kind,
                },
                gles::GlesRenderer,
                utils::{draw_render_elements, on_commit_buffer_handler},
                Bind, Color32F, Frame, Renderer,
            },
            x11::{WindowBuilder, X11Backend, X11Event, X11Input, X11Surface},
        },
        delegate_compositor, delegate_foreign_toplevel_list, delegate_layer_shell, delegate_output,
        delegate_seat, delegate_shm, delegate_xdg_shell,
        desktop::utils::send_frames_surface_tree,
        input::{
            keyboard::{FilterResult, XkbConfig},
            pointer::{
                AxisFrame, ButtonEvent, CursorImageStatus, CursorImageSurfaceData, Focus,
                GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
                GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
                GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData, MotionEvent,
                PointerGrab, PointerInnerHandle, RelativeMotionEvent,
            },
            Seat, SeatHandler, SeatState,
        },
        output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
        reexports::{
            calloop::{
                generic::Generic, EventLoop, Interest, LoopHandle, LoopSignal, Mode as CalloopMode,
                PostAction,
            },
            wayland_server::{
                backend::{ClientData, ClientId, DisconnectReason},
                protocol::{wl_output, wl_surface::WlSurface},
                Display, DisplayHandle, Resource,
            },
        },
        utils::{
            Clock, DeviceFd, Logical, Monotonic, Physical, Point, Rectangle, Serial, Size,
            Transform,
        },
        wayland::{
            buffer::BufferHandler,
            compositor::{with_states, CompositorClientState, CompositorHandler, CompositorState},
            foreign_toplevel_list::{
                ForeignToplevelHandle, ForeignToplevelListHandler, ForeignToplevelListState,
            },
            output::{OutputHandler, OutputManagerState},
            selection::{
                data_device::{
                    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler,
                    DataDeviceState, ServerDndGrabHandler,
                },
                primary_selection::{
                    set_primary_focus, PrimarySelectionHandler, PrimarySelectionState,
                },
                SelectionHandler, SelectionSource, SelectionTarget,
            },
            shell::wlr_layer::{
                Anchor, Layer, LayerSurface, LayerSurfaceCachedState, Margins,
                WlrLayerShellHandler, WlrLayerShellState,
            },
            shell::xdg::{
                decoration::{XdgDecorationHandler, XdgDecorationState},
                PopupSurface, PositionerState, SurfaceCachedState, ToplevelSurface,
                XdgShellHandler, XdgShellState, XdgToplevelSurfaceData,
            },
            shm::{ShmHandler, ShmState},
            socket::ListeningSocketSource,
        },
    };
    use smithay::{delegate_primary_selection, delegate_xwayland_shell};

    // Retro gray: rgb(152, 152, 148) — the classic Mac OS desktop fill
    const RETRO_GRAY: (u8, u8, u8) = (152, 152, 148);

    // Window placeholder colors (cycling palette for distinguishing windows)
    const WIN_COLORS: &[(f32, f32, f32)] = &[
        (0.502, 0.502, 1.000), // soft blue
        (0.502, 1.000, 0.502), // soft green
        (1.000, 0.502, 0.502), // soft red
        (1.000, 1.000, 0.502), // soft yellow
        (0.502, 1.000, 1.000), // soft cyan
        (1.000, 0.502, 1.000), // soft magenta
    ];

    /// Compositor-owned selection payload keyed by mime type.
    /// Used as [`SelectionHandler::SelectionUserData`] for server-set selections.
    type MimePayload = Arc<HashMap<String, Vec<u8>>>;

    // -----------------------------------------------------------------------
    // Per-client data
    // -----------------------------------------------------------------------

    #[derive(Default)]
    struct ClientState {
        compositor_state: CompositorClientState,
    }

    impl ClientData for ClientState {
        fn initialized(&self, _client_id: ClientId) {
            eprintln!("[slopos-compositor] client connected");
        }
        fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
            eprintln!("[slopos-compositor] client disconnected");
        }
    }

    // -----------------------------------------------------------------------
    // Tracked surface: a mapped xdg_toplevel with a compositor-space position
    // -----------------------------------------------------------------------

    #[derive(Clone)]
    struct MappedWindow {
        toplevel: ToplevelSurface,
        /// Foreign-toplevel-list handle for task list / Force Quit / overview.
        foreign: ForeignToplevelHandle,
        /// Stable id for workspace visibility (`foreign.identifier()` at map).
        window_id: String,
        /// Wayland app_id captured at map time; the shell uses compositor focus
        /// state to select the corresponding global-menu manifest.
        app_id: String,
        /// Top-left position in logical compositor space
        position: Point<i32, Logical>,
        /// Last committed size (logical pixels)
        size: Size<i32, Logical>,
        /// Single-authority presentation state (Normal, Minimized, SmartZoomed, Filled, Fullscreen, Tiled).
        presentation_state: WindowPresentationState,
        /// Saved restore state prior to zoom/fill/fullscreen/tiling.
        restore_state: Option<WindowRestoreState>,
        /// Minimized windows stay mapped but are excluded from hit-testing/painting.
        minimized: bool,
    }

    struct MappedLayer {
        surface: LayerSurface,
        layer: Layer,
        namespace: String,
        /// Authoritative compositor-space placement of the layer surface.
        geo: Rectangle<i32, Logical>,
        /// Exclusive work-area reservation requested by the layer client.
        exclusive_zone: i32,
    }

    #[derive(Clone)]
    struct PointerPress {
        serial: Serial,
        /// Mapped toplevel that owns the hit toplevel/popup surface tree.
        window_id: String,
    }

    struct InteractivePointerGrab {
        start_data: GrabStartData<SloposCompositor>,
    }

    impl PointerGrab<SloposCompositor> for InteractivePointerGrab {
        fn motion(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            _focus: Option<(WlSurface, Point<f64, Logical>)>,
            event: &MotionEvent,
        ) {
            if !data.update_interactive_grab() {
                handle.unset_grab(self, data, event.serial, event.time, true);
                return;
            }
            handle.motion(data, self.start_data.focus.clone(), event);
        }

        fn relative_motion(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            _focus: Option<(WlSurface, Point<f64, Logical>)>,
            event: &RelativeMotionEvent,
        ) {
            if !data.update_interactive_grab() {
                let serial = data.next_serial();
                let time = u32::try_from(event.utime / 1_000)
                    .unwrap_or(u32::MAX)
                    .max(1);
                handle.unset_grab(self, data, serial, time, true);
                return;
            }
            handle.relative_motion(data, self.start_data.focus.clone(), event);
        }

        fn button(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &ButtonEvent,
        ) {
            handle.button(data, event);
            if event.state == ButtonState::Released && handle.current_pressed().is_empty() {
                handle.unset_grab(self, data, event.serial, event.time, true);
            }
        }

        fn axis(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            details: AxisFrame,
        ) {
            handle.axis(data, details);
        }

        fn frame(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
        ) {
            handle.frame(data);
        }

        fn gesture_swipe_begin(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GestureSwipeBeginEvent,
        ) {
            handle.gesture_swipe_begin(data, event);
        }

        fn gesture_swipe_update(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GestureSwipeUpdateEvent,
        ) {
            handle.gesture_swipe_update(data, event);
        }

        fn gesture_swipe_end(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GestureSwipeEndEvent,
        ) {
            handle.gesture_swipe_end(data, event);
        }

        fn gesture_pinch_begin(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GesturePinchBeginEvent,
        ) {
            handle.gesture_pinch_begin(data, event);
        }

        fn gesture_pinch_update(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GesturePinchUpdateEvent,
        ) {
            handle.gesture_pinch_update(data, event);
        }

        fn gesture_pinch_end(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GesturePinchEndEvent,
        ) {
            handle.gesture_pinch_end(data, event);
        }

        fn gesture_hold_begin(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GestureHoldBeginEvent,
        ) {
            handle.gesture_hold_begin(data, event);
        }

        fn gesture_hold_end(
            &mut self,
            data: &mut SloposCompositor,
            handle: &mut PointerInnerHandle<'_, SloposCompositor>,
            event: &GestureHoldEndEvent,
        ) {
            handle.gesture_hold_end(data, event);
        }

        fn start_data(&self) -> &GrabStartData<SloposCompositor> {
            &self.start_data
        }

        fn unset(&mut self, data: &mut SloposCompositor) {
            data.finish_interactive_grab();
        }
    }

    fn layer_policy_defaults(
        namespace: &str,
        output: Size<i32, Logical>,
    ) -> (Size<i32, Logical>, Anchor) {
        match namespace {
            "slopos-i-menu" | "menu-bar" => (
                Size::from((output.w, 24)),
                Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            ),
            "slopos-i-dock" | "dock" => (
                Size::from((output.w, 64)),
                Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            ),
            "slopos-i-menu-popup" => (Size::from((1, 1)), Anchor::TOP | Anchor::LEFT),
            _ => (
                output,
                Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            ),
        }
    }

    fn layer_geometry_for(
        namespace: &str,
        layer: Layer,
        output: Size<i32, Logical>,
        requested: Size<i32, Logical>,
        anchor: Anchor,
        margins: Margins,
    ) -> Rectangle<i32, Logical> {
        let (fallback_size, fallback_anchor) = layer_policy_defaults(namespace, output);
        let anchor = if anchor.is_empty() {
            fallback_anchor
        } else {
            anchor
        };
        let left = margins.left.max(0);
        let right = margins.right.max(0);
        let top = margins.top.max(0);
        let bottom = margins.bottom.max(0);

        let width = if requested.w == 0 {
            if anchor.anchored_horizontally() {
                (output.w - left - right).max(1)
            } else {
                fallback_size.w
            }
        } else {
            requested.w
        }
        .clamp(1, output.w.max(1));
        let height = if requested.h == 0 {
            if anchor.anchored_vertically() {
                (output.h - top - bottom).max(1)
            } else {
                fallback_size.h
            }
        } else {
            requested.h
        }
        .clamp(1, output.h.max(1));

        let x = if anchor.contains(Anchor::LEFT) && anchor.contains(Anchor::RIGHT) {
            left
        } else if anchor.contains(Anchor::RIGHT) {
            (output.w - width - right).max(0)
        } else if anchor.contains(Anchor::LEFT) {
            left
        } else {
            (output.w - width) / 2
        };
        let y = if anchor.contains(Anchor::TOP) && anchor.contains(Anchor::BOTTOM) {
            top
        } else if anchor.contains(Anchor::BOTTOM) {
            (output.h - height - bottom).max(0)
        } else if anchor.contains(Anchor::TOP) {
            top
        } else {
            (output.h - height) / 2
        };

        // `layer` is intentionally part of the policy signature: layer order
        // controls composition, while anchors control geometry. Keeping both
        // here prevents callers from accidentally treating a Bottom surface as
        // a normal xdg window when adding new chrome roles.
        let _ = layer;
        Rectangle::new((x, y).into(), (width, height).into())
    }

    fn layer_surface_request(surface: &LayerSurface) -> (Size<i32, Logical>, Anchor, Margins, i32) {
        with_states(surface.wl_surface(), |states| {
            let mut cached = states.cached_state.get::<LayerSurfaceCachedState>();
            let current = *cached.current();
            (
                current.size,
                current.anchor,
                current.margin,
                current.exclusive_zone.into(),
            )
        })
    }

    /// Convert a surface-tree hit origin (relative to the layer surface) into
    /// compositor-space logical coordinates.
    pub(super) fn layer_surface_hit_origin(
        layer_origin: Point<i32, Logical>,
        surface_origin: Point<i32, Logical>,
    ) -> Point<f64, Logical> {
        Point::from((
            layer_origin.x as f64 + surface_origin.x as f64,
            layer_origin.y as f64 + surface_origin.y as f64,
        ))
    }

    impl MappedWindow {
        fn geometry(&self) -> WindowGeometry {
            WindowGeometry::new(self.position.x, self.position.y, self.size.w, self.size.h)
        }
    }

    pub(super) fn x11_resize_edge_to_resize_edges(edge: ResizeEdge) -> ResizeEdges {
        match edge {
            ResizeEdge::Top => ResizeEdges::TOP,
            ResizeEdge::Bottom => ResizeEdges::BOTTOM,
            ResizeEdge::Left => ResizeEdges::LEFT,
            ResizeEdge::Right => ResizeEdges::RIGHT,
            ResizeEdge::TopLeft => ResizeEdges::TOP_LEFT,
            ResizeEdge::BottomLeft => ResizeEdges::BOTTOM_LEFT,
            ResizeEdge::TopRight => ResizeEdges::TOP_RIGHT,
            ResizeEdge::BottomRight => ResizeEdges::BOTTOM_RIGHT,
        }
    }

    // -----------------------------------------------------------------------
    // Main compositor state
    // -----------------------------------------------------------------------

    struct SloposCompositor {
        display_handle: DisplayHandle,
        _loop_signal: LoopSignal,
        loop_handle: LoopHandle<'static, SloposCompositor>,
        clock: Clock<Monotonic>,

        // Smithay protocol states
        compositor_state: CompositorState,
        shm_state: ShmState,
        seat_state: SeatState<SloposCompositor>,
        xdg_shell_state: XdgShellState,
        data_device_state: DataDeviceState,
        primary_selection_state: PrimarySelectionState,
        _output_manager_state: OutputManagerState,
        xwayland_shell_state: XWaylandShellState,
        layer_shell_state: WlrLayerShellState,
        foreign_toplevel_list: ForeignToplevelListState,
        _xdg_decoration_state: XdgDecorationState,
        /// Present when SLOPOS_TEXT_INPUT enables text-input-v3 global.
        _text_input_state: Option<smithay::wayland::text_input::TextInputManagerState>,
        /// Present when SLOPOS_TEXT_INPUT=full|im enables input-method-v2.
        _input_method_state: Option<smithay::wayland::input_method::InputMethodManagerState>,
        /// Input-method popup surfaces (IME UI).
        im_popups: Vec<smithay::wayland::input_method::PopupSurface>,

        seat: Seat<SloposCompositor>,
        /// Registered wl_output objects (one or more; multi-output via SLOPOS_OUTPUTS).
        /// Kept alive so globals stay registered for the compositor lifetime.
        #[allow(dead_code)]
        outputs: Vec<Output>,
        /// Normalized logical output rectangles used for window assignment.
        laid_out_outputs: Vec<LaidOutOutput>,
        /// Connector or synthetic names parallel to `laid_out_outputs`.
        output_names: Vec<String>,
        running: bool,

        // Mapped windows (in painting order, bottom → top)
        windows: Vec<MappedWindow>,
        /// Virtual workspaces: only active-workspace windows are painted.
        workspace_state: WorkspaceState,
        // Layer-shell chrome (menu bar, dock, notifications, …)
        layer_surfaces: Vec<MappedLayer>,
        /// Tracks xdg popup trees independently of ordinary toplevel windows.
        popup_manager: PopupManager,
        /// The currently active popup grab, if a client requested one.
        popup_grab: Option<PopupGrab<SloposCompositor>>,
        /// Window whose xdg_toplevel state currently carries Activated.
        activated_window_id: Option<String>,
        /// Generic Restore targets the most recently minimized client. Focus
        /// moves to another visible window after minimize, so the active id
        /// alone cannot identify the Dock restore target.
        last_minimized_window_id: Option<String>,
        // Counter for cascading new window placement
        next_window_offset: i32,
        // Current pointer position (logical)
        pointer_pos: Point<f64, Logical>,
        /// Client requested cursor surface/name; Named always has a software fallback.
        cursor_status: CursorImageStatus,
        /// Current compositor-owned xdg_toplevel move/resize operation.
        interactive_grab: Option<InteractiveGrab>,
        /// Tracks BTN_LEFT so stale xdg move/resize requests cannot start a grab.
        left_button_down: bool,
        /// The most recent left-button press delivered to an application surface.
        /// xdg_toplevel.move/resize must consume this exact serial while held.
        last_pointer_press: Option<PointerPress>,
        /// A frame is produced only after damage, input, commit, or a frame event.
        frame_dirty: bool,
        // Output size advertised for X11 input transforms (union of all outputs).
        output_size: Size<i32, Physical>,
        // Serial counter for synthetic events
        serial: u32,

        // GL rendering
        renderer: Option<GlesRenderer>,
        x11_surface: Option<X11Surface>,

        // ---- selection / DnD store (P1.1) ----
        /// Last client clipboard SelectionSource (for tracking / XWayland bridge).
        clipboard_source: Option<SelectionSource>,
        /// Last client primary SelectionSource.
        primary_source: Option<SelectionSource>,
        /// Compositor-owned clipboard mime → bytes (server-set selections).
        clipboard_data: HashMap<String, Vec<u8>>,
        /// Compositor-owned primary mime → bytes.
        primary_data: HashMap<String, Vec<u8>>,
        /// Server-initiated DnD mime payloads (written in ServerDndGrabHandler::send).
        server_dnd_data: HashMap<String, Vec<u8>>,
        /// Client DnD icon surface (if any).
        dnd_icon: Option<WlSurface>,

        // ---- HDR / VRR (P1.4) ----
        /// Applied policy snapshot (logged at startup; retained for introspection).
        #[allow(dead_code)]
        display_policy: DisplayPolicy,
        #[allow(dead_code)]
        hdr_caps: HdrCapabilities,
        frame_scheduler: FrameScheduler,

        // ---- Damage / present honesty ----
        /// Union of dirty regions from window moves/resizes (partial present plan).
        pending_damage: Option<DamageRect>,
        /// Set on workspace switch so the next frame redraws the full output.
        need_full_redraw: bool,
        /// Counts frames that fell back to solid placeholders; logs once per session.
        placeholder_stats: PlaceholderPresentStats,

        // ---- XWayland (P1.3) ----
        xwm: Option<X11Wm>,
        xdisplay: Option<u32>,
        /// X11 surfaces we know about (not fully managed yet under nested X11).
        x11_surfaces: Vec<X11WmSurface>,
        /// XWayland windows associated with live Wayland surfaces.
        x11_surface_associations: HashMap<X11Window, WlSurface>,
        /// Wayland socket name advertised to spawned clients (Super+O/L shortcuts).
        wayland_socket_name: String,
    }

    pub(super) fn bind_session_control_listener(
        runtime: &std::path::Path,
    ) -> anyhow::Result<SessionControlListener> {
        SessionControlListener::bind(runtime)
            .map_err(|error| anyhow::anyhow!("bind session control socket: {error}"))
    }

    impl SloposCompositor {
        /// Allocate the next serial (wrapping)
        fn next_serial(&mut self) -> Serial {
            self.serial = self.serial.wrapping_add(1);
            Serial::from(self.serial)
        }

        fn popup_origin(
            root_origin: Point<i32, Logical>,
            popup: &PopupKind,
            popup_offset: Point<i32, Logical>,
        ) -> Point<i32, Logical> {
            let geometry = popup.geometry();
            Point::from((
                root_origin.x + popup_offset.x - geometry.loc.x,
                root_origin.y + popup_offset.y - geometry.loc.y,
            ))
        }

        /// Find the topmost surface under a compositor-space point.
        ///
        /// Hit testing follows the committed surface trees rather than the
        /// compositor's configured rectangles. This preserves subsurface
        /// offsets, actual committed buffer sizes, and client input regions.
        fn layer_surface_under(
            layer: &MappedLayer,
            pt: Point<f64, Logical>,
        ) -> Option<(WlSurface, Point<f64, Logical>)> {
            for (popup, popup_offset) in
                PopupManager::popups_for_surface(layer.surface.wl_surface())
            {
                let origin = Self::popup_origin(layer.geo.loc, &popup, popup_offset);
                if let Some((surface, surface_origin)) =
                    under_from_surface_tree(popup.wl_surface(), pt, origin, WindowSurfaceType::ALL)
                {
                    return Some((surface, surface_origin.to_f64()));
                }
            }

            let local = Point::from((pt.x - layer.geo.loc.x as f64, pt.y - layer.geo.loc.y as f64));
            let (surface, origin) = under_from_surface_tree(
                layer.surface.wl_surface(),
                local,
                (0, 0),
                WindowSurfaceType::ALL,
            )?;
            Some((surface, layer_surface_hit_origin(layer.geo.loc, origin)))
        }

        fn surface_under(
            &self,
            pt: Point<f64, Logical>,
        ) -> Option<(WlSurface, Point<f64, Logical>)> {
            for layer in self.layer_surfaces.iter().rev() {
                if matches!(layer.layer, Layer::Overlay | Layer::Top) {
                    if let Some(hit) = Self::layer_surface_under(layer, pt) {
                        return Some(hit);
                    }
                }
            }

            for window in self
                .windows
                .iter()
                .rev()
                .filter(|w| !w.minimized && self.workspace_state.is_visible(&w.window_id))
            {
                for (popup, popup_offset) in
                    PopupManager::popups_for_surface(window.toplevel.wl_surface())
                {
                    let origin = Self::popup_origin(window.position, &popup, popup_offset);
                    if let Some((surface, surface_origin)) = under_from_surface_tree(
                        popup.wl_surface(),
                        pt,
                        origin,
                        WindowSurfaceType::ALL,
                    ) {
                        return Some((surface, surface_origin.to_f64()));
                    }
                }
                if let Some((surface, surface_origin)) = under_from_surface_tree(
                    window.toplevel.wl_surface(),
                    pt,
                    window.position,
                    WindowSurfaceType::ALL,
                ) {
                    return Some((surface, surface_origin.to_f64()));
                }
            }

            for layer in self.layer_surfaces.iter().rev() {
                if matches!(layer.layer, Layer::Bottom | Layer::Background) {
                    if let Some(hit) = Self::layer_surface_under(layer, pt) {
                        return Some(hit);
                    }
                }
            }
            None
        }

        /// Resolve a surface to its mapped toplevel owner. Subsurfaces are
        /// normalized to their role-bearing tree root; popup roots are then
        /// accepted only when tracked under a known mapped toplevel.
        fn mapped_window_index_for_surface(&self, surface: &WlSurface) -> Option<usize> {
            let tree_root = surface_tree_root(surface);
            if let Some(index) = self
                .windows
                .iter()
                .position(|window| window.toplevel.wl_surface() == &tree_root)
            {
                return Some(index);
            }
            let popup = self.popup_manager.find_popup(&tree_root)?;
            let root = find_popup_root_surface(&popup).ok()?;
            self.windows
                .iter()
                .position(|window| window.toplevel.wl_surface() == &root)
        }

        fn popup_root_origin(&self, popup: &PopupKind) -> Option<Point<i32, Logical>> {
            let root = find_popup_root_surface(popup).ok()?;
            if let Some(window) = self
                .windows
                .iter()
                .find(|window| window.toplevel.wl_surface() == &root)
            {
                return Some(window.position);
            }
            self.layer_surfaces
                .iter()
                .find(|layer| layer.surface.wl_surface() == &root)
                .map(|layer| layer.geo.loc)
        }

        fn constrained_popup_geometry(
            &self,
            popup: &PopupKind,
            positioner: PositionerState,
        ) -> Rectangle<i32, Logical> {
            let Some(root_origin) = self.popup_root_origin(popup) else {
                return positioner.get_geometry();
            };
            let parent_offset = get_popup_toplevel_coords(popup);
            let output = self.output_area_for_point(root_origin);
            let target = Rectangle::new(
                Point::from((
                    output.x - root_origin.x - parent_offset.x,
                    output.y - root_origin.y - parent_offset.y,
                )),
                Size::from((output.width.max(1), output.height.max(1))),
            );
            positioner.get_unconstrained_geometry(target)
        }

        fn activated_window_for_surface(&self, surface: &WlSurface) -> Option<String> {
            self.mapped_window_index_for_surface(surface)
                .map(|index| self.windows[index].window_id.clone())
        }

        /// Keep xdg_toplevel.Activated synchronized with compositor focus.
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
            }
        }

        fn begin_popup_grab(
            &mut self,
            surface: PopupSurface,
            seat: wl_seat::WlSeat,
            serial: Serial,
        ) {
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
            self.request_redraw();
        }

        /// Bring window at `idx` to the top and focus keyboard+pointer on it.
        fn focus_window(&mut self, idx: usize) {
            if idx >= self.windows.len() {
                return;
            }
            let app_id = self.windows[idx].app_id.clone();
            self.windows[idx].minimized = false;
            // Rotate to top
            let surface = self.windows[idx].toplevel.wl_surface().clone();
            move_to_top(&mut self.windows, idx);

            self.focus_surface(Some(surface.clone()));
            if let Err(err) = slopos_compositor::publish_active_toplevel(Some(&app_id)) {
                tracing::debug!(error = %err, app_id = %app_id, "could not publish active application");
            }
            let serial = self.next_serial();
            // Move pointer focus to surface at (0,0) within the window
            if let Some(ptr) = self.seat.get_pointer() {
                let win = self.windows.last().unwrap();
                let local = Point::from((
                    (self.pointer_pos.x - win.position.x as f64),
                    (self.pointer_pos.y - win.position.y as f64),
                ));
                ptr.motion(
                    self,
                    Some((surface.clone(), local)),
                    &MotionEvent {
                        location: self.pointer_pos,
                        serial,
                        time: 0,
                    },
                );
                ptr.frame(self);
            }
        }

        fn focus_surface(&mut self, surface: Option<WlSurface>) {
            let keeps_interactive_grab = self.interactive_grab.as_ref().is_some_and(|grab| {
                surface.as_ref().is_some_and(|surface| {
                    self.windows.iter().any(|window| {
                        window.window_id == grab.window_id
                            && window.toplevel.wl_surface() == surface
                    })
                })
            });
            if self.interactive_grab.is_some() && !keeps_interactive_grab {
                self.cancel_interactive_grab();
            }
            self.sync_activated_for_surface(surface.as_ref());
            if surface.is_none() {
                if let Err(err) = slopos_compositor::publish_active_toplevel(None) {
                    tracing::debug!(error = %err, "could not clear active application");
                }
            }
            let serial = self.next_serial();
            if let Some(kb) = self.seat.get_keyboard() {
                kb.set_focus(self, surface.clone(), serial);
            }
            let client = surface.and_then(|surface| surface.client());
            set_data_device_focus(&self.display_handle, &self.seat, client.clone());
            set_primary_focus(&self.display_handle, &self.seat, client);
        }

        /// Retarget pointer focus immediately before button delivery.
        ///
        /// A button event carries no position. Replaying motion at the last
        /// compositor pointer location ensures Smithay sends the press to the
        /// surface currently under the click even when the backend did not
        /// report a motion immediately beforehand.
        fn forward_pointer_motion(&mut self, time: u32) {
            let pos = self.pointer_pos;
            let focus = self.surface_under(pos);
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

        /// Remove dead windows (client disconnected / surface destroyed).
        fn prune_dead_windows(&mut self) {
            let dead_ids: HashSet<String> = self
                .windows
                .iter()
                .filter(|window| !window.toplevel.alive())
                .map(|window| window.window_id.clone())
                .collect();
            if dead_ids.is_empty() {
                return;
            }

            if self
                .interactive_grab
                .as_ref()
                .is_some_and(|grab| dead_ids.contains(&grab.window_id))
            {
                self.cancel_interactive_grab();
            }
            if self
                .last_pointer_press
                .as_ref()
                .is_some_and(|press| dead_ids.contains(&press.window_id))
            {
                self.last_pointer_press = None;
                self.left_button_down = false;
            }

            let mut retained =
                Vec::with_capacity(self.windows.len().saturating_sub(dead_ids.len()));
            for window in self.windows.drain(..) {
                if dead_ids.contains(&window.window_id) {
                    self.workspace_state.remove_window(&window.window_id);
                    window.foreign.send_closed();
                } else {
                    retained.push(window);
                }
            }
            self.windows = retained;

            if self
                .last_minimized_window_id
                .as_ref()
                .is_some_and(|id| dead_ids.contains(id))
            {
                self.last_minimized_window_id = None;
            }

            self.request_full_redraw();
            self.apply_focus_after_workspace_switch();
        }

        /// After Super+workspace switch: unfocus windows now hidden; focus topmost
        /// visible window (paint order bottom→top). Clears keyboard focus when none.
        fn apply_focus_after_workspace_switch(&mut self) {
            let order: Vec<&str> = self
                .windows
                .iter()
                .filter(|w| !w.minimized)
                .map(|w| w.window_id.as_str())
                .collect();
            let target = focus_window_after_workspace_switch(&self.workspace_state, &order)
                .map(str::to_owned);
            if let Some(id) = target {
                if let Some(idx) = self.windows.iter().position(|w| w.window_id == id) {
                    self.focus_window(idx);
                    return;
                }
            }
            // No visible window on this workspace — drop keyboard/selection focus so a
            // hidden window does not keep receiving keys.
            let serial = self.next_serial();
            if let Some(kb) = self.seat.get_keyboard() {
                kb.set_focus(self, None, serial);
            }
            self.sync_activated_for_surface(None);
            set_data_device_focus(&self.display_handle, &self.seat, None);
            set_primary_focus(&self.display_handle, &self.seat, None);
        }

        fn request_full_redraw(&mut self) {
            self.need_full_redraw = true;
            self.pending_damage = None;
            self.frame_dirty = true;
        }

        fn request_redraw(&mut self) {
            self.frame_dirty = true;
        }

        /// Record dirty rects when a window moves/resizes (`accumulate_damage` over old+new).
        fn note_window_geometry_change(
            &mut self,
            window_id: &str,
            old: WindowGeometry,
            new: WindowGeometry,
        ) {
            if old == new {
                return;
            }
            if let Some(d) = accumulate_damage_for_window_move(window_id, old, new) {
                self.pending_damage = Some(accumulate_damage_rect(self.pending_damage, d));
            }
            self.frame_dirty = true;
        }

        /// Move a mapped window and accumulate damage over old+new extents.
        #[allow(dead_code)] // used when interactive move/shell rules land
        fn set_window_position(&mut self, idx: usize, x: i32, y: i32) {
            if idx >= self.windows.len() {
                return;
            }
            let old = self.windows[idx].geometry();
            self.windows[idx].position = Point::from((x, y));
            let new = self.windows[idx].geometry();
            let id = self.windows[idx].window_id.clone();
            self.note_window_geometry_change(&id, old, new);
        }

        fn begin_interactive_grab(
            &mut self,
            surface: &ToplevelSurface,
            kind: InteractiveGrabKind,
            seat: &wl_seat::WlSeat,
            serial: Serial,
        ) {
            let requested_surface = surface.wl_surface();
            let Some((window_id, start_geometry, window_position)) = self
                .windows
                .iter()
                .find(|w| w.toplevel.wl_surface() == requested_surface)
                .map(|window| (window.window_id.clone(), window.geometry(), window.position))
            else {
                tracing::debug!(?kind, "rejecting interactive request for an unknown window");
                return;
            };
            let pressed_serial = self
                .last_pointer_press
                .as_ref()
                .map(|press| u32::from(press.serial));
            let pressed_window_id = self
                .last_pointer_press
                .as_ref()
                .map(|press| press.window_id.as_str());
            let same_client = match (requested_surface.client(), seat.client()) {
                (Some(surface_client), Some(seat_client)) => surface_client == seat_client,
                _ => false,
            };
            let authorized = pointer_grab_request_is_valid_for_window(
                u32::from(serial),
                pressed_serial,
                &window_id,
                pressed_window_id,
                self.left_button_down,
                self.seat.owns(seat),
                same_client,
            );
            if !authorized {
                tracing::debug!(
                    request_serial = u32::from(serial),
                    ?kind,
                    pressed_window_id,
                    requested_window_id = %window_id,
                    same_client,
                    "rejecting unauthorized xdg move/resize request"
                );
                return;
            }
            let Some(pointer) = self.seat.get_pointer() else {
                tracing::debug!(?kind, "rejecting interactive request without a pointer");
                return;
            };
            let pointer_location = pointer.current_location();
            let pointer_x = pointer_location.x.round() as i32;
            let pointer_y = pointer_location.y.round() as i32;
            self.interactive_grab = Some(InteractiveGrab {
                window_id: window_id.clone(),
                kind,
                start_pointer_x: pointer_x,
                start_pointer_y: pointer_y,
                start_geometry,
            });
            pointer.set_grab(
                self,
                InteractivePointerGrab {
                    start_data: GrabStartData {
                        focus: Some((
                            requested_surface.clone(),
                            Point::from((window_position.x as f64, window_position.y as f64)),
                        )),
                        button: 0x110,
                        location: pointer_location,
                    },
                },
                serial,
                Focus::Keep,
            );
            if matches!(kind, InteractiveGrabKind::Resize(_)) {
                surface.with_pending_state(|state| {
                    state.size = Some(Size::from((start_geometry.width, start_geometry.height)));
                    state.states.set(xdg_toplevel::State::Resizing);
                });
                surface.send_configure();
            }
            tracing::debug!(
                window_id = %window_id,
                ?kind,
                pointer_x,
                pointer_y,
                "interactive grab started"
            );
        }

        fn update_interactive_grab(&mut self) -> bool {
            let Some(grab) = self.interactive_grab.clone() else {
                return false;
            };
            let Some(idx) = self
                .windows
                .iter()
                .position(|w| w.window_id == grab.window_id)
            else {
                self.finish_interactive_grab();
                return false;
            };
            let min_size = with_states(self.windows[idx].toplevel.wl_surface(), |states| {
                let mut cached = states.cached_state.get::<SurfaceCachedState>();
                cached.current().min_size
            });
            let new = geometry_for_interactive_grab(
                &grab,
                self.pointer_pos.x.round() as i32,
                self.pointer_pos.y.round() as i32,
                160.max(min_size.w),
                96.max(min_size.h),
                self.output_size.w,
                self.output_size.h,
            );
            let old = self.windows[idx].geometry();
            if old == new {
                return true;
            }
            self.windows[idx].position = Point::from((new.x, new.y));
            self.windows[idx].size = Size::from((new.width, new.height));
            let surface = self.windows[idx].toplevel.clone();
            let id = self.windows[idx].window_id.clone();
            if matches!(grab.kind, InteractiveGrabKind::Resize(_)) {
                surface.with_pending_state(|state| {
                    state.size = Some(Size::from((new.width, new.height)));
                    state.states.set(xdg_toplevel::State::Resizing);
                });
                surface.send_configure();
            }
            self.note_window_geometry_change(&id, old, new);
            true
        }

        fn finish_interactive_grab(&mut self) {
            let Some(grab) = clear_interactive_grab_state(
                &mut self.interactive_grab,
                &mut self.last_pointer_press,
                &mut self.left_button_down,
            ) else {
                return;
            };
            if matches!(grab.kind, InteractiveGrabKind::Resize(_)) {
                if let Some(window) = self
                    .windows
                    .iter()
                    .find(|w| w.window_id == grab.window_id && w.toplevel.alive())
                {
                    let surface = window.toplevel.clone();
                    surface.with_pending_state(|state| {
                        state.states.unset(xdg_toplevel::State::Resizing);
                        state.size = Some(window.size);
                    });
                    surface.send_configure();
                }
            }
            tracing::debug!(window_id = %grab.window_id, "interactive grab finished");
            self.request_redraw();
        }

        fn cancel_interactive_grab(&mut self) {
            if self.interactive_grab.is_some() {
                if let Some(pointer) = self.seat.get_pointer() {
                    let serial = self.next_serial();
                    pointer.unset_grab(self, serial, 0);
                } else {
                    self.finish_interactive_grab();
                }
            } else {
                self.left_button_down = false;
                self.last_pointer_press = None;
            }
        }

        fn associated_x11_surface(&self, window: &X11WmSurface) -> Option<WlSurface> {
            self.x11_surface_associations
                .get(&window.window_id())
                .cloned()
        }

        fn x11_toplevel_for_surface(&self, surface: &WlSurface) -> Option<ToplevelSurface> {
            self.windows
                .iter()
                .find(|window| window.toplevel.wl_surface() == surface)
                .map(|window| window.toplevel.clone())
        }

        fn x11_client_seat(&self, surface: &WlSurface) -> Option<wl_seat::WlSeat> {
            let client = surface.client()?;
            self.seat.client_seats(&client).into_iter().next()
        }

        fn begin_x11_interactive_grab(
            &mut self,
            window: &X11WmSurface,
            kind: InteractiveGrabKind,
            button: u32,
        ) {
            let primary_button = button == 0x110 || button == 1;
            if !primary_button {
                tracing::debug!(
                    button,
                    ?kind,
                    "rejecting X11 interactive grab on non-primary button"
                );
                return;
            }

            let Some(surface) = self.associated_x11_surface(window) else {
                tracing::debug!(
                    window = window.window_id(),
                    ?kind,
                    "rejecting X11 interactive grab without associated wl_surface"
                );
                return;
            };
            let Some(toplevel) = self.x11_toplevel_for_surface(&surface) else {
                tracing::debug!(
                    window = window.window_id(),
                    ?kind,
                    "rejecting X11 interactive grab without mapped toplevel"
                );
                return;
            };
            let Some(seat) = self.x11_client_seat(&surface) else {
                tracing::debug!(
                    window = window.window_id(),
                    ?kind,
                    "rejecting X11 interactive grab without client seat resource"
                );
                return;
            };
            let Some(serial) = self.last_pointer_press.as_ref().map(|press| press.serial) else {
                tracing::debug!(
                    window = window.window_id(),
                    ?kind,
                    "rejecting X11 interactive grab without prior pointer press serial"
                );
                return;
            };

            self.begin_interactive_grab(&toplevel, kind, &seat, serial);
        }

        fn canvas_area(&self) -> WindowGeometry {
            WindowGeometry::new(0, 0, self.output_size.w, self.output_size.h)
        }

        fn output_area_for_point(&self, point: Point<i32, Logical>) -> WindowGeometry {
            output_index_for_point(&self.laid_out_outputs, point.x, point.y)
                .and_then(|index| self.laid_out_outputs.get(index))
                .map(output_geometry)
                .unwrap_or_else(|| self.canvas_area())
        }

        fn work_area_for_output(&self, output: WindowGeometry) -> WindowGeometry {
            let reservations = self.layer_surfaces.iter().filter_map(|layer| {
                let layer_geometry = WindowGeometry::new(
                    layer.geo.loc.x,
                    layer.geo.loc.y,
                    layer.geo.size.w,
                    layer.geo.size.h,
                );
                if !geometries_intersect(output, layer_geometry) {
                    return None;
                }
                let (_, anchor, margins, _) = layer_surface_request(&layer.surface);
                Some(ExclusiveZoneReservation {
                    exclusive_zone: layer.exclusive_zone,
                    anchor_top: anchor.contains(Anchor::TOP),
                    anchor_bottom: anchor.contains(Anchor::BOTTOM),
                    anchor_left: anchor.contains(Anchor::LEFT),
                    anchor_right: anchor.contains(Anchor::RIGHT),
                    margin_top: margins.top,
                    margin_bottom: margins.bottom,
                    margin_left: margins.left,
                    margin_right: margins.right,
                })
            });
            compute_exclusive_work_area(output, reservations)
        }

        /// Keep normal windows inside the current compositor-owned work area
        /// after a layer-shell surface changes its exclusive reservation.
        fn clamp_normal_windows_to_work_area(&mut self) {
            let fallback_work_area = self.work_area_for_output(self.canvas_area());
            let output_work_areas: Vec<WindowGeometry> = self
                .laid_out_outputs
                .iter()
                .map(|output| self.work_area_for_output(output_geometry(output)))
                .collect();
            let mut changed = false;
            for window in &mut self.windows {
                if window.minimized
                    || window.presentation_state != WindowPresentationState::Normal
                    || window.app_id.starts_with("com.slopos.shell")
                {
                    continue;
                }
                let current = window.geometry();
                let work_area = output_index_for_geometry(&self.laid_out_outputs, current)
                    .and_then(|index| output_work_areas.get(index).copied())
                    .unwrap_or(fallback_work_area);
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
                self.request_redraw();
            }
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
        /// The shell sends only a semantic application id; this backend owns
        /// the actual restore, stacking, focus, and active-toplevel update.
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
            self.focus_window(idx);
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
                self.focus_window(idx);
            } else {
                self.request_redraw();
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
            let output_index = output_index_for_geometry(&self.laid_out_outputs, old).unwrap_or(0);
            let output_area = self
                .laid_out_outputs
                .get(output_index)
                .map(output_geometry)
                .unwrap_or_else(|| self.canvas_area());
            let work_area = self.work_area_for_output(output_area);
            let output_id = self
                .output_names
                .get(output_index)
                .cloned()
                .unwrap_or_else(|| format!("output-{output_index}"));
            let transition = transition_presentation_state(
                current_state,
                old,
                current_restore_state.as_ref(),
                target_state,
                work_area,
                output_area,
                None,
                output_id,
                self.workspace_state.active.as_usize(),
            );
            self.windows[idx].presentation_state = transition.state;
            self.windows[idx].restore_state = transition.restore_state;
            self.windows[idx].position =
                Point::from((transition.geometry.x, transition.geometry.y));
            self.windows[idx].size =
                Size::from((transition.geometry.width, transition.geometry.height));
            let new = self.windows[idx].geometry();
            let toplevel = self.windows[idx].toplevel.clone();
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
                state.size = Some(Size::from((new.width, new.height)));
            });
            toplevel.send_configure();
            let id = self.windows[idx].window_id.clone();
            self.note_window_geometry_change(&id, old, new);
        }

        fn cycle_workspace_next(&mut self) {
            self.workspace_state.cycle_next();
            self.request_full_redraw();
            eprintln!(
                "[slopos-compositor] {}",
                self.workspace_state.summary_line()
            );
            self.apply_focus_after_workspace_switch();
        }

        fn cycle_workspace_prev(&mut self) {
            self.workspace_state.cycle_prev();
            self.request_full_redraw();
            eprintln!(
                "[slopos-compositor] {}",
                self.workspace_state.summary_line()
            );
            self.apply_focus_after_workspace_switch();
        }

        fn activate_workspace_index(&mut self, index: u8) {
            if let Some(ws) = WorkspaceId::new(index) {
                if self.workspace_state.activate(ws) {
                    self.request_full_redraw();
                    eprintln!(
                        "[slopos-compositor] {}",
                        self.workspace_state.summary_line()
                    );
                    self.apply_focus_after_workspace_switch();
                }
            }
        }

        /// Render a frame using the GlesRenderer:
        ///   1. Acquire an X11 dmabuf
        ///   2. Bind it to the GL renderer
        ///   3. Clear to retro gray; composite layer-shell (under) → windows → layer-shell (over)
        ///   4. Finish the frame and present
        ///
        /// Client presentation honesty:
        /// - Prefer real SHM/client surface trees (`render_elements_from_surface_tree`).
        /// - Solid `WIN_COLORS` placeholders are used **only** for visible windows whose
        ///   surface tree yields zero elements (no committed buffer yet). They never
        ///   replace real content when a buffer has been committed.
        /// - Inactive-workspace windows are not painted (workspace filter).
        /// - Workspace switch requests a full redraw; window moves accumulate damage.
        fn render_frame(&mut self) {
            self.prune_dead_windows();
            self.cleanup_popup_state();
            self.layer_surfaces.retain(|l| l.surface.alive());

            // Present plan: workspace switch forces full redraw; otherwise use pending
            // damage heuristic (still full clear today — partial clip is follow-on).
            let full_redraw = self.need_full_redraw
                || self
                    .pending_damage
                    .is_some_and(|d| prefer_full_redraw(d, self.output_size.w, self.output_size.h));
            self.need_full_redraw = false;
            let _damage_for_present = if full_redraw {
                None
            } else {
                self.pending_damage.take()
            };
            if full_redraw {
                self.pending_damage = None;
            }

            let cursor_status = self.cursor_status.clone();
            let cursor_position = self.pointer_pos;
            let (renderer, x11_surface) =
                match (self.renderer.as_mut(), self.x11_surface.as_mut()) {
                    (Some(r), Some(s)) => (r, s),
                    _ => {
                        let now = self.clock.now();
                        if let Some(output) = self.outputs.first().cloned() {
                            let workspace_state = &self.workspace_state;
                            for w in self.windows.iter().filter(|w| {
                                !w.minimized && workspace_state.is_visible(&w.window_id)
                            }) {
                                send_frames_surface_tree(
                                    w.toplevel.wl_surface(),
                                    &output,
                                    now,
                                    Some(Duration::ZERO),
                                    |_, _| None,
                                );
                            }
                            for layer in &self.layer_surfaces {
                                send_frames_surface_tree(
                                    layer.surface.wl_surface(),
                                    &output,
                                    now,
                                    Some(Duration::ZERO),
                                    |_, _| None,
                                );
                            }
                        }
                        self.frame_dirty = false;
                        return;
                    }
                };

            // Paint order: bottom layers → xdg windows → top/overlay layers.
            use slopos_compositor::{plan_compose_order, ChromeLayer};
            let layer_z: Vec<u8> = self
                .layer_surfaces
                .iter()
                .map(|l| match l.layer {
                    Layer::Background => ChromeLayer::Background.z_priority(),
                    Layer::Bottom => ChromeLayer::Bottom.z_priority(),
                    Layer::Top => ChromeLayer::Top.z_priority(),
                    Layer::Overlay => ChromeLayer::Overlay.z_priority(),
                })
                .collect();
            let compose = plan_compose_order(&layer_z);
            let under: Vec<usize> = compose
                .layer_indices_bottom_first
                .iter()
                .copied()
                .filter(|&i| layer_z.get(i).copied().unwrap_or(0) <= 1)
                .collect();
            let over: Vec<usize> = compose
                .layer_indices_bottom_first
                .iter()
                .copied()
                .filter(|&i| layer_z.get(i).copied().unwrap_or(0) > 1)
                .collect();

            // Collect SHM render elements BEFORE binding the render target.
            // Per-window: real surface elements when available; placeholders only when empty.
            let mut surface_elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = Vec::new();
            // (rect, color) for windows with no committed buffer on the active workspace.
            let mut placeholders: Vec<(Rectangle<i32, Physical>, Color32F)> = Vec::new();

            for &i in &under {
                let layer = &self.layer_surfaces[i];
                let popup_elements = PopupManager::popups_for_surface(layer.surface.wl_surface())
                    .flat_map(|(popup, popup_offset)| {
                        let popup_loc = Self::popup_origin(layer.geo.loc, &popup, popup_offset);
                        render_elements_from_surface_tree(
                            renderer,
                            popup.wl_surface(),
                            Point::<i32, Physical>::from((popup_loc.x, popup_loc.y)),
                            1.0_f64,
                            1.0_f32,
                            Kind::Unspecified,
                        )
                    });
                surface_elements.extend(popup_elements);
                let loc = Point::<i32, Physical>::from((layer.geo.loc.x, layer.geo.loc.y));
                surface_elements.extend(render_elements_from_surface_tree(
                    renderer,
                    layer.surface.wl_surface(),
                    loc,
                    1.0_f64,
                    1.0_f32,
                    Kind::Unspecified,
                ));
            }
            // Workspace filter: hide surfaces not on the active virtual desktop.
            let workspace_state = &self.workspace_state;
            let visible_windows: Vec<&MappedWindow> = self
                .windows
                .iter()
                .filter(|w| !w.minimized && workspace_state.is_visible(&w.window_id))
                .collect();
            for (i, w) in visible_windows.iter().enumerate() {
                let loc = Point::<i32, Physical>::from((w.position.x, w.position.y));
                let popup_elements = PopupManager::popups_for_surface(w.toplevel.wl_surface())
                    .flat_map(|(popup, popup_offset)| {
                        let popup_loc = Self::popup_origin(w.position, &popup, popup_offset);
                        render_elements_from_surface_tree(
                            renderer,
                            popup.wl_surface(),
                            Point::<i32, Physical>::from((popup_loc.x, popup_loc.y)),
                            1.0_f64,
                            1.0_f32,
                            Kind::Unspecified,
                        )
                    });
                surface_elements.extend(popup_elements);
                let els = render_elements_from_surface_tree(
                    renderer,
                    w.toplevel.wl_surface(),
                    loc,
                    1.0_f64,
                    1.0_f32,
                    Kind::Unspecified,
                );
                match window_paint_source(!els.is_empty()) {
                    WindowPaintSource::SurfaceTree => {
                        surface_elements.extend(els);
                    }
                    WindowPaintSource::Placeholder => {
                        // No committed buffer: solid rect so the window still appears.
                        let color_idx = i % WIN_COLORS.len();
                        let (r, g, b) = WIN_COLORS[color_idx];
                        let rect = Rectangle::new(
                            Point::<i32, Physical>::from((w.position.x, w.position.y)),
                            Size::<i32, Physical>::from((w.size.w, w.size.h)),
                        );
                        placeholders.push((rect, Color32F::from([r, g, b, 1.0_f32])));
                    }
                }
            }
            for &i in &over {
                let layer = &self.layer_surfaces[i];
                let popup_elements = PopupManager::popups_for_surface(layer.surface.wl_surface())
                    .flat_map(|(popup, popup_offset)| {
                        let popup_loc = Self::popup_origin(layer.geo.loc, &popup, popup_offset);
                        render_elements_from_surface_tree(
                            renderer,
                            popup.wl_surface(),
                            Point::<i32, Physical>::from((popup_loc.x, popup_loc.y)),
                            1.0_f64,
                            1.0_f32,
                            Kind::Unspecified,
                        )
                    });
                surface_elements.extend(popup_elements);
                let loc = Point::<i32, Physical>::from((layer.geo.loc.x, layer.geo.loc.y));
                surface_elements.extend(render_elements_from_surface_tree(
                    renderer,
                    layer.surface.wl_surface(),
                    loc,
                    1.0_f64,
                    1.0_f32,
                    Kind::Unspecified,
                ));
            }

            // Client cursor surfaces are real Wayland surfaces and must be the
            // top-most render element.  If a client does not provide one, the
            // permanent software fallback below is used.
            let mut client_cursor_drawn = false;
            if let CursorImageStatus::Surface(surface) = &cursor_status {
                let hotspot = with_states(surface, |states| {
                    states
                        .data_map
                        .get::<CursorImageSurfaceData>()
                        .and_then(|attrs| attrs.lock().ok().map(|attrs| attrs.hotspot))
                        .unwrap_or_else(|| Point::from((0, 0)))
                });
                let cursor_loc = Point::<i32, Physical>::from((
                    cursor_position.x.round() as i32 - hotspot.x,
                    cursor_position.y.round() as i32 - hotspot.y,
                ));
                let cursor_elements = render_elements_from_surface_tree(
                    renderer,
                    surface,
                    cursor_loc,
                    1.0_f64,
                    1.0_f32,
                    Kind::Cursor,
                );
                client_cursor_drawn = !cursor_elements.is_empty();
                surface_elements.extend(cursor_elements);
            }

            // Acquire the next buffer from the X11 swapchain
            let (mut dmabuf, _age) = match x11_surface.buffer() {
                Ok(pair) => pair,
                Err(e) => {
                    eprintln!("[render] failed to get X11 buffer: {e}");
                    return;
                }
            };

            let output_size = self.output_size;

            // Bind the dmabuf as GL render target
            let mut target = match renderer.bind(&mut dmabuf) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[render] failed to bind dmabuf: {e}");
                    return;
                }
            };

            // Open a render frame
            let mut frame = match renderer.render(&mut target, output_size, Transform::Normal) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("[render] failed to start frame: {e}");
                    return;
                }
            };

            // Clear to retro gray: rgb(152, 152, 148) → linear ≈ (0.596, 0.596, 0.580)
            let retro_gray = Color32F::from([
                RETRO_GRAY.0 as f32 / 255.0,
                RETRO_GRAY.1 as f32 / 255.0,
                RETRO_GRAY.2 as f32 / 255.0,
                1.0_f32,
            ]);
            let full_screen = Rectangle::new(Point::<i32, Physical>::from((0, 0)), output_size);
            if let Err(e) = frame.clear(retro_gray, &[full_screen]) {
                eprintln!("[render] clear failed: {e}");
            }

            if !placeholders.is_empty() && self.placeholder_stats.note_frame_with_placeholders() {
                eprintln!(
                    "[slopos-compositor] present honesty: frame used solid placeholders \
                     (no committed SHM buffer for {} window(s)); session counter starts at {}",
                    placeholders.len(),
                    self.placeholder_stats.frames_with_placeholders
                );
            }
            for (rect, color) in &placeholders {
                if let Err(e) = frame.clear(*color, &[*rect]) {
                    eprintln!("[render] window placeholder clear failed: {e}");
                }
            }

            if !surface_elements.is_empty() {
                surface_elements.reverse();
                if let Err(e) = draw_render_elements::<GlesRenderer, _, _>(
                    &mut frame,
                    1.0_f64,
                    &surface_elements,
                    &[full_screen],
                ) {
                    eprintln!("[render] draw_render_elements failed: {e}");
                }
            }

            // Permanent compositor-owned software cursor fallback.  It remains
            // visible for Named cursors and whenever a client cursor surface has
            // not committed a buffer. Hidden is respected exactly.
            let fallback_cursor =
                !matches!(cursor_status, CursorImageStatus::Hidden) && !client_cursor_drawn;
            if fallback_cursor {
                let origin_x = cursor_position.x.round() as i32;
                let origin_y = cursor_position.y.round() as i32;
                let black = Color32F::from([0.0, 0.0, 0.0, 1.0]);
                let white = Color32F::from([1.0, 1.0, 1.0, 1.0]);
                // Classic high-contrast arrow, represented as horizontal runs.
                const OUTLINE: &[(i32, i32, i32)] = &[
                    (0, 0, 1),
                    (0, 1, 2),
                    (0, 2, 3),
                    (0, 3, 4),
                    (0, 4, 5),
                    (0, 5, 6),
                    (0, 6, 7),
                    (0, 7, 8),
                    (0, 8, 9),
                    (0, 9, 10),
                    (0, 10, 11),
                    (0, 11, 12),
                    (0, 12, 8),
                    (0, 13, 5),
                    (0, 14, 4),
                    (0, 15, 3),
                    (5, 12, 4),
                    (6, 13, 4),
                    (7, 14, 4),
                    (8, 15, 4),
                    (9, 16, 3),
                    (10, 17, 3),
                ];
                const FILL: &[(i32, i32, i32)] = &[
                    (1, 2, 1),
                    (1, 3, 2),
                    (1, 4, 3),
                    (1, 5, 4),
                    (1, 6, 5),
                    (1, 7, 6),
                    (1, 8, 7),
                    (1, 9, 8),
                    (1, 10, 9),
                    (1, 11, 6),
                    (1, 12, 3),
                ];
                for &(x, y, width) in OUTLINE {
                    let rect = Rectangle::new(
                        Point::<i32, Physical>::from((origin_x + x, origin_y + y)),
                        Size::<i32, Physical>::from((width, 1)),
                    );
                    let _ = frame.clear(black, &[rect]);
                }
                for &(x, y, width) in FILL {
                    let rect = Rectangle::new(
                        Point::<i32, Physical>::from((origin_x + x, origin_y + y)),
                        Size::<i32, Physical>::from((width, 1)),
                    );
                    let _ = frame.clear(white, &[rect]);
                }
            }

            // Finish the frame (flushes GL commands)
            if let Err(e) = frame.finish() {
                eprintln!("[render] frame finish failed: {e}");
            }

            // Present to the X11 window
            if let Err(e) = x11_surface.submit() {
                eprintln!("[render] submit failed: {e}");
            }

            // Release frame callbacks for everything we just presented. Clients
            // that throttle drawing on wl_surface.frame (winit/wgpu apps, and
            // therefore every SLOPOS-I app) render exactly one frame and then
            // wait forever without this.
            let now = self.clock.now();
            if let Some(output) = self.outputs.first().cloned() {
                // Throttle ZERO: always release, even for surfaces with no
                // primary scan-out output (nothing assigns one on this path).
                for w in self
                    .windows
                    .iter()
                    .filter(|w| !w.minimized && self.workspace_state.is_visible(&w.window_id))
                {
                    send_frames_surface_tree(
                        w.toplevel.wl_surface(),
                        &output,
                        now,
                        Some(Duration::ZERO),
                        |_, _| None,
                    );
                }
                for layer in &self.layer_surfaces {
                    send_frames_surface_tree(
                        layer.surface.wl_surface(),
                        &output,
                        now,
                        Some(Duration::ZERO),
                        |_, _| None,
                    );
                    for (popup, _) in PopupManager::popups_for_surface(layer.surface.wl_surface()) {
                        send_frames_surface_tree(
                            popup.wl_surface(),
                            &output,
                            now,
                            Some(Duration::ZERO),
                            |_, _| None,
                        );
                    }
                }
            }

            self.frame_scheduler.record_frame();
            self.frame_dirty = false;
        }
    }

    // -----------------------------------------------------------------------
    // BufferHandler (required by on_commit_buffer_handler)
    // -----------------------------------------------------------------------

    impl BufferHandler for SloposCompositor {
        fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
    }

    // -----------------------------------------------------------------------
    // CompositorHandler
    // -----------------------------------------------------------------------

    impl CompositorHandler for SloposCompositor {
        fn compositor_state(&mut self) -> &mut CompositorState {
            &mut self.compositor_state
        }

        fn client_compositor_state<'a>(
            &self,
            client: &'a smithay::reexports::wayland_server::Client,
        ) -> &'a CompositorClientState {
            &client.get_data::<ClientState>().unwrap().compositor_state
        }

        fn commit(&mut self, surface: &WlSurface) {
            on_commit_buffer_handler::<Self>(surface);
            self.popup_manager.commit(surface);
            // Update size of the matching window after the client commits.
            // ToplevelSurface::current_state gives us the server-side acknowledged size;
            // use that or fall back to DEFAULT_WIN. Size changes accumulate damage.
            let mut geometry_change: Option<(String, WindowGeometry, WindowGeometry)> = None;
            for w in self.windows.iter_mut() {
                if w.toplevel.wl_surface() == surface {
                    let old = w.geometry();
                    let st = w.toplevel.current_state();
                    let (sw, sh) = (
                        if st.size.map_or(0, |s| s.w) > 0 {
                            st.size.unwrap().w
                        } else {
                            DEFAULT_WINDOW_W
                        },
                        if st.size.map_or(0, |s| s.h) > 0 {
                            st.size.unwrap().h
                        } else {
                            DEFAULT_WINDOW_H
                        },
                    );
                    w.size = Size::from((sw, sh));
                    let new = w.geometry();
                    if old != new {
                        geometry_change = Some((w.window_id.clone(), old, new));
                    }
                    break;
                }
            }
            if let Some((id, old, new)) = geometry_change {
                self.note_window_geometry_change(&id, old, new);
            }

            // Apply the client-requested layer-shell anchors, margins, and
            // size to compositor-space placement. A layer surface is allowed
            // to extend outside its parent's notion of a window rectangle.
            let output = Size::<i32, Logical>::from((self.output_size.w, self.output_size.h));
            for layer in self.layer_surfaces.iter_mut() {
                if layer.surface.wl_surface() != surface {
                    continue;
                }
                let (requested, anchor, margins, exclusive_zone) =
                    layer_surface_request(&layer.surface);
                let geo = layer_geometry_for(
                    &layer.namespace,
                    layer.layer,
                    output,
                    requested,
                    anchor,
                    margins,
                );
                let current = layer.surface.current_state();
                if current.size != Some(geo.size) {
                    layer.surface.with_pending_state(|state| {
                        state.size = Some(geo.size);
                    });
                    layer.surface.send_configure();
                }
                layer.geo = geo;
                layer.exclusive_zone = exclusive_zone;
                break;
            }
            self.clamp_normal_windows_to_work_area();
            self.request_redraw();
        }
    }

    delegate_compositor!(SloposCompositor);

    // -----------------------------------------------------------------------
    // ShmHandler
    // -----------------------------------------------------------------------

    impl ShmHandler for SloposCompositor {
        fn shm_state(&self) -> &ShmState {
            &self.shm_state
        }
    }

    delegate_shm!(SloposCompositor);

    // -----------------------------------------------------------------------
    // SeatHandler
    // -----------------------------------------------------------------------

    impl SeatHandler for SloposCompositor {
        type KeyboardFocus = WlSurface;
        type PointerFocus = WlSurface;
        type TouchFocus = WlSurface;

        fn seat_state(&mut self) -> &mut SeatState<SloposCompositor> {
            &mut self.seat_state
        }

        fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
            self.cursor_status = image;
            self.request_redraw();
        }

        fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
            let client = focused.and_then(|s| s.client());
            set_data_device_focus(&self.display_handle, seat, client.clone());
            set_primary_focus(&self.display_handle, seat, client);
        }
    }

    delegate_seat!(SloposCompositor);

    // -----------------------------------------------------------------------
    // SelectionHandler / DataDeviceHandler (P1.1)
    // -----------------------------------------------------------------------

    /// Write mime payload to the client-provided fd on a background thread so the
    /// compositor event loop never blocks on a full pipe. Missing data → EOF only.
    fn write_selection_fd(mime_type: String, fd: OwnedFd, data: Option<Vec<u8>>) {
        if let Err(err) = std::thread::Builder::new()
            .name("selection-send".into())
            .spawn(move || {
                let mut file = std::fs::File::from(fd);
                if let Some(bytes) = data {
                    if let Err(err) = file.write_all(&bytes) {
                        tracing::debug!(
                            mime_type = %mime_type,
                            error = %err,
                            "selection send write failed"
                        );
                    }
                }
                // Dropping `file` closes the fd → EOF for the receiving client.
                let _ = file.flush();
            })
        {
            // On spawn failure the closure (and thus `fd`) was dropped → EOF.
            tracing::warn!(error = %err, "failed to spawn selection-send thread; fd closed");
        }
    }

    impl SelectionHandler for SloposCompositor {
        type SelectionUserData = MimePayload;

        fn new_selection(
            &mut self,
            ty: SelectionTarget,
            source: Option<SelectionSource>,
            _seat: Seat<Self>,
        ) {
            let mime_types = source.as_ref().map(|s| s.mime_types()).unwrap_or_default();
            match ty {
                SelectionTarget::Clipboard => {
                    self.clipboard_source = source;
                    if self.clipboard_source.is_none() {
                        self.clipboard_data.clear();
                    }
                    tracing::debug!(?mime_types, "clipboard selection updated");
                }
                SelectionTarget::Primary => {
                    self.primary_source = source;
                    if self.primary_source.is_none() {
                        self.primary_data.clear();
                    }
                    tracing::debug!(?mime_types, "primary selection updated");
                }
            }

            // Bridge Wayland → X11 selection when XWayland WM is live.
            if let Some(xwm) = self.xwm.as_mut() {
                let offered = if mime_types.is_empty() {
                    None
                } else {
                    Some(mime_types)
                };
                if let Err(err) = xwm.new_selection(ty, offered) {
                    tracing::debug!(?err, ?ty, "XWayland new_selection failed");
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
            // Prefer compositor-owned user_data (server-set selection via set_data_device_selection).
            let from_user = selection_bytes_for_mime_with_text_fallback(user_data, &mime_type)
                .map(|b| b.to_vec());
            let from_store = match ty {
                SelectionTarget::Clipboard => {
                    selection_bytes_for_mime_with_text_fallback(&self.clipboard_data, &mime_type)
                        .map(|b| b.to_vec())
                }
                SelectionTarget::Primary => {
                    selection_bytes_for_mime_with_text_fallback(&self.primary_data, &mime_type)
                        .map(|b| b.to_vec())
                }
            };
            let data = from_user.or(from_store);

            if data.is_none() {
                // Last resort: ask XWayland WM to fill the fd (X11 → Wayland).
                if let Some(xwm) = self.xwm.as_mut() {
                    if let Err(err) =
                        xwm.send_selection(ty, mime_type.clone(), fd, self.loop_handle.clone())
                    {
                        tracing::debug!(?err, "XWayland send_selection failed; EOF");
                    }
                    return;
                }
                tracing::debug!(
                    %mime_type,
                    ?ty,
                    "send_selection: no mime data; closing fd (EOF)"
                );
                drop(fd);
                return;
            }

            tracing::debug!(
                %mime_type,
                ?ty,
                bytes = data.as_ref().map(|d| d.len()).unwrap_or(0),
                "send_selection writing mime data"
            );
            write_selection_fd(mime_type, fd, data);
        }
    }

    impl DataDeviceHandler for SloposCompositor {
        fn data_device_state(&self) -> &DataDeviceState {
            &self.data_device_state
        }
    }

    impl ClientDndGrabHandler for SloposCompositor {
        fn started(
            &mut self,
            _source: Option<WlDataSource>,
            icon: Option<WlSurface>,
            _seat: Seat<Self>,
        ) {
            // Client-initiated DnD: smithay routes offer.receive to the client's
            // WlDataSource directly. We only track the optional drag icon here.
            self.dnd_icon = icon;
            tracing::debug!("client DnD started");
        }

        fn dropped(&mut self, _target: Option<WlSurface>, _validated: bool, _seat: Seat<Self>) {
            self.dnd_icon = None;
            tracing::debug!("client DnD dropped");
        }
    }

    impl ServerDndGrabHandler for SloposCompositor {
        fn send(&mut self, mime_type: String, fd: OwnedFd, _seat: Seat<Self>) {
            // Server-initiated DnD: write tracked mime payloads, or EOF if none.
            let data =
                selection_bytes_for_mime_with_text_fallback(&self.server_dnd_data, &mime_type)
                    .map(|b| b.to_vec());
            if data.is_none() {
                tracing::debug!(
                    %mime_type,
                    "ServerDndGrabHandler::send: no tracked source data; EOF"
                );
                drop(fd);
                return;
            }
            tracing::debug!(
                %mime_type,
                bytes = data.as_ref().map(|d| d.len()).unwrap_or(0),
                "ServerDndGrabHandler::send writing mime data"
            );
            write_selection_fd(mime_type, fd, data);
        }

        fn cancelled(&mut self, _seat: Seat<Self>) {
            self.server_dnd_data.clear();
        }

        fn finished(&mut self, _seat: Seat<Self>) {
            self.server_dnd_data.clear();
        }
    }

    smithay::delegate_data_device!(SloposCompositor);

    impl PrimarySelectionHandler for SloposCompositor {
        fn primary_selection_state(&self) -> &PrimarySelectionState {
            &self.primary_selection_state
        }
    }

    delegate_primary_selection!(SloposCompositor);

    // -----------------------------------------------------------------------
    // XdgShellHandler
    // -----------------------------------------------------------------------

    impl XdgShellHandler for SloposCompositor {
        fn xdg_shell_state(&mut self) -> &mut XdgShellState {
            &mut self.xdg_shell_state
        }

        fn new_toplevel(&mut self, surface: ToplevelSurface) {
            // Cascade new windows
            let offset = self.next_window_offset;
            self.next_window_offset = next_cascade_offset(offset);
            let (x, y) = cascade_position(offset);
            let requested_geometry = WindowGeometry::new(x, y, DEFAULT_WINDOW_W, DEFAULT_WINDOW_H);
            let output_area = output_index_for_geometry(&self.laid_out_outputs, requested_geometry)
                .and_then(|index| self.laid_out_outputs.get(index))
                .map(output_geometry)
                .unwrap_or_else(|| self.canvas_area());
            let geometry = clamp_window_to_work_area(
                requested_geometry,
                self.work_area_for_output(output_area),
            );
            surface.with_pending_state(|state| {
                // The compositor owns the logical work area, including scale
                // and layer-shell exclusive zones. Do not let a default
                // client request cover the Dock on a small logical output.
                state.size = Some(Size::from((geometry.width, geometry.height)));
                state.states.set(xdg_toplevel::State::Activated);
            });
            surface.send_configure();
            let position = Point::from((geometry.x, geometry.y));

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
            let foreign = self
                .foreign_toplevel_list
                .new_toplevel::<SloposCompositor>(&title, &app_id);

            eprintln!(
                "[slopos-compositor] surface mapped at ({},{}) title={title}",
                position.x, position.y
            );

            let window_id = foreign.identifier();
            // New maps land on the active virtual workspace (not an untracked id).
            if !assign_new_window_to_active(&mut self.workspace_state, window_id.clone()) {
                // Active id is always valid after WorkspaceState::new / activate.
                let _ = self
                    .workspace_state
                    .assign_window(window_id.clone(), WorkspaceId::FIRST);
            }
            eprintln!(
                "[slopos-compositor] assign window_id={window_id} {}",
                self.workspace_state.summary_line()
            );

            self.windows.push(MappedWindow {
                toplevel: surface,
                foreign,
                window_id,
                app_id,
                position,
                size: Size::from((geometry.width, geometry.height)),
                presentation_state: WindowPresentationState::Normal,
                restore_state: None,
                minimized: false,
            });
            self.request_full_redraw();

            // Focus the new window
            let idx = self.windows.len() - 1;
            self.focus_window(idx);
        }

        fn move_request(
            &mut self,
            surface: ToplevelSurface,
            seat: wl_seat::WlSeat,
            serial: Serial,
        ) {
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
            self.begin_interactive_grab(
                &surface,
                InteractiveGrabKind::Resize(edges),
                &seat,
                serial,
            );
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
            _output: Option<wl_output::WlOutput>,
        ) {
            self.set_window_presentation_state(&surface, WindowPresentationState::Fullscreen);
        }

        fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
            self.set_window_presentation_state(&surface, WindowPresentationState::Normal);
        }

        fn minimize_request(&mut self, surface: ToplevelSurface) {
            let Some(idx) = self
                .windows
                .iter()
                .position(|window| window.toplevel.wl_surface() == surface.wl_surface())
            else {
                return;
            };
            let window_id = self.windows[idx].window_id.clone();
            self.set_window_presentation_state(&surface, WindowPresentationState::Minimized);
            self.windows[idx].minimized = true;
            self.last_minimized_window_id = Some(window_id);
            self.request_full_redraw();
            self.apply_focus_after_workspace_switch();
        }

        fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
            let destroyed_surface = surface.wl_surface();
            let destroys_grab = self.interactive_grab.as_ref().is_some_and(|grab| {
                self.windows.iter().any(|window| {
                    window.window_id == grab.window_id
                        && window.toplevel.wl_surface() == destroyed_surface
                })
            });
            if destroys_grab {
                self.cancel_interactive_grab();
            } else if self.last_pointer_press.as_ref().is_some_and(|press| {
                self.windows.iter().any(|window| {
                    window.window_id == press.window_id
                        && window.toplevel.wl_surface() == destroyed_surface
                })
            }) {
                self.last_pointer_press = None;
                self.left_button_down = false;
            }
            if let Some(idx) = self
                .windows
                .iter()
                .position(|w| w.toplevel.wl_surface() == destroyed_surface)
            {
                let win = self.windows.remove(idx);
                self.workspace_state.remove_window(&win.window_id);
                if self.last_minimized_window_id.as_deref() == Some(win.window_id.as_str()) {
                    self.last_minimized_window_id = None;
                }
                win.foreign.send_closed();
            }
            // Focus topmost **visible** remaining window (not hidden by workspace).
            self.apply_focus_after_workspace_switch();
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
                    if let Err(err) = slopos_compositor::publish_active_toplevel(Some(&app_id)) {
                        tracing::debug!(
                            error = %err,
                            app_id = %app_id,
                            "could not refresh active application"
                        );
                    }
                }
            }
        }

        fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
            let popup = PopupKind::from(surface.clone());
            if let Err(err) = self.popup_manager.track_popup(popup.clone()) {
                tracing::debug!(?err, "failed to track xdg popup");
                return;
            }
            let root_ready = find_popup_root_surface(&popup).is_ok();
            let geometry = self.constrained_popup_geometry(&popup, positioner);
            surface.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });
            if root_ready {
                if let Err(err) = surface.send_configure() {
                    tracing::debug!(?err, "failed to configure xdg popup");
                }
            } else {
                tracing::debug!(
                    "deferring parentless popup configure until layer-shell association"
                );
            }
            self.request_redraw();
        }

        fn grab(&mut self, surface: PopupSurface, seat: wl_seat::WlSeat, serial: WlSerial) {
            self.begin_popup_grab(surface, seat, serial);
        }

        fn reposition_request(
            &mut self,
            surface: PopupSurface,
            positioner: PositionerState,
            token: u32,
        ) {
            let popup = PopupKind::from(surface.clone());
            let geometry = self.constrained_popup_geometry(&popup, positioner);
            surface.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });
            let _serial = surface.send_repositioned(token);
            self.request_redraw();
        }
    }

    delegate_xdg_shell!(SloposCompositor);

    // -----------------------------------------------------------------------
    // Layer shell (menu bar / dock / notifications chrome)
    // -----------------------------------------------------------------------

    impl WlrLayerShellHandler for SloposCompositor {
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
                "[slopos-compositor] layer-shell surface namespace={namespace} layer={layer:?}"
            );
            let output = Size::<i32, Logical>::from((self.output_size.w, self.output_size.h));
            let (requested, anchor, margins, exclusive_zone) = layer_surface_request(&surface);
            let geo = layer_geometry_for(&namespace, layer, output, requested, anchor, margins);
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
            self.request_redraw();
        }

        fn new_popup(&mut self, _parent: LayerSurface, surface: PopupSurface) {
            let popup = PopupKind::from(surface.clone());
            let positioner = surface.with_pending_state(|state| state.positioner);
            let geometry = self.constrained_popup_geometry(&popup, positioner);
            surface.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });
            if let Err(err) = surface.send_configure() {
                tracing::debug!(?err, "failed to configure layer-shell popup");
            }
            self.request_redraw();
        }

        fn layer_destroyed(&mut self, surface: LayerSurface) {
            self.layer_surfaces
                .retain(|l| l.surface.wl_surface() != surface.wl_surface());
        }
    }

    delegate_layer_shell!(SloposCompositor);

    // -----------------------------------------------------------------------
    // Foreign toplevel list (task list / overview / Force Quit)
    // -----------------------------------------------------------------------

    impl ForeignToplevelListHandler for SloposCompositor {
        fn foreign_toplevel_list_state(&mut self) -> &mut ForeignToplevelListState {
            &mut self.foreign_toplevel_list
        }
    }

    delegate_foreign_toplevel_list!(SloposCompositor);

    // -----------------------------------------------------------------------
    // xdg-decoration (server-side preference for external apps)
    // -----------------------------------------------------------------------

    impl XdgDecorationHandler for SloposCompositor {
        fn new_decoration(&mut self, toplevel: ToplevelSurface) {
            use slopos_compositor::{decoration_preference_for_app_id, DecorationPreference};
            use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
            let app_id = with_states(toplevel.wl_surface(), |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .and_then(|d| d.lock().unwrap().app_id.clone())
                    .unwrap_or_default()
            });
            let mode = match decoration_preference_for_app_id(&app_id) {
                DecorationPreference::ServerSide => Mode::ServerSide,
                DecorationPreference::ClientSide => Mode::ClientSide,
            };
            toplevel.with_pending_state(|state| {
                state.decoration_mode = Some(mode);
            });
            toplevel.send_configure();
        }

        fn request_mode(
            &mut self,
            toplevel: ToplevelSurface,
            mode: smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
        ) {
            toplevel.with_pending_state(|state| {
                state.decoration_mode = Some(mode);
            });
            toplevel.send_configure();
        }

        fn unset_mode(&mut self, toplevel: ToplevelSurface) {
            use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
            // Prefer server-side for unknown clients when unset.
            toplevel.with_pending_state(|state| {
                state.decoration_mode = Some(Mode::ServerSide);
            });
            toplevel.send_configure();
        }
    }

    smithay::delegate_xdg_decoration!(SloposCompositor);

    // text-input-v3 manager (global advertised when policy enables it)
    smithay::delegate_text_input_manager!(SloposCompositor);

    // input-method-v2 (paired with text-input for IME clients)
    impl smithay::wayland::input_method::InputMethodHandler for SloposCompositor {
        fn new_popup(&mut self, surface: smithay::wayland::input_method::PopupSurface) {
            tracing::debug!("input-method popup created");
            self.im_popups.push(surface);
        }

        fn dismiss_popup(&mut self, surface: smithay::wayland::input_method::PopupSurface) {
            self.im_popups.retain(|p| p != &surface);
        }

        fn popup_repositioned(&mut self, _surface: smithay::wayland::input_method::PopupSurface) {}

        fn parent_geometry(&self, parent: &WlSurface) -> smithay::utils::Rectangle<i32, Logical> {
            // Use focused window geometry when the parent matches a toplevel.
            for w in &self.windows {
                if w.toplevel.wl_surface() == parent {
                    return smithay::utils::Rectangle::new(w.position, w.size);
                }
            }
            smithay::utils::Rectangle::default()
        }
    }

    smithay::delegate_input_method_manager!(SloposCompositor);

    // -----------------------------------------------------------------------
    // OutputHandler (required by delegate_output!)
    // -----------------------------------------------------------------------

    impl OutputHandler for SloposCompositor {}

    delegate_output!(SloposCompositor);

    // -----------------------------------------------------------------------
    // XWayland (P1.3) — best-effort under nested X11
    //
    // Nested under Xvfb/X11 the compositor already owns DISPLAY. XWayland is
    // still spawned (own display number) so the code path exists and X clients
    // can attach when the binary + runtime allow it. Full rootless WM mapping
    // of X11 windows into the GL scene is incomplete under nested X11; handlers
    // accept maps and track surfaces so the path is live for native Linux.
    // -----------------------------------------------------------------------

    impl XWaylandShellHandler for SloposCompositor {
        fn xwayland_shell_state(&mut self) -> &mut XWaylandShellState {
            &mut self.xwayland_shell_state
        }

        fn surface_associated(
            &mut self,
            _xwm: XwmId,
            wl_surface: WlSurface,
            surface: X11WmSurface,
        ) {
            tracing::info!(
                title = %surface.title(),
                "XWayland surface associated with wl_surface"
            );
            self.x11_surface_associations
                .insert(surface.window_id(), wl_surface);
            if !self
                .x11_surfaces
                .iter()
                .any(|s| s.window_id() == surface.window_id())
            {
                self.x11_surfaces.push(surface);
            }
        }
    }

    delegate_xwayland_shell!(SloposCompositor);

    impl XwmHandler for SloposCompositor {
        fn xwm_state(&mut self, _xwm: XwmId) -> &mut X11Wm {
            self.xwm.as_mut().expect("X11Wm missing for XwmHandler")
        }

        fn new_window(&mut self, _xwm: XwmId, window: X11WmSurface) {
            tracing::debug!(title = %window.title(), "X11 new_window");
            self.x11_surfaces.push(window);
        }

        fn new_override_redirect_window(&mut self, _xwm: XwmId, window: X11WmSurface) {
            tracing::debug!(title = %window.title(), "X11 override-redirect window");
            self.x11_surfaces.push(window);
        }

        fn map_window_request(&mut self, _xwm: XwmId, window: X11WmSurface) {
            // Grant map so X clients don't hang waiting for the WM.
            if let Err(err) = window.set_mapped(true) {
                tracing::debug!(?err, "X11 set_mapped failed");
            }
            let geo = window.geometry();
            if let Err(err) = window.configure(Some(geo)) {
                tracing::debug!(?err, "X11 configure failed");
            }
            tracing::info!(title = %window.title(), "X11 map_window_request granted");
        }

        fn mapped_override_redirect_window(&mut self, _xwm: XwmId, window: X11WmSurface) {
            tracing::debug!(title = %window.title(), "X11 override-redirect mapped");
        }

        fn unmapped_window(&mut self, _xwm: XwmId, window: X11WmSurface) {
            self.x11_surfaces
                .retain(|s| s.window_id() != window.window_id());
            self.x11_surface_associations.remove(&window.window_id());
        }

        fn destroyed_window(&mut self, _xwm: XwmId, window: X11WmSurface) {
            self.x11_surfaces
                .retain(|s| s.window_id() != window.window_id());
            self.x11_surface_associations.remove(&window.window_id());
        }

        fn configure_request(
            &mut self,
            _xwm: XwmId,
            window: X11WmSurface,
            x: Option<i32>,
            y: Option<i32>,
            w: Option<u32>,
            h: Option<u32>,
            _reorder: Option<Reorder>,
        ) {
            let mut geo = window.geometry();
            if let Some(x) = x {
                geo.loc.x = x;
            }
            if let Some(y) = y {
                geo.loc.y = y;
            }
            if let Some(w) = w {
                geo.size.w = w as i32;
            }
            if let Some(h) = h {
                geo.size.h = h as i32;
            }
            let _ = window.configure(Some(geo));
        }

        fn configure_notify(
            &mut self,
            _xwm: XwmId,
            _window: X11WmSurface,
            _geometry: Rectangle<i32, Logical>,
            _above: Option<X11Window>,
        ) {
        }

        fn resize_request(
            &mut self,
            _xwm: XwmId,
            window: X11WmSurface,
            button: u32,
            resize_edge: ResizeEdge,
        ) {
            let edges = x11_resize_edge_to_resize_edges(resize_edge);
            self.begin_x11_interactive_grab(&window, InteractiveGrabKind::Resize(edges), button);
        }

        fn move_request(&mut self, _xwm: XwmId, window: X11WmSurface, button: u32) {
            self.begin_x11_interactive_grab(&window, InteractiveGrabKind::Move, button);
        }

        fn allow_selection_access(&mut self, _xwm: XwmId, _selection: SelectionTarget) -> bool {
            // Allow X clients to read the Wayland selection store.
            true
        }

        fn send_selection(
            &mut self,
            _xwm: XwmId,
            selection: SelectionTarget,
            mime_type: String,
            fd: OwnedFd,
        ) {
            let store = match selection {
                SelectionTarget::Clipboard => &self.clipboard_data,
                SelectionTarget::Primary => &self.primary_data,
            };
            let data =
                selection_bytes_for_mime_with_text_fallback(store, &mime_type).map(|b| b.to_vec());
            write_selection_fd(mime_type, fd, data);
        }

        fn new_selection(
            &mut self,
            _xwm: XwmId,
            selection: SelectionTarget,
            mime_types: Vec<String>,
        ) {
            tracing::debug!(?selection, ?mime_types, "X11 client set selection");
        }

        fn cleared_selection(&mut self, _xwm: XwmId, selection: SelectionTarget) {
            match selection {
                SelectionTarget::Clipboard => self.clipboard_data.clear(),
                SelectionTarget::Primary => self.primary_data.clear(),
            }
        }

        fn disconnected(&mut self, _xwm: XwmId) {
            tracing::warn!("XWayland WM disconnected");
            self.xwm = None;
            self.xdisplay = None;
            self.x11_surfaces.clear();
            self.x11_surface_associations.clear();
        }
    }

    // -----------------------------------------------------------------------
    // Input dispatch helpers (called from the X11 event handler)
    // -----------------------------------------------------------------------

    fn handle_keyboard_event<E>(state: &mut SloposCompositor, ev: &E)
    where
        E: KeyboardKeyEvent<X11Input>,
    {
        use smithay::backend::input::KeyState;
        use smithay::input::keyboard::Keysym;

        let serial = state.next_serial();
        let time = ev.time_msec();
        let keycode = ev.key_code();
        let key_state = ev.state();

        if let Some(kb) = state.seat.get_keyboard() {
            kb.input::<(), _>(
                state,
                keycode,
                key_state,
                serial,
                time,
                |data, mods, keysym| {
                    // Super+Right / Super+Left: cycle virtual workspaces (live filter).
                    if key_state == KeyState::Pressed && mods.logo {
                        let sym = keysym.modified_sym();
                        if sym == Keysym::o || sym == Keysym::O {
                            slopos_compositor::client_spawn::spawn_client(
                                &data.wayland_socket_name,
                                "finder",
                            );
                            return FilterResult::Intercept(());
                        }
                        if sym == Keysym::l || sym == Keysym::L {
                            slopos_compositor::client_spawn::spawn_client(
                                &data.wayland_socket_name,
                                "slopos-lock",
                            );
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
                        // Super+1..8 → activate workspace 0..7
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
    }

    fn handle_pointer_motion<E>(state: &mut SloposCompositor, ev: &E)
    where
        E: PointerMotionAbsoluteEvent<X11Input>,
    {
        let logical = Size::<i32, Logical>::from((state.output_size.w, state.output_size.h));
        let pos = ev.position_transformed(logical);
        state.pointer_pos = pos;
        state.request_redraw();

        // Hit-test layer chrome, popup trees, then ordinary toplevels.
        let focus = state.surface_under(pos);

        let serial = state.next_serial();
        let time = ev.time_msec();

        if let Some(ptr) = state.seat.get_pointer() {
            ptr.motion(
                state,
                focus,
                &MotionEvent {
                    location: pos,
                    serial,
                    time,
                },
            );
            ptr.frame(state);
        }
    }

    fn handle_pointer_button<E>(state: &mut SloposCompositor, ev: &E)
    where
        E: PointerButtonEvent<X11Input>,
    {
        let serial = state.next_serial();
        let time = ev.time_msec();
        let button = ev.button_code();
        let btn_state = ev.state();

        let primary_button = button == 0x110 || button == 1;
        if primary_button {
            state.left_button_down = btn_state == ButtonState::Pressed;
        }

        // On press: hit-test surfaces and focus the topmost one.
        if btn_state == ButtonState::Pressed {
            let pos = state.pointer_pos;
            let hit = state.surface_under(pos);
            let mapped_window_index = hit
                .as_ref()
                .and_then(|(surface, _)| state.mapped_window_index_for_surface(surface));
            if primary_button {
                state.last_pointer_press = mapped_window_index.map(|index| PointerPress {
                    serial,
                    window_id: state.windows[index].window_id.clone(),
                });
            }
            match hit {
                Some((surface, _)) => match mapped_window_index {
                    Some(idx) => {
                        state.focus_window(idx);
                    }
                    None => {
                        state.focus_surface(Some(surface));
                    }
                },
                None => state.focus_surface(None),
            }
            // Retarget pointer focus so the button is delivered to the
            // surface under the current click coordinates.
            state.forward_pointer_motion(time);
        }

        if let Some(ptr) = state.seat.get_pointer() {
            ptr.button(
                state,
                &ButtonEvent {
                    serial,
                    time,
                    button,
                    state: btn_state,
                },
            );
            ptr.frame(state);
        }
        if primary_button && btn_state == ButtonState::Released {
            state.finish_interactive_grab();
        }
        state.request_redraw();
    }

    /// Create one or more wl_output globals at the given logical origins.
    ///
    /// `laid_out` positions come from shell `SLOPOS_OUTPUTS_LAYOUT` or from
    /// `SLOPOS_OUTPUTS` + layout mode. `names` are connector names when known
    /// (else synthetic `X11-N`). `scale` is advertised on each output (HiDPI);
    /// mode sizes stay logical width×height; scale is the wl_output scale factor.
    ///
    /// Nested path only places logical outputs — no DRM modeset for external
    /// connectors in this pass.
    fn create_outputs(
        display_handle: &DisplayHandle,
        laid_out: &[LaidOutOutput],
        names: &[String],
        refresh_mhz: i32,
        scale: OutputScale,
    ) -> (Vec<Output>, Size<i32, Physical>) {
        let scale_i32 = scale.as_f64().round().max(1.0) as i32;
        let total = total_output_size(laid_out);
        // Physical canvas size for the nested X11 window (logical × scale).
        let total_phys = apply_scale_to_output_config(total, scale);
        let mut outputs = Vec::with_capacity(laid_out.len());

        for (i, o) in laid_out.iter().enumerate() {
            let name = names
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("X11-{}", i + 1));
            let output = Output::new(
                name.clone(),
                PhysicalProperties {
                    size: (0, 0).into(),
                    subpixel: Subpixel::Unknown,
                    make: "SLOPOS-I".into(),
                    model: format!("X11 Output {}", i + 1),
                },
            );
            let mode = Mode {
                size: (o.config.width, o.config.height).into(),
                refresh: refresh_mhz,
            };
            output.change_current_state(
                Some(mode),
                Some(Transform::Normal),
                Some(Scale::Integer(scale_i32)),
                Some((o.x, o.y).into()),
            );
            output.set_preferred(mode);
            output.create_global::<SloposCompositor>(display_handle);
            tracing::info!(
                "wl_output {} ({}) {}x{} at ({},{}) refresh={} mHz {}",
                i + 1,
                name,
                o.config.width,
                o.config.height,
                o.x,
                o.y,
                refresh_mhz,
                output_scale_summary(scale)
            );
            outputs.push(output);
        }

        let output_size = Size::<i32, Physical>::from((total_phys.width, total_phys.height));
        (outputs, output_size)
    }

    /// Best-effort XWayland startup. Returns false when the binary is missing or spawn fails.
    ///
    /// Under nested X11 this is still useful: XWayland gets its own display number and
    /// clients can set DISPLAY=:N. Full scene integration of X11 surfaces remains limited
    /// because the compositor itself is an X11 client of the host server.
    fn try_start_xwayland(state: &mut SloposCompositor) {
        // Allow opt-out: SLOPOS_XWAYLAND=0
        if std::env::var("SLOPOS_XWAYLAND")
            .map(|v| matches!(v.as_str(), "0" | "false" | "off" | "no"))
            .unwrap_or(false)
        {
            tracing::info!("XWayland disabled via SLOPOS_XWAYLAND");
            return;
        }

        use std::process::Stdio;

        match XWayland::spawn(
            &state.display_handle,
            None,
            std::iter::empty::<(String, String)>(),
            true,
            Stdio::null(),
            Stdio::null(),
            |_| (),
        ) {
            Ok((xwayland, client)) => {
                let display_number_hint = xwayland.display_number();
                tracing::info!(
                    "XWayland spawning (will claim DISPLAY=:{} when ready)",
                    display_number_hint
                );
                let ret = state.loop_handle.insert_source(xwayland, move |event, _, data| {
                    match event {
                        XWaylandEvent::Ready {
                            x11_socket,
                            display_number,
                        } => {
                            tracing::info!(
                                "XWayland ready on DISPLAY=:{} — starting X11 WM",
                                display_number
                            );
                            match X11Wm::start_wm(data.loop_handle.clone(), x11_socket, client.clone())
                            {
                                Ok(wm) => {
                                    data.xwm = Some(wm);
                                    data.xdisplay = Some(display_number);
                                    // Expose DISPLAY for child processes launched later.
                                    std::env::set_var("SLOPOS_XWAYLAND_DISPLAY", format!(":{display_number}"));
                                    eprintln!(
                                        "[slopos-compositor] XWayland ready DISPLAY=:{}",
                                        display_number
                                    );
                                }
                                Err(err) => {
                                    tracing::warn!(?err, "Failed to start X11Wm for XWayland");
                                }
                            }
                        }
                        XWaylandEvent::Error => {
                            tracing::warn!(
                                "XWayland failed to start (binary missing, nested X11 conflict, or crash)"
                            );
                        }
                    }
                });
                if let Err(err) = ret {
                    tracing::warn!(?err, "Failed to insert XWayland event source");
                }
            }
            Err(err) => {
                // Nested X11 or missing Xwayland package — document, don't abort.
                tracing::warn!(
                    error = %err,
                    "XWayland spawn failed (install `xwayland` package for X11 client support; nested X11 may still be limited)"
                );
                eprintln!(
                    "[slopos-compositor] XWayland unavailable: {err} (continuing without it)"
                );
            }
        }
    }

    #[cfg(test)]
    pub fn parse_bool_env(key: &str) -> bool {
        match std::env::var(key) {
            Ok(v) => matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            ),
            Err(_) => false,
        }
    }

    pub(crate) fn default_backend_for_host(
        display: Option<&str>,
        _wayland_display: Option<&str>,
    ) -> &'static str {
        // The nested implementation below is Smithay's X11 backend. A host
        // Wayland socket is not a valid transport for it.
        if display.is_some_and(|value| !value.is_empty()) {
            "nested"
        } else {
            "drm"
        }
    }

    pub(crate) fn validate_nested_transport(
        requested_backend: &str,
        display: Option<&str>,
    ) -> Result<(), String> {
        if matches!(requested_backend, "nested" | "x11" | "winit")
            && !display.is_some_and(|value| !value.is_empty())
        {
            return Err(
                "nested backend requires a non-empty DISPLAY (nested transport is X11-only); use --backend drm or --backend headless"
                    .to_owned(),
            );
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub fn run() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();

        let args: Vec<String> = std::env::args().collect();
        let mut backend_arg: Option<String> = None;
        let mut idx = 1;
        while idx < args.len() {
            if args[idx] == "--backend" && idx + 1 < args.len() {
                backend_arg = Some(args[idx + 1].clone());
                idx += 2;
            } else if args[idx].starts_with("--backend=") {
                backend_arg = Some(args[idx].trim_start_matches("--backend=").to_string());
                idx += 1;
            } else {
                idx += 1;
            }
        }

        // Backend selection is explicit and fail-fast. The production session
        // never substitutes labwc/sway or silently changes the requested backend.
        let requested_backend = backend_arg.unwrap_or_else(|| {
            default_backend_for_host(
                std::env::var("DISPLAY").ok().as_deref(),
                std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
            )
            .to_owned()
        });

        if let Err(error) =
            validate_nested_transport(&requested_backend, std::env::var("DISPLAY").ok().as_deref())
        {
            anyhow::bail!(error);
        }

        if requested_backend == "drm" {
            eprintln!("[slopos-compositor] backend: SessionDrm (explicit)");
            return slopos_compositor::session_drm::run_drm_session();
        }

        let headless = requested_backend == "headless";
        if !matches!(
            requested_backend.as_str(),
            "nested" | "x11" | "winit" | "headless"
        ) {
            anyhow::bail!(
                "unsupported backend '{requested_backend}'; expected drm, nested, x11, winit, or headless"
            );
        }
        if requested_backend == "winit" {
            tracing::warn!(
                "--backend winit currently uses Smithay's nested X11 transport; use --backend nested"
            );
        }
        let backend_kind = if headless {
            CompositorBackendKind::Headless
        } else {
            CompositorBackendKind::NestedX11
        };
        eprintln!(
            "[slopos-compositor] backend: {} (explicit)",
            if headless { "Headless" } else { "NestedX11" }
        );

        // ---- Display policy (HDR / VRR / refresh / color) ----
        let display_policy = DisplayPolicy::resolve();
        let mut hdr_caps = HdrCapabilities::detect();
        let color_applied =
            hdr_caps.apply_request(display_policy.hdr_requested, display_policy.color_space);
        let effective_refresh = display_policy.effective_refresh_rate();
        let frame_scheduler = FrameScheduler::new(effective_refresh);
        let refresh_mhz: i32 = match effective_refresh {
            RefreshRate::Adaptive => 60_000, // advertise 60; pacing is free-run
            r => (r.as_hz() as i32) * 1000,
        };

        let policy_line = display_policy.summary_line(hdr_caps.hdr_supported);
        tracing::info!("display policy applied: {policy_line} color_applied={color_applied}");
        eprintln!("[slopos-compositor] display policy: {policy_line}");
        if display_policy.hdr_requested && !hdr_caps.hdr_supported {
            tracing::info!(
                "HDR requested but not supported under nested X11/no-KMS probe; staying SDR ({})",
                hdr_caps.current_color_space.as_str()
            );
        }

        let mut event_loop: EventLoop<SloposCompositor> = EventLoop::try_new()?;
        let display: Display<SloposCompositor> = Display::new()?;
        let display_handle = display.handle();
        let loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();

        // Protocol states
        let compositor_state = CompositorState::new::<SloposCompositor>(&display_handle);
        let shm_state = ShmState::new::<SloposCompositor>(&display_handle, vec![]);
        let mut seat_state = SeatState::new();
        let xdg_shell_state = XdgShellState::new::<SloposCompositor>(&display_handle);
        let data_device_state = DataDeviceState::new::<SloposCompositor>(&display_handle);
        let primary_selection_state =
            PrimarySelectionState::new::<SloposCompositor>(&display_handle);
        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<SloposCompositor>(&display_handle);
        let xwayland_shell_state = XWaylandShellState::new::<SloposCompositor>(&display_handle);
        let layer_shell_state = WlrLayerShellState::new::<SloposCompositor>(&display_handle);
        let foreign_toplevel_list =
            ForeignToplevelListState::new::<SloposCompositor>(&display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<SloposCompositor>(&display_handle);

        // text-input-v3 global when SLOPOS_TEXT_INPUT requests it (default: on)
        // Default "full" advertises text-input-v3 + input-method-v2 for IME clients.
        // Set SLOPOS_TEXT_INPUT=0 to disable, or v3 for text-input only.
        let text_input_cap = text_input_capability_from_env(
            std::env::var("SLOPOS_TEXT_INPUT")
                .ok()
                .as_deref()
                .or(Some("full")),
        );
        let text_input_state = if matches!(
            text_input_cap,
            TextInputCapability::TextInputV3 | TextInputCapability::InputMethodAndTextInput
        ) {
            eprintln!(
                "[slopos-compositor] {}",
                text_input_capability_summary(text_input_cap)
            );
            Some(smithay::wayland::text_input::TextInputManagerState::new::<
                SloposCompositor,
            >(&display_handle))
        } else {
            eprintln!(
                "[slopos-compositor] {}",
                text_input_capability_summary(TextInputCapability::None)
            );
            None
        };
        let input_method_state =
            if matches!(text_input_cap, TextInputCapability::InputMethodAndTextInput) {
                eprintln!("[slopos-compositor] input_method=zwp_input_method_v2");
                Some(
                    smithay::wayland::input_method::InputMethodManagerState::new::<
                        SloposCompositor,
                        _,
                    >(&display_handle, |_client| true),
                )
            } else {
                None
            };

        // Seat: keyboard + pointer
        let mut seat: Seat<SloposCompositor> = seat_state.new_wl_seat(&display_handle, "seat0");
        seat.add_keyboard(XkbConfig::default(), 200, 25)?;
        seat.add_pointer();

        // ---- Outputs (P1.2 multi-output) + HiDPI scale ----
        let output_scale = detect_output_scale_from_env().unwrap_or(OutputScale::IDENTITY);
        eprintln!(
            "[slopos-compositor] {}",
            session_mode_note(backend_kind, output_scale)
        );
        // Prefer SLOPOS_OUTPUTS_LAYOUT (shell display arrange), else
        // SLOPOS_OUTPUTS + layout mode, else WIDTH/HEIGHT defaults.
        let resolved = resolve_laid_out_outputs_from_env();
        eprintln!("[slopos-compositor] {}", resolved.summary());
        let laid_out_outputs = resolved.laid_out.clone();
        let output_names = resolved.names.clone();
        let (outputs, output_size) = create_outputs(
            &display_handle,
            &laid_out_outputs,
            &output_names,
            refresh_mhz,
            output_scale,
        );
        if resolved.laid_out.len() > 1 || !output_scale.is_identity() {
            eprintln!(
                "[slopos-compositor] multi-output/scale: {} heads, canvas {}x{} {}",
                resolved.laid_out.len(),
                output_size.w,
                output_size.h,
                output_scale_summary(output_scale)
            );
        }

        // -----------------------------------------------------------------------
        // Backend + GL renderer setup
        // -----------------------------------------------------------------------

        let x11_backend = if headless {
            None
        } else {
            Some(X11Backend::new().map_err(|err| {
                anyhow::anyhow!(
                    "requested nested backend could not initialize Smithay X11 transport: {err:#}"
                )
            })?)
        };

        let mut renderer_opt = None;
        let mut x11_surface_opt = None;

        if let Some(ref x11_backend) = x11_backend {
            let x11_handle = x11_backend.handle();
            let nested_window_size = Size::<u16, Logical>::from((
                output_size.w.clamp(1, u16::MAX as i32) as u16,
                output_size.h.clamp(1, u16::MAX as i32) as u16,
            ));
            if let Ok(window) = WindowBuilder::new()
                .title("slopos-compositor")
                .size(nested_window_size)
                .build(&x11_handle)
            {
                if let Ok((_drm_node, fd)) = x11_handle.drm_node() {
                    if let Ok(device) = GbmDevice::new(DeviceFd::from(fd)) {
                        if let Ok(egl_display) = unsafe { EGLDisplay::new(device.clone()) } {
                            if let Ok(egl_context) = EGLContext::new(&egl_display) {
                                let modifiers: HashSet<_> = egl_context
                                    .dmabuf_render_formats()
                                    .iter()
                                    .map(|fmt| fmt.modifier)
                                    .collect();
                                if let Ok(surf) = x11_handle.create_surface(
                                    &window,
                                    DmabufAllocator(GbmAllocator::new(
                                        device,
                                        GbmBufferFlags::RENDERING,
                                    )),
                                    modifiers.into_iter(),
                                ) {
                                    x11_surface_opt = Some(surf);
                                }
                                if let Ok(r) = unsafe { GlesRenderer::new(egl_context) } {
                                    renderer_opt = Some(r);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Wayland listening socket. Created only AFTER the X11 backend and GL
        // renderer are up: the socket name (and the wayland-display handshake
        // file the session entrypoint polls) must never be advertised by a
        // compositor that can still fail backend init and exit.
        let socket = ListeningSocketSource::new_auto()?;
        let socket_name = socket.socket_name().to_string_lossy().into_owned();
        tracing::info!("Listening on WAYLAND_DISPLAY={}", socket_name);
        eprintln!("[slopos-compositor] WAYLAND_DISPLAY={}", socket_name);
        println!("WAYLAND_DISPLAY={}", socket_name);
        // Bind the session control endpoint before publishing readiness. The
        // session supervisor starts shell clients as soon as readiness is
        // visible; constructing this listener later in `SloposCompositor`
        // otherwise leaves a startup window where menu actions are lost.
        let control_listener = std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR")
            .map(PathBuf::from)
            .map(|runtime| bind_session_control_listener(&runtime))
            .transpose()?;
        // Write the actual socket name to a file so the entrypoint can read it,
        // and set the env var so child processes launched by the compositor see the right name.
        slopos_compositor::publish_session_readiness(&socket_name, output_size.w, output_size.h)
            .map_err(|err| anyhow::anyhow!("publish private session readiness: {err}"))?;
        std::env::set_var("SLOPOS_CLIENT_WAYLAND_DISPLAY", &socket_name);
        std::env::set_var("WAYLAND_DISPLAY", &socket_name);

        // Insert socket source: accept new Wayland client connections
        loop_handle
            .insert_source(socket, |client_stream, _, state| {
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .expect("failed to insert client");
            })
            .expect("failed to insert wayland socket source");
        register_wayland_display_source(&loop_handle, display)
            .context("insert Wayland display source")?;

        if let Some(x11_backend) = x11_backend {
            loop_handle
                .insert_source(x11_backend, |event, _, state| match event {
                    X11Event::CloseRequested { .. } => {
                        tracing::info!("X11 close requested");
                        state.running = false;
                    }
                    X11Event::Refresh { .. } | X11Event::PresentCompleted { .. } => {
                        // Coalesce host refresh with pending compositor damage.
                        state.request_redraw();
                    }
                    X11Event::Resized { new_size, .. } => {
                        tracing::debug!("resized: {:?}", new_size);
                    }
                    X11Event::Input { event, .. } => match event {
                        BackendInputEvent::Keyboard { event: ev } => {
                            handle_keyboard_event(state, &ev);
                        }
                        BackendInputEvent::PointerMotionAbsolute { event: ev } => {
                            handle_pointer_motion(state, &ev);
                        }
                        BackendInputEvent::PointerButton { event: ev } => {
                            handle_pointer_button(state, &ev);
                        }
                        _ => {}
                    },
                    X11Event::Focus { .. } => {}
                })
                .expect("failed to insert x11 backend source");
        }

        // The session control socket is part of the nested event loop, not a
        // polled side-channel. This keeps the compositor asleep when idle
        // while still waking immediately for shell requests such as Minimize
        // or Fill. The listener is the exact socket bound in this session's
        // runtime directory; no Wayland socket discovery is involved.
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
                .map_err(|error| anyhow::anyhow!("insert session control socket: {error}"))?;
        }

        let clock = Clock::<Monotonic>::new();
        let mut state = SloposCompositor {
            display_handle,
            _loop_signal: loop_signal,
            loop_handle,
            clock,
            compositor_state,
            shm_state,
            seat_state,
            xdg_shell_state,
            data_device_state,
            primary_selection_state,
            _output_manager_state: output_manager_state,
            xwayland_shell_state,
            layer_shell_state,
            foreign_toplevel_list,
            _xdg_decoration_state: xdg_decoration_state,
            _text_input_state: text_input_state,
            _input_method_state: input_method_state,
            im_popups: Vec::new(),
            seat,
            outputs,
            laid_out_outputs,
            output_names,
            running: true,
            windows: Vec::new(),
            workspace_state: WorkspaceState::new(),
            layer_surfaces: Vec::new(),
            popup_manager: PopupManager::default(),
            popup_grab: None,
            activated_window_id: None,
            last_minimized_window_id: None,
            next_window_offset: 0,
            pointer_pos: Point::from((0.0_f64, 0.0_f64)),
            cursor_status: CursorImageStatus::default_named(),
            interactive_grab: None,
            left_button_down: false,
            last_pointer_press: None,
            frame_dirty: true,
            output_size,
            serial: 0,
            renderer: renderer_opt,
            x11_surface: x11_surface_opt,
            clipboard_source: None,
            primary_source: None,
            clipboard_data: HashMap::new(),
            primary_data: HashMap::new(),
            server_dnd_data: HashMap::new(),
            dnd_icon: None,
            display_policy,
            hdr_caps,
            frame_scheduler,
            pending_damage: None,
            need_full_redraw: true, // first frame is always full
            placeholder_stats: PlaceholderPresentStats::new(),
            xwm: None,
            xdisplay: None,
            x11_surfaces: Vec::new(),
            x11_surface_associations: HashMap::new(),
            wayland_socket_name: socket_name.clone(),
        };

        // P1.3: best-effort XWayland after state exists (needs loop_handle).
        try_start_xwayland(&mut state);

        tracing::info!("slopos-compositor event loop starting");
        while state.running {
            // Pace the loop with FrameScheduler when not adaptive (VRR).
            // Adaptive uses a short poll so PresentCompleted / input wake us quickly.
            let dispatch_timeout = if !state.frame_dirty {
                // File-descriptor sources, including the Wayland display, wake
                // calloop immediately. An idle compositor can therefore block
                // until input, a client request, or host output activity.
                None
            } else if state.frame_scheduler.refresh_rate().is_fixed() {
                let wait = state.frame_scheduler.time_until_next_frame();
                let ms = wait.as_millis().clamp(1, 32) as u64;
                Some(Duration::from_millis(ms))
            } else {
                Some(Duration::from_millis(16))
            };

            event_loop.dispatch(dispatch_timeout, &mut state)?;

            // Damage-driven rendering: commits, pointer motion, output refresh,
            // workspace changes and animations explicitly mark the frame dirty.
            // Static desktops therefore sleep instead of saturating LLVMpipe.
            if state.frame_dirty {
                state.render_frame();
            }
        }

        tracing::info!("slopos-compositor exiting");
        Ok(())
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use linux::parse_bool_env;
    use smithay::utils::Point;

    #[test]
    fn test_parse_bool_env() {
        std::env::set_var("TEST_BOOL_ENV_TRUE_1", "1");
        std::env::set_var("TEST_BOOL_ENV_TRUE_2", "true");
        std::env::set_var("TEST_BOOL_ENV_TRUE_3", "YES");
        std::env::set_var("TEST_BOOL_ENV_TRUE_4", "On");
        std::env::set_var("TEST_BOOL_ENV_FALSE_1", "0");
        std::env::set_var("TEST_BOOL_ENV_FALSE_2", "false");
        std::env::set_var("TEST_BOOL_ENV_FALSE_3", "no");
        std::env::set_var("TEST_BOOL_ENV_FALSE_4", "OFF");

        assert!(parse_bool_env("TEST_BOOL_ENV_TRUE_1"));
        assert!(parse_bool_env("TEST_BOOL_ENV_TRUE_2"));
        assert!(parse_bool_env("TEST_BOOL_ENV_TRUE_3"));
        assert!(parse_bool_env("TEST_BOOL_ENV_TRUE_4"));
        assert!(!parse_bool_env("TEST_BOOL_ENV_FALSE_1"));
        assert!(!parse_bool_env("TEST_BOOL_ENV_FALSE_2"));
        assert!(!parse_bool_env("TEST_BOOL_ENV_FALSE_3"));
        assert!(!parse_bool_env("TEST_BOOL_ENV_FALSE_4"));
        assert!(!parse_bool_env("TEST_BOOL_ENV_UNSET"));
    }

    #[test]
    fn automatic_backend_requires_x11_display_for_nested() {
        assert_eq!(linux::default_backend_for_host(Some(":99"), None), "nested");
        assert_eq!(
            linux::default_backend_for_host(None, Some("wayland-0")),
            "drm",
            "a Wayland-only host must not select the X11 nested backend"
        );
        assert_eq!(linux::default_backend_for_host(Some(""), None), "drm");
        assert_eq!(linux::default_backend_for_host(None, None), "drm");
    }

    #[test]
    fn explicit_nested_backend_fails_without_x11_display() {
        let error = linux::validate_nested_transport("nested", None).unwrap_err();
        assert!(error.contains("DISPLAY"));
        assert!(linux::validate_nested_transport("nested", Some(":99")).is_ok());
        assert!(linux::validate_nested_transport("drm", None).is_ok());
        assert!(linux::validate_nested_transport("headless", None).is_ok());
    }

    #[test]
    fn nested_layer_surface_hit_origin_is_translated_to_compositor_space() {
        assert_eq!(
            linux::layer_surface_hit_origin(Point::from((120, 80)), Point::from((7, 11))),
            Point::from((127.0, 91.0)),
        );
    }

    #[test]
    fn nested_control_binds_before_readiness_and_delivers_request() {
        let runtime = std::env::temp_dir().join(format!(
            "slopos-nested-control-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock before Unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&runtime).expect("create test runtime");

        let listener = linux::bind_session_control_listener(&runtime)
            .expect("bind control listener before readiness");
        let control_socket = runtime.join(slopos_bus::SESSION_CONTROL_SOCKET);
        assert!(
            control_socket.exists(),
            "control socket must precede readiness"
        );

        std::fs::write(runtime.join("readiness"), b"wayland-9\n").expect("write readiness marker");
        let request = slopos_bus::SessionControlRequest::FocusedWindow {
            action: slopos_bus::WindowPresentationAction::Fill,
        };
        let previous_runtime = std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR");
        std::env::set_var("SLOPOS_SESSION_RUNTIME_DIR", &runtime);
        slopos_bus::send_session_control(&request).expect("deliver semantic request");
        if let Some(previous_runtime) = previous_runtime {
            std::env::set_var("SLOPOS_SESSION_RUNTIME_DIR", previous_runtime);
        } else {
            std::env::remove_var("SLOPOS_SESSION_RUNTIME_DIR");
        }

        assert_eq!(listener.drain(), vec![request]);
        drop(listener);
        std::fs::remove_dir_all(&runtime).expect("remove test runtime");
    }

    #[test]
    fn nested_control_source_wakes_calloop_and_drains_request() {
        use std::os::unix::net::UnixDatagram;
        use std::time::{Duration, Instant};

        use smithay::reexports::calloop::{
            generic::Generic, EventLoop, Interest, Mode as CalloopMode, PostAction,
        };

        let runtime = std::env::temp_dir().join(format!(
            "slo-evt-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock before Unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&runtime).expect("create test runtime");

        let listener = linux::bind_session_control_listener(&runtime)
            .expect("bind exact session control socket");
        let sender = UnixDatagram::unbound().expect("create control sender");
        let request = slopos_bus::SessionControlRequest::FocusedWindow {
            action: slopos_bus::WindowPresentationAction::Fill,
        };

        let mut event_loop: EventLoop<Vec<slopos_bus::SessionControlRequest>> =
            EventLoop::try_new().expect("create calloop");
        event_loop
            .handle()
            .insert_source(
                Generic::new(listener, Interest::READ, CalloopMode::Level),
                |_, listener, requests| {
                    requests.extend(listener.drain());
                    Ok(PostAction::Continue)
                },
            )
            .expect("register exact control fd");

        // Do not queue the datagram before dispatch: the sender waits briefly
        // after dispatch is entered so the test exercises an idle poll wake.
        let send_after = Duration::from_millis(50);
        let payload = serde_json::to_vec(&request).expect("serialize control request");
        let socket_path = runtime.join(slopos_bus::SESSION_CONTROL_SOCKET);
        let sender_thread = std::thread::spawn(move || {
            std::thread::sleep(send_after);
            sender
                .send_to(&payload, socket_path)
                .expect("send control request");
        });

        let dispatch_timeout = Duration::from_secs(1);
        let dispatch_started = Instant::now();
        let mut observed = Vec::new();
        event_loop
            .dispatch(Some(dispatch_timeout), &mut observed)
            .expect("dispatch control fd");
        let dispatch_elapsed = dispatch_started.elapsed();
        sender_thread.join().expect("join control sender");

        assert!(
            dispatch_elapsed >= Duration::from_millis(25),
            "dispatch returned before the delayed request could wake it: {dispatch_elapsed:?}"
        );
        assert!(
            dispatch_elapsed < dispatch_timeout,
            "dispatch reached its timeout instead of waking for the request: {dispatch_elapsed:?}"
        );
        assert_eq!(observed, vec![request]);

        drop(event_loop);
        std::fs::remove_dir_all(&runtime).expect("remove test runtime");
    }

    #[test]
    fn x11_resize_edges_use_shared_interactive_mapping() {
        assert_eq!(
            linux::x11_resize_edge_to_resize_edges(smithay::xwayland::xwm::ResizeEdge::Top),
            slopos_compositor::ResizeEdges::TOP
        );
        assert_eq!(
            linux::x11_resize_edge_to_resize_edges(smithay::xwayland::xwm::ResizeEdge::BottomRight,),
            slopos_compositor::ResizeEdges::BOTTOM_RIGHT
        );
    }
}
