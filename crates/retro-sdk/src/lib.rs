use retro_bus::RetroBus;
use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers, MouseButton};
use retro_kit::icon_view::{IconItem, IconView};
use retro_kit::label::Label;
use retro_kit::layout::{Layout, LayoutView};
use retro_kit::list_view::ListView;
use retro_kit::menu::{Menu, MenuItem, MenuItemKind};
use retro_kit::menu_bar::MenuBar;
use retro_kit::scroll_view::ScrollView;
use retro_kit::slider::Slider;
use retro_kit::progress_bar::ProgressBar;
use retro_kit::tab_view::TabView;
use retro_kit::dock_view::DockView;
use retro_kit::split_view::SplitView;
use retro_kit::status_bar::StatusBar;
use retro_kit::text_field::TextField;
use retro_kit::toolbar::Toolbar;
use retro_kit::tree_view::{TreeNode, TreeView};
use retro_kit::window::Window;
use retro_kit::{Color, LayoutConstraint, MonospaceView, Point, Rect, Size, Widget};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::util::DeviceExt;

static RENDER_DARK_MODE: AtomicBool = AtomicBool::new(false);

fn set_render_dark_mode(is_dark: bool) {
    RENDER_DARK_MODE.store(is_dark, Ordering::Relaxed);
}

fn render_dark_mode() -> bool {
    RENDER_DARK_MODE.load(Ordering::Relaxed)
}

fn load_dark_mode_preference() -> bool {
    let config_dir = std::env::var_os("RETROSHELL_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config/retroshell"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/retroshell"));
    let path = config_dir.join("settings.conf");
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    parse_dark_mode_preference(&content)
}

fn parse_dark_mode_preference(content: &str) -> bool {
    content.lines().any(|line| {
        let Some((key, value)) = line.split_once('=') else {
            return false;
        };
        key.trim() == "appearance" && value.trim().eq_ignore_ascii_case("dark")
    })
}

pub fn menu_manifest_dir() -> Option<PathBuf> {
    std::env::var_os("RETROSHELL_MENU_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(|runtime| PathBuf::from(runtime).join("retroshell").join("menus"))
        })
}

