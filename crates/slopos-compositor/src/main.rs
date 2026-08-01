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
    use std::collections::{HashMap, HashSet};
    use std::io::Write;
    use std::os::unix::io::OwnedFd;
    use std::sync::Arc;
    use std::time::Duration;

    use slopos_compositor::frame_timing::{FrameScheduler, RefreshRate};
    use slopos_compositor::hdr::HdrCapabilities;
    use slopos_compositor::{
        accumulate_damage_for_window_move, accumulate_damage_rect, apply_scale_to_output_config,
        assign_new_window_to_active, calculate_presentation_geometry, cascade_position,
        detect_dri3_from_env, detect_output_scale_from_env, focus_window_after_workspace_switch,
        geometry_for_interactive_grab, move_to_top, next_cascade_offset, output_scale_summary,
        prefer_full_redraw, resolve_laid_out_outputs_from_env,
        selection_bytes_for_mime_with_text_fallback, session_mode_note,
        text_input_capability_from_env, text_input_capability_summary, topmost_window_at,
        total_output_size, window_paint_source, CompositorBackendKind, DamageRect, DisplayPolicy,
        InteractiveGrab, InteractiveGrabKind, LaidOutOutput, OutputScale, PlaceholderPresentStats,
        ResizeEdges, TextInputCapability, TilePlacement, WindowGeometry, WindowPaintSource,
        WindowPresentationState, WindowRestoreState, WorkspaceId, WorkspaceState, ZoomAction,
        ZoomPolicyConfig, DEFAULT_WINDOW_H, DEFAULT_WINDOW_W,
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
            pointer::{ButtonEvent, CursorImageStatus, CursorImageSurfaceData, MotionEvent},
            Seat, SeatHandler, SeatState,
        },
        output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
        reexports::{
            calloop::{EventLoop, LoopHandle, LoopSignal},
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
            shell::wlr_layer::{Layer, LayerSurface, WlrLayerShellHandler, WlrLayerShellState},
            shell::xdg::{
                decoration::{XdgDecorationHandler, XdgDecorationState},
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
                XdgToplevelSurfaceData,
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
        /// Top-left position in logical compositor space
        position: Point<i32, Logical>,
        /// Last committed size (logical pixels)
        size: Size<i32, Logical>,
        /// Geometry restored after maximize/fullscreen.
        restore_geometry: Option<WindowGeometry>,
        /// Single-authority presentation state (Normal, Minimized, SmartZoomed, Filled, Fullscreen, Tiled).
        presentation_state: WindowPresentationState,
        /// Saved restore state prior to zoom/fill/fullscreen/tiling.
        restore_state: Option<WindowRestoreState>,
        /// Minimized windows stay mapped but are excluded from hit-testing/painting.
        minimized: bool,
    }

    struct MappedLayer {
        surface: LayerSurface,
        #[allow(dead_code)]
        layer: Layer,
        #[allow(dead_code)]
        namespace: String,
    }

    impl MappedWindow {
        fn geometry(&self) -> WindowGeometry {
            WindowGeometry::new(self.position.x, self.position.y, self.size.w, self.size.h)
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
        running: bool,

        // Mapped windows (in painting order, bottom → top)
        windows: Vec<MappedWindow>,
        /// Virtual workspaces: only active-workspace windows are painted.
        workspace_state: WorkspaceState,
        // Layer-shell chrome (menu bar, dock, notifications, …)
        layer_surfaces: Vec<MappedLayer>,
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
        /// Wayland socket name advertised to spawned clients (Super+O/L shortcuts).
        wayland_socket_name: String,
    }

    impl SloposCompositor {
        /// Allocate the next serial (wrapping)
        fn next_serial(&mut self) -> Serial {
            self.serial = self.serial.wrapping_add(1);
            Serial::from(self.serial)
        }

        /// Find the topmost **visible** window that contains `pt`, returning its index.
        fn window_at(&self, pt: Point<f64, Logical>) -> Option<usize> {
            // Walk top→bottom; skip windows on inactive workspaces.
            for (idx, w) in self.windows.iter().enumerate().rev() {
                if w.minimized || !self.workspace_state.is_visible(&w.window_id) {
                    continue;
                }
                if w.geometry().contains_f64(pt.x, pt.y) {
                    return Some(idx);
                }
            }
            None
        }

        /// Bring window at `idx` to the top and focus keyboard+pointer on it.
        fn focus_window(&mut self, idx: usize) {
            if idx >= self.windows.len() {
                return;
            }
            self.windows[idx].minimized = false;
            // Rotate to top
            let surface = self.windows[idx].toplevel.wl_surface().clone();
            move_to_top(&mut self.windows, idx);

            let serial = self.next_serial();
            if let Some(kb) = self.seat.get_keyboard() {
                kb.set_focus(self, Some(surface.clone()), serial);
            }
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

            // Clipboard/primary selection focus follows keyboard focus (smithay seat data).
            let client = surface.client();
            set_data_device_focus(&self.display_handle, &self.seat, client.clone());
            set_primary_focus(&self.display_handle, &self.seat, client);
        }

        /// Remove dead windows (client disconnected / surface destroyed).
        fn prune_dead_windows(&mut self) {
            let before: Vec<String> = self.windows.iter().map(|w| w.window_id.clone()).collect();
            self.windows.retain(|w| w.toplevel.alive());
            let alive: HashSet<&str> = self.windows.iter().map(|w| w.window_id.as_str()).collect();
            for id in before {
                if !alive.contains(id.as_str()) {
                    self.workspace_state.remove_window(&id);
                }
            }
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

        fn begin_interactive_grab(&mut self, surface: &ToplevelSurface, kind: InteractiveGrabKind) {
            if !self.left_button_down {
                tracing::debug!("ignoring xdg move/resize without pressed left button");
                return;
            }
            let Some(window) = self
                .windows
                .iter()
                .find(|w| w.toplevel.wl_surface() == surface.wl_surface())
            else {
                return;
            };
            let pointer_x = self.pointer_pos.x.round() as i32;
            let pointer_y = self.pointer_pos.y.round() as i32;
            self.interactive_grab = Some(InteractiveGrab {
                window_id: window.window_id.clone(),
                kind,
                start_pointer_x: pointer_x,
                start_pointer_y: pointer_y,
                start_geometry: window.geometry(),
            });
            tracing::debug!(
                window_id = %window.window_id,
                ?kind,
                pointer_x,
                pointer_y,
                "interactive grab started"
            );
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
            let new = geometry_for_interactive_grab(
                &grab,
                self.pointer_pos.x.round() as i32,
                self.pointer_pos.y.round() as i32,
                160,
                96,
                self.output_size.w,
                self.output_size.h,
            );
            let old = self.windows[idx].geometry();
            if old == new {
                return;
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
        }

        fn finish_interactive_grab(&mut self) {
            let Some(grab) = self.interactive_grab.take() else {
                return;
            };
            if matches!(grab.kind, InteractiveGrabKind::Resize(_)) {
                if let Some(window) = self.windows.iter().find(|w| w.window_id == grab.window_id) {
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

        fn set_window_state_geometry(
            &mut self,
            surface: &ToplevelSurface,
            state_flag: xdg_toplevel::State,
            enabled: bool,
        ) {
            let Some(idx) = self
                .windows
                .iter()
                .position(|w| w.toplevel.wl_surface() == surface.wl_surface())
            else {
                return;
            };
            let old = self.windows[idx].geometry();
            if enabled {
                if self.windows[idx].restore_geometry.is_none() {
                    self.windows[idx].restore_geometry = Some(old);
                }
                self.windows[idx].position = Point::from((0, 0));
                self.windows[idx].size = Size::from((self.output_size.w, self.output_size.h));
            } else if let Some(restore) = self.windows[idx].restore_geometry.take() {
                self.windows[idx].position = Point::from((restore.x, restore.y));
                self.windows[idx].size = Size::from((restore.width, restore.height));
            }
            let new = self.windows[idx].geometry();
            let toplevel = self.windows[idx].toplevel.clone();
            toplevel.with_pending_state(|state| {
                if enabled {
                    state.states.set(state_flag);
                } else {
                    state.states.unset(state_flag);
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
            self.layer_surfaces.retain(|l| l.surface.alive());

            // Present plan: workspace switch forces full redraw; otherwise use pending
            // damage heuristic (still full clear today — partial clip is follow-on).
            let full_redraw = self.need_full_redraw
                || self.pending_damage.map_or(false, |d| {
                    prefer_full_redraw(d, self.output_size.w, self.output_size.h)
                });
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
                let loc = Point::<i32, Physical>::from((0, 0));
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
                        let rect = Rectangle::from_loc_and_size(
                            Point::<i32, Physical>::from((w.position.x, w.position.y)),
                            Size::<i32, Physical>::from((w.size.w, w.size.h)),
                        );
                        placeholders.push((rect, Color32F::from([r, g, b, 1.0_f32])));
                    }
                }
            }
            for &i in &over {
                let layer = &self.layer_surfaces[i];
                let loc = Point::<i32, Physical>::from((0, 0));
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
            let full_screen =
                Rectangle::from_loc_and_size(Point::<i32, Physical>::from((0, 0)), output_size);
            if let Err(e) = frame.clear(retro_gray, &[full_screen]) {
                eprintln!("[render] clear failed: {e}");
            }

            if !placeholders.is_empty() {
                if self.placeholder_stats.note_frame_with_placeholders() {
                    eprintln!(
                        "[slopos-compositor] present honesty: frame used solid placeholders \
                         (no committed SHM buffer for {} window(s)); session counter starts at {}",
                        placeholders.len(),
                        self.placeholder_stats.frames_with_placeholders
                    );
                }
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
                    let rect = Rectangle::from_loc_and_size(
                        Point::<i32, Physical>::from((origin_x + x, origin_y + y)),
                        Size::<i32, Physical>::from((width, 1)),
                    );
                    let _ = frame.clear(black, &[rect]);
                }
                for &(x, y, width) in FILL {
                    let rect = Rectangle::from_loc_and_size(
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
            surface.with_pending_state(|state| {
                // Tell the client what size we'd like
                state.size = Some(Size::from((DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)));
                state.states.set(xdg_toplevel::State::Activated);
            });
            surface.send_configure();

            // Cascade new windows
            let offset = self.next_window_offset;
            self.next_window_offset = next_cascade_offset(offset);
            let position = Point::from(cascade_position(offset));

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
                position,
                size: Size::from((DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)),
                restore_geometry: None,
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
            _seat: wl_seat::WlSeat,
            _serial: Serial,
        ) {
            self.begin_interactive_grab(&surface, InteractiveGrabKind::Move);
        }

        fn resize_request(
            &mut self,
            surface: ToplevelSurface,
            _seat: wl_seat::WlSeat,
            _serial: Serial,
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
            self.begin_interactive_grab(&surface, InteractiveGrabKind::Resize(edges));
        }

        fn maximize_request(&mut self, surface: ToplevelSurface) {
            self.set_window_state_geometry(&surface, xdg_toplevel::State::Maximized, true);
        }

        fn unmaximize_request(&mut self, surface: ToplevelSurface) {
            self.set_window_state_geometry(&surface, xdg_toplevel::State::Maximized, false);
        }

        fn fullscreen_request(
            &mut self,
            surface: ToplevelSurface,
            _output: Option<wl_output::WlOutput>,
        ) {
            self.set_window_state_geometry(&surface, xdg_toplevel::State::Fullscreen, true);
        }

        fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
            self.set_window_state_geometry(&surface, xdg_toplevel::State::Fullscreen, false);
        }

        fn minimize_request(&mut self, surface: ToplevelSurface) {
            if let Some(idx) = self
                .windows
                .iter()
                .position(|w| w.toplevel.wl_surface() == surface.wl_surface())
            {
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
            if let Some(w) = self
                .windows
                .iter()
                .find(|w| w.toplevel.wl_surface() == surface.wl_surface())
            {
                w.foreign.send_app_id(&app_id);
                w.foreign.send_done();
            }
        }

        fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

        fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: WlSerial) {}

        fn reposition_request(
            &mut self,
            _surface: PopupSurface,
            _positioner: PositionerState,
            _token: u32,
        ) {
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
            let size = self.output_size;
            surface.with_pending_state(|state| {
                state.size = Some(Size::from((size.w, size.h)));
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
            _wl_surface: WlSurface,
            surface: X11WmSurface,
        ) {
            tracing::info!(
                title = %surface.title(),
                "XWayland surface associated with wl_surface"
            );
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
        }

        fn destroyed_window(&mut self, _xwm: XwmId, window: X11WmSurface) {
            self.x11_surfaces
                .retain(|s| s.window_id() != window.window_id());
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
            _window: X11WmSurface,
            _button: u32,
            _resize_edge: ResizeEdge,
        ) {
        }

        fn move_request(&mut self, _xwm: XwmId, _window: X11WmSurface, _button: u32) {}

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
        state.update_interactive_grab();
        state.request_redraw();

        // Find which window (if any) the pointer is over
        let focus = state.window_at(pos).map(|idx| {
            let w = &state.windows[idx];
            let local = Point::from((pos.x - w.position.x as f64, pos.y - w.position.y as f64));
            (w.toplevel.wl_surface().clone(), local)
        });

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
            if btn_state == ButtonState::Released {
                state.finish_interactive_grab();
            }
        }

        // On press: hit-test surfaces and focus the topmost one
        if btn_state == ButtonState::Pressed {
            let pos = state.pointer_pos;
            if let Some(idx) = state.window_at(pos) {
                state.focus_window(idx);
            } else {
                // Click on desktop: clear keyboard focus
                let serial = state.next_serial();
                if let Some(kb) = state.seat.get_keyboard() {
                    kb.set_focus(state, None, serial);
                }
                set_data_device_focus(&state.display_handle, &state.seat, None);
                set_primary_focus(&state.display_handle, &state.seat, None);
            }
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

    pub fn parse_bool_env(key: &str) -> bool {
        match std::env::var(key) {
            Ok(v) => matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            ),
            Err(_) => false,
        }
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
            if std::env::var_os("DISPLAY").is_some()
                || std::env::var_os("SLOPOS_HOST_WAYLAND_DISPLAY").is_some()
            {
                "nested".to_owned()
            } else {
                "drm".to_owned()
            }
        });

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
        let backend_kind = CompositorBackendKind::NestedX11;
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
        let mut display: Display<SloposCompositor> = Display::new()?;
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
        let (outputs, output_size) = create_outputs(
            &display_handle,
            &resolved.laid_out,
            &resolved.names,
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
            if let Ok(window) = WindowBuilder::new()
                .title("slopos-compositor")
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
        // Write the actual socket name to a file so the entrypoint can read it,
        // and set the env var so child processes launched by the compositor see the right name.
        let runtime_dir =
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp/runtime-root".to_string());
        let _ = std::fs::write(
            std::path::Path::new(&runtime_dir).join("wayland-display"),
            &socket_name,
        );
        let _ = std::fs::write(
            std::path::Path::new(&runtime_dir).join("slopos-client-wayland-display"),
            &socket_name,
        );
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
            running: true,
            windows: Vec::new(),
            workspace_state: WorkspaceState::new(),
            layer_surfaces: Vec::new(),
            next_window_offset: 0,
            pointer_pos: Point::from((0.0_f64, 0.0_f64)),
            cursor_status: CursorImageStatus::default_named(),
            interactive_grab: None,
            left_button_down: false,
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
            wayland_socket_name: socket_name.clone(),
        };

        // P1.3: best-effort XWayland after state exists (needs loop_handle + display).
        try_start_xwayland(&mut state);

        tracing::info!("slopos-compositor event loop starting");
        while state.running {
            display.flush_clients()?;

            // Pace the loop with FrameScheduler when not adaptive (VRR).
            // Adaptive uses a short poll so PresentCompleted / input wake us quickly.
            let dispatch_timeout = if !state.frame_dirty {
                // File-descriptor sources wake calloop immediately. A long idle
                // timeout prevents an empty desktop from polling at 1 kHz.
                Some(Duration::from_millis(250))
            } else if state.frame_scheduler.refresh_rate().is_fixed() {
                let wait = state.frame_scheduler.time_until_next_frame();
                let ms = wait.as_millis().min(32).max(1) as u64;
                Some(Duration::from_millis(ms))
            } else {
                Some(Duration::from_millis(16))
            };

            event_loop.dispatch(dispatch_timeout, &mut state)?;

            // Process pending client requests. The listening-socket source only
            // ACCEPTS connections; without this call no client request (bind,
            // commit, …) is ever read, and every client hangs on its first
            // roundtrip. Mirrors the DRM session loop in session_drm.rs.
            display.dispatch_clients(&mut state)?;
            display.flush_clients()?;

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
}
