//! retro-compositor — minimal Wayland compositor using Smithay.
//!
//! This compositor replaces labwc in the RetroShell stack. It:
//!   - Opens an X11 window (running nested under Xvfb on DISPLAY=:99)
//!   - Exposes a Wayland socket so retro-shell (winit/wgpu) can connect
//!   - Implements xdg_shell, wl_shm, wl_seat for basic window management
//!   - Implements wl_data_device selection send (clipboard + primary store)
//!   - Optionally multi-output via RETROSHELL_OUTPUTS=WxH,WxH
//!   - Optionally starts XWayland (best-effort under nested X11)
//!
//! Linux-only: requires libgbm, libdrm, libEGL, libxcb and libwayland-server.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("retro-compositor is Linux-only (requires Wayland/DRM/GBM system libraries).");
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

    use retro_compositor::{
        cascade_position, detect_dri3_from_env, layout_outputs_side_by_side, move_to_top,
        next_cascade_offset, outputs_from_env, select_backend_kind, selection_bytes_for_mime_with_text_fallback,
        session_mode_summary, topmost_window_at, total_output_size, CompositorBackendKind,
        DisplayPolicy, WindowGeometry, DEFAULT_WINDOW_H, DEFAULT_WINDOW_W,
    };
    use retro_compositor::frame_timing::{FrameScheduler, RefreshRate};
    use retro_compositor::hdr::HdrCapabilities;
    use smithay::{
        backend::{
            allocator::{
                dmabuf::DmabufAllocator,
                gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            },
            egl::{EGLContext, EGLDisplay},
            input::{
                ButtonState, InputEvent as BackendInputEvent, KeyboardKeyEvent,
                PointerButtonEvent, PointerMotionAbsoluteEvent,
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
        delegate_compositor, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell,
        input::{
            keyboard::{FilterResult, XkbConfig},
            pointer::{ButtonEvent, CursorImageStatus, MotionEvent},
            Seat, SeatHandler, SeatState,
        },
        output::{Mode, Output, PhysicalProperties, Subpixel},
        reexports::{
            calloop::{EventLoop, LoopHandle, LoopSignal},
            wayland_server::{
                backend::{ClientData, ClientId, DisconnectReason},
                protocol::wl_surface::WlSurface,
                Display, DisplayHandle, Resource,
            },
        },
        utils::{
            Clock, DeviceFd, Logical, Monotonic, Physical, Point, Rectangle, Serial, Size, Transform,
        },
        wayland::{
            buffer::BufferHandler,
            compositor::{CompositorClientState, CompositorHandler, CompositorState},
            output::{OutputHandler, OutputManagerState},
            selection::{
                data_device::{
                    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
                    ServerDndGrabHandler,
                },
                primary_selection::{
                    set_primary_focus, PrimarySelectionHandler, PrimarySelectionState,
                },
                SelectionHandler, SelectionSource, SelectionTarget,
            },
            shell::xdg::{
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            },
            shm::{ShmHandler, ShmState},
            socket::ListeningSocketSource,
        },
    };
    use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
    use smithay::reexports::wayland_server::protocol::{wl_buffer, wl_data_source::WlDataSource, wl_seat};
    use smithay::utils::Serial as WlSerial;
    use smithay::xwayland::{X11Surface as X11WmSurface, X11Wm, XWayland, XWaylandEvent, XwmHandler};
    use smithay::xwayland::xwm::{Reorder, ResizeEdge, X11Window, XwmId};
    use smithay::wayland::xwayland_shell::{
        XWaylandShellHandler, XWaylandShellState,
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
            eprintln!("[retro-compositor] client connected");
        }
        fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
            eprintln!("[retro-compositor] client disconnected");
        }
    }

    // -----------------------------------------------------------------------
    // Tracked surface: a mapped xdg_toplevel with a compositor-space position
    // -----------------------------------------------------------------------

    #[derive(Clone)]
    struct MappedWindow {
        toplevel: ToplevelSurface,
        /// Top-left position in logical compositor space
        position: Point<i32, Logical>,
        /// Last committed size (logical pixels)
        size: Size<i32, Logical>,
    }

    impl MappedWindow {
        fn geometry(&self) -> WindowGeometry {
            WindowGeometry::new(self.position.x, self.position.y, self.size.w, self.size.h)
        }
    }

    // -----------------------------------------------------------------------
    // Main compositor state
    // -----------------------------------------------------------------------

    struct RetroCompositor {
        display_handle: DisplayHandle,
        _loop_signal: LoopSignal,
        loop_handle: LoopHandle<'static, RetroCompositor>,
        _clock: Clock<Monotonic>,

        // Smithay protocol states
        compositor_state: CompositorState,
        shm_state: ShmState,
        seat_state: SeatState<RetroCompositor>,
        xdg_shell_state: XdgShellState,
        data_device_state: DataDeviceState,
        primary_selection_state: PrimarySelectionState,
        _output_manager_state: OutputManagerState,
        xwayland_shell_state: XWaylandShellState,

        seat: Seat<RetroCompositor>,
        /// Registered wl_output objects (one or more; multi-output via RETROSHELL_OUTPUTS).
        /// Kept alive so globals stay registered for the compositor lifetime.
        #[allow(dead_code)]
        outputs: Vec<Output>,
        running: bool,

        // Mapped windows (in painting order, bottom → top)
        windows: Vec<MappedWindow>,
        // Counter for cascading new window placement
        next_window_offset: i32,
        // Current pointer position (logical)
        pointer_pos: Point<f64, Logical>,
        // Output size advertised for X11 input transforms (union of all outputs).
        output_size: Size<i32, Physical>,
        // Serial counter for synthetic events
        serial: u32,

        // GL rendering
        renderer: GlesRenderer,
        x11_surface: X11Surface,

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

        // ---- XWayland (P1.3) ----
        xwm: Option<X11Wm>,
        xdisplay: Option<u32>,
        /// X11 surfaces we know about (not fully managed yet under nested X11).
        x11_surfaces: Vec<X11WmSurface>,
    }

    impl RetroCompositor {
        /// Allocate the next serial (wrapping)
        fn next_serial(&mut self) -> Serial {
            self.serial = self.serial.wrapping_add(1);
            Serial::from(self.serial)
        }

        /// Find the topmost window that contains `pt`, returning its index.
        fn window_at(&self, pt: Point<f64, Logical>) -> Option<usize> {
            let windows: Vec<_> = self.windows.iter().map(MappedWindow::geometry).collect();
            topmost_window_at(&windows, pt.x, pt.y)
        }

        /// Bring window at `idx` to the top and focus keyboard+pointer on it.
        fn focus_window(&mut self, idx: usize) {
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
            self.windows.retain(|w| w.toplevel.alive());
        }

        /// Render a frame using the GlesRenderer:
        ///   1. Acquire an X11 dmabuf
        ///   2. Bind it to the GL renderer
        ///   3. Clear to retro gray, draw placeholder rectangles for each window
        ///   4. Finish the frame and present
        fn render_frame(&mut self) {
            self.prune_dead_windows();

            // Collect SHM render elements for each window BEFORE binding the render target.
            // render_elements_from_surface_tree() borrows &mut self.renderer, which must be
            // done before renderer.bind() / renderer.render() take over the borrow.
            let surface_elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = self
                .windows
                .iter()
                .flat_map(|w| {
                    let loc = Point::<i32, Physical>::from((w.position.x, w.position.y));
                    render_elements_from_surface_tree(
                        &mut self.renderer,
                        w.toplevel.wl_surface(),
                        loc,
                        1.0_f64,
                        1.0_f32,
                        Kind::Unspecified,
                    )
                })
                .collect();

            // Acquire the next buffer from the X11 swapchain
            let (mut dmabuf, _age) = match self.x11_surface.buffer() {
                Ok(pair) => pair,
                Err(e) => {
                    eprintln!("[render] failed to get X11 buffer: {e}");
                    return;
                }
            };

            let output_size = self.output_size;

            // Bind the dmabuf as GL render target
            let mut target = match self.renderer.bind(&mut dmabuf) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[render] failed to bind dmabuf: {e}");
                    return;
                }
            };

            // Open a render frame
            let mut frame = match self.renderer.render(&mut target, output_size, Transform::Normal) {
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
            let full_screen = Rectangle::from_loc_and_size(
                Point::<i32, Physical>::from((0, 0)),
                output_size,
            );
            if let Err(e) = frame.clear(retro_gray, &[full_screen]) {
                eprintln!("[render] clear failed: {e}");
            }

            // Render actual SHM buffer content for windows that have committed a buffer.
            // If no surface elements were collected (client hasn't committed yet), fall back
            // to solid colored placeholder rectangles so the compositor always shows something.
            if !surface_elements.is_empty() {
                if let Err(e) = draw_render_elements::<GlesRenderer, _, _>(
                    &mut frame,
                    1.0_f64,
                    &surface_elements,
                    &[full_screen],
                ) {
                    eprintln!("[render] draw_render_elements failed: {e}");
                }
            } else {
                // Fallback: draw solid colored placeholder rectangles for each mapped window
                let windows: Vec<_> = self.windows.iter().enumerate().map(|(i, w)| {
                    let color_idx = i % WIN_COLORS.len();
                    let (r, g, b) = WIN_COLORS[color_idx];
                    let rect = Rectangle::from_loc_and_size(
                        Point::<i32, Physical>::from((w.position.x, w.position.y)),
                        Size::<i32, Physical>::from((w.size.w, w.size.h)),
                    );
                    (rect, Color32F::from([r, g, b, 1.0_f32]))
                }).collect();

                for (rect, color) in &windows {
                    if let Err(e) = frame.clear(*color, &[*rect]) {
                        eprintln!("[render] window clear failed: {e}");
                    }
                }
            }

            // Finish the frame (flushes GL commands)
            if let Err(e) = frame.finish() {
                eprintln!("[render] frame finish failed: {e}");
            }

            // Present to the X11 window
            if let Err(e) = self.x11_surface.submit() {
                eprintln!("[render] submit failed: {e}");
            }

            self.frame_scheduler.record_frame();
        }
    }

    // -----------------------------------------------------------------------
    // BufferHandler (required by on_commit_buffer_handler)
    // -----------------------------------------------------------------------

    impl BufferHandler for RetroCompositor {
        fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
    }

    // -----------------------------------------------------------------------
    // CompositorHandler
    // -----------------------------------------------------------------------

    impl CompositorHandler for RetroCompositor {
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
            // use that or fall back to DEFAULT_WIN.
            for w in self.windows.iter_mut() {
                if w.toplevel.wl_surface() == surface {
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
                    break;
                }
            }
        }
    }

    delegate_compositor!(RetroCompositor);

    // -----------------------------------------------------------------------
    // ShmHandler
    // -----------------------------------------------------------------------

    impl ShmHandler for RetroCompositor {
        fn shm_state(&self) -> &ShmState {
            &self.shm_state
        }
    }

    delegate_shm!(RetroCompositor);

    // -----------------------------------------------------------------------
    // SeatHandler
    // -----------------------------------------------------------------------

    impl SeatHandler for RetroCompositor {
        type KeyboardFocus = WlSurface;
        type PointerFocus = WlSurface;
        type TouchFocus = WlSurface;

        fn seat_state(&mut self) -> &mut SeatState<RetroCompositor> {
            &mut self.seat_state
        }

        fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}

        fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
            let client = focused.and_then(|s| s.client());
            set_data_device_focus(&self.display_handle, seat, client.clone());
            set_primary_focus(&self.display_handle, seat, client);
        }
    }

    delegate_seat!(RetroCompositor);

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

    impl SelectionHandler for RetroCompositor {
        type SelectionUserData = MimePayload;

        fn new_selection(
            &mut self,
            ty: SelectionTarget,
            source: Option<SelectionSource>,
            _seat: Seat<Self>,
        ) {
            let mime_types = source
                .as_ref()
                .map(|s| s.mime_types())
                .unwrap_or_default();
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

    impl DataDeviceHandler for RetroCompositor {
        fn data_device_state(&self) -> &DataDeviceState {
            &self.data_device_state
        }
    }

    impl ClientDndGrabHandler for RetroCompositor {
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

        fn dropped(
            &mut self,
            _target: Option<WlSurface>,
            _validated: bool,
            _seat: Seat<Self>,
        ) {
            self.dnd_icon = None;
            tracing::debug!("client DnD dropped");
        }
    }

    impl ServerDndGrabHandler for RetroCompositor {
        fn send(
            &mut self,
            mime_type: String,
            fd: OwnedFd,
            _seat: Seat<Self>,
        ) {
            // Server-initiated DnD: write tracked mime payloads, or EOF if none.
            let data = selection_bytes_for_mime_with_text_fallback(&self.server_dnd_data, &mime_type)
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

    smithay::delegate_data_device!(RetroCompositor);

    impl PrimarySelectionHandler for RetroCompositor {
        fn primary_selection_state(&self) -> &PrimarySelectionState {
            &self.primary_selection_state
        }
    }

    delegate_primary_selection!(RetroCompositor);

    // -----------------------------------------------------------------------
    // XdgShellHandler
    // -----------------------------------------------------------------------

    impl XdgShellHandler for RetroCompositor {
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

            eprintln!(
                "[retro-compositor] surface mapped at ({},{})",
                position.x, position.y
            );

            self.windows.push(MappedWindow {
                toplevel: surface,
                position,
                size: Size::from((DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)),
            });

            // Focus the new window
            let idx = self.windows.len() - 1;
            self.focus_window(idx);
        }

        fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
            self.windows.retain(|w| w.toplevel.wl_surface() != surface.wl_surface());
            // Move focus to the topmost remaining window, if any
            if let Some(top) = self.windows.last() {
                let surf = top.toplevel.wl_surface().clone();
                let serial = self.next_serial();
                if let Some(kb) = self.seat.get_keyboard() {
                    kb.set_focus(self, Some(surf.clone()), serial);
                }
                let client = surf.client();
                set_data_device_focus(&self.display_handle, &self.seat, client.clone());
                set_primary_focus(&self.display_handle, &self.seat, client);
            } else if let Some(kb) = self.seat.get_keyboard() {
                let serial = self.next_serial();
                kb.set_focus(self, None, serial);
                set_data_device_focus(&self.display_handle, &self.seat, None);
                set_primary_focus(&self.display_handle, &self.seat, None);
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

    delegate_xdg_shell!(RetroCompositor);

    // -----------------------------------------------------------------------
    // OutputHandler (required by delegate_output!)
    // -----------------------------------------------------------------------

    impl OutputHandler for RetroCompositor {}

    delegate_output!(RetroCompositor);

    // -----------------------------------------------------------------------
    // XWayland (P1.3) — best-effort under nested X11
    //
    // Nested under Xvfb/X11 the compositor already owns DISPLAY. XWayland is
    // still spawned (own display number) so the code path exists and X clients
    // can attach when the binary + runtime allow it. Full rootless WM mapping
    // of X11 windows into the GL scene is incomplete under nested X11; handlers
    // accept maps and track surfaces so the path is live for native Linux.
    // -----------------------------------------------------------------------

    impl XWaylandShellHandler for RetroCompositor {
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

    delegate_xwayland_shell!(RetroCompositor);

    impl XwmHandler for RetroCompositor {
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
            let data = selection_bytes_for_mime_with_text_fallback(store, &mime_type)
                .map(|b| b.to_vec());
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

    fn handle_keyboard_event<E>(state: &mut RetroCompositor, ev: &E)
    where
        E: KeyboardKeyEvent<X11Input>,
    {
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
                |_data, _mods, _keysym| FilterResult::Forward,
            );
        }
    }

    fn handle_pointer_motion<E>(state: &mut RetroCompositor, ev: &E)
    where
        E: PointerMotionAbsoluteEvent<X11Input>,
    {
        let logical = Size::<i32, Logical>::from((state.output_size.w, state.output_size.h));
        let pos = ev.position_transformed(logical);
        state.pointer_pos = pos;

        // Find which window (if any) the pointer is over
        let focus = state.window_at(pos).map(|idx| {
            let w = &state.windows[idx];
            let local = Point::from((
                pos.x - w.position.x as f64,
                pos.y - w.position.y as f64,
            ));
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

    fn handle_pointer_button<E>(state: &mut RetroCompositor, ev: &E)
    where
        E: PointerButtonEvent<X11Input>,
    {
        let serial = state.next_serial();
        let time = ev.time_msec();
        let button = ev.button_code();
        let btn_state = ev.state();

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
    }

    /// Create one or more wl_output globals laid out side-by-side.
    fn create_outputs(
        display_handle: &DisplayHandle,
        configs: &[retro_compositor::OutputConfig],
        refresh_mhz: i32,
    ) -> (Vec<Output>, Size<i32, Physical>) {
        let laid_out = layout_outputs_side_by_side(configs);
        let total = total_output_size(&laid_out);
        let mut outputs = Vec::with_capacity(laid_out.len());

        for (i, o) in laid_out.iter().enumerate() {
            let name = format!("X11-{}", i + 1);
            let output = Output::new(
                name,
                PhysicalProperties {
                    size: (0, 0).into(),
                    subpixel: Subpixel::Unknown,
                    make: "RetroShell".into(),
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
                None,
                Some((o.x, o.y).into()),
            );
            output.set_preferred(mode);
            output.create_global::<RetroCompositor>(display_handle);
            tracing::info!(
                "wl_output {} {}x{} at ({},{}) refresh={} mHz",
                i + 1,
                o.config.width,
                o.config.height,
                o.x,
                o.y,
                refresh_mhz
            );
            outputs.push(output);
        }

        let output_size = Size::<i32, Physical>::from((total.width, total.height));
        (outputs, output_size)
    }

    /// Best-effort XWayland startup. Returns false when the binary is missing or spawn fails.
    ///
    /// Under nested X11 this is still useful: XWayland gets its own display number and
    /// clients can set DISPLAY=:N. Full scene integration of X11 surfaces remains limited
    /// because the compositor itself is an X11 client of the host server.
    fn try_start_xwayland(state: &mut RetroCompositor) {
        // Allow opt-out: RETROSHELL_XWAYLAND=0
        if std::env::var("RETROSHELL_XWAYLAND")
            .map(|v| matches!(v.as_str(), "0" | "false" | "off" | "no"))
            .unwrap_or(false)
        {
            tracing::info!("XWayland disabled via RETROSHELL_XWAYLAND");
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
                                    std::env::set_var("RETROSHELL_XWAYLAND_DISPLAY", format!(":{display_number}"));
                                    eprintln!(
                                        "[retro-compositor] XWayland ready DISPLAY=:{}",
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
                    "[retro-compositor] XWayland unavailable: {err} (continuing without it)"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub fn run() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();

        // ---- Backend mode honesty (session DRM vs nested X11 vs labwc) ----
        // This binary is the nested-X11 / session candidate; labwc is chosen by
        // start-retroshell/entrypoint when we die early. Log the selected kind.
        let force_labwc = std::env::var_os("RETROSHELL_FORCE_LABWC").is_some()
            || std::env::var("RETROSHELL_COMPOSITOR")
                .map(|v| v.eq_ignore_ascii_case("labwc"))
                .unwrap_or(false);
        let prefer_drm = std::env::var("RETROSHELL_PREFER_DRM")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(std::path::Path::new("/dev/dri").exists());
        let dri3 = detect_dri3_from_env().unwrap_or(prefer_drm && !force_labwc);
        let backend_kind = select_backend_kind(prefer_drm, dri3, force_labwc);
        let mode_line = session_mode_summary(backend_kind);
        tracing::info!("compositor backend selection: {mode_line}");
        eprintln!("[retro-compositor] backend: {mode_line}");
        if matches!(backend_kind, CompositorBackendKind::LabwcFallback) {
            anyhow::bail!(
                "RETROSHELL_FORCE_LABWC / COMPOSITOR=labwc set; refusing to start nested compositor"
            );
        }
        if matches!(backend_kind, CompositorBackendKind::SessionDrm) {
            tracing::warn!(
                "SessionDrm selected by policy; this build still uses nested X11 backend code path until DRM backend lands — running NestedX11 with honest log"
            );
            eprintln!(
                "[retro-compositor] NOTE: SessionDrm preferred but runtime is NestedX11 until DRM backend ships"
            );
        }

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
        eprintln!("[retro-compositor] display policy: {policy_line}");
        if display_policy.hdr_requested && !hdr_caps.hdr_supported {
            tracing::info!(
                "HDR requested but not supported under nested X11/no-KMS probe; staying SDR ({})",
                hdr_caps.current_color_space.as_str()
            );
        }

        let mut event_loop: EventLoop<RetroCompositor> = EventLoop::try_new()?;
        let mut display: Display<RetroCompositor> = Display::new()?;
        let display_handle = display.handle();
        let loop_handle = event_loop.handle();
        let loop_signal = event_loop.get_signal();

        // Protocol states
        let compositor_state = CompositorState::new::<RetroCompositor>(&display_handle);
        let shm_state = ShmState::new::<RetroCompositor>(&display_handle, vec![]);
        let mut seat_state = SeatState::new();
        let xdg_shell_state = XdgShellState::new::<RetroCompositor>(&display_handle);
        let data_device_state = DataDeviceState::new::<RetroCompositor>(&display_handle);
        let primary_selection_state = PrimarySelectionState::new::<RetroCompositor>(&display_handle);
        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<RetroCompositor>(&display_handle);
        let xwayland_shell_state = XWaylandShellState::new::<RetroCompositor>(&display_handle);

        // Seat: keyboard + pointer
        let mut seat: Seat<RetroCompositor> =
            seat_state.new_wl_seat(&display_handle, "seat0");
        seat.add_keyboard(XkbConfig::default(), 200, 25)?;
        seat.add_pointer();

        // ---- Outputs (P1.2 multi-output) ----
        let output_configs = outputs_from_env();
        let (outputs, output_size) =
            create_outputs(&display_handle, &output_configs, refresh_mhz);
        if output_configs.len() > 1 {
            eprintln!(
                "[retro-compositor] multi-output: {} heads, canvas {}x{}",
                output_configs.len(),
                output_size.w,
                output_size.h
            );
        }

        // Wayland listening socket
        let socket = ListeningSocketSource::new_auto()?;
        let socket_name = socket.socket_name().to_string_lossy().into_owned();
        tracing::info!("Listening on WAYLAND_DISPLAY={}", socket_name);
        eprintln!("[retro-compositor] WAYLAND_DISPLAY={}", socket_name);
        println!("WAYLAND_DISPLAY={}", socket_name);
        // Write the actual socket name to a file so the entrypoint can read it,
        // and set the env var so child processes launched by the compositor see the right name.
        let _ = std::fs::write("/tmp/runtime-root/wayland-display", &socket_name);
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

        // -----------------------------------------------------------------------
        // X11 backend + GL renderer setup
        // -----------------------------------------------------------------------

        let x11_backend = X11Backend::new()?;
        let x11_handle = x11_backend.handle();
        // Single X11 host window covering the union of all logical outputs.
        let window = WindowBuilder::new()
            .title("retro-compositor")
            .build(&x11_handle)?;

        // Obtain the DRM render node used by the X server
        let (_drm_node, fd) = x11_handle.drm_node()?;

        // Create a GBM device on that node for buffer allocation
        let device = GbmDevice::new(DeviceFd::from(fd))?;

        // Create an EGL display backed by the GBM device, then an EGL context
        let egl_display = unsafe { EGLDisplay::new(device.clone())? };
        let egl_context = EGLContext::new(&egl_display)?;

        // Collect dmabuf modifiers supported by this GL context
        let modifiers: HashSet<_> = egl_context
            .dmabuf_render_formats()
            .iter()
            .map(|fmt| fmt.modifier)
            .collect();

        // Create the X11 surface (swapchain backed by GBM dmabufs)
        let x11_surface = x11_handle.create_surface(
            &window,
            DmabufAllocator(GbmAllocator::new(device, GbmBufferFlags::RENDERING)),
            modifiers.into_iter(),
        )?;

        // Build the GlesRenderer from the EGL context
        // SAFETY: we are the sole owner of `egl_context` and it is not current on any other thread.
        let renderer = unsafe { GlesRenderer::new(egl_context)? };

        loop_handle
            .insert_source(x11_backend, |event, _, state| match event {
                X11Event::CloseRequested { .. } => {
                    tracing::info!("X11 close requested");
                    state.running = false;
                }
                X11Event::Refresh { .. } | X11Event::PresentCompleted { .. } => {
                    // Frame pacing hint from X server — render immediately
                    state.render_frame();
                }
                X11Event::Resized { new_size, .. } => {
                    tracing::debug!("resized: {:?}", new_size);
                }
                X11Event::Input { event, .. } => {
                    match event {
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
                    }
                }
                X11Event::Focus { .. } => {}
            })
            .expect("failed to insert X11 backend");

        let clock = Clock::<Monotonic>::new();
        let mut state = RetroCompositor {
            display_handle,
            _loop_signal: loop_signal,
            loop_handle,
            _clock: clock,
            compositor_state,
            shm_state,
            seat_state,
            xdg_shell_state,
            data_device_state,
            primary_selection_state,
            _output_manager_state: output_manager_state,
            xwayland_shell_state,
            seat,
            outputs,
            running: true,
            windows: Vec::new(),
            next_window_offset: 0,
            pointer_pos: Point::from((0.0_f64, 0.0_f64)),
            output_size,
            serial: 0,
            renderer,
            x11_surface,
            clipboard_source: None,
            primary_source: None,
            clipboard_data: HashMap::new(),
            primary_data: HashMap::new(),
            server_dnd_data: HashMap::new(),
            dnd_icon: None,
            display_policy,
            hdr_caps,
            frame_scheduler,
            xwm: None,
            xdisplay: None,
            x11_surfaces: Vec::new(),
        };

        // P1.3: best-effort XWayland after state exists (needs loop_handle + display).
        try_start_xwayland(&mut state);

        tracing::info!("retro-compositor event loop starting");
        let mut frame_counter: u32 = 0;
        while state.running {
            display.flush_clients()?;

            // Pace the loop with FrameScheduler when not adaptive (VRR).
            // Adaptive uses a short poll so PresentCompleted / input wake us quickly.
            let dispatch_timeout = if state.frame_scheduler.refresh_rate().is_fixed() {
                let wait = state.frame_scheduler.time_until_next_frame();
                // Cap wait so input stays responsive; floor at 1ms.
                let ms = wait.as_millis().min(32).max(1) as u64;
                Some(Duration::from_millis(ms))
            } else {
                Some(Duration::from_millis(1))
            };

            event_loop.dispatch(dispatch_timeout, &mut state)?;

            // Steady-state frames: fixed rates render ~every N ticks; adaptive more often.
            frame_counter = frame_counter.wrapping_add(1);
            let render_every = match state.frame_scheduler.refresh_rate() {
                RefreshRate::Hz60 => 4,
                RefreshRate::Hz120 => 2,
                RefreshRate::Hz144 | RefreshRate::Hz165 => 2,
                RefreshRate::Adaptive => 1,
            };
            if frame_counter % render_every == 1 {
                state.render_frame();
            }
        }

        tracing::info!("retro-compositor exiting");
        Ok(())
    }
}