pub fn global_menu_mode_enabled() -> bool {
    std::env::var_os("RETROSHELL_GLOBAL_MENU")
        .and_then(|value| value.into_string().ok())
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn sanitize_manifest_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn ui(light: [f32; 4], dark: [f32; 4]) -> [f32; 4] {
    if render_dark_mode() {
        dark
    } else {
        light
    }
}

pub struct Application {
    pub name: String,
    pub bundle_id: String,
    pub main_window: Option<Window>,
    pub initial_size: Size,
    pub menus: Vec<Menu>,
    pub bus: Option<RetroBus>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuManifest {
    pub app_name: String,
    pub bundle_id: String,
    pub menus: Vec<Menu>,
    pub updated_at_millis: u64,
}

impl Application {
    pub fn new(name: &str, bundle_id: &str) -> Self {
        Self {
            name: name.to_string(),
            bundle_id: bundle_id.to_string(),
            main_window: None,
            initial_size: Size::new(960.0, 640.0),
            menus: vec![],
            bus: None,
            running: false,
        }
    }

    pub fn with_bus(mut self, bus: RetroBus) -> Self {
        self.bus = Some(bus);
        self
    }

    pub fn set_main_window(&mut self, window: Window) {
        self.main_window = Some(window);
    }

    pub fn set_initial_size(&mut self, size: Size) {
        self.initial_size = Size::new(size.width.max(1.0), size.height.max(1.0));
    }

    pub fn set_menus(&mut self, menus: Vec<Menu>) {
        self.menus = menus;
    }

    fn complete_menus(&self) -> Vec<Menu> {
        let mut menus = self.menus.clone();
        let mut app_menu = Menu::new(&self.name);
        app_menu.add_action(format!("About {}", self.name));
        app_menu.add_separator();
        app_menu.add_action(format!("Hide {}", self.name));
        app_menu.add_separator();
        app_menu.add_action(format!("Quit {}", self.name));
        menus.insert(0, app_menu);
        assign_default_menu_actions(&mut menus, &self.bundle_id);
        menus
    }

    pub fn menu_manifest(&self) -> MenuManifest {
        MenuManifest {
            app_name: self.name.clone(),
            bundle_id: self.bundle_id.clone(),
            menus: self.complete_menus(),
            updated_at_millis: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    pub fn publish_menu_manifest(&self) -> std::io::Result<Option<PathBuf>> {
        if self.menus.is_empty() {
            return Ok(None);
        }

        let Some(dir) = menu_manifest_dir() else {
            return Ok(None);
        };
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", sanitize_manifest_name(&self.bundle_id)));
        let json =
            serde_json::to_vec_pretty(&self.menu_manifest()).map_err(std::io::Error::other)?;
        fs::write(&path, json)?;
        Ok(Some(path))
    }

    fn attach_menu_bar(&mut self) {
        let Some(mut window) = self.main_window.take() else {
            return;
        };
        if self.menus.is_empty() {
            self.main_window = Some(window);
            return;
        }

        let menus = self.complete_menus();

        let content = window.content.take();
        let mut root = Layout::vertical(0.0);
        root.add(Box::new(MenuBar::new(menus)));
        if let Some(content) = content {
            root.add(content);
        }
        window.set_content(Box::new(LayoutView::new(root)));
        self.main_window = Some(window);
    }

    pub fn run(&mut self) {
        if let Err(err) = self.publish_menu_manifest() {
            tracing::warn!("failed to publish menu manifest: {err}");
        }
        if !global_menu_mode_enabled() {
            self.attach_menu_bar();
        }
        self.running = true;
        tracing::info!("Application '{}' started", self.name);

        let event_loop = retro_render::event_loop::RetroEventLoop::new();
        let main_window = self.main_window.take();

        struct AppHandler {
            name: String,
            window: Option<Window>,
            initial_size: Size,
            platform_window: Option<Arc<winit::window::Window>>,
            presenter: Option<WgpuPresenter>,
            modifiers: winit::keyboard::ModifiersState,
            cursor_position: Point,
            last_click: Option<(MouseButton, Point, std::time::Instant)>,
            dirty: bool,
            dark_mode: bool,
            scale: f32,
        }

        impl AppHandler {
            fn modifiers(&self) -> Modifiers {
                modifiers_from_winit(self.modifiers)
            }

            fn dispatch(&mut self, event: retro_kit::Event) -> retro_kit::EventResult {
                let result = if let Some(ref mut win) = self.window {
                    win.handle_event(&event)
                } else {
                    retro_kit::EventResult::Ignored
                };
                self.dirty = true;
                if let Some(window) = &self.platform_window {
                    window.request_redraw();
                }
                result
            }

            fn layout_window(&mut self, width: u32, height: u32) {
                if let Some(ref mut win) = self.window {
                    let logical_width = (width as f32 / self.scale).max(1.0);
                    let logical_height = (height as f32 / self.scale).max(1.0);
                    let size = Size::new(logical_width, logical_height);
                    win.set_rect(Rect::new(0.0, 0.0, size.width, size.height));
                    win.layout(LayoutConstraint::tight(size));
                    self.dirty = true;
                }
            }

            fn paint(&mut self) {
                let Some(window) = &self.window else {
                    return;
                };
                let Some(presenter) = &mut self.presenter else {
                    return;
                };
                set_render_dark_mode(self.dark_mode);
                let scale = self.scale;
                if let Err(err) = presenter.render(|canvas| {
                    canvas.width /= scale;
                    canvas.height /= scale;
                    draw_desktop_backdrop(canvas);
                    draw_window(canvas, window);
                }) {
                    tracing::error!("failed to render frame: {err}");
                } else {
                    self.dirty = false;
                }
            }
        }

        impl retro_render::event_loop::RetroAppHandler for AppHandler {
            fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
                let initial_size = self.initial_size;
                let mut attrs = winit::window::Window::default_attributes()
                    .with_title(&self.name)
                    .with_inner_size(winit::dpi::LogicalSize::new(
                        initial_size.width,
                        initial_size.height,
                    ));
                // FIXME: Hardcoded application name comparison ("RetroShell") to determine borderless state.
                // This property should ideally be configured dynamically via options in App.toml or manifest attributes.
                if self.name == "RetroShell" {
                    attrs = attrs.with_decorations(false);
                }

                match event_loop.create_window(attrs) {
                    Ok(window) => {
                        let window = Arc::new(window);
                        self.scale = window.scale_factor() as f32;
                        let size = window.inner_size();
                        match futures::executor::block_on(WgpuPresenter::new(window.clone())) {
                            Ok(presenter) => {
                                self.layout_window(size.width, size.height);
                                window.request_redraw();
                                self.presenter = Some(presenter);
                                self.platform_window = Some(window);
                            }
                            Err(err) => {
                                tracing::error!("failed to create presenter: {err}");
                                event_loop.exit();
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("failed to create application window: {err}");
                        event_loop.exit();
                    }
                }
            }

            fn handle_window_event(
                &mut self,
                event_loop: &winit::event_loop::ActiveEventLoop,
                event: winit::event::WindowEvent,
            ) {
                match event {
                    winit::event::WindowEvent::CloseRequested => event_loop.exit(),
                    winit::event::WindowEvent::RedrawRequested => self.paint(),
                    winit::event::WindowEvent::Resized(size) => {
                        if let Some(presenter) = &mut self.presenter {
                            presenter.resize(size.width, size.height);
                        }
                        self.layout_window(size.width, size.height);
                        if let Some(window) = &self.platform_window {
                            window.request_redraw();
                        }
                    }
                    winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        self.scale = scale_factor as f32;
                        let size_and_win = self
                            .platform_window
                            .as_ref()
                            .map(|w| (w.inner_size(), w.clone()));
                        if let Some((size, window)) = size_and_win {
                            if let Some(presenter) = &mut self.presenter {
                                presenter.resize(size.width, size.height);
                            }
                            self.layout_window(size.width, size.height);
                            window.request_redraw();
                        }
                    }
                    winit::event::WindowEvent::ModifiersChanged(new_mods) => {
                        self.modifiers = new_mods.state();
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        let scale = self.scale;
                        self.cursor_position =
                            Point::new(position.x as f32 / scale, position.y as f32 / scale);
                        let _ = self.dispatch(retro_kit::Event::MouseMove {
                            point: self.cursor_position,
                            modifiers: self.modifiers(),
                        });
                    }
                    winit::event::WindowEvent::CursorEntered { .. } => {
                        let _ = self.dispatch(retro_kit::Event::MouseEnter);
                    }
                    winit::event::WindowEvent::CursorLeft { .. } => {
                        let _ = self.dispatch(retro_kit::Event::MouseLeave);
                    }
                    winit::event::WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(button) = winit_to_retro_mouse_button(button) {
                            let event = match state {
                                winit::event::ElementState::Pressed => {
                                    let now = std::time::Instant::now();
                                    let is_double_click = self
                                        .last_click
                                        .as_ref()
                                        .map(|(last_button, last_point, last_time)| {
                                            *last_button == button
                                                && now.duration_since(*last_time)
                                                    <= std::time::Duration::from_millis(500)
                                                && distance_squared(
                                                    *last_point,
                                                    self.cursor_position,
                                                ) <= 16.0
                                        })
                                        .unwrap_or(false);
                                    self.last_click = Some((button, self.cursor_position, now));
                                    if is_double_click {
                                        retro_kit::Event::DoubleClick {
                                            button,
                                            point: self.cursor_position,
                                            modifiers: self.modifiers(),
                                        }
                                    } else {
                                        retro_kit::Event::MouseDown {
                                            button,
                                            point: self.cursor_position,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                }
                                winit::event::ElementState::Released => retro_kit::Event::MouseUp {
                                    button,
                                    point: self.cursor_position,
                                    modifiers: self.modifiers(),
                                },
                            };
                            let _ = self.dispatch(event);
                        }
                    }
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        let delta = winit_to_retro_scroll_delta(delta);
                        let _ = self.dispatch(retro_kit::Event::Scroll {
                            delta,
                            modifiers: self.modifiers(),
                        });
                    }
                    winit::event::WindowEvent::Focused(true) => {
                        let _ = self.dispatch(retro_kit::Event::FocusIn);
                    }
                    winit::event::WindowEvent::Focused(false) => {
                        let _ = self.dispatch(retro_kit::Event::FocusOut);
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        event: key_event, ..
                    } => {
                        let mut handled = false;
                        if let winit::keyboard::PhysicalKey::Code(phys_key) = key_event.physical_key
                        {
                            if let Some(rkey) = winit_to_retro_key(phys_key) {
                                let retro_event = match key_event.state {
                                    winit::event::ElementState::Pressed => {
                                        retro_kit::Event::KeyDown {
                                            key: rkey,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                    winit::event::ElementState::Released => {
                                        retro_kit::Event::KeyUp {
                                            key: rkey,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                };
                                handled = matches!(
                                    self.dispatch(retro_event),
                                    retro_kit::EventResult::Handled
                                        | retro_kit::EventResult::StopPropagation
                                );
                            }
                        }
                        if key_event.state == winit::event::ElementState::Pressed && !handled {
                            if let Some(ref text) = key_event.text {
                                for character in text.chars() {
                                    if !character.is_control() {
                                        let _ = self.dispatch(retro_kit::Event::Char { character });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
                let next_dark_mode = load_dark_mode_preference();
                if next_dark_mode != self.dark_mode {
                    self.dark_mode = next_dark_mode;
                    self.dirty = true;
                }
                if let Some(ref mut win) = self.window {
                    win.update();
                    self.dirty = true;
                }
                if self.dirty {
                    if let Some(window) = &self.platform_window {
                        window.request_redraw();
                    }
                }
            }
        }

        let mut handler = AppHandler {
            name: self.name.clone(),
            window: main_window,
            initial_size: self.initial_size,
            platform_window: None,
            presenter: None,
            modifiers: winit::keyboard::ModifiersState::default(),
            cursor_position: Point::ZERO,
            last_click: None,
            dirty: true,
            dark_mode: load_dark_mode_preference(),
            scale: 1.0,
        };
        if let Err(err) = event_loop.run(&mut handler) {
            tracing::error!("application event loop failed: {err}");
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
        tracing::info!("Application '{}' quit", self.name);
    }
}

pub trait AppDelegate {
    fn app_did_finish_launching(&mut self);
    fn app_will_terminate(&mut self);
    fn app_did_resign_active(&mut self);
    fn app_did_become_active(&mut self);
}

pub fn build_menu(title: &str) -> Menu {
    Menu::new(title)
}

fn assign_default_menu_actions(menus: &mut [Menu], bundle_id: &str) {
    for menu in menus {
        let menu_slug = action_slug(&menu.title);
        for item in &mut menu.items {
            if matches!(item.kind, MenuItemKind::Action) && item.action_id.is_empty() {
                item.action_id = format!("{bundle_id}.{}.{}", menu_slug, action_slug(&item.label));
            }
            if let Some(submenu) = &mut item.submenu {
                assign_default_menu_actions(std::slice::from_mut(submenu), bundle_id);
            }
        }
    }
}

fn action_slug(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for ch in label.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    if slug.is_empty() {
        "action".to_string()
    } else {
        slug
    }
}

pub fn menu_item(label: &str, action: &str) -> MenuItem {
    let mut item = MenuItem::action(label);
    item.with_action(action);
    item
}

pub fn separator() -> MenuItem {
    MenuItem::separator()
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

struct WgpuPresenter {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

impl WgpuPresenter {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(Default::default());
        let surface = instance
            .create_surface(window)
            .map_err(|err| format!("surface creation failed: {err}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "no compatible graphics adapter found".to_string())?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RetroSDK Device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|err| format!("device creation failed: {err}"))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RetroSDK Immediate UI Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VsOut {
    var out: VsOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RetroSDK Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RetroSDK Immediate UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[Vertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self, draw: impl FnOnce(&mut Canvas<'_>)) -> Result<(), String> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                self.surface
                    .get_current_texture()
                    .map_err(|err| format!("surface acquire failed after reconfigure: {err}"))?
            }
            Err(err) => return Err(format!("surface acquire failed: {err}")),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut canvas = Canvas::new(self.config.width as f32, self.config.height as f32);
        draw(&mut canvas);

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("RetroSDK Immediate UI Vertex Buffer"),
                contents: bytemuck::cast_slice(&canvas.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RetroSDK Frame Encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RetroSDK Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if !canvas.vertices.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw(0..canvas.vertices.len() as u32, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

struct Canvas<'a> {
    width: f32,
    height: f32,
    vertices: Vec<Vertex>,
    clip: Option<Rect>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Canvas<'a> {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            vertices: Vec::with_capacity(8192),
            clip: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn rect(&mut self, rect: Rect, color: [f32; 4]) {
        let mut x0 = rect.x.max(0.0);
        let mut y0 = rect.y.max(0.0);
        let mut x1 = (rect.x + rect.width).min(self.width);
        let mut y1 = (rect.y + rect.height).min(self.height);
        if let Some(clip) = self.clip {
            x0 = x0.max(clip.x);
            y0 = y0.max(clip.y);
            x1 = x1.min(clip.x + clip.width);
            y1 = y1.min(clip.y + clip.height);
        }
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let p0 = self.ndc(x0, y0);
        let p1 = self.ndc(x1, y0);
        let p2 = self.ndc(x1, y1);
        let p3 = self.ndc(x0, y1);
        self.vertices.extend_from_slice(&[
            Vertex {
                position: p0,
                color,
            },
            Vertex {
                position: p1,
                color,
            },
            Vertex {
                position: p2,
                color,
            },
            Vertex {
                position: p0,
                color,
            },
            Vertex {
                position: p2,
                color,
            },
            Vertex {
                position: p3,
                color,
            },
        ]);
    }

    fn stroke(&mut self, rect: Rect, color: [f32; 4]) {
        self.rect(Rect::new(rect.x, rect.y, rect.width, 1.0), color);
        self.rect(
            Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
            color,
        );
        self.rect(Rect::new(rect.x, rect.y, 1.0, rect.height), color);
        self.rect(
            Rect::new(rect.x + rect.width - 1.0, rect.y, 1.0, rect.height),
            color,
        );
    }

    fn text(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        let mut cursor_x = x;
        let mut cursor_y = y;
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                cursor_y += 12.0;
                continue;
            }
            cursor_x += self.glyph(ch, cursor_x, cursor_y, color);
        }
    }

    fn glyph(&mut self, ch: char, x: f32, y: f32, color: [f32; 4]) -> f32 {
        if let Some((data, w, h, advance)) = retro_render::rasterize_char(ch, 11.0) {
            for row in 0..h {
                for col in 0..w {
                    let idx = (row * w + col) as usize;
                    let alpha = data[idx] as f32 / 255.0;
                    if alpha > 0.05 {
                        let mut c = color;
                        c[3] *= alpha;
                        self.rect(Rect::new(x + col as f32, y + row as f32, 1.0, 1.0), c);
                    }
                }
            }
            advance.max(4.0)
        } else {
            for (row, bits) in glyph_pattern(ch).iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(Rect::new(x + col as f32, y + row as f32, 1.0, 1.0), color);
                    }
                }
            }
            7.0
        }
    }

    fn with_clip(&mut self, clip: Rect, draw: impl FnOnce(&mut Self)) {
        let old = self.clip;
        self.clip = Some(if let Some(old) = old {
            intersect_rect(old, clip).unwrap_or(Rect::ZERO)
        } else {
            clip
        });
        draw(self);
        self.clip = old;
    }

    fn ndc(&self, x: f32, y: f32) -> [f32; 2] {
        [(x / self.width) * 2.0 - 1.0, 1.0 - (y / self.height) * 2.0]
    }
}

fn draw_desktop_backdrop(canvas: &mut Canvas<'_>) {
    canvas.rect(
        Rect::new(0.0, 0.0, canvas.width, canvas.height),
        ui(rgb(152, 152, 148), rgb(26, 28, 30)),
    );
    let width = canvas.width as usize;
    let height = canvas.height as usize;
    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            let pattern_x = x / 4;
            let pattern_y = y / 4;
            let shade = match (pattern_x + pattern_y) % 4 {
                0 => {
                    if render_dark_mode() {
                        34
                    } else {
                        168
                    }
                }
                1 => {
                    if render_dark_mode() {
                        24
                    } else {
                        148
                    }
                }
                2 => {
                    if render_dark_mode() {
                        30
                    } else {
                        160
                    }
                }
                _ => {
                    if render_dark_mode() {
                        28
                    } else {
                        152
                    }
                }
            };
            let size = 2.0;
            canvas.rect(
                Rect::new(x as f32, y as f32, size, size),
                rgb(shade, shade, shade),
            );
            if x + 2 < width {
                canvas.rect(
                    Rect::new(x as f32 + 2.0, y as f32, size, size),
                    rgb(shade, shade, shade),
                );
            }
            if y + 2 < height {
                canvas.rect(
                    Rect::new(x as f32, y as f32 + 2.0, size, size),
                    rgb(shade, shade, shade),
                );
            }
            if x + 2 < width && y + 2 < height {
                canvas.rect(
                    Rect::new(x as f32 + 2.0, y as f32 + 2.0, size, size),
                    rgb(shade, shade, shade),
                );
            }
        }
    }
}

fn draw_window(canvas: &mut Canvas<'_>, window: &Window) {
    let rect = window.rect();
    if window.title() == "RetroShell Desktop" {
        canvas.rect(rect, rgb(152, 152, 148));
        draw_desktop_backdrop(canvas);
        for child in window.children() {
            draw_widget(canvas, child);
        }
        for child in window.children() {
            draw_menu_overlays(canvas, child);
        }
        return;
    }

    // Draw high-quality window drop shadows
    if window.is_active {
        for i in 1..=6 {
            let offset = i as f32 * 1.5;
            let alpha = 0.07 * (7 - i) as f32 / 6.0;
            canvas.rect(
                Rect::new(rect.x + offset, rect.y + offset, rect.width, rect.height),
                [0.0, 0.0, 0.0, alpha],
            );
        }
    } else {
        for i in 1..=3 {
            let offset = i as f32 * 1.0;
            let alpha = 0.04 * (4 - i) as f32 / 3.0;
            canvas.rect(
                Rect::new(rect.x + offset, rect.y + offset, rect.width, rect.height),
                [0.0, 0.0, 0.0, alpha],
            );
        }
    }

    canvas.rect(rect, ui(rgb(236, 235, 229), rgb(32, 34, 36)));
    draw_beveled_rect(canvas, rect, ui(rgb(238, 238, 232), rgb(38, 40, 42)), true);

    let titlebar = Rect::new(rect.x, rect.y, rect.width, 24.0);
    draw_classic_titlebar(canvas, titlebar, window.title(), window.is_active);

    canvas.with_clip(
        Rect::new(
            rect.x + 1.0,
            rect.y + 25.0,
            rect.width - 2.0,
            rect.height - 26.0,
        ),
        |canvas| {
            for child in window.children() {
                draw_widget(canvas, child);
            }
            for child in window.children() {
                draw_menu_overlays(canvas, child);
            }
        },
    );

    draw_resize_grow_box(canvas, rect);
}

fn draw_classic_titlebar(canvas: &mut Canvas<'_>, rect: Rect, title: &str, is_active: bool) {
    let titlebar_bg = ui(rgb(224, 224, 218), rgb(46, 48, 52));
    canvas.rect(rect, titlebar_bg);
    draw_beveled_rect(canvas, rect, titlebar_bg, true);

    if is_active {
        for y in (rect.y as i32 + 4..rect.y as i32 + rect.height as i32 - 4).step_by(3) {
            canvas.rect(
                Rect::new(rect.x + 26.0, y as f32, rect.width - 52.0, 1.0),
                ui(rgb(112, 112, 108), rgb(112, 116, 120)),
            );
            canvas.rect(
                Rect::new(rect.x + 26.0, y as f32 + 1.0, rect.width - 52.0, 1.0),
                ui(rgb(246, 246, 242), rgb(22, 24, 26)),
            );
        }

        let close_box = Rect::new(rect.x + 8.0, rect.y + 7.0, 11.0, 11.0);
        canvas.rect(close_box, ui(rgb(238, 238, 232), rgb(58, 60, 64)));
        canvas.stroke(close_box, ui(rgb(60, 60, 58), rgb(168, 170, 174)));
        canvas.rect(
            Rect::new(
                close_box.x + 2.0,
                close_box.y + 2.0,
                close_box.width - 4.0,
                1.0,
            ),
            ui(rgb(255, 255, 255), rgb(92, 94, 98)),
        );

        // Minimize box (classic Mac OS style dash)
        let minimize_box = Rect::new(rect.x + 22.0, rect.y + 7.0, 11.0, 11.0);
        canvas.rect(minimize_box, ui(rgb(238, 238, 232), rgb(58, 60, 64)));
        canvas.stroke(minimize_box, ui(rgb(60, 60, 58), rgb(168, 170, 174)));
        canvas.rect(
            Rect::new(
                minimize_box.x + 2.0,
                minimize_box.y + 5.0,
                minimize_box.width - 4.0,
                1.0,
            ),
            ui(rgb(255, 255, 255), rgb(92, 94, 98)),
        );

        let zoom_box = Rect::new(rect.x + rect.width - 19.0, rect.y + 7.0, 11.0, 11.0);
        canvas.rect(zoom_box, ui(rgb(238, 238, 232), rgb(58, 60, 64)));
        canvas.stroke(zoom_box, ui(rgb(60, 60, 58), rgb(168, 170, 174)));
        canvas.rect(
            Rect::new(
                zoom_box.x + 2.0,
                zoom_box.y + 2.0,
                zoom_box.width - 4.0,
                1.0,
            ),
            ui(rgb(255, 255, 255), rgb(92, 94, 98)),
        );
    }

    let title_width = title.len() as f32 * 7.0 + 18.0;
    let title_rect = Rect::new(
        rect.x + (rect.width - title_width) * 0.5,
        rect.y + 3.0,
        title_width,
        18.0,
    );
    canvas.rect(title_rect, titlebar_bg);

    let text_color = if is_active {
        ui(rgb(24, 24, 24), rgb(235, 235, 232))
    } else {
        ui(rgb(140, 140, 140), rgb(110, 112, 115))
    };
    canvas.text(title, title_rect.x + 9.0, rect.y + 8.0, text_color);

    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        ui(rgb(92, 92, 88), rgb(14, 16, 18)),
    );
}

fn draw_resize_grow_box(canvas: &mut Canvas<'_>, window_rect: Rect) {
    let box_rect = Rect::new(
        window_rect.x + window_rect.width - 16.0,
        window_rect.y + window_rect.height - 16.0,
        15.0,
        15.0,
    );
    canvas.rect(box_rect, ui(rgb(232, 232, 226), rgb(50, 52, 54)));
    canvas.stroke(box_rect, ui(rgb(132, 132, 126), rgb(148, 150, 152)));

    for offset in [4.0, 8.0, 12.0] {
        canvas.rect(
            Rect::new(box_rect.x + offset, box_rect.y + 13.0, 1.0, 1.0),
            ui(rgb(82, 82, 78), rgb(180, 182, 184)),
        );
        canvas.rect(
            Rect::new(box_rect.x + 13.0, box_rect.y + offset, 1.0, 1.0),
            ui(rgb(82, 82, 78), rgb(180, 182, 184)),
        );
        canvas.rect(
            Rect::new(box_rect.x + offset, box_rect.y + offset, 1.0, 1.0),
            ui(rgb(164, 164, 158), rgb(84, 86, 88)),
        );
    }
}

fn draw_widget(canvas: &mut Canvas<'_>, widget: &dyn Widget) {
    let rect = widget.rect();
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    if let Some(window) = widget.as_any().downcast_ref::<Window>() {
        draw_window(canvas, window);
        return;
    }

    if let Some(label) = widget.as_any().downcast_ref::<Label>() {
        canvas.text(
            &label.text,
            rect.x + 2.0,
            rect.y + 5.0,
            ui(rgb(24, 24, 24), rgb(226, 226, 222)),
        );
    } else if let Some(button) = widget.as_any().downcast_ref::<Button>() {
        if rect.height <= 24.0 {
            canvas.text(
                button.label(),
                rect.x + 8.0,
                rect.y + 7.0,
                ui(rgb(8, 8, 8), rgb(232, 232, 228)),
            );
            return;
        }
        let bg = if button.widget_state().hovered {
            ui(rgb(226, 235, 246), rgb(70, 76, 84))
        } else {
            ui(rgb(222, 222, 218), rgb(58, 60, 64))
        };
        canvas.rect(rect, bg);
        draw_beveled_rect(canvas, rect, bg, true);
        canvas.text(
            button.label(),
            rect.x + 12.0,
            rect.y + 9.0,
            ui(rgb(20, 20, 20), rgb(236, 236, 232)),
        );
    } else if let Some(text_field) = widget.as_any().downcast_ref::<TextField>() {
        canvas.rect(rect, ui(rgb(255, 255, 252), rgb(18, 20, 22)));
        canvas.stroke(rect, ui(rgb(115, 115, 110), rgb(120, 124, 128)));
        let text = if text_field.text().is_empty() {
            &text_field.placeholder
        } else {
            text_field.text()
        };
        canvas.text(
            text,
            rect.x + 6.0,
            rect.y + 8.0,
            ui(rgb(25, 25, 25), rgb(232, 232, 228)),
        );
    } else if let Some(slider) = widget.as_any().downcast_ref::<Slider>() {
        let track = Rect::new(
            rect.x + 9.0,
            rect.y + rect.height * 0.5 - 3.0,
            rect.width - 18.0,
            6.0,
        );
        canvas.rect(track, ui(rgb(196, 196, 190), rgb(30, 32, 34)));
        canvas.stroke(track, ui(rgb(104, 104, 98), rgb(112, 114, 116)));
        let filled = Rect::new(
            track.x + 1.0,
            track.y + 1.0,
            (track.width - 2.0) * slider.normalized_value(),
            track.height - 2.0,
        );
        canvas.rect(filled, ui(rgb(92, 122, 176), rgb(120, 150, 208)));
        let thumb_x = track.x + track.width * slider.normalized_value() - 5.0;
        let thumb = Rect::new(thumb_x, rect.y + 3.0, 10.0, rect.height - 6.0);
        let thumb_bg = if slider.dragging {
            ui(rgb(236, 240, 246), rgb(78, 84, 92))
        } else {
            ui(rgb(226, 226, 220), rgb(58, 60, 64))
        };
        canvas.rect(thumb, thumb_bg);
        draw_beveled_rect(canvas, thumb, thumb_bg, true);
    } else if let Some(tree) = widget.as_any().downcast_ref::<TreeView>() {
        draw_tree(canvas, rect, tree);
    } else if let Some(icon_view) = widget.as_any().downcast_ref::<IconView>() {
        draw_icon_view(canvas, icon_view);
    } else if let Some(list) = widget.as_any().downcast_ref::<ListView>() {
        draw_list(canvas, rect, list);
    } else if let Some(menu_bar) = widget.as_any().downcast_ref::<MenuBar>() {
        draw_menu_bar_widget(canvas, rect, menu_bar);
        return;
    } else if let Some(toolbar) = widget.as_any().downcast_ref::<Toolbar>() {
        if rect.y <= 1.0 && rect.width > 500.0 {
            draw_menu_bar(canvas, rect, toolbar);
        } else {
            canvas.rect(rect, ui(rgb(218, 218, 214), rgb(42, 44, 46)));
            canvas.rect(
                Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
                ui(rgb(145, 145, 140), rgb(92, 94, 96)),
            );
            for child in toolbar.children() {
                draw_widget(canvas, child);
            }
        }
        return;
    } else if let Some(scroll) = widget.as_any().downcast_ref::<ScrollView>() {
        canvas.rect(rect, ui(rgb(248, 248, 244), rgb(28, 30, 32)));
        canvas.stroke(rect, ui(rgb(160, 160, 154), rgb(92, 94, 96)));
        canvas.with_clip(rect, |canvas| {
            for child in scroll.children() {
                draw_widget(canvas, child);
            }
        });
        return;
    } else if widget.as_any().is::<SplitView>() {
        canvas.rect(rect, ui(rgb(230, 230, 225), rgb(34, 36, 38)));
        if let Some(split) = widget.as_any().downcast_ref::<SplitView>() {
            let divider = match split.direction {
                retro_kit::split_view::SplitDirection::Horizontal => Rect::new(
                    rect.x + rect.width * split.divider_position,
                    rect.y,
                    split.divider_size,
                    rect.height,
                ),
                retro_kit::split_view::SplitDirection::Vertical => Rect::new(
                    rect.x,
                    rect.y + rect.height * split.divider_position,
                    rect.width,
                    split.divider_size,
                ),
            };
            canvas.rect(divider, ui(rgb(180, 180, 176), rgb(70, 72, 74)));
            canvas.stroke(divider, ui(rgb(110, 110, 106), rgb(112, 114, 116)));
        }
    } else if let Some(grid) = widget.as_any().downcast_ref::<MonospaceView>() {
        draw_monospace_view(canvas, rect, grid);
        return;
    } else if let Some(status) = widget.as_any().downcast_ref::<StatusBar>() {
        canvas.rect(rect, ui(rgb(220, 220, 216), rgb(36, 38, 40)));
        canvas.rect(
            Rect::new(rect.x, rect.y, rect.width, 1.0),
            ui(rgb(150, 150, 145), rgb(96, 98, 100)),
        );
        let mut x = rect.x + 8.0;
        for item in &status.items {
            canvas.text(
                &item.text,
                x,
                rect.y + 8.0,
                ui(rgb(35, 35, 35), rgb(228, 228, 224)),
            );
            x += item.width.max(item.text.len() as f32 * 7.0 + 12.0);
        }
    } else if let Some(pb) = widget.as_any().downcast_ref::<ProgressBar>() {
        draw_progress_bar(canvas, rect, pb);
        return;
    } else if let Some(tv) = widget.as_any().downcast_ref::<TabView>() {
        draw_tab_view(canvas, rect, tv);
        return;
    } else if let Some(dock) = widget.as_any().downcast_ref::<DockView>() {
        draw_dock_view(canvas, rect, dock);
        return;
    } else if let Some(layout_view) = widget.as_any().downcast_ref::<LayoutView>() {
        draw_layout(canvas, &layout_view.layout);
        return;
    }

    for child in widget.children() {
        draw_widget(canvas, child);
    }
    for child in widget.children() {
        if let Some(menu_bar) = child.as_any().downcast_ref::<MenuBar>() {
            if menu_bar.open_menu.is_some() {
                draw_menu_bar_widget(canvas, menu_bar.rect(), menu_bar);
            }
        }
    }
}

fn draw_progress_bar(canvas: &mut Canvas<'_>, rect: Rect, pb: &ProgressBar) {
    canvas.rect(rect, ui(rgb(236, 236, 232), rgb(24, 26, 28)));
    canvas.stroke(rect, ui(rgb(145, 145, 140), rgb(92, 94, 96)));
    let ratio = if pb.max > 0.0 { pb.value / pb.max } else { 0.0 };
    let fill_width = (rect.width - 4.0) * ratio.clamp(0.0, 1.0);
    if fill_width > 0.0 {
        let fill = Rect::new(rect.x + 2.0, rect.y + 2.0, fill_width, rect.height - 4.0);
        canvas.rect(fill, ui(rgb(90, 140, 220), rgb(110, 160, 240)));
    }
}

fn draw_tab_view(canvas: &mut Canvas<'_>, rect: Rect, tv: &TabView) {
    let header_height = 30.0;
    let divider_y = rect.y + header_height - 1.0;
    canvas.rect(
        Rect::new(rect.x, divider_y, rect.width, 1.0),
        ui(rgb(150, 150, 145), rgb(96, 98, 100)),
    );
    let mut current_x = rect.x + 8.0;
    for (i, tab) in tv.tabs.iter().enumerate() {
        let tab_width = tab.title.len() as f32 * 7.0 + 24.0;
        let tab_rect = Rect::new(current_x, rect.y + 4.0, tab_width, 25.0);
        let is_selected = tv.selected_tab_index == i;
        if is_selected {
            canvas.rect(tab_rect, ui(rgb(236, 235, 229), rgb(48, 50, 52)));
            draw_beveled_rect(canvas, tab_rect, ui(rgb(238, 238, 232), rgb(52, 54, 56)), true);
            canvas.rect(
                Rect::new(tab_rect.x + 1.0, divider_y, tab_rect.width - 2.0, 1.0),
                ui(rgb(238, 238, 232), rgb(52, 54, 56)),
            );
        } else {
            let inactive_bg = ui(rgb(210, 210, 204), rgb(32, 34, 36));
            canvas.rect(tab_rect, inactive_bg);
            draw_beveled_rect(canvas, tab_rect, inactive_bg, false);
        }
        canvas.text(
            &tab.title,
            tab_rect.x + 12.0,
            tab_rect.y + 8.0,
            if is_selected {
                ui(rgb(24, 24, 24), rgb(240, 240, 235))
            } else {
                ui(rgb(100, 100, 95), rgb(140, 140, 135))
            },
        );
        current_x += tab_width + 4.0;
    }
    if let Some(content) = tv.selected_content() {
        draw_widget(canvas, content);
    }
}

fn draw_dock_view(canvas: &mut Canvas<'_>, rect: Rect, dock: &DockView) {
    if dock.items.is_empty() {
        return;
    }
    
    let item_size = 48.0;
    let padding = 8.0;
    let item_spacing = 6.0;
    let total_width = dock.items.len() as f32 * (item_size + item_spacing) - item_spacing + padding * 2.0;
    
    let dock_x = rect.x + (rect.width - total_width) * 0.5;
    let dock_y = rect.y + rect.height - item_size - padding * 2.0;
    let dock_rect = Rect::new(dock_x, dock_y, total_width, item_size + padding * 2.0);
    
    let bg_color = ui(rgb(230, 230, 226), rgb(28, 30, 32));
    canvas.rect(dock_rect, bg_color);
    draw_beveled_rect(canvas, dock_rect, bg_color, true);
    
    let mut current_x = dock_x + padding;
    for item in &dock.items {
        let item_rect = Rect::new(current_x, dock_y + padding, item_size, item_size);
        
        if item.is_focused {
            let highlight_rect = Rect::new(item_rect.x - 2.0, item_rect.y - 2.0, item_rect.width + 4.0, item_rect.height + 4.0);
            canvas.rect(highlight_rect, ui(rgb(180, 200, 240), rgb(60, 80, 120)));
            draw_beveled_rect(canvas, highlight_rect, ui(rgb(180, 200, 240), rgb(60, 80, 120)), false);
        }
        
        let icon_bg = ui(rgb(250, 250, 246), rgb(44, 46, 50));
        canvas.rect(item_rect, icon_bg);
        draw_beveled_rect(canvas, item_rect, icon_bg, true);
        
        let symbol_x = item_rect.x + (item_size - 32.0) * 0.5;
        let symbol_y = item_rect.y + (item_size - 32.0) * 0.5 - 2.0;
        
        match item.label.as_str() {
            "Finder" => draw_app_icon(canvas, symbol_x - 6.0, symbol_y - 6.0),
            "Settings" => draw_drive_icon(canvas, symbol_x - 6.0, symbol_y - 6.0),
            "TextEdit" => draw_document_icon(canvas, symbol_x - 6.0, symbol_y - 6.0),
            "Trash" => draw_trash_icon(canvas, symbol_x - 6.0, symbol_y - 6.0),
            _ => draw_app_icon(canvas, symbol_x - 6.0, symbol_y - 6.0),
        }
        
        if item.is_running {
            canvas.rect(
                Rect::new(item_rect.x + item_rect.width * 0.5 - 2.0, item_rect.y + item_rect.height - 5.0, 4.0, 4.0),
                ui(rgb(60, 60, 55), rgb(200, 200, 195)),
            );
        }
        
        current_x += item_size + item_spacing;
    }
}

fn draw_layout(canvas: &mut Canvas<'_>, layout: &Layout) {
    match layout {
        Layout::Horizontal { children, .. }
        | Layout::Vertical { children, .. }
        | Layout::Grid { children, .. }
        | Layout::Stack { children }
        | Layout::Overlay { children } => {
            for child in children {
                draw_widget(canvas, child.as_ref());
            }
            for child in children {
                if child
                    .as_any()
                    .downcast_ref::<MenuBar>()
                    .is_some_and(|menu_bar| menu_bar.open_menu.is_some())
                {
                    draw_widget(canvas, child.as_ref());
                }
                draw_menu_overlays(canvas, child.as_ref());
            }
        }
    }
}

fn draw_menu_overlays(canvas: &mut Canvas<'_>, widget: &dyn Widget) {
    if let Some(menu_bar) = widget.as_any().downcast_ref::<MenuBar>() {
        if menu_bar.open_menu.is_some() {
            draw_menu_bar_widget(canvas, menu_bar.rect(), menu_bar);
        }
    }
    for child in widget.children() {
        draw_menu_overlays(canvas, child);
    }
}

fn draw_menu_bar(canvas: &mut Canvas<'_>, rect: Rect, toolbar: &Toolbar) {
    canvas.rect(rect, ui(rgb(238, 238, 238), rgb(28, 30, 32)));
    canvas.rect(
        Rect::new(rect.x, rect.y, rect.width, 1.0),
        ui(rgb(255, 255, 255), rgb(58, 60, 62)),
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 2.0, rect.width, 1.0),
        ui(rgb(95, 95, 95), rgb(92, 94, 96)),
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        ui(rgb(35, 35, 35), rgb(8, 10, 12)),
    );

    let mut x = rect.x + 10.0;
    draw_apple_icon(canvas, x + 1.0, rect.y + 7.0, false);
    x += 18.0;

    for child in toolbar.children() {
        if let Some(button) = child.as_any().downcast_ref::<Button>() {
            let label = button.label();
            canvas.text(label, x, rect.y + 8.0, ui(rgb(8, 8, 8), rgb(232, 232, 228)));
            x += label.len() as f32 * 8.0 + 18.0;
        }
    }

    let clock = current_time_string();
    canvas.text(
        &clock,
        rect.x + rect.width - clock.len() as f32 * 7.0 - 72.0,
        rect.y + 8.0,
        ui(rgb(8, 8, 8), rgb(232, 232, 228)),
    );
    draw_status_glyph(canvas, rect.x + rect.width - 42.0, rect.y + 7.0);
    draw_status_glyph(canvas, rect.x + rect.width - 22.0, rect.y + 7.0);
}

fn draw_menu_bar_widget(canvas: &mut Canvas<'_>, rect: Rect, menu_bar: &MenuBar) {
    canvas.rect(rect, ui(rgb(238, 238, 238), rgb(28, 30, 32)));
    canvas.rect(
        Rect::new(rect.x, rect.y, rect.width, 1.0),
        ui(rgb(255, 255, 255), rgb(58, 60, 62)),
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 2.0, rect.width, 1.0),
        ui(rgb(95, 95, 95), rgb(92, 94, 96)),
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        ui(rgb(35, 35, 35), rgb(8, 10, 12)),
    );

    for (index, menu) in menu_bar.menus.iter().enumerate() {
        let Some(menu_rect) = menu_bar.menu_rects().get(index).copied() else {
            continue;
        };
        let active = menu_bar.open_menu == Some(index) || menu_bar.hovered_menu == Some(index);
        if active {
            canvas.rect(
                Rect::new(
                    menu_rect.x + 1.0,
                    menu_rect.y + 2.0,
                    menu_rect.width - 2.0,
                    20.0,
                ),
                ui(rgb(24, 24, 24), rgb(78, 86, 98)),
            );
        }
        if index == 0 {
            draw_apple_icon(canvas, menu_rect.x + 4.0, menu_rect.y + 7.0, active);
            canvas.text(
                &menu.title,
                menu_rect.x + 18.0,
                menu_rect.y + 8.0,
                if active {
                    rgb(255, 255, 255)
                } else {
                    ui(rgb(8, 8, 8), rgb(232, 232, 228))
                },
            );
        } else {
            canvas.text(
                &menu.title,
                menu_rect.x + 8.0,
                menu_rect.y + 8.0,
                if active {
                    rgb(255, 255, 255)
                } else {
                    ui(rgb(8, 8, 8), rgb(232, 232, 228))
                },
            );
        }
    }

    let clock = current_time_string();
    canvas.text(
        &clock,
        rect.x + rect.width - clock.len() as f32 * 7.0 - 72.0,
        rect.y + 8.0,
        ui(rgb(8, 8, 8), rgb(232, 232, 228)),
    );
    draw_status_glyph(canvas, rect.x + rect.width - 42.0, rect.y + 7.0);
    draw_status_glyph(canvas, rect.x + rect.width - 22.0, rect.y + 7.0);

    if let Some(menu_index) = menu_bar.open_menu {
        draw_open_menu(canvas, menu_bar, menu_index);
    }
}

fn draw_apple_icon(canvas: &mut Canvas<'_>, x: f32, y: f32, active: bool) {
    let color = if active {
        rgb(255, 255, 255)
    } else {
        ui(rgb(22, 22, 22), rgb(232, 232, 228))
    };
    canvas.rect(Rect::new(x + 1.0, y + 1.0, 5.0, 5.0), color);
    canvas.rect(Rect::new(x + 1.0, y + 2.0, 1.0, 3.0), color);
    canvas.rect(Rect::new(x + 5.0, y + 2.0, 1.0, 3.0), color);
    canvas.rect(Rect::new(x + 2.0, y, 3.0, 1.0), color);
    canvas.rect(Rect::new(x + 1.0, y + 6.0, 1.0, 1.0), color);
    canvas.rect(Rect::new(x + 5.0, y + 6.0, 1.0, 1.0), color);
    canvas.rect(
        Rect::new(x + 2.0, y + 6.0, 3.0, 1.0),
        if active {
            rgb(22, 22, 22)
        } else {
            ui(rgb(238, 238, 238), rgb(28, 30, 32))
        },
    );
}

fn draw_open_menu(canvas: &mut Canvas<'_>, menu_bar: &MenuBar, menu_index: usize) {
    let Some(menu) = menu_bar.menus.get(menu_index) else {
        return;
    };
    let Some(dropdown) = menu_bar.dropdown_rect(menu_index) else {
        return;
    };

    canvas.rect(
        Rect::new(
            dropdown.x + 3.0,
            dropdown.y + 3.0,
            dropdown.width,
            dropdown.height,
        ),
        rgba(0, 0, 0, 0.24),
    );
    draw_beveled_rect(
        canvas,
        dropdown,
        ui(rgb(244, 244, 238), rgb(42, 44, 48)),
        true,
    );
    canvas.rect(
        Rect::new(
            dropdown.x + 4.0,
            dropdown.y + 4.0,
            dropdown.width - 8.0,
            1.0,
        ),
        ui(rgb(255, 255, 255), rgb(66, 68, 72)),
    );
    canvas.rect(
        Rect::new(
            dropdown.x + 4.0,
            dropdown.y + 4.0,
            1.0,
            dropdown.height - 8.0,
        ),
        ui(rgb(255, 255, 255), rgb(66, 68, 72)),
    );

    for (item_index, item) in menu.items.iter().enumerate() {
        let Some(item_rect) = menu_bar.item_rect(menu_index, item_index) else {
            continue;
        };
        if matches!(item.kind, MenuItemKind::Separator) {
            canvas.rect(
                Rect::new(
                    item_rect.x + 12.0,
                    item_rect.y + 9.0,
                    item_rect.width - 24.0,
                    1.0,
                ),
                ui(rgb(120, 120, 116), rgb(116, 118, 122)),
            );
            canvas.rect(
                Rect::new(
                    item_rect.x + 12.0,
                    item_rect.y + 10.0,
                    item_rect.width - 24.0,
                    1.0,
                ),
                ui(rgb(255, 255, 255), rgb(28, 30, 32)),
            );
            continue;
        }

        let hovered = menu_bar.hovered_item == Some(item_index);
        if hovered && item.enabled {
            canvas.rect(item_rect, ui(rgb(22, 22, 22), rgb(82, 90, 104)));
        }
        let text_color = if !item.enabled {
            ui(rgb(132, 132, 128), rgb(116, 118, 120))
        } else if hovered {
            rgb(255, 255, 255)
        } else {
            ui(rgb(8, 8, 8), rgb(232, 232, 228))
        };
        match item.kind {
            MenuItemKind::Checkbox if item.checked => {
                canvas.text("✓", item_rect.x + 8.0, item_rect.y + 7.0, text_color);
            }
            MenuItemKind::Radio if item.checked => {
                canvas.rect(
                    Rect::new(item_rect.x + 10.0, item_rect.y + 8.0, 5.0, 5.0),
                    text_color,
                );
            }
            _ => {}
        }
        canvas.text(
            &item.label,
            item_rect.x + 24.0,
            item_rect.y + 7.0,
            text_color,
        );
        if let Some((key, modifiers)) = item.shortcut {
            let shortcut = shortcut_label(key, modifiers);
            canvas.text(
                &shortcut,
                item_rect.x + item_rect.width - shortcut.len() as f32 * 7.0 - 8.0,
                item_rect.y + 7.0,
                text_color,
            );
        }
    }
}

fn shortcut_label(key: KeyCode, modifiers: Modifiers) -> String {
    let mut parts = Vec::new();
    if modifiers.control {
        parts.push("Ctrl".to_string());
    }
    if modifiers.alt {
        parts.push("Alt".to_string());
    }
    if modifiers.shift {
        parts.push("Shift".to_string());
    }
    if modifiers.meta {
        parts.push("Cmd".to_string());
    }
    parts.push(key_label(key).to_string());
    parts.join("+")
}

fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::A => "A",
        KeyCode::B => "B",
        KeyCode::C => "C",
        KeyCode::D => "D",
        KeyCode::E => "E",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::H => "H",
        KeyCode::I => "I",
        KeyCode::J => "J",
        KeyCode::K => "K",
        KeyCode::L => "L",
        KeyCode::M => "M",
        KeyCode::N => "N",
        KeyCode::O => "O",
        KeyCode::P => "P",
        KeyCode::Q => "Q",
        KeyCode::R => "R",
        KeyCode::S => "S",
        KeyCode::T => "T",
        KeyCode::U => "U",
        KeyCode::V => "V",
        KeyCode::W => "W",
        KeyCode::X => "X",
        KeyCode::Y => "Y",
        KeyCode::Z => "Z",
        KeyCode::Backspace => "Del",
        KeyCode::Escape => "Esc",
        KeyCode::Enter => "Ret",
        KeyCode::Space => "Space",
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",
        _ => "?",
    }
}

fn current_time_string() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format_clock_from_seconds(duration.as_secs())
}

fn format_clock_from_seconds(seconds_since_epoch: u64) -> String {
    let local_secs = seconds_since_epoch as i64;
    let minutes = (local_secs / 60).rem_euclid(60);
    let hours_24 = (local_secs / 3600).rem_euclid(24);
    let hour_12 = match hours_24 % 12 {
        0 => 12,
        h => h,
    };
    let am_pm = if hours_24 < 12 { "AM" } else { "PM" };
    format!("{}:{:02} {}", hour_12, minutes, am_pm)
}

fn draw_status_glyph(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x, y, 13.0, 13.0),
        rgb(220, 220, 216),
        true,
    );
    canvas.rect(Rect::new(x + 4.0, y + 3.0, 5.0, 7.0), rgb(78, 92, 132));
    canvas.rect(Rect::new(x + 5.0, y + 4.0, 3.0, 5.0), rgb(176, 194, 222));
}

fn draw_beveled_rect(canvas: &mut Canvas<'_>, rect: Rect, fill: [f32; 4], raised: bool) {
    canvas.rect(rect, fill);
    let light = if raised {
        rgb(255, 255, 255)
    } else {
        rgb(72, 72, 72)
    };
    let mid = if raised {
        rgb(190, 190, 186)
    } else {
        rgb(132, 132, 128)
    };
    let dark = if raised {
        rgb(74, 74, 72)
    } else {
        rgb(255, 255, 255)
    };
    canvas.rect(Rect::new(rect.x, rect.y, rect.width, 1.0), light);
    canvas.rect(Rect::new(rect.x, rect.y, 1.0, rect.height), light);
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, 1.0),
        mid,
    );
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, 1.0, rect.height - 2.0),
        mid,
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        dark,
    );
    canvas.rect(
        Rect::new(rect.x + rect.width - 1.0, rect.y, 1.0, rect.height),
        dark,
    );
}

fn draw_tree(canvas: &mut Canvas<'_>, rect: Rect, tree: &TreeView) {
    canvas.rect(rect, ui(rgb(222, 226, 230), rgb(30, 33, 36)));
    canvas.stroke(rect, ui(rgb(145, 150, 154), rgb(92, 96, 100)));
    let mut y = rect.y + 8.0;
    for (index, node) in tree.roots.iter().enumerate() {
        draw_tree_node(
            canvas,
            node,
            &tree.selected_path,
            &[index],
            rect.x + 10.0,
            &mut y,
            0,
        );
    }
}

fn draw_tree_node(
    canvas: &mut Canvas<'_>,
    node: &TreeNode,
    selected_path: &Option<Vec<usize>>,
    path: &[usize],
    x: f32,
    y: &mut f32,
    depth: usize,
) {
    let selected = selected_path
        .as_ref()
        .is_some_and(|selected| selected == path);
    if selected {
        canvas.rect(
            Rect::new(x - 4.0, *y - 3.0, 170.0, 16.0),
            ui(rgb(64, 111, 171), rgb(82, 98, 126)),
        );
    }
    canvas.text(
        &node.label,
        x + depth as f32 * 12.0,
        *y,
        if selected {
            rgb(255, 255, 255)
        } else {
            ui(rgb(30, 30, 30), rgb(226, 226, 222))
        },
    );
    *y += 18.0;
    if node.expanded {
        for (index, child) in node.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index);
            draw_tree_node(canvas, child, selected_path, &child_path, x, y, depth + 1);
        }
    }
}

/// Truncates a string label to a maximum length, preserving file extensions if possible.
///
/// # Assumptions:
/// - **FIXME**: Characters are assumed to have a fixed layout width (7px width spacing inside `Canvas`).
///   This function only checks character length (`label.len()`) rather than visual bounding boxes.
fn truncate_label(label: &str, max_len: usize) -> String {
    if label.len() <= max_len {
        return label.to_string();
    }
    if max_len <= 4 {
        return format!("{}...", &label[..max_len.max(3) - 3]);
    }
    if let Some(pos) = label.rfind('.') {
        let ext = &label[pos..];
        if ext.len() < max_len - 3 {
            let base_len = max_len - 3 - ext.len();
            return format!("{}...{}", &label[..base_len], ext);
        }
    }
    format!("{}...", &label[..max_len - 3])
}

/// Renders the `IconView` grid.
///
/// # Limitations:
/// - **FIXME**: The current renderer uses the built-in system pixel font, which only supports
///   uppercase characters (lower-case is automatically mapped to upper-case by the rasterizer).
fn draw_icon_view(canvas: &mut Canvas<'_>, icon_view: &IconView) {
    let rect = icon_view.rect();
    let is_desktop = rect.width >= 600.0
        && rect.height >= 360.0
        && icon_view.items.iter().any(|item| item.label == "Hard Disk")
        && icon_view.items.iter().any(|item| item.label == "Trash");
    if is_desktop {
        canvas.with_clip(rect, draw_desktop_backdrop);
    } else {
        canvas.rect(rect, rgb(248, 248, 244));
    }
    for item in &icon_view.items {
        let display_label = truncate_label(&item.label, 12);
        if item.selected {
            let sel_rect = Rect::new(
                item.rect.x - 6.0,
                item.rect.y - 6.0,
                item.rect.width + 12.0,
                icon_view.icon_size + 32.0,
            );
            draw_selection_highlight(canvas, sel_rect);
        }
        draw_desktop_icon(canvas, item);
        let label_y = item.rect.y + icon_view.icon_size + 6.0;
        canvas.text(
            &display_label,
            item.rect.x + (item.rect.width - display_label.len() as f32 * 6.0) * 0.5,
            label_y,
            if item.selected {
                rgb(255, 255, 255)
            } else {
                rgb(20, 20, 20)
            },
        );
    }
}

fn draw_selection_highlight(canvas: &mut Canvas<'_>, rect: Rect) {
    canvas.rect(rect, rgb(64, 111, 171));
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, 1.0),
        rgb(120, 160, 220),
    );
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, 1.0, rect.height - 2.0),
        rgb(120, 160, 220),
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        rgb(40, 70, 130),
    );
    canvas.rect(
        Rect::new(rect.x + rect.width - 1.0, rect.y, 1.0, rect.height),
        rgb(40, 70, 130),
    );
}

