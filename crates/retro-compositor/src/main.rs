//! retro-compositor — minimal Wayland compositor using Smithay.
//!
//! This compositor replaces labwc in the RetroShell stack. It:
//!   - Opens an X11 window (running nested under Xvfb on DISPLAY=:99)
//!   - Exposes a Wayland socket so retro-shell (winit/wgpu) can connect
//!   - Implements xdg_shell, wl_shm, wl_seat for basic window management
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
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::Duration;

    use smithay::{
        backend::{
            allocator::{
                dmabuf::DmabufAllocator,
                gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            },
            egl::{EGLContext, EGLDisplay},
            input::{
                ButtonState,
                InputBackend, InputEvent as BackendInputEvent,
                KeyboardKeyEvent, PointerButtonEvent, PointerMotionAbsoluteEvent,
            },
            renderer::{
                Bind, Color32F, Frame, Renderer, ImportAll,
                element::{
                    Kind,
                    surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                },
                gles::GlesRenderer,
                utils::{on_commit_buffer_handler, draw_render_elements},
            },
            x11::{X11Backend, X11Event, X11Input, X11Surface, WindowBuilder},
        },
        delegate_compositor, delegate_shm, delegate_seat, delegate_xdg_shell, delegate_output,
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
                Display, DisplayHandle,
                protocol::wl_surface::WlSurface,
            },
        },
        utils::{Clock, DeviceFd, Logical, Monotonic, Physical, Point, Rectangle, Size, Transform, Serial},
        wayland::{
            buffer::BufferHandler,
            compositor::{
                CompositorClientState, CompositorHandler, CompositorState,
            },
            output::{OutputHandler, OutputManagerState},
            selection::{
                data_device::{
                    ClientDndGrabHandler, DataDeviceHandler, DataDeviceState,
                    ServerDndGrabHandler,
                },
                SelectionHandler,
            },
            shell::xdg::{
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            },
            shm::{ShmHandler, ShmState},
            socket::ListeningSocketSource,
        },
    };
    use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
    use smithay::reexports::wayland_server::protocol::{wl_buffer, wl_seat};
    use smithay::utils::Serial as WlSerial;

    // Retro gray: rgb(152, 152, 148) — the classic Mac OS desktop fill
    const RETRO_GRAY: (u8, u8, u8) = (152, 152, 148);
    // Default size given to new toplevel windows (placeholder rectangle)
    const DEFAULT_WIN_W: i32 = 640;
    const DEFAULT_WIN_H: i32 = 480;
    // Output resolution
    const OUTPUT_W: i32 = 1024;
    const OUTPUT_H: i32 = 768;

    // Window placeholder colors (cycling palette for distinguishing windows)
    const WIN_COLORS: &[(f32, f32, f32)] = &[
        (0.502, 0.502, 1.000), // soft blue
        (0.502, 1.000, 0.502), // soft green
        (1.000, 0.502, 0.502), // soft red
        (1.000, 1.000, 0.502), // soft yellow
        (0.502, 1.000, 1.000), // soft cyan
        (1.000, 0.502, 1.000), // soft magenta
    ];

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
        fn contains(&self, pt: Point<f64, Logical>) -> bool {
            let x = pt.x as i32;
            let y = pt.y as i32;
            x >= self.position.x
                && x < self.position.x + self.size.w
                && y >= self.position.y
                && y < self.position.y + self.size.h
        }
    }

    // -----------------------------------------------------------------------
    // Main compositor state
    // -----------------------------------------------------------------------

    struct RetroCompositor {
        display_handle: DisplayHandle,
        _loop_signal: LoopSignal,
        _loop_handle: LoopHandle<'static, RetroCompositor>,
        _clock: Clock<Monotonic>,

        // Smithay protocol states
        compositor_state: CompositorState,
        shm_state: ShmState,
        seat_state: SeatState<RetroCompositor>,
        xdg_shell_state: XdgShellState,
        data_device_state: DataDeviceState,
        _output_manager_state: OutputManagerState,

        seat: Seat<RetroCompositor>,
        _output: Output,
        running: bool,

        // Mapped windows (in painting order, bottom → top)
        windows: Vec<MappedWindow>,
        // Counter for cascading new window placement
        next_window_offset: i32,
        // Current pointer position (logical)
        pointer_pos: Point<f64, Logical>,
        // Serial counter for synthetic events
        serial: u32,

        // GL rendering
        renderer: GlesRenderer,
        x11_surface: X11Surface,
    }

    impl RetroCompositor {
        /// Allocate the next serial (wrapping)
        fn next_serial(&mut self) -> Serial {
            self.serial = self.serial.wrapping_add(1);
            Serial::from(self.serial)
        }

        /// Find the topmost window that contains `pt`, returning its index.
        fn window_at(&self, pt: Point<f64, Logical>) -> Option<usize> {
            self.windows
                .iter()
                .enumerate()
                .rev()
                .find(|(_, w)| w.contains(pt))
                .map(|(i, _)| i)
        }

        /// Bring window at `idx` to the top and focus keyboard+pointer on it.
        fn focus_window(&mut self, idx: usize) {
            // Rotate to top
            let win = self.windows.remove(idx);
            let surface = win.toplevel.wl_surface().clone();
            self.windows.push(win);

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
                    Some((surface, local)),
                    &MotionEvent {
                        location: self.pointer_pos,
                        serial,
                        time: 0,
                    },
                );
                ptr.frame(self);
            }
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

            let output_size: Size<i32, Physical> = Size::from((OUTPUT_W, OUTPUT_H));

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
                            DEFAULT_WIN_W
                        },
                        if st.size.map_or(0, |s| s.h) > 0 {
                            st.size.unwrap().h
                        } else {
                            DEFAULT_WIN_H
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
    }

    delegate_seat!(RetroCompositor);

    // -----------------------------------------------------------------------
    // SelectionHandler / DataDeviceHandler (required by delegate_data_device)
    // -----------------------------------------------------------------------

    impl SelectionHandler for RetroCompositor {
        type SelectionUserData = ();
    }

    impl DataDeviceHandler for RetroCompositor {
        fn data_device_state(&self) -> &DataDeviceState {
            &self.data_device_state
        }
    }

    impl ClientDndGrabHandler for RetroCompositor {}
    impl ServerDndGrabHandler for RetroCompositor {
        fn send(
            &mut self,
            _mime_type: String,
            _fd: std::os::unix::io::OwnedFd,
            _seat: Seat<Self>,
        ) {
        }
    }

    smithay::delegate_data_device!(RetroCompositor);

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
                state.size = Some(Size::from((DEFAULT_WIN_W, DEFAULT_WIN_H)));
                state.states.set(xdg_toplevel::State::Activated);
            });
            surface.send_configure();

            // Cascade new windows
            let offset = self.next_window_offset;
            self.next_window_offset = (offset + 32) % 256;
            let position = Point::from((64 + offset, 64 + offset));

            eprintln!(
                "[retro-compositor] surface mapped at ({},{})",
                position.x, position.y
            );

            self.windows.push(MappedWindow {
                toplevel: surface,
                position,
                size: Size::from((DEFAULT_WIN_W, DEFAULT_WIN_H)),
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
                    kb.set_focus(self, Some(surf), serial);
                }
            } else if let Some(kb) = self.seat.get_keyboard() {
                let serial = self.next_serial();
                kb.set_focus(self, None, serial);
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
        let output_size = Size::from((OUTPUT_W, OUTPUT_H));
        let pos = ev.position_transformed(output_size);
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

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub fn run() -> anyhow::Result<()> {
        tracing_subscriber::fmt::init();

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
        let output_manager_state =
            OutputManagerState::new_with_xdg_output::<RetroCompositor>(&display_handle);

        // Seat: keyboard + pointer
        let mut seat: Seat<RetroCompositor> =
            seat_state.new_wl_seat(&display_handle, "seat0");
        seat.add_keyboard(XkbConfig::default(), 200, 25)?;
        seat.add_pointer();

        // Output (OUTPUT_W x OUTPUT_H @ 60 Hz)
        let output = Output::new(
            "X11-1".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "RetroShell".into(),
                model: "X11 Output".into(),
            },
        );
        let mode = Mode {
            size: (OUTPUT_W, OUTPUT_H).into(),
            refresh: 60_000,
        };
        output.change_current_state(
            Some(mode),
            Some(Transform::Normal),
            None,
            Some((0, 0).into()),
        );
        output.set_preferred(mode);
        output.create_global::<RetroCompositor>(&display_handle);

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
            _loop_handle: loop_handle,
            _clock: clock,
            compositor_state,
            shm_state,
            seat_state,
            xdg_shell_state,
            data_device_state,
            _output_manager_state: output_manager_state,
            seat,
            _output: output,
            running: true,
            windows: Vec::new(),
            next_window_offset: 0,
            pointer_pos: Point::from((0.0_f64, 0.0_f64)),
            serial: 0,
            renderer,
            x11_surface,
        };

        tracing::info!("retro-compositor event loop starting");
        let mut frame_counter: u32 = 0;
        while state.running {
            display.flush_clients()?;
            // Dispatch with a 16 ms timeout → ~60 fps
            event_loop.dispatch(Some(Duration::from_millis(16)), &mut state)?;
            // After each dispatch tick, render a frame (so we hit ~60 fps
            // even when no X11 Refresh event arrives)
            frame_counter += 1;
            if frame_counter % 4 == 1 {
                // Render every 4 ticks (~15 fps steady state, spikes to 60 on X11 Refresh)
                state.render_frame();
            }
        }

        tracing::info!("retro-compositor exiting");
        Ok(())
    }
}