fn draw_monospace_view(canvas: &mut Canvas<'_>, rect: Rect, grid: &MonospaceView) {
    canvas.rect(rect, rgb(12, 12, 12));
    canvas.stroke(rect, rgb(90, 90, 86));
    let cols = grid.cols;
    let rows = grid.rows;
    for row in 0..rows {
        for col in 0..cols {
            let idx = row * cols + col;
            let Some(cell) = grid.cells.get(idx) else {
                continue;
            };
            let x = rect.x + col as f32 * grid.cell_width;
            let y = rect.y + row as f32 * grid.cell_height;
            if cell.bg[3] > 0.0 {
                canvas.rect(Rect::new(x, y, grid.cell_width, grid.cell_height), cell.bg);
            }
            if cell.ch != ' ' {
                canvas.glyph(cell.ch, x + 1.0, y + 4.0, cell.fg);
            }
        }
    }
}

fn draw_desktop_icon(canvas: &mut Canvas<'_>, item: &IconItem) {
    let x = item.rect.x + 9.0;
    let y = item.rect.y + 3.0;
    match item.label.as_str() {
        "Hard Disk" | "Home" => draw_drive_icon(canvas, x, y),
        "Trash" => draw_trash_icon(canvas, x + 5.0, y),
        "Applications" => draw_folder_icon(canvas, x, y, rgb(226, 216, 142)),
        _ => {
            if item.icon.as_deref() == Some("folder") {
                draw_folder_icon(canvas, x, y, rgb(226, 216, 142));
            } else if item.icon.as_deref() == Some("document") {
                draw_document_icon(canvas, x, y);
            } else {
                draw_app_icon(canvas, x, y);
            }
        }
    }
}

fn draw_document_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    // Page body
    draw_beveled_rect(
        canvas,
        Rect::new(x + 8.0, y + 4.0, 28.0, 36.0),
        rgb(255, 255, 252),
        true,
    );
    // Page content lines (blue left margin line, gray text lines)
    canvas.rect(Rect::new(x + 13.0, y + 12.0, 1.0, 20.0), rgb(140, 140, 220));
    canvas.rect(Rect::new(x + 16.0, y + 15.0, 14.0, 1.0), rgb(160, 160, 155));
    canvas.rect(Rect::new(x + 16.0, y + 21.0, 16.0, 1.0), rgb(160, 160, 155));
    canvas.rect(Rect::new(x + 16.0, y + 27.0, 12.0, 1.0), rgb(160, 160, 155));
    
    // Top right folded corner
    canvas.rect(Rect::new(x + 29.0, y + 4.0, 7.0, 7.0), rgb(210, 210, 205));
    canvas.rect(Rect::new(x + 29.0, y + 11.0, 8.0, 1.0), rgb(130, 130, 125));
    canvas.rect(Rect::new(x + 28.0, y + 4.0, 1.0, 8.0), rgb(130, 130, 125));
}

fn draw_drive_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    // Disk casing
    draw_beveled_rect(
        canvas,
        Rect::new(x, y + 8.0, 44.0, 28.0),
        rgb(210, 210, 205),
        true,
    );
    // Disc slot
    canvas.rect(Rect::new(x + 6.0, y + 14.0, 32.0, 3.0), rgb(50, 50, 50));
    // LED Dot
    canvas.rect(Rect::new(x + 34.0, y + 26.0, 4.0, 4.0), rgb(80, 220, 80));
}

fn draw_folder_icon(canvas: &mut Canvas<'_>, x: f32, y: f32, color: [f32; 4]) {
    // Back tab
    canvas.rect(Rect::new(x + 3.0, y + 10.0, 16.0, 6.0), rgb(180, 160, 90));
    canvas.rect(Rect::new(x + 4.0, y + 9.0, 14.0, 1.0), rgb(230, 220, 160));
    // Front body
    draw_beveled_rect(canvas, Rect::new(x, y + 15.0, 44.0, 26.0), color, true);
    // Folder accent highlights
    canvas.rect(Rect::new(x + 1.0, y + 16.0, 42.0, 1.0), rgb(250, 245, 210));
    canvas.rect(Rect::new(x, y + 40.0, 44.0, 1.0), rgb(120, 110, 60));
}

fn draw_app_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    // Monitor frame
    draw_beveled_rect(
        canvas,
        Rect::new(x + 4.0, y + 4.0, 36.0, 32.0),
        rgb(220, 220, 216),
        true,
    );
    // Screen area
    canvas.rect(
        Rect::new(x + 8.0, y + 8.0, 28.0, 20.0),
        rgb(40, 44, 52),
    );
    // Stand/base
    canvas.rect(Rect::new(x + 14.0, y + 36.0, 16.0, 4.0), rgb(180, 180, 175));
    canvas.rect(Rect::new(x + 10.0, y + 40.0, 24.0, 2.0), rgb(140, 140, 135));
    
    // Stylized logo graphic
    let logo_color = rgb(90, 160, 240);
    canvas.rect(Rect::new(x + 20.0, y + 12.0, 4.0, 3.0), logo_color);
    canvas.rect(Rect::new(x + 17.0, y + 15.0, 4.0, 9.0), logo_color);
    canvas.rect(Rect::new(x + 23.0, y + 15.0, 4.0, 9.0), logo_color);
    canvas.rect(Rect::new(x + 17.0, y + 18.0, 10.0, 3.0), logo_color);
}

fn draw_trash_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    let lid_color = rgb(190, 190, 185);
    let body_color = rgb(170, 170, 165);
    let shadow_color = rgb(110, 110, 105);
    
    // Handle
    canvas.rect(Rect::new(x + 18.0, y + 2.0, 8.0, 3.0), lid_color);
    canvas.rect(Rect::new(x + 19.0, y + 1.0, 6.0, 1.0), rgb(240, 240, 240));
    
    // Lid rim
    draw_beveled_rect(canvas, Rect::new(x + 6.0, y + 5.0, 32.0, 5.0), lid_color, true);
    
    // Can body
    draw_beveled_rect(canvas, Rect::new(x + 9.0, y + 10.0, 26.0, 34.0), body_color, true);
    
    // Rib highlights
    for offset in [14.0, 20.0, 26.0, 32.0] {
        canvas.rect(Rect::new(x + offset, y + 14.0, 1.0, 26.0), shadow_color);
        canvas.rect(Rect::new(x + offset + 1.0, y + 14.0, 1.0, 26.0), rgb(220, 220, 215));
    }
}

fn draw_list(canvas: &mut Canvas<'_>, rect: Rect, list: &ListView) {
    canvas.rect(rect, ui(rgb(255, 255, 252), rgb(24, 26, 28)));
    canvas.stroke(rect, ui(rgb(150, 150, 144), rgb(92, 94, 96)));
    for (index, item) in list.items.iter().enumerate() {
        let y = rect.y + 6.0 + index as f32 * 18.0;
        if list.selected_index == Some(index) {
            canvas.rect(
                Rect::new(rect.x + 3.0, y - 3.0, rect.width - 6.0, 16.0),
                ui(rgb(64, 111, 171), rgb(82, 98, 126)),
            );
        }
        canvas.text(
            item,
            rect.x + 8.0,
            y,
            if list.selected_index == Some(index) {
                rgb(255, 255, 255)
            } else {
                ui(rgb(25, 25, 25), rgb(230, 230, 226))
            },
        );
    }
}

fn intersect_rect(a: Rect, b: Rect) -> Option<Rect> {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.width).min(b.x + b.width);
    let y1 = (a.y + a.height).min(b.y + b.height);
    (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
}

fn rgb(r: u8, g: u8, b: u8) -> [f32; 4] {
    rgba(r, g, b, 1.0)
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a.clamp(0.0, 1.0),
    ]
}

fn _color_to_rgb(color: Color) -> [f32; 4] {
    [
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        color.a.clamp(0.0, 1.0),
    ]
}

fn glyph_pattern(ch: char) -> [u8; 7] {
    match ch {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001,
        ],
        'Y' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        'a' => [
            0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111,
        ],
        'b' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110,
        ],
        'c' => [
            0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10000, 0b01110,
        ],
        'd' => [
            0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111,
        ],
        'e' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110,
        ],
        'f' => [
            0b00110, 0b01001, 0b01000, 0b11110, 0b01000, 0b01000, 0b01000,
        ],
        'g' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b01111, 0b00001, 0b01110,
        ],
        'h' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ],
        'i' => [
            0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'j' => [
            0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'k' => [
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
        ],
        'l' => [
            0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'm' => [
            0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10101, 0b10101,
        ],
        'n' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ],
        'o' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'p' => [
            0b00000, 0b00000, 0b01100, 0b01010, 0b01100, 0b01000, 0b01000,
        ],
        'q' => [
            0b00000, 0b00000, 0b01100, 0b10100, 0b01100, 0b00100, 0b00100,
        ],
        'r' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000,
        ],
        's' => [
            0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110,
        ],
        't' => [
            0b00100, 0b00100, 0b11110, 0b00100, 0b00100, 0b00100, 0b00011,
        ],
        'u' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101,
        ],
        'v' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        'w' => [
            0b00000, 0b00000, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'x' => [
            0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001,
        ],
        'y' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
        ],
        'z' => [
            0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b11111],
        '.' => [0, 0, 0, 0, 0, 0b01100, 0b01100],
        ':' => [0, 0b01100, 0b01100, 0, 0b01100, 0b01100, 0],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '\\' => [
            0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        ',' => [0, 0, 0, 0, 0, 0b01100, 0b00100],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100, 0],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0, 0b00100],
        '=' => [0, 0, 0b11111, 0, 0b11111, 0, 0],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        _ => [
            0b11111, 0b10001, 0b00010, 0b00100, 0b00000, 0b00100, 0b00100,
        ],
    }
}

pub fn modifiers_from_winit(modifiers: winit::keyboard::ModifiersState) -> Modifiers {
    Modifiers {
        shift: modifiers.shift_key(),
        control: modifiers.control_key(),
        alt: modifiers.alt_key(),
        meta: modifiers.super_key(),
    }
}

pub fn winit_to_retro_mouse_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Back => Some(MouseButton::Back),
        winit::event::MouseButton::Forward => Some(MouseButton::Forward),
        winit::event::MouseButton::Other(_) => None,
    }
}

pub fn winit_to_retro_scroll_delta(delta: winit::event::MouseScrollDelta) -> Point {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(x, y) => Point::new(x * 16.0, y * 16.0),
        winit::event::MouseScrollDelta::PixelDelta(pos) => Point::new(pos.x as f32, pos.y as f32),
    }
}

pub fn winit_to_retro_key(key: winit::keyboard::KeyCode) -> Option<KeyCode> {
    use retro_kit::event::KeyCode as RKey;
    use winit::keyboard::KeyCode as WKey;

    match key {
        WKey::KeyA => Some(RKey::A),
        WKey::KeyB => Some(RKey::B),
        WKey::KeyC => Some(RKey::C),
        WKey::KeyD => Some(RKey::D),
        WKey::KeyE => Some(RKey::E),
        WKey::KeyF => Some(RKey::F),
        WKey::KeyG => Some(RKey::G),
        WKey::KeyH => Some(RKey::H),
        WKey::KeyI => Some(RKey::I),
        WKey::KeyJ => Some(RKey::J),
        WKey::KeyK => Some(RKey::K),
        WKey::KeyL => Some(RKey::L),
        WKey::KeyM => Some(RKey::M),
        WKey::KeyN => Some(RKey::N),
        WKey::KeyO => Some(RKey::O),
        WKey::KeyP => Some(RKey::P),
        WKey::KeyQ => Some(RKey::Q),
        WKey::KeyR => Some(RKey::R),
        WKey::KeyS => Some(RKey::S),
        WKey::KeyT => Some(RKey::T),
        WKey::KeyU => Some(RKey::U),
        WKey::KeyV => Some(RKey::V),
        WKey::KeyW => Some(RKey::W),
        WKey::KeyX => Some(RKey::X),
        WKey::KeyY => Some(RKey::Y),
        WKey::KeyZ => Some(RKey::Z),
        WKey::Digit0 => Some(RKey::Key0),
        WKey::Digit1 => Some(RKey::Key1),
        WKey::Digit2 => Some(RKey::Key2),
        WKey::Digit3 => Some(RKey::Key3),
        WKey::Digit4 => Some(RKey::Key4),
        WKey::Digit5 => Some(RKey::Key5),
        WKey::Digit6 => Some(RKey::Key6),
        WKey::Digit7 => Some(RKey::Key7),
        WKey::Digit8 => Some(RKey::Key8),
        WKey::Digit9 => Some(RKey::Key9),
        WKey::F1 => Some(RKey::F1),
        WKey::F2 => Some(RKey::F2),
        WKey::F3 => Some(RKey::F3),
        WKey::F4 => Some(RKey::F4),
        WKey::F5 => Some(RKey::F5),
        WKey::F6 => Some(RKey::F6),
        WKey::F7 => Some(RKey::F7),
        WKey::F8 => Some(RKey::F8),
        WKey::F9 => Some(RKey::F9),
        WKey::F10 => Some(RKey::F10),
        WKey::F11 => Some(RKey::F11),
        WKey::F12 => Some(RKey::F12),
        WKey::Escape => Some(RKey::Escape),
        WKey::Tab => Some(RKey::Tab),
        WKey::CapsLock => Some(RKey::CapsLock),
        WKey::ShiftLeft => Some(RKey::ShiftLeft),
        WKey::ShiftRight => Some(RKey::ShiftRight),
        WKey::ControlLeft => Some(RKey::ControlLeft),
        WKey::ControlRight => Some(RKey::ControlRight),
        WKey::AltLeft => Some(RKey::AltLeft),
        WKey::AltRight => Some(RKey::AltRight),
        WKey::Space => Some(RKey::Space),
        WKey::Enter => Some(RKey::Enter),
        WKey::Backspace => Some(RKey::Backspace),
        WKey::Delete => Some(RKey::Delete),
        WKey::Insert => Some(RKey::Insert),
        WKey::Home => Some(RKey::Home),
        WKey::End => Some(RKey::End),
        WKey::PageUp => Some(RKey::PageUp),
        WKey::PageDown => Some(RKey::PageDown),
        WKey::ArrowUp => Some(RKey::ArrowUp),
        WKey::ArrowDown => Some(RKey::ArrowDown),
        WKey::ArrowLeft => Some(RKey::ArrowLeft),
        WKey::ArrowRight => Some(RKey::ArrowRight),
        WKey::SuperLeft => Some(RKey::MetaLeft),
        WKey::SuperRight => Some(RKey::MetaRight),
        WKey::Minus => Some(RKey::Minus),
        WKey::Equal => Some(RKey::Equals),
        WKey::BracketLeft => Some(RKey::LeftBracket),
        WKey::BracketRight => Some(RKey::RightBracket),
        WKey::Backslash => Some(RKey::Backslash),
        WKey::Semicolon => Some(RKey::Semicolon),
        WKey::Quote => Some(RKey::Quote),
        WKey::Comma => Some(RKey::Comma),
        WKey::Period => Some(RKey::Period),
        WKey::Slash => Some(RKey::Slash),
        _ => None,
    }
}

fn distance_squared(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::{format_clock_from_seconds, parse_dark_mode_preference};

    #[test]
    fn parses_dark_appearance_preference() {
        assert!(parse_dark_mode_preference("appearance=dark\n"));
        assert!(parse_dark_mode_preference("appearance=Dark\n"));
    }

    #[test]
    fn ignores_non_dark_appearance_preferences() {
        assert!(!parse_dark_mode_preference("appearance=light\n"));
        assert!(!parse_dark_mode_preference("appearance=system\n"));
        assert!(!parse_dark_mode_preference("other=dark\n"));
    }

    #[test]
    fn formats_menu_clock_with_minute_precision() {
        assert_eq!(format_clock_from_seconds(0), "12:00 AM");
        assert_eq!(format_clock_from_seconds(60), "12:01 AM");
        assert_eq!(format_clock_from_seconds(11 * 3600 + 59 * 60), "11:59 AM");
        assert_eq!(format_clock_from_seconds(12 * 3600), "12:00 PM");
        assert_eq!(format_clock_from_seconds(23 * 3600 + 5 * 60), "11:05 PM");
    }
}
