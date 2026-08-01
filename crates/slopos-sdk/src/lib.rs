#![allow(dead_code)]

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use slopos_bus::SloposBus;
use slopos_kit::button::Button;
use slopos_kit::dialog::Dialog;
use slopos_kit::dock_view::DockView;
use slopos_kit::event::{KeyCode, Modifiers, MouseButton};
use slopos_kit::icon_view::{IconItem, IconView};
use slopos_kit::label::Label;
use slopos_kit::layout::{Layout, LayoutView};
use slopos_kit::list_view::ListView;
use slopos_kit::menu::{Menu, MenuItem, MenuItemKind};
use slopos_kit::menu_bar::MenuBar;
use slopos_kit::panel::Panel;
use slopos_kit::popup_button::PopupButton;
use slopos_kit::progress_bar::ProgressBar;
use slopos_kit::scroll_view::ScrollView;
use slopos_kit::slider::Slider;
use slopos_kit::split_view::SplitView;
use slopos_kit::status_bar::StatusBar;
use slopos_kit::tab_view::TabView;
use slopos_kit::text_field::TextField;
use slopos_kit::toolbar::Toolbar;
use slopos_kit::tree_view::{TreeNode, TreeView};
use slopos_kit::window::Window;
use slopos_kit::workspace_grid_view::WorkspaceGridView;
use slopos_kit::{Color, LayoutConstraint, MonospaceView, Point, Rect, Size, Widget};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::util::DeviceExt;

static RENDER_DARK_MODE: AtomicBool = AtomicBool::new(false);
static RENDER_ACCENT_COLOR: Mutex<[f32; 4]> = Mutex::new([0.36, 0.54, 0.85, 1.0]); // default Mac OS 7 blue

/// Snaps a float value to the nearest integer pixel.
pub fn snap_to_pixel(val: f32) -> f32 {
    val.round()
}

/// Snaps a 2D point (x, y) to integer pixel boundaries.
pub fn snap_point_to_pixel(x: f32, y: f32) -> (f32, f32) {
    (x.round(), y.round())
}

/// Snaps a rectangle to integer pixel boundaries.
pub fn snap_rect_to_pixel(rect: Rect) -> Rect {
    let x = rect.x.round();
    let y = rect.y.round();
    let width = rect.width.round().max(1.0);
    let height = rect.height.round().max(1.0);
    Rect::new(x, y, width, height)
}

/// Snaps 1-pixel strokes to half-pixel raster alignment.
pub fn snap_stroke_1px(val: f32) -> f32 {
    val.floor() + 0.5
}

// System 7 Classic palette — aligned to Calculable/System7Components Assets.xcassets
// See docs/UI-REFERENCES.md
const S7_BG: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // Background #FFFFFF
const S7_FG: [f32; 4] = [0.0, 0.0, 0.0, 1.0]; // Foreground #000000
const S7_GRAY100: [f32; 4] = [0.937, 0.937, 0.937, 1.0]; // #EFEFEF
const S7_GRAY200: [f32; 4] = [0.855, 0.855, 0.855, 1.0]; // #DADADA
const S7_GRAY300: [f32; 4] = [0.647, 0.647, 0.647, 1.0]; // #A5A5A5
const S7_GRAY400: [f32; 4] = [0.525, 0.525, 0.525, 1.0]; // #868686
const S7_GRAY500: [f32; 4] = [0.400, 0.400, 0.400, 1.0]; // #666666
const S7_LAVENDER100: [f32; 4] = [0.855, 0.855, 0.988, 1.0]; // #DADAFC
const S7_LAVENDER300: [f32; 4] = [0.529, 0.529, 0.690, 1.0]; // #8787B0

const COLOR_PLATINUM_BG: [f32; 4] = S7_GRAY100;
const COLOR_BUTTON_BG: [f32; 4] = S7_GRAY100;
const COLOR_BUTTON_HOVER: [f32; 4] = S7_GRAY200;
const COLOR_WINDOW_BORDER: [f32; 4] = S7_FG;
const COLOR_TEXT_PRIMARY: [f32; 4] = S7_FG;
const COLOR_TEXT_SECONDARY: [f32; 4] = S7_GRAY500;
const COLOR_SELECTION_BG: [f32; 4] = [0.39, 0.59, 0.86, 1.0]; // classic Mac blue
const COLOR_SELECTION_TEXT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const COLOR_FOCUS_RING: [f32; 4] = [0.39, 0.59, 0.86, 1.0];
const COLOR_EDGE_LIGHT: [f32; 4] = S7_BG;
const COLOR_EDGE_DARK: [f32; 4] = S7_GRAY500;

// Graphite / dark mode
const COLOR_DARK_BG: [f32; 4] = [0.14, 0.14, 0.15, 1.0];
const COLOR_DARK_BUTTON_BG: [f32; 4] = [0.22, 0.22, 0.24, 1.0];
const COLOR_DARK_BUTTON_HOVER: [f32; 4] = [0.30, 0.30, 0.34, 1.0];
const COLOR_DARK_BORDER: [f32; 4] = [0.02, 0.02, 0.02, 1.0];
const COLOR_DARK_TEXT: [f32; 4] = [0.92, 0.92, 0.93, 1.0];
const COLOR_DARK_EDGE_LIGHT: [f32; 4] = [0.42, 0.42, 0.45, 1.0];
const COLOR_DARK_EDGE_DARK: [f32; 4] = [0.02, 0.02, 0.02, 1.0];
const COLOR_DARK_MENU: [f32; 4] = [0.18, 0.18, 0.19, 1.0];

fn theme_face() -> [f32; 4] {
    if render_dark_mode() {
        COLOR_DARK_BUTTON_BG
    } else {
        S7_GRAY100
    }
}

fn theme_menu() -> [f32; 4] {
    if render_dark_mode() {
        COLOR_DARK_MENU
    } else {
        S7_BG
    }
}

fn theme_paper() -> [f32; 4] {
    if render_dark_mode() {
        COLOR_DARK_BG
    } else {
        S7_BG
    }
}

fn theme_ink() -> [f32; 4] {
    if render_dark_mode() {
        COLOR_DARK_TEXT
    } else {
        S7_FG
    }
}

fn theme_muted() -> [f32; 4] {
    if render_dark_mode() {
        COLOR_DARK_EDGE_LIGHT
    } else {
        S7_GRAY400
    }
}

fn set_render_dark_mode(is_dark: bool) {
    RENDER_DARK_MODE.store(is_dark, Ordering::Relaxed);
}

fn render_dark_mode() -> bool {
    RENDER_DARK_MODE.load(Ordering::Relaxed)
}

fn set_render_accent(color: [f32; 4]) {
    *RENDER_ACCENT_COLOR.lock() = color;
}

fn render_accent() -> [f32; 4] {
    *RENDER_ACCENT_COLOR.lock()
}

/// Get a color value based on current theme (light/dark mode).
/// Maps semantic color names to System 7 palette values.
fn theme_color(color_name: &str) -> [f32; 4] {
    if render_dark_mode() {
        match color_name {
            "window_bg" => COLOR_DARK_BG,
            "button_bg" => COLOR_DARK_BUTTON_BG,
            "button_hover" => COLOR_DARK_BUTTON_HOVER,
            "border" => COLOR_DARK_BORDER,
            "text" => COLOR_DARK_TEXT,
            "edge_light" => COLOR_DARK_EDGE_LIGHT,
            "edge_dark" => COLOR_DARK_EDGE_DARK,
            _ => [0.5, 0.5, 0.5, 1.0], // fallback gray
        }
    } else {
        match color_name {
            "window_bg" => COLOR_PLATINUM_BG,
            "button_bg" => COLOR_BUTTON_BG,
            "button_hover" => COLOR_BUTTON_HOVER,
            "border" => COLOR_WINDOW_BORDER,
            "text" => COLOR_TEXT_PRIMARY,
            "edge_light" => COLOR_EDGE_LIGHT,
            "edge_dark" => COLOR_EDGE_DARK,
            _ => [0.5, 0.5, 0.5, 1.0], // fallback gray
        }
    }
}

/// Apply both dark mode flag and accent color together (used when theme changes).
pub fn apply_theme(is_dark: bool, accent: [f32; 4]) {
    set_render_dark_mode(is_dark);
    set_render_accent(accent);
}

/// Accent color definitions for each named theme.
pub mod theme_accents {
    /// Classic (Mac OS 7 Platinum) — blue
    pub const CLASSIC: [f32; 4] = [0.36, 0.54, 0.85, 1.0];
    /// Dark — same blue in dark mode
    pub const DARK: [f32; 4] = [0.36, 0.54, 0.85, 1.0];
    /// Grape — purple
    pub const GRAPE: [f32; 4] = [0.55, 0.28, 0.72, 1.0];
    /// Blueberry — deep blue
    pub const BLUEBERRY: [f32; 4] = [0.15, 0.25, 0.62, 1.0];
    /// Strawberry — red-pink
    pub const STRAWBERRY: [f32; 4] = [0.82, 0.23, 0.28, 1.0];
    /// Solarized — #268bd2 (matches ThemeName::Solarized in slopos-shell)
    pub const SOLARIZED: [f32; 4] = [0.15, 0.55, 0.82, 1.0];
    /// Dracula — #bd93f9
    pub const DRACULA: [f32; 4] = [0.74, 0.58, 0.98, 1.0];
    /// HighContrast — yellow
    pub const HIGH_CONTRAST: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
}

/// Read settings.conf and return (is_dark, accent_color) for the current theme.
fn load_theme_preference() -> (bool, [f32; 4]) {
    let config_dir = std::env::var_os("SLOPOS_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config/slopos-i"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/slopos-i"));
    let path = config_dir.join("settings.conf");
    let Ok(content) = std::fs::read_to_string(path) else {
        return (false, theme_accents::CLASSIC);
    };
    parse_theme_preference(&content)
}

fn parse_theme_preference(content: &str) -> (bool, [f32; 4]) {
    let mut theme_name: Option<String> = None;
    let mut appearance: Option<String> = None;
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "theme" => theme_name = Some(value.trim().to_ascii_lowercase()),
            "appearance" => appearance = Some(value.trim().to_ascii_lowercase()),
            _ => {}
        }
    }
    // Named theme takes precedence over appearance
    if let Some(name) = theme_name {
        // Must stay in sync with slopos_shell::theme_manager::ThemeName
        // (accent + is_dark); a name missing here silently renders as Classic.
        return match name.as_str() {
            "grape" => (true, theme_accents::GRAPE),
            "blueberry" => (true, theme_accents::BLUEBERRY),
            "strawberry" => (false, theme_accents::STRAWBERRY),
            "dark" => (true, theme_accents::DARK),
            "solarized" => (true, theme_accents::SOLARIZED),
            "dracula" => (true, theme_accents::DRACULA),
            "highcontrast" => (false, theme_accents::HIGH_CONTRAST),
            _ => (false, theme_accents::CLASSIC), // classic and unknown
        };
    }
    // Fall back to appearance key
    let is_dark = appearance.as_deref().map(|a| a == "dark").unwrap_or(false);
    let accent = if is_dark {
        theme_accents::DARK
    } else {
        theme_accents::CLASSIC
    };
    (is_dark, accent)
}

pub fn menu_manifest_dir() -> Option<PathBuf> {
    std::env::var_os("SLOPOS_MENU_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(|runtime| PathBuf::from(runtime).join("slopos-i").join("menus"))
        })
}

pub fn global_menu_mode_enabled() -> bool {
    std::env::var_os("SLOPOS_GLOBAL_MENU")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowChromeHit {
    Content,
    Titlebar,
    Close,
    Zoom,
    ResizeSouthEast,
}

fn hit_test_window_chrome(point: Point, size: Size) -> WindowChromeHit {
    const TITLEBAR_HEIGHT: f32 = 24.0;
    const CONTROL_TOP: f32 = 5.0;
    const CONTROL_SIZE: f32 = 14.0;
    const CONTROL_MARGIN: f32 = 11.0;
    const RESIZE_GRIP: f32 = 18.0;

    if point.x >= (size.width - RESIZE_GRIP).max(0.0)
        && point.y >= (size.height - RESIZE_GRIP).max(0.0)
    {
        return WindowChromeHit::ResizeSouthEast;
    }
    if point.y >= CONTROL_TOP
        && point.y <= CONTROL_TOP + CONTROL_SIZE
        && point.x >= CONTROL_MARGIN
        && point.x <= CONTROL_MARGIN + CONTROL_SIZE
    {
        return WindowChromeHit::Close;
    }
    let zoom_left = (size.width - CONTROL_MARGIN - CONTROL_SIZE).max(0.0);
    if point.y >= CONTROL_TOP
        && point.y <= CONTROL_TOP + CONTROL_SIZE
        && point.x >= zoom_left
        && point.x <= zoom_left + CONTROL_SIZE
    {
        return WindowChromeHit::Zoom;
    }
    if point.y >= 1.0 && point.y < TITLEBAR_HEIGHT {
        return WindowChromeHit::Titlebar;
    }
    WindowChromeHit::Content
}

pub struct Application {
    pub name: String,
    pub bundle_id: String,
    pub main_window: Option<Window>,
    pub initial_size: Size,
    pub menus: Vec<Menu>,
    pub bus: Option<SloposBus>,
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

    pub fn with_bus(mut self, bus: SloposBus) -> Self {
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

    pub fn run(&mut self) {
        if let Err(err) = self.publish_menu_manifest() {
            tracing::warn!("failed to publish menu manifest: {err}");
        }
        self.running = true;
        tracing::info!("Application '{}' started", self.name);

        let event_loop = match slopos_render::event_loop::RetroEventLoop::new() {
            Ok(event_loop) => event_loop,
            Err(err) => {
                tracing::error!(
                    app = %self.name,
                    wayland_display = ?std::env::var("WAYLAND_DISPLAY").ok(),
                    display = ?std::env::var("DISPLAY").ok(),
                    "cannot start: no display server connection: {err}"
                );
                eprintln!(
                    "[{}] cannot start: no Wayland/X11 display server reachable ({err})",
                    self.name
                );
                std::process::exit(1);
            }
        };
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
            accent_color: [f32; 4],
            scale: f32,
        }

        impl AppHandler {
            fn modifiers(&self) -> Modifiers {
                modifiers_from_winit(self.modifiers)
            }

            fn dispatch(&mut self, event: slopos_kit::Event) -> slopos_kit::EventResult {
                let result = if let Some(ref mut win) = self.window {
                    win.handle_event(&event)
                } else {
                    slopos_kit::EventResult::Ignored
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
                // Re-layout before drawing. update() can swap in entirely new
                // content (lock screen fields, a new terminal tab, dialogs);
                // without this those widgets keep Rect::ZERO until the next
                // resize and draw_widget skips them, painting an empty window.
                if let Some(ref mut win) = self.window {
                    let size = Size::new(win.rect().width, win.rect().height);
                    if size.width > 0.0 && size.height > 0.0 {
                        win.layout(LayoutConstraint::tight(size));
                    }
                }
                let Some(window) = &self.window else {
                    return;
                };
                let Some(presenter) = &mut self.presenter else {
                    return;
                };
                apply_theme(self.dark_mode, self.accent_color);
                let scale = self.scale;
                if let Err(err) = presenter.render(|canvas| {
                    canvas.set_scale(scale);
                    draw_desktop_backdrop(canvas);
                    draw_window(canvas, window);
                }) {
                    tracing::error!("failed to render frame: {err}");
                } else {
                    self.dirty = false;
                }
            }
        }

        impl slopos_render::event_loop::RetroAppHandler for AppHandler {
            fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
                let initial_size = self.initial_size;
                // No winit/Adwaita CSD — classic Mac chrome is drawn by the kit
                // (title bar) and the global menu lives in slopos-shell layer-shell.
                let attrs = winit::window::Window::default_attributes()
                    .with_title(&self.name)
                    .with_inner_size(winit::dpi::LogicalSize::new(
                        initial_size.width,
                        initial_size.height,
                    ))
                    .with_decorations(false);

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
                        if let Some(window) = &self.platform_window {
                            let logical = window.inner_size().to_logical::<f32>(self.scale as f64);
                            let hit = hit_test_window_chrome(
                                self.cursor_position,
                                Size::new(logical.width, logical.height),
                            );
                            window.set_cursor(match hit {
                                WindowChromeHit::ResizeSouthEast => {
                                    winit::window::CursorIcon::NwseResize
                                }
                                _ => winit::window::CursorIcon::Default,
                            });
                        }
                        let _ = self.dispatch(slopos_kit::Event::MouseMove {
                            point: self.cursor_position,
                            modifiers: self.modifiers(),
                        });
                    }
                    winit::event::WindowEvent::CursorEntered { .. } => {
                        let _ = self.dispatch(slopos_kit::Event::MouseEnter);
                    }
                    winit::event::WindowEvent::CursorLeft { .. } => {
                        let _ = self.dispatch(slopos_kit::Event::MouseLeave);
                    }
                    winit::event::WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(button) = winit_to_retro_mouse_button(button) {
                            let now = std::time::Instant::now();
                            let is_double_click = state == winit::event::ElementState::Pressed
                                && self
                                    .last_click
                                    .as_ref()
                                    .map(|(last_button, last_point, last_time)| {
                                        *last_button == button
                                            && now.duration_since(*last_time)
                                                <= std::time::Duration::from_millis(500)
                                            && distance_squared(*last_point, self.cursor_position)
                                                <= 16.0
                                    })
                                    .unwrap_or(false);

                            if state == winit::event::ElementState::Pressed {
                                self.last_click = Some((button, self.cursor_position, now));
                            }

                            if button == MouseButton::Left
                                && state == winit::event::ElementState::Pressed
                            {
                                if let Some(window) = &self.platform_window {
                                    let logical =
                                        window.inner_size().to_logical::<f32>(self.scale as f64);
                                    match hit_test_window_chrome(
                                        self.cursor_position,
                                        Size::new(logical.width, logical.height),
                                    ) {
                                        WindowChromeHit::Close => {
                                            event_loop.exit();
                                            return;
                                        }
                                        WindowChromeHit::Zoom => {
                                            window.set_maximized(!window.is_maximized());
                                            return;
                                        }
                                        WindowChromeHit::Titlebar => {
                                            if is_double_click {
                                                window.set_maximized(!window.is_maximized());
                                            } else if let Err(err) = window.drag_window() {
                                                tracing::warn!("failed to request compositor window move: {err}");
                                            }
                                            return;
                                        }
                                        WindowChromeHit::ResizeSouthEast => {
                                            if let Err(err) = window.drag_resize_window(
                                                winit::window::ResizeDirection::SouthEast,
                                            ) {
                                                tracing::warn!(
                                                    "failed to request compositor resize: {err}"
                                                );
                                            }
                                            return;
                                        }
                                        WindowChromeHit::Content => {}
                                    }
                                }
                            }

                            let event = match state {
                                winit::event::ElementState::Pressed if is_double_click => {
                                    slopos_kit::Event::DoubleClick {
                                        button,
                                        point: self.cursor_position,
                                        modifiers: self.modifiers(),
                                    }
                                }
                                winit::event::ElementState::Pressed => {
                                    slopos_kit::Event::MouseDown {
                                        button,
                                        point: self.cursor_position,
                                        modifiers: self.modifiers(),
                                    }
                                }
                                winit::event::ElementState::Released => {
                                    slopos_kit::Event::MouseUp {
                                        button,
                                        point: self.cursor_position,
                                        modifiers: self.modifiers(),
                                    }
                                }
                            };
                            let _ = self.dispatch(event);
                        }
                    }
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        let delta = winit_to_retro_scroll_delta(delta);
                        let _ = self.dispatch(slopos_kit::Event::Scroll {
                            delta,
                            modifiers: self.modifiers(),
                        });
                    }
                    winit::event::WindowEvent::Focused(true) => {
                        let _ = self.dispatch(slopos_kit::Event::FocusIn);
                    }
                    winit::event::WindowEvent::Focused(false) => {
                        let _ = self.dispatch(slopos_kit::Event::FocusOut);
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
                                        slopos_kit::Event::KeyDown {
                                            key: rkey,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                    winit::event::ElementState::Released => {
                                        slopos_kit::Event::KeyUp {
                                            key: rkey,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                };
                                handled = matches!(
                                    self.dispatch(retro_event),
                                    slopos_kit::EventResult::Handled
                                        | slopos_kit::EventResult::StopPropagation
                                );
                            }
                        }
                        if key_event.state == winit::event::ElementState::Pressed && !handled {
                            if let Some(ref text) = key_event.text {
                                for character in text.chars() {
                                    if !character.is_control() {
                                        let _ =
                                            self.dispatch(slopos_kit::Event::Char { character });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
                let (next_dark_mode, next_accent) = load_theme_preference();
                if next_dark_mode != self.dark_mode || next_accent != self.accent_color {
                    self.dark_mode = next_dark_mode;
                    self.accent_color = next_accent;
                    self.dirty = true;
                }
                if let Some(ref mut win) = self.window {
                    win.update();
                }
                if self.dirty {
                    if let Some(window) = &self.platform_window {
                        window.request_redraw();
                    }
                }
            }
        }

        let (init_dark_mode, init_accent) = load_theme_preference();
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
            dark_mode: init_dark_mode,
            accent_color: init_accent,
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

/// Build a [`slopos_render::DisplayRenderPolicy`] from env and optional settings.conf.
fn display_render_policy_from_env() -> slopos_render::DisplayRenderPolicy {
    let mut hdr_enabled = env_flag_true("SLOPOS_HDR");
    let mut vrr_adaptive = env_flag_true("SLOPOS_VRR");

    if let Some(path) = dirs_settings_conf() {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                match key.trim() {
                    "hdr_requested" | "hdr_request" => {
                        hdr_enabled = parse_conf_bool(value, hdr_enabled);
                    }
                    "vrr_adaptive" => {
                        vrr_adaptive = parse_conf_bool(value, vrr_adaptive);
                    }
                    _ => {}
                }
            }
        }
    }

    slopos_render::DisplayRenderPolicy {
        hdr_enabled,
        vrr_adaptive,
    }
}

fn env_flag_true(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn parse_conf_bool(value: &str, fallback: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => fallback,
    }
}

fn dirs_settings_conf() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/slopos-i/settings.conf"))
        .or_else(|| {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .map(|base| base.join("slopos-i/settings.conf"))
        })
}

pub struct WgpuPresenter {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

/// Renders immediate-mode UI onto a Wayland surface created outside winit
/// (e.g. a wlr-layer-shell surface owned by slopos-shell).
pub struct RawSurfaceRenderer {
    presenter: WgpuPresenter,
}

impl RawSurfaceRenderer {
    /// Create a renderer from raw Wayland handles for a layer-shell surface.
    ///
    /// # Safety
    ///
    /// `display` must be a valid `*mut wl_display` and `surface` must be a valid
    /// `*mut wl_surface`. Both must outlive the returned renderer. They will not
    /// be freed by this renderer — the caller retains ownership and responsibility
    /// for cleanup.
    pub async unsafe fn new(
        display: *mut std::ffi::c_void,
        surface: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        Ok(Self {
            presenter: WgpuPresenter::new_raw(display, surface, width, height).await?,
        })
    }

    /// Resize the rendering surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.presenter.resize(width, height);
    }

    /// Render a frame by calling the draw closure with a mutable Canvas.
    pub fn render(&mut self, draw: impl FnOnce(&mut Canvas<'_>)) -> Result<(), String> {
        self.presenter.render(draw)
    }
}

/// Backend-agnostic UI runtime: owns a Window's widget tree, lays it out,
/// paints it via a RawSurfaceRenderer, and accepts neutral input events.
/// Mirrors the logic of the winit `AppHandler` without any winit dependency,
/// so a wlr-layer-shell driver (slopos-shell) can drive the same UI.
pub struct UiRuntime {
    window: Option<Window>,
    scale: f32,
    dark_mode: bool,
    accent_color: [f32; 4],
    modifiers: Modifiers,
    cursor_position: Point,
    last_click: Option<(MouseButton, Point, u128)>,
    dirty: bool,
    /// Last wall-clock minute painted; used so idle drivers wake the menu clock.
    last_clock_minute: Option<u64>,
}

impl UiRuntime {
    /// Create a new UI runtime with the given widget tree, sized in physical pixels.
    /// The widget is wrapped in a Window and laid out at the logical size (px / scale).
    pub fn new(
        content: Box<dyn slopos_kit::Widget>,
        width_px: u32,
        height_px: u32,
        scale: f32,
    ) -> Self {
        // NOTE: the title MUST be "SLOPOS-I Desktop" — draw_window special-cases
        // that exact title to render chromeless (no titlebar, no content clip), so
        // the menu bar sits at y=0 and the dock reaches the bottom edge.
        let mut window = Window::new("SLOPOS-I Desktop");
        window.set_content(content);

        let mut rt = Self {
            window: Some(window),
            scale: scale.max(1.0),
            dark_mode: false,
            accent_color: theme_accents::CLASSIC,
            modifiers: Modifiers::NONE,
            cursor_position: Point::ZERO,
            last_click: None,
            dirty: true,
            last_clock_minute: None,
        };

        rt.layout_window(width_px, height_px);
        rt
    }

    /// Resize and re-layout the widget tree at the new physical pixel dimensions.
    pub fn resize(&mut self, width_px: u32, height_px: u32, scale: f32) {
        self.scale = scale.max(1.0);
        self.layout_window(width_px, height_px);
    }

    /// Update the dark mode and accent color theme.
    pub fn set_theme(&mut self, dark_mode: bool, accent_color: [f32; 4]) {
        self.dark_mode = dark_mode;
        self.accent_color = accent_color;
        self.dirty = true;
    }

    /// Per-frame tick: reload theme preference and drive the content's
    /// `update()` so dynamic content (dock items, notifications, etc.) is
    /// rebuilt — mirrors the winit `AppHandler::about_to_wait` logic. A driver
    /// must call this each event-loop iteration or the dock never populates.
    /// Also dirties when the wall-clock minute changes so the menu clock advances
    /// even when the driver only wakes on a timer (no pointer/keyboard events).
    pub fn tick(&mut self) {
        let (dark, accent) = load_theme_preference();
        if dark != self.dark_mode || accent != self.accent_color {
            self.dark_mode = dark;
            self.accent_color = accent;
            self.dirty = true;
        }
        if let Some(ref mut win) = self.window {
            win.update();
        }
        let minute = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 60;
        if self.last_clock_minute != Some(minute) {
            self.last_clock_minute = Some(minute);
            self.dirty = true;
        }
    }

    /// Update the current modifier key state.
    pub fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    /// Handle pointer movement at logical coordinates.
    pub fn pointer_moved(&mut self, x: f32, y: f32) {
        self.cursor_position = Point::new(x, y);
        let _ = self.dispatch(slopos_kit::Event::MouseMove {
            point: self.cursor_position,
            modifiers: self.modifiers,
        });
    }

    /// Handle pointer button press/release at logical coordinates.
    /// Implements double-click detection: if the same button is pressed within
    /// 400ms and within 4 logical units (distance squared <= 16.0), emits DoubleClick;
    /// otherwise MouseDown on press, MouseUp on release.
    pub fn pointer_button(
        &mut self,
        button: MouseButton,
        pressed: bool,
        time_ms: u128,
    ) -> slopos_kit::EventResult {
        if pressed {
            let is_double_click = self
                .last_click
                .as_ref()
                .map(|(last_button, last_point, last_time)| {
                    *last_button == button
                        && time_ms.saturating_sub(*last_time) <= 400
                        && distance_squared(*last_point, self.cursor_position) <= 16.0
                })
                .unwrap_or(false);
            self.last_click = Some((button, self.cursor_position, time_ms));
            if is_double_click {
                self.dispatch(slopos_kit::Event::DoubleClick {
                    button,
                    point: self.cursor_position,
                    modifiers: self.modifiers,
                })
            } else {
                self.dispatch(slopos_kit::Event::MouseDown {
                    button,
                    point: self.cursor_position,
                    modifiers: self.modifiers,
                })
            }
        } else {
            self.dispatch(slopos_kit::Event::MouseUp {
                button,
                point: self.cursor_position,
                modifiers: self.modifiers,
            })
        }
    }

    /// Handle mouse wheel scroll.
    pub fn wheel(&mut self, delta_x: f32, delta_y: f32) {
        let _ = self.dispatch(slopos_kit::Event::Scroll {
            delta: Point::new(delta_x, delta_y),
            modifiers: self.modifiers,
        });
    }

    /// Handle a keyboard event (caller builds the neutral Event).
    pub fn key(&mut self, event: slopos_kit::Event) {
        let _ = self.dispatch(event);
    }

    /// Set window focus state.
    pub fn set_focus(&mut self, focused: bool) {
        if focused {
            let _ = self.dispatch(slopos_kit::Event::FocusIn);
        } else {
            let _ = self.dispatch(slopos_kit::Event::FocusOut);
        }
    }

    /// Check if a redraw is needed.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Layout (if needed), paint the widget tree and desktop backdrop through the renderer.
    /// Clears the dirty flag on success.
    pub fn paint(&mut self, renderer: &mut RawSurfaceRenderer) -> Result<(), String> {
        self.paint_ex(renderer, true, true)
    }

    /// Paint with optional backdrop and optional re-layout.
    ///
    /// Layer-shell chrome strips call this with `relayout = false` after manually
    /// positioning menu/dock widgets onto strip-sized surfaces.
    pub fn paint_ex(
        &mut self,
        renderer: &mut RawSurfaceRenderer,
        backdrop: bool,
        relayout: bool,
    ) -> Result<(), String> {
        if relayout {
            if let Some(ref mut win) = self.window {
                let size = Size::new(win.rect().width, win.rect().height);
                if size.width > 0.0 && size.height > 0.0 {
                    win.layout(LayoutConstraint::tight(size));
                }
            }
        }
        let Some(window) = &self.window else {
            return Ok(());
        };
        apply_theme(self.dark_mode, self.accent_color);
        let scale = self.scale;
        renderer.render(|canvas| {
            canvas.set_scale(scale);
            if backdrop {
                draw_desktop_backdrop(canvas);
            }
            draw_window(canvas, window);
        })?;
        self.dirty = false;
        Ok(())
    }

    /// Access the root content widget (under the chromeless Window).
    pub fn with_root_content_mut<R>(
        &mut self,
        f: impl FnOnce(&mut dyn slopos_kit::Widget) -> R,
    ) -> Option<R> {
        let win = self.window.as_mut()?;
        let content = win.content.as_mut()?;
        Some(f(content.as_mut()))
    }

    /// Mark the UI dirty so the next driver iteration repaints.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark the current pixels/layout as synchronized. Layer-shell drivers use
    /// this after restoring hit-test layout following a multi-surface paint.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Dispatch an event to the window and mark as dirty on any result.
    fn dispatch(&mut self, event: slopos_kit::Event) -> slopos_kit::EventResult {
        let result = if let Some(ref mut win) = self.window {
            win.handle_event(&event)
        } else {
            slopos_kit::EventResult::Ignored
        };
        self.dirty = true;
        result
    }

    /// Re-layout the window at the new physical pixel dimensions.
    fn layout_window(&mut self, width_px: u32, height_px: u32) {
        if let Some(ref mut win) = self.window {
            let logical_width = (width_px as f32 / self.scale).max(1.0);
            let logical_height = (height_px as f32 / self.scale).max(1.0);
            let size = Size::new(logical_width, logical_height);
            win.set_rect(Rect::new(0.0, 0.0, size.width, size.height));
            win.layout(LayoutConstraint::tight(size));
            self.dirty = true;
        }
    }
}

impl WgpuPresenter {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(Default::default());
        let surface = instance
            .create_surface(window)
            .map_err(|err| format!("surface creation failed: {err}"))?;
        Self::from_surface(instance, surface, size.width, size.height).await
    }

    /// Build a presenter from raw Wayland handles for a layer-shell surface
    /// created outside winit. `display` = `*mut wl_display`, `surface` =
    /// `*mut wl_surface`.
    ///
    /// # Safety
    /// both pointers must reference a valid `wl_display` / `wl_surface`
    /// that outlive the returned presenter.
    pub async unsafe fn new_raw(
        display: *mut std::ffi::c_void,
        surface: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        use raw_window_handle::{
            RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
        };
        let instance = wgpu::Instance::new(Default::default());
        let display_nn =
            std::ptr::NonNull::new(display).ok_or_else(|| "null wl_display".to_string())?;
        let surface_nn =
            std::ptr::NonNull::new(surface).ok_or_else(|| "null wl_surface".to_string())?;
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display_nn));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(surface_nn));
        let wgpu_surface = instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
            .map_err(|err| format!("raw surface creation failed: {err}"))?;
        Self::from_surface(instance, wgpu_surface, width, height).await
    }

    async fn from_surface(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
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
        let policy = display_render_policy_from_env();
        let format = slopos_render::select_surface_format(&caps.formats, policy);
        let present_mode = slopos_render::select_present_mode(&caps.present_modes, policy);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
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

pub struct Canvas<'a> {
    width: f32,
    height: f32,
    /// Number of physical framebuffer pixels per logical UI unit.
    pixel_scale: f32,
    vertices: Vec<Vertex>,
    clip: Option<Rect>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Canvas<'a> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            pixel_scale: 1.0,
            vertices: Vec::with_capacity(8192),
            clip: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// Configure logical layout coordinates while preserving the physical
    /// framebuffer scale for text rasterization and pixel snapping.
    pub fn set_scale(&mut self, scale: f32) {
        let scale = scale.max(1.0);
        if (self.pixel_scale - scale).abs() <= f32::EPSILON {
            return;
        }
        self.width /= scale;
        self.height /= scale;
        self.pixel_scale = scale;
    }

    pub fn pixel_scale(&self) -> f32 {
        self.pixel_scale
    }

    pub fn rect(&mut self, rect: Rect, color: [f32; 4]) {
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

    pub fn stroke(&mut self, rect: Rect, color: [f32; 4]) {
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

    pub fn measure_text(&self, text: &str) -> f32 {
        let scale = self.pixel_scale.max(1.0);
        let mut width = 0.0;
        for ch in text.chars() {
            if ch == '\n' {
                break;
            }
            if let Some(glyph) = slopos_render::rasterize_char(ch, 13.0 * scale) {
                width += glyph.advance / scale;
            } else {
                width += 6.0;
            }
        }
        width
    }

    /// Return text that fits the requested logical width, adding a measured
    /// ellipsis when truncation is required.
    pub fn ellipsize_text(&self, text: &str, max_width: f32) -> String {
        if max_width <= 0.0 {
            return String::new();
        }
        if self.measure_text(text) <= max_width {
            return text.to_owned();
        }
        let ellipsis = "...";
        let ellipsis_width = self.measure_text(ellipsis);
        if ellipsis_width >= max_width {
            return ellipsis.to_owned();
        }
        let mut out = String::new();
        for ch in text.chars() {
            let mut candidate = out.clone();
            candidate.push(ch);
            candidate.push_str(ellipsis);
            if self.measure_text(&candidate) > max_width {
                break;
            }
            out.push(ch);
        }
        out.push_str(ellipsis);
        out
    }

    pub fn text(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        let mut cursor_x = x;
        let mut cursor_y = y;
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                cursor_y += 14.0;
                continue;
            }
            cursor_x += self.glyph(ch, cursor_x, cursor_y, color);
        }
    }

    fn glyph(&mut self, ch: char, x: f32, y: f32, color: [f32; 4]) -> f32 {
        let scale = self.pixel_scale.max(1.0);
        if let Some(glyph) = slopos_render::rasterize_char(ch, 13.0 * scale) {
            let baseline_y_px = y * scale + glyph.ascent;
            let start_x_px = x * scale + glyph.bearing_x;
            let start_y_px = baseline_y_px + glyph.bearing_y;
            let logical_pixel = 1.0 / scale;

            for row in 0..glyph.height {
                for col in 0..glyph.width {
                    let idx = (row * glyph.width + col) as usize;
                    let alpha = glyph.data[idx] as f32 / 255.0;
                    if alpha > 0.05 {
                        let mut c = color;
                        c[3] *= alpha;
                        self.rect(
                            Rect::new(
                                (start_x_px + col as f32).round() / scale,
                                (start_y_px + row as f32).round() / scale,
                                logical_pixel,
                                logical_pixel,
                            ),
                            c,
                        );
                    }
                }
            }
            glyph.advance / scale
        } else {
            let logical_pixel = 1.0 / scale;
            for (row, bits) in glyph_pattern(ch).iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(
                            Rect::new(
                                (x * scale + col as f32).round() / scale,
                                (y * scale + row as f32).round() / scale,
                                logical_pixel,
                                logical_pixel,
                            ),
                            color,
                        );
                    }
                }
            }
            7.0
        }
    }

    pub fn with_clip(&mut self, clip: Rect, draw: impl FnOnce(&mut Self)) {
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
    let backdrop_base = if render_dark_mode() {
        [0.10, 0.11, 0.11, 1.0]
    } else {
        [0.60, 0.60, 0.58, 1.0]
    };
    canvas.rect(
        Rect::new(0.0, 0.0, canvas.width, canvas.height),
        backdrop_base,
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

    // Menu bar area under layer chrome is painted by the menu surface; for winit
    // fallback keep a subtle strip matching theme.
    if !render_dark_mode() {
        canvas.rect(Rect::new(0.0, 0.0, canvas.width, 24.0), rgb(239, 239, 239));
        canvas.rect(Rect::new(0.0, 24.0, canvas.width, 1.0), S7_FG);
    } else {
        canvas.rect(Rect::new(0.0, 0.0, canvas.width, 24.0), COLOR_DARK_MENU);
        canvas.rect(
            Rect::new(0.0, 24.0, canvas.width, 1.0),
            COLOR_DARK_EDGE_LIGHT,
        );
    }
}

fn draw_window(canvas: &mut Canvas<'_>, window: &Window) {
    let rect = window.rect();
    if window.title() == "SLOPOS-I Desktop" {
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

    // System7 frame: content fill, then 3D border (black + offset shadow)
    let window_bg = if render_dark_mode() {
        theme_color("window_bg")
    } else {
        S7_BG
    };
    canvas.rect(rect, window_bg);
    draw_system7_3d_border(canvas, rect);

    let titlebar = Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, 22.0);
    draw_classic_titlebar(canvas, titlebar, window.title(), window.is_active);

    // Content below title bar (inside black border)
    canvas.with_clip(
        Rect::new(
            rect.x + 2.0,
            rect.y + 24.0,
            (rect.width - 4.0).max(0.0),
            (rect.height - 26.0).max(0.0),
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

fn draw_window_grip(canvas: &mut Canvas<'_>, x: f32, y: f32, width: f32, height: f32) {
    // System7WindowGrip: 6 horizontal Gray400 lines
    let grip = if render_dark_mode() {
        COLOR_DARK_EDGE_LIGHT
    } else {
        S7_GRAY400
    };
    let line_h = 1.0;
    let gap = 1.0;
    let total = 6.0 * line_h + 5.0 * gap;
    let start_y = y + ((height - total) * 0.5).max(0.0);
    for i in 0..6 {
        let ly = start_y + i as f32 * (line_h + gap);
        canvas.rect(Rect::new(x, ly, width, line_h), grip);
    }
}

fn draw_classic_titlebar(canvas: &mut Canvas<'_>, rect: Rect, title: &str, is_active: bool) {
    let title_w = canvas.measure_text(title);
    if !is_active {
        let bg = if render_dark_mode() {
            theme_color("window_bg")
        } else {
            S7_BG
        };
        canvas.rect(rect, bg);
        canvas.stroke(
            rect,
            if render_dark_mode() {
                COLOR_DARK_BORDER
            } else {
                S7_FG
            },
        );
        let text_color = if render_dark_mode() {
            [0.43, 0.44, 0.45, 1.0]
        } else {
            S7_GRAY300
        };
        let title_x = (rect.x + (rect.width - title_w) * 0.5).round();
        canvas.text(title, title_x, rect.y + 6.0, text_color);
        return;
    }

    // Focused: lavender rail behind, gray100 face, grips + boxes
    let rail = if render_dark_mode() {
        [0.22, 0.22, 0.28, 1.0]
    } else {
        S7_LAVENDER100
    };
    let face = if render_dark_mode() {
        COLOR_DARK_BUTTON_BG
    } else {
        S7_GRAY100
    };
    canvas.rect(rect, rail);
    let inner = Rect::new(
        rect.x + 1.0,
        rect.y + 1.0,
        rect.width - 2.0,
        rect.height - 2.0,
    );
    canvas.rect(inner, face);
    canvas.stroke(
        rect,
        if render_dark_mode() {
            COLOR_DARK_BORDER
        } else {
            S7_FG
        },
    );

    // Boxes layout
    let box_size = 13.0;
    let box_y = inner.y + (inner.height - box_size) * 0.5;

    // Left close box
    let mut x = inner.x + 3.0;
    draw_window_grip(canvas, x, inner.y + 2.0, 5.0, inner.height - 4.0);
    x += 7.0;
    let close_box = Rect::new(x, box_y, box_size, box_size);
    draw_beveled_rect(canvas, close_box, face, true);
    canvas.stroke(
        close_box,
        if render_dark_mode() {
            COLOR_DARK_TEXT
        } else {
            S7_FG
        },
    );
    let left_end = close_box.x + close_box.width + 2.0;

    // Right zoom box
    let zoom_x = inner.x + inner.width - 3.0 - 5.0 - box_size - 2.0;
    let zoom_box = Rect::new(zoom_x, box_y, box_size, box_size);
    let right_start = zoom_box.x - 2.0;

    // Centered Title Pill
    let pill_w = (title_w + 16.0).min(inner.width - 70.0).max(20.0);
    let pill_x = (rect.x + (rect.width - pill_w) * 0.5).round();
    let pill_rect = Rect::new(pill_x, inner.y + 2.0, pill_w, inner.height - 4.0);

    // Left & Right Grips (Symmetrical around title pill)
    let left_grip_w = (pill_x - 2.0 - left_end).max(0.0);
    if left_grip_w > 4.0 {
        draw_window_grip(
            canvas,
            left_end,
            inner.y + 2.0,
            left_grip_w,
            inner.height - 4.0,
        );
    }

    let right_grip_x = pill_x + pill_w + 2.0;
    let right_grip_w = (right_start - right_grip_x).max(0.0);
    if right_grip_w > 4.0 {
        draw_window_grip(
            canvas,
            right_grip_x,
            inner.y + 2.0,
            right_grip_w,
            inner.height - 4.0,
        );
    }

    // Title face pill + text
    canvas.rect(pill_rect, face);
    let text_x = (pill_rect.x + (pill_w - title_w) * 0.5).round();
    let text_color = theme_color("text");
    canvas.text(title, text_x, rect.y + 6.0, text_color);

    // Zoom box (right)
    draw_beveled_rect(canvas, zoom_box, face, true);
    canvas.stroke(
        zoom_box,
        if render_dark_mode() {
            COLOR_DARK_TEXT
        } else {
            S7_FG
        },
    );
    draw_window_grip(
        canvas,
        zoom_box.x + zoom_box.width + 2.0,
        inner.y + 2.0,
        5.0,
        inner.height - 4.0,
    );
    // Inner zoom mark
    canvas.rect(
        Rect::new(
            zoom_box.x + 3.0,
            zoom_box.y + 3.0,
            zoom_box.width - 6.0,
            zoom_box.height - 6.0,
        ),
        face,
    );
    canvas.stroke(
        Rect::new(
            zoom_box.x + 3.0,
            zoom_box.y + 3.0,
            zoom_box.width - 6.0,
            zoom_box.height - 6.0,
        ),
        if render_dark_mode() {
            COLOR_DARK_TEXT
        } else {
            S7_FG
        },
    );

    draw_window_grip(
        canvas,
        zoom_box.x + box_size + 2.0,
        inner.y + 2.0,
        5.0,
        inner.height - 4.0,
    );
}

fn draw_resize_grow_box(canvas: &mut Canvas<'_>, window_rect: Rect) {
    let box_rect = Rect::new(
        window_rect.x + window_rect.width - 16.0,
        window_rect.y + window_rect.height - 16.0,
        15.0,
        15.0,
    );
    let box_bg = theme_color("button_bg");
    let box_stroke = if render_dark_mode() {
        [0.58, 0.58, 0.59, 1.0]
    } else {
        [0.52, 0.52, 0.49, 1.0]
    };
    canvas.rect(box_rect, box_bg);
    canvas.stroke(box_rect, box_stroke);

    let glyph_color = if render_dark_mode() {
        [0.71, 0.71, 0.72, 1.0]
    } else {
        [0.32, 0.32, 0.31, 1.0]
    };
    for offset in [4.0, 8.0, 12.0] {
        canvas.rect(
            Rect::new(box_rect.x + offset, box_rect.y + 13.0, 1.0, 1.0),
            glyph_color,
        );
        canvas.rect(
            Rect::new(box_rect.x + 13.0, box_rect.y + offset, 1.0, 1.0),
            glyph_color,
        );
        canvas.rect(
            Rect::new(box_rect.x + offset, box_rect.y + offset, 1.0, 1.0),
            if render_dark_mode() {
                [0.33, 0.33, 0.34, 1.0]
            } else {
                [0.64, 0.64, 0.62, 1.0]
            },
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
        canvas.text(&label.text, rect.x + 2.0, rect.y + 5.0, theme_color("text"));
    } else if let Some(button) = widget.as_any().downcast_ref::<Button>() {
        if rect.height <= 24.0 {
            canvas.text(
                button.label(),
                rect.x + 8.0,
                rect.y + 7.0,
                theme_color("text"),
            );
            if button.widget_state().focused {
                canvas.stroke(rect, COLOR_FOCUS_RING);
            }
            return;
        }
        // Use theme-aware colors for beveled button
        let bg = if button.widget_state().hovered {
            theme_color("button_hover")
        } else {
            theme_color("button_bg")
        };
        canvas.rect(rect, bg);
        draw_beveled_rect(canvas, rect, bg, true);
        canvas.text(
            button.label(),
            rect.x + 12.0,
            rect.y + 9.0,
            if render_dark_mode() {
                COLOR_DARK_TEXT
            } else {
                COLOR_TEXT_PRIMARY
            },
        );
        if button.widget_state().focused {
            canvas.stroke(rect, COLOR_FOCUS_RING);
        }
    } else if let Some(text_field) = widget.as_any().downcast_ref::<TextField>() {
        // Text field uses theme-aware colors
        let tf_bg = if render_dark_mode() {
            [0.12, 0.12, 0.12, 1.0]
        } else {
            [0.99, 0.99, 0.98, 1.0]
        };
        canvas.rect(rect, tf_bg);
        canvas.stroke(rect, theme_color("border"));
        let text = if text_field.text().is_empty() {
            &text_field.placeholder
        } else {
            text_field.text()
        };
        canvas.text(
            text,
            rect.x + 6.0,
            rect.y + 8.0,
            if render_dark_mode() {
                COLOR_DARK_TEXT
            } else {
                COLOR_TEXT_PRIMARY
            },
        );
    } else if let Some(slider) = widget.as_any().downcast_ref::<Slider>() {
        let track = Rect::new(
            rect.x + 9.0,
            rect.y + rect.height * 0.5 - 3.0,
            rect.width - 18.0,
            6.0,
        );
        let track_bg = if render_dark_mode() {
            [0.12, 0.12, 0.13, 1.0]
        } else {
            [0.77, 0.77, 0.75, 1.0]
        };
        canvas.rect(track, track_bg);
        canvas.stroke(track, theme_color("border"));
        let filled = Rect::new(
            track.x + 1.0,
            track.y + 1.0,
            (track.width - 2.0) * slider.normalized_value(),
            track.height - 2.0,
        );
        canvas.rect(filled, render_accent());
        let thumb_x = track.x + track.width * slider.normalized_value() - 5.0;
        let thumb = Rect::new(thumb_x, rect.y + 3.0, 10.0, rect.height - 6.0);
        let thumb_bg = if slider.dragging {
            theme_color("button_hover")
        } else {
            theme_color("button_bg")
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
            let toolbar_bg = if render_dark_mode() {
                [0.16, 0.16, 0.18, 1.0]
            } else {
                [0.85, 0.85, 0.84, 1.0]
            };
            canvas.rect(rect, toolbar_bg);
            canvas.rect(
                Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
                theme_color("border"),
            );
            for child in toolbar.children() {
                draw_widget(canvas, child);
            }
        }
        return;
    } else if let Some(scroll) = widget.as_any().downcast_ref::<ScrollView>() {
        let scroll_bg = if render_dark_mode() {
            [0.11, 0.11, 0.12, 1.0]
        } else {
            [0.97, 0.97, 0.96, 1.0]
        };
        canvas.rect(rect, scroll_bg);
        canvas.stroke(rect, theme_color("border"));
        canvas.with_clip(rect, |canvas| {
            for child in scroll.children() {
                draw_widget(canvas, child);
            }
        });
        return;
    } else if widget.as_any().is::<SplitView>() {
        let split_bg = if render_dark_mode() {
            [0.13, 0.14, 0.14, 1.0]
        } else {
            [0.90, 0.90, 0.88, 1.0]
        };
        canvas.rect(rect, split_bg);
        if let Some(split) = widget.as_any().downcast_ref::<SplitView>() {
            let divider = match split.direction {
                slopos_kit::split_view::SplitDirection::Horizontal => Rect::new(
                    rect.x + rect.width * split.divider_position,
                    rect.y,
                    split.divider_size,
                    rect.height,
                ),
                slopos_kit::split_view::SplitDirection::Vertical => Rect::new(
                    rect.x,
                    rect.y + rect.height * split.divider_position,
                    rect.width,
                    split.divider_size,
                ),
            };
            let divider_bg = if render_dark_mode() {
                [0.27, 0.28, 0.29, 1.0]
            } else {
                [0.71, 0.71, 0.69, 1.0]
            };
            canvas.rect(divider, divider_bg);
            canvas.stroke(divider, theme_color("border"));
        }
    } else if let Some(grid) = widget.as_any().downcast_ref::<MonospaceView>() {
        draw_monospace_view(canvas, rect, grid);
        return;
    } else if let Some(status) = widget.as_any().downcast_ref::<StatusBar>() {
        let status_bg = if render_dark_mode() {
            [0.14, 0.15, 0.15, 1.0]
        } else {
            [0.86, 0.86, 0.85, 1.0]
        };
        canvas.rect(rect, status_bg);
        canvas.rect(
            Rect::new(rect.x, rect.y, rect.width, 1.0),
            theme_color("border"),
        );
        let mut x = rect.x + 8.0;
        for item in &status.items {
            canvas.text(&item.text, x, rect.y + 8.0, theme_color("text"));
            x += item.width.max(item.text.len() as f32 * 7.0 + 12.0);
        }
    } else if let Some(dialog) = widget.as_any().downcast_ref::<Dialog>() {
        draw_dialog(canvas, rect, dialog);
        return;
    } else if let Some(pb) = widget.as_any().downcast_ref::<PopupButton>() {
        draw_popup_button(canvas, rect, pb);
        return;
    } else if let Some(pb) = widget.as_any().downcast_ref::<ProgressBar>() {
        draw_progress_bar(canvas, rect, pb);
        return;
    } else if let Some(tv) = widget.as_any().downcast_ref::<TabView>() {
        draw_tab_view(canvas, rect, tv);
        return;
    } else if let Some(dock) = widget.as_any().downcast_ref::<DockView>() {
        draw_dock_view(canvas, rect, dock);
        return;
    } else if let Some(grid) = widget.as_any().downcast_ref::<WorkspaceGridView>() {
        draw_workspace_grid_view(canvas, rect, grid);
        return;
    } else if let Some(layout_view) = widget.as_any().downcast_ref::<LayoutView>() {
        draw_layout(canvas, &layout_view.layout);
        return;
    } else if let Some(panel) = widget.as_any().downcast_ref::<Panel>() {
        let fill = if panel.themed {
            theme_color("window_bg")
        } else {
            panel.fill
        };
        canvas.rect(rect, fill);
        if panel.beveled {
            draw_beveled_rect(canvas, rect, fill, panel.raised);
        } else if panel.bordered {
            canvas.stroke(rect, theme_color("border"));
        }
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

fn draw_dialog(canvas: &mut Canvas<'_>, rect: Rect, dialog: &Dialog) {
    // Background and outer border - use theme colors
    let bg = theme_color("window_bg");
    canvas.rect(rect, bg);
    canvas.stroke(rect, theme_color("border"));

    // Title bar area
    let titlebar_rect = Rect::new(rect.x, rect.y, rect.width, 32.0);
    let titlebar_bg = if render_dark_mode() {
        [0.21, 0.22, 0.23, 1.0]
    } else {
        [0.85, 0.85, 0.84, 1.0]
    };
    canvas.rect(titlebar_rect, titlebar_bg);

    // Title bar highlight (raised bevel top edge)
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, 1.0),
        theme_color("edge_light"),
    );

    // Title text centered in title bar
    let title = &dialog.title;
    let title_w = canvas.measure_text(title);
    let title_x = (rect.x + (rect.width - title_w) * 0.5).round();
    canvas.text(title, title_x, rect.y + 10.0, theme_color("text"));

    // Horizontal separator below title
    canvas.rect(
        Rect::new(rect.x, rect.y + 32.0, rect.width, 1.0),
        if render_dark_mode() {
            [0.31, 0.32, 0.33, 1.0]
        } else {
            [0.57, 0.57, 0.55, 1.0]
        },
    );

    // Message text
    canvas.text(
        &dialog.message,
        rect.x + 12.0,
        rect.y + 42.0,
        theme_color("text"),
    );

    // Draw buttons right-aligned at the bottom
    let btn_h = 24.0;
    let btn_y = rect.y + rect.height - btn_h - 10.0;
    let mut btn_x = rect.x + rect.width - 10.0;
    for btn in dialog.buttons.iter().rev() {
        let label = btn.label();
        let label_w = canvas.measure_text(label);
        let btn_w = (label_w + 20.0).max(72.0);
        btn_x -= btn_w;
        let btn_rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
        let btn_bg = theme_color("button_bg");
        canvas.rect(btn_rect, btn_bg);
        draw_beveled_rect(canvas, btn_rect, btn_bg, true);
        let text_x = (btn_rect.x + (btn_w - label_w) * 0.5).round();
        canvas.text(label, text_x, btn_rect.y + 6.0, theme_color("text"));
        btn_x -= 8.0;
    }
}

fn draw_popup_button(canvas: &mut Canvas<'_>, rect: Rect, pb: &PopupButton) {
    // Background with beveled raised look
    let bg = theme_color("button_bg");
    canvas.rect(rect, bg);
    draw_beveled_rect(canvas, rect, bg, true);

    // Selected title text, left-aligned with some padding
    let label = pb.selected_title().unwrap_or("");
    canvas.text(
        label,
        rect.x + 8.0,
        rect.y + (rect.height - 12.0) * 0.5,
        theme_color("text"),
    );

    // Down-arrow indicator on the right side
    // Draw a small triangle using three thin horizontal rects
    let arrow_x = rect.x + rect.width - 14.0;
    let arrow_y = rect.y + rect.height * 0.5 - 2.0;
    let arrow_color = if render_dark_mode() {
        [0.71, 0.71, 0.69, 1.0]
    } else {
        [0.24, 0.24, 0.23, 1.0]
    };
    canvas.rect(Rect::new(arrow_x, arrow_y, 7.0, 1.0), arrow_color);
    canvas.rect(
        Rect::new(arrow_x + 1.0, arrow_y + 1.0, 5.0, 1.0),
        arrow_color,
    );
    canvas.rect(
        Rect::new(arrow_x + 2.0, arrow_y + 2.0, 3.0, 1.0),
        arrow_color,
    );
    canvas.rect(
        Rect::new(arrow_x + 3.0, arrow_y + 3.0, 1.0, 1.0),
        arrow_color,
    );

    // Separator line between label area and arrow area
    canvas.rect(
        Rect::new(
            rect.x + rect.width - 18.0,
            rect.y + 2.0,
            1.0,
            rect.height - 4.0,
        ),
        theme_color("border"),
    );

    // Shadow line at bottom-right for depth
    let shadow_color = theme_color("edge_dark");
    canvas.rect(
        Rect::new(
            rect.x + 1.0,
            rect.y + rect.height - 1.0,
            rect.width - 1.0,
            1.0,
        ),
        shadow_color,
    );
    canvas.rect(
        Rect::new(
            rect.x + rect.width - 1.0,
            rect.y + 1.0,
            1.0,
            rect.height - 1.0,
        ),
        shadow_color,
    );
}

fn draw_progress_bar(canvas: &mut Canvas<'_>, rect: Rect, pb: &ProgressBar) {
    let pb_bg = if render_dark_mode() {
        [0.09, 0.10, 0.11, 1.0]
    } else {
        [0.93, 0.93, 0.91, 1.0]
    };
    canvas.rect(rect, pb_bg);
    canvas.stroke(rect, theme_color("border"));
    let ratio = if pb.max > 0.0 { pb.value / pb.max } else { 0.0 };
    let fill_width = (rect.width - 4.0) * ratio.clamp(0.0, 1.0);
    if fill_width > 0.0 {
        let fill = Rect::new(rect.x + 2.0, rect.y + 2.0, fill_width, rect.height - 4.0);
        let accent = render_accent();
        canvas.rect(fill, accent);
    }
}

fn draw_workspace_grid_view(canvas: &mut Canvas<'_>, _rect: Rect, grid: &WorkspaceGridView) {
    // Cell geometry comes from the widget — the same rects its
    // `handle_event` hit-tests, so paint and input cannot drift.
    for i in 0..4 {
        let cell_r = grid.cell_rect(i);
        let cell_w = cell_r.width;
        let cell_h = cell_r.height;
        let cell_x = cell_r.x;
        let cell_y = cell_r.y;

        let bg_color = if i == grid.active_index {
            if render_dark_mode() {
                [0.19, 0.27, 0.38, 1.0]
            } else {
                [0.80, 0.87, 0.94, 1.0]
            }
        } else {
            if render_dark_mode() {
                [0.15, 0.15, 0.16, 1.0]
            } else {
                [0.94, 0.94, 0.92, 1.0]
            }
        };
        canvas.rect(cell_r, bg_color);

        let border_color = if i == grid.active_index {
            if render_dark_mode() {
                [0.55, 0.71, 0.94, 1.0]
            } else {
                [0.04, 0.31, 0.63, 1.0]
            }
        } else {
            theme_color("border")
        };

        canvas.stroke(cell_r, border_color);
        if i == grid.active_index {
            canvas.stroke(
                Rect::new(
                    cell_r.x + 1.0,
                    cell_r.y + 1.0,
                    cell_r.width - 2.0,
                    cell_r.height - 2.0,
                ),
                border_color,
            );
        }

        let label = grid
            .items
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("Desktop {}", i + 1));
        let text_color = if i == grid.active_index {
            if render_dark_mode() {
                [0.90, 0.94, 1.0, 1.0]
            } else {
                [0.04, 0.16, 0.31, 1.0]
            }
        } else {
            theme_color("text")
        };
        canvas.text(
            &label,
            cell_x + (cell_w - label.len() as f32 * 7.0) * 0.5,
            cell_y + (cell_h - 12.0) * 0.5 + 2.0,
            text_color,
        );
    }
}

fn draw_tab_view(canvas: &mut Canvas<'_>, rect: Rect, tv: &TabView) {
    let header_height = 30.0;
    let divider_y = rect.y + header_height - 1.0;
    canvas.rect(
        Rect::new(rect.x, divider_y, rect.width, 1.0),
        theme_color("border"),
    );
    let mut current_x = rect.x + 8.0;
    for (i, tab) in tv.tabs.iter().enumerate() {
        let title_w = canvas.measure_text(&tab.title);
        let tab_width = title_w + 24.0;
        let tab_rect = Rect::new(current_x, rect.y + 4.0, tab_width, 25.0);
        let is_selected = tv.selected_tab_index == i;
        if is_selected {
            let tab_bg = theme_color("button_bg");
            canvas.rect(tab_rect, tab_bg);
            draw_beveled_rect(canvas, tab_rect, tab_bg, true);
            canvas.rect(
                Rect::new(tab_rect.x + 1.0, divider_y, tab_rect.width - 2.0, 1.0),
                tab_bg,
            );
            // Accent underline on selected tab
            canvas.rect(
                Rect::new(tab_rect.x + 2.0, tab_rect.y, tab_rect.width - 4.0, 2.0),
                render_accent(),
            );
        } else {
            let inactive_bg = if render_dark_mode() {
                [0.12, 0.12, 0.13, 1.0]
            } else {
                [0.82, 0.82, 0.80, 1.0]
            };
            canvas.rect(tab_rect, inactive_bg);
            draw_beveled_rect(canvas, tab_rect, inactive_bg, false);
        }
        let text_color = if is_selected {
            theme_color("text")
        } else {
            if render_dark_mode() {
                [0.55, 0.55, 0.53, 1.0]
            } else {
                [0.39, 0.39, 0.37, 1.0]
            }
        };
        let text_x = (tab_rect.x + (tab_width - title_w) * 0.5).round();
        canvas.text(&tab.title, text_x, tab_rect.y + 7.0, text_color);
        current_x += tab_width + 4.0;
    }
    if let Some(content) = tv.selected_content() {
        draw_widget(canvas, content);
    }
}

fn draw_dock_view(canvas: &mut Canvas<'_>, _rect: Rect, dock: &DockView) {
    if dock.items.is_empty() {
        return;
    }

    // Geometry comes from the widget itself — the same rects its
    // `handle_event` hit-tests, so paint and input cannot drift.
    let dock_rect = dock.strip_rect();

    let bg_color = theme_face();
    canvas.rect(dock_rect, bg_color);
    draw_beveled_rect(canvas, dock_rect, bg_color, true);
    draw_system7_3d_border(canvas, dock_rect);

    for (i, item) in dock.items.iter().enumerate() {
        let item_rect = dock.item_rect(i);

        if item.is_focused {
            let highlight_rect = Rect::new(
                item_rect.x - 2.0,
                item_rect.y - 2.0,
                item_rect.width + 4.0,
                item_rect.height + 4.0,
            );
            let focus_color = if render_dark_mode() {
                S7_LAVENDER300
            } else {
                S7_LAVENDER100
            };
            canvas.rect(highlight_rect, focus_color);
            draw_beveled_rect(canvas, highlight_rect, focus_color, false);
        }

        let icon_bg = theme_paper();
        canvas.rect(item_rect, icon_bg);
        draw_beveled_rect(canvas, item_rect, icon_bg, true);

        let symbol_x = item_rect.x + (item_rect.width - 32.0) * 0.5;
        let symbol_y = item_rect.y + (item_rect.height - 32.0) * 0.5;
        draw_labeled_icon(canvas, item.label.as_str(), symbol_x, symbol_y);

        if item.is_running {
            canvas.rect(
                Rect::new(
                    item_rect.x + item_rect.width * 0.5 - 2.0,
                    item_rect.y + item_rect.height - 5.0,
                    4.0,
                    4.0,
                ),
                theme_ink(),
            );
        }
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
    let menu_bar_bg = if render_dark_mode() {
        [0.11, 0.11, 0.12, 1.0]
    } else {
        [0.93, 0.93, 0.93, 1.0]
    };
    canvas.rect(rect, menu_bar_bg);
    canvas.rect(
        Rect::new(rect.x, rect.y, rect.width, 1.0),
        theme_color("edge_light"),
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 2.0, rect.width, 1.0),
        if render_dark_mode() {
            [0.4, 0.4, 0.4, 1.0]
        } else {
            [0.37, 0.37, 0.37, 1.0]
        },
    );
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        theme_color("edge_dark"),
    );

    let mut x = rect.x + 10.0;
    draw_slopos_menu_logo(canvas, x + 1.0, rect.y + 6.0, false);
    x += 18.0;

    for child in toolbar.children() {
        if let Some(button) = child.as_any().downcast_ref::<Button>() {
            let label = button.label();
            canvas.text(label, x, rect.y + 8.0, theme_color("text"));
            x += label.len() as f32 * 8.0 + 18.0;
        }
    }

    let right_label = menu_status_label();
    let right_w = canvas.measure_text(&right_label);
    canvas.text(
        &right_label,
        rect.x + rect.width - right_w - 72.0,
        rect.y + 8.0,
        theme_color("text"),
    );
    draw_status_glyph(canvas, rect.x + rect.width - 42.0, rect.y + 7.0);
    draw_status_glyph(canvas, rect.x + rect.width - 22.0, rect.y + 7.0);
}

fn draw_menu_bar_widget(canvas: &mut Canvas<'_>, rect: Rect, menu_bar: &MenuBar) {
    if menu_bar.layer_popup_origin {
        if let Some(menu_index) = menu_bar.open_menu {
            draw_open_menu_at_origin(canvas, menu_bar, menu_index);
        }
        return;
    }

    // System 7 menu bar: graphite/platinum face + ink bottom rule
    let menu_bar_bg = theme_menu();
    canvas.rect(rect, menu_bar_bg);
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        theme_ink(),
    );

    for (index, menu) in menu_bar.menus.iter().enumerate() {
        let Some(menu_rect) = menu_bar.menu_rects().get(index).copied() else {
            continue;
        };
        let active = menu_bar.open_menu == Some(index) || menu_bar.hovered_menu == Some(index);
        if active {
            // Classic inverted selection — System 7 style
            let highlight_color = if render_dark_mode() {
                S7_LAVENDER300
            } else {
                S7_FG
            };
            canvas.rect(
                Rect::new(
                    menu_rect.x + 1.0,
                    menu_rect.y + 2.0,
                    menu_rect.width - 2.0,
                    20.0,
                ),
                highlight_color,
            );
        }
        if index == 0 {
            draw_slopos_menu_logo(canvas, menu_rect.x + 4.0, menu_rect.y + 6.0, active);
            canvas.text(
                &menu.title,
                menu_rect.x + 18.0,
                menu_rect.y + 8.0,
                if active {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    theme_ink()
                },
            );
        } else {
            canvas.text(
                &menu.title,
                menu_rect.x + 8.0,
                menu_rect.y + 8.0,
                if active {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    theme_ink()
                },
            );
        }
    }

    let right_label = menu_status_label();
    let right_w = canvas.measure_text(&right_label);
    canvas.text(
        &right_label,
        rect.x + rect.width - right_w - 72.0,
        rect.y + 8.0,
        theme_ink(),
    );
    draw_status_glyph(canvas, rect.x + rect.width - 42.0, rect.y + 7.0);
    draw_status_glyph(canvas, rect.x + rect.width - 22.0, rect.y + 7.0);

    if let Some(menu_index) = menu_bar.open_menu {
        if !menu_bar.suppress_dropdown_paint {
            draw_open_menu(canvas, menu_bar, menu_index);
        }
    }
}

fn draw_slopos_menu_logo(canvas: &mut Canvas<'_>, x: f32, y: f32, active: bool) {
    let main_color = if active {
        [1.0, 1.0, 1.0, 1.0]
    } else {
        theme_ink()
    };
    let accent_color = if active {
        S7_LAVENDER300
    } else if render_dark_mode() {
        [0.4, 0.4, 0.6, 1.0]
    } else {
        S7_LAVENDER100
    };

    // Retro monitor bezel (10x8)
    canvas.rect(Rect::new(x, y, 10.0, 8.0), main_color);
    // Monitor screen interior (6x5)
    canvas.rect(Rect::new(x + 2.0, y + 1.5, 6.0, 5.0), accent_color);
    // 'S' logo symbol inside screen
    let s_color = if active {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        theme_paper()
    };
    canvas.rect(Rect::new(x + 3.0, y + 2.0, 4.0, 1.0), s_color);
    canvas.rect(Rect::new(x + 3.0, y + 3.0, 2.0, 1.0), s_color);
    canvas.rect(Rect::new(x + 3.0, y + 4.0, 4.0, 1.0), s_color);
    canvas.rect(Rect::new(x + 5.0, y + 5.0, 2.0, 1.0), s_color);

    // Keyboard base (12x2)
    canvas.rect(Rect::new(x - 1.0, y + 9.0, 12.0, 2.0), main_color);
}

fn draw_open_menu(canvas: &mut Canvas<'_>, menu_bar: &MenuBar, menu_index: usize) {
    let Some(dropdown) = menu_bar.dropdown_rect(menu_index) else {
        return;
    };
    draw_open_menu_box(canvas, menu_bar, menu_index, dropdown, 0.0, 0.0);
}

/// Draw the open dropdown with its top-left at (0,0) for an Overlay layer surface.
fn draw_open_menu_at_origin(canvas: &mut Canvas<'_>, menu_bar: &MenuBar, menu_index: usize) {
    let Some(dropdown) = menu_bar.dropdown_rect(menu_index) else {
        return;
    };
    draw_open_menu_box(
        canvas,
        menu_bar,
        menu_index,
        Rect::new(0.0, 0.0, dropdown.width, dropdown.height),
        -dropdown.x,
        -dropdown.y,
    );
}

fn draw_open_menu_box(
    canvas: &mut Canvas<'_>,
    menu_bar: &MenuBar,
    menu_index: usize,
    dropdown: Rect,
    item_dx: f32,
    item_dy: f32,
) {
    let Some(menu) = menu_bar.menus.get(menu_index) else {
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
    let menu_bg = if render_dark_mode() {
        [0.16, 0.17, 0.18, 1.0]
    } else {
        [0.96, 0.96, 0.93, 1.0]
    };
    draw_beveled_rect(canvas, dropdown, menu_bg, true);
    canvas.rect(
        Rect::new(
            dropdown.x + 4.0,
            dropdown.y + 4.0,
            dropdown.width - 8.0,
            1.0,
        ),
        theme_color("edge_light"),
    );
    canvas.rect(
        Rect::new(
            dropdown.x + 4.0,
            dropdown.y + 4.0,
            1.0,
            dropdown.height - 8.0,
        ),
        theme_color("edge_light"),
    );

    for (item_index, item) in menu.items.iter().enumerate() {
        let Some(mut item_rect) = menu_bar.item_rect(menu_index, item_index) else {
            continue;
        };
        item_rect.x += item_dx;
        item_rect.y += item_dy;
        if matches!(item.kind, MenuItemKind::Separator) {
            let sep_dark = if render_dark_mode() {
                [0.45, 0.46, 0.47, 1.0]
            } else {
                [0.47, 0.47, 0.45, 1.0]
            };
            let sep_light = if render_dark_mode() {
                [0.11, 0.12, 0.12, 1.0]
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };
            canvas.rect(
                Rect::new(
                    item_rect.x + 12.0,
                    item_rect.y + 9.0,
                    item_rect.width - 24.0,
                    1.0,
                ),
                sep_dark,
            );
            canvas.rect(
                Rect::new(
                    item_rect.x + 12.0,
                    item_rect.y + 10.0,
                    item_rect.width - 24.0,
                    1.0,
                ),
                sep_light,
            );
            continue;
        }

        let hovered = menu_bar.hovered_item == Some(item_index);
        if hovered && item.enabled {
            let highlight_color = if render_dark_mode() {
                [0.32, 0.35, 0.41, 1.0]
            } else {
                [0.09, 0.09, 0.09, 1.0]
            };
            canvas.rect(item_rect, highlight_color);
        }
        let text_color = if !item.enabled {
            if render_dark_mode() {
                [0.45, 0.46, 0.47, 1.0]
            } else {
                [0.52, 0.52, 0.50, 1.0]
            }
        } else if hovered {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            theme_color("text")
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
            let shortcut_w = canvas.measure_text(&shortcut);
            canvas.text(
                &shortcut,
                item_rect.x + item_rect.width - shortcut_w - 8.0,
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

/// Battery + clock for the menu bar right edge, with a single separating space.
fn menu_status_label() -> String {
    let battery = battery_status_string();
    let clock = current_time_string();
    if battery.is_empty() {
        clock
    } else {
        format!("{} {}", battery.trim_end(), clock)
    }
}

/// Returns a compact battery indicator like "[87%]" or "[87% CHG]" when a
/// battery is present, or an empty string on desktops/VMs without one.
fn battery_status_string() -> String {
    let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok());
    let Some(pct) = capacity else {
        return String::new();
    };
    if pct >= 100 {
        return String::new();
    }
    let charging = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .ok()
        .map(|s| !s.trim().eq_ignore_ascii_case("Discharging"))
        .unwrap_or(false);
    if charging {
        format!("[{}% CHG]", pct)
    } else {
        format!("[{}%]", pct)
    }
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

/// Edge helpers for System 7 multi-layer borders (System7Components recipes).
fn stroke_edges(canvas: &mut Canvas<'_>, rect: Rect, top_left: [f32; 4], bottom_right: [f32; 4]) {
    // Top + leading
    canvas.rect(Rect::new(rect.x, rect.y, rect.width, 1.0), top_left);
    canvas.rect(Rect::new(rect.x, rect.y, 1.0, rect.height), top_left);
    // Bottom + trailing
    canvas.rect(
        Rect::new(rect.x, rect.y + rect.height - 1.0, rect.width, 1.0),
        bottom_right,
    );
    canvas.rect(
        Rect::new(rect.x + rect.width - 1.0, rect.y, 1.0, rect.height),
        bottom_right,
    );
}

/// Port of System7Components `system73DBorder`:
/// black outer border + 1px bottom/right offset shadow.
fn draw_system7_3d_border(canvas: &mut Canvas<'_>, rect: Rect) {
    let fg = if render_dark_mode() {
        COLOR_DARK_BORDER
    } else {
        S7_FG
    };
    // Offset shadow (bottom/trailing)
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + rect.height, rect.width, 1.0),
        fg,
    );
    canvas.rect(
        Rect::new(rect.x + rect.width, rect.y + 1.0, 1.0, rect.height),
        fg,
    );
    // Outer black border
    canvas.stroke(rect, fg);
}

/// Port of System73DButtonStyle edge stack (raised or pressed/inset).
fn draw_beveled_rect(canvas: &mut Canvas<'_>, rect: Rect, fill: [f32; 4], raised: bool) {
    if rect.width < 4.0 || rect.height < 4.0 {
        canvas.rect(rect, fill);
        return;
    }
    canvas.rect(rect, fill);

    if render_dark_mode() {
        let light = if raised {
            COLOR_DARK_EDGE_LIGHT
        } else {
            COLOR_DARK_EDGE_DARK
        };
        let dark = if raised {
            COLOR_DARK_EDGE_DARK
        } else {
            COLOR_DARK_EDGE_LIGHT
        };
        stroke_edges(canvas, rect, light, dark);
        let inner = Rect::new(
            rect.x + 1.0,
            rect.y + 1.0,
            rect.width - 2.0,
            rect.height - 2.0,
        );
        if inner.width > 2.0 && inner.height > 2.0 {
            stroke_edges(canvas, inner, light, dark);
        }
        return;
    }

    // Light mode: three nested edge pairs from System73DButtonStyle
    if raised {
        // Outer: top/left Gray500, bottom/right Foreground
        stroke_edges(canvas, rect, S7_GRAY500, S7_FG);
        let mid = Rect::new(
            rect.x + 1.0,
            rect.y + 1.0,
            rect.width - 2.0,
            rect.height - 2.0,
        );
        if mid.width > 2.0 && mid.height > 2.0 {
            // Mid: top/left Gray100, bottom/right Gray300
            stroke_edges(canvas, mid, S7_GRAY100, S7_GRAY300);
            let inner = Rect::new(mid.x + 1.0, mid.y + 1.0, mid.width - 2.0, mid.height - 2.0);
            if inner.width > 1.0 && inner.height > 1.0 {
                // Inner: top/left White, bottom/right Gray300
                stroke_edges(canvas, inner, S7_BG, S7_GRAY300);
            }
        }
    } else {
        // Pressed: edges reverse (inset)
        stroke_edges(canvas, rect, S7_FG, S7_GRAY100);
        let mid = Rect::new(
            rect.x + 1.0,
            rect.y + 1.0,
            rect.width - 2.0,
            rect.height - 2.0,
        );
        if mid.width > 2.0 && mid.height > 2.0 {
            stroke_edges(canvas, mid, S7_GRAY500, S7_GRAY100);
            let inner = Rect::new(mid.x + 1.0, mid.y + 1.0, mid.width - 2.0, mid.height - 2.0);
            if inner.width > 1.0 && inner.height > 1.0 {
                stroke_edges(canvas, inner, S7_GRAY500, S7_GRAY300);
            }
        }
    }
}

fn draw_tree(canvas: &mut Canvas<'_>, rect: Rect, tree: &TreeView) {
    let tree_bg = if render_dark_mode() {
        [0.12, 0.13, 0.14, 1.0]
    } else {
        [0.87, 0.89, 0.90, 1.0]
    };
    canvas.rect(rect, tree_bg);
    canvas.stroke(rect, theme_color("border"));
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
        let selection_color = if render_dark_mode() {
            [0.32, 0.38, 0.49, 1.0]
        } else {
            [0.25, 0.43, 0.67, 1.0]
        };
        canvas.rect(Rect::new(x - 4.0, *y - 3.0, 170.0, 16.0), selection_color);
    }
    canvas.text(
        &node.label,
        x + depth as f32 * 12.0,
        *y,
        if selected {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            theme_color("text")
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
    // Counted and sliced in chars, never bytes: labels are user filenames and
    // may be multi-byte UTF-8; byte slicing panics on non-boundaries.
    let char_count = label.chars().count();
    if char_count <= max_len {
        return label.to_string();
    }
    let prefix = |n: usize| -> String { label.chars().take(n).collect() };
    if max_len <= 4 {
        return format!("{}...", prefix(max_len.max(3) - 3));
    }
    if let Some(pos) = label.rfind('.') {
        let ext = &label[pos..];
        let ext_chars = ext.chars().count();
        if ext_chars < max_len - 3 {
            let base_len = max_len - 3 - ext_chars;
            return format!("{}...{}", prefix(base_len), ext);
        }
    }
    format!("{}...", prefix(max_len - 3))
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
        canvas.rect(rect, theme_paper());
    }
    for item in &icon_view.items {
        let display_label = canvas.ellipsize_text(&item.label, (item.rect.width + 8.0).max(36.0));
        if item.selected {
            let sel_rect = Rect::new(
                item.rect.x - 4.0,
                item.rect.y - 2.0,
                item.rect.width + 8.0,
                52.0,
            );
            draw_selection_highlight(canvas, sel_rect);
        }
        draw_desktop_icon(canvas, item);
        let label_y = item.rect.y + 36.0;
        let text_w = canvas.measure_text(&display_label);
        let label_x = (item.rect.x + (item.rect.width - text_w) * 0.5).round();
        if item.selected {
            let plate = Rect::new(label_x - 3.0, label_y - 2.0, text_w + 6.0, 14.0);
            canvas.rect(plate, render_accent());
            canvas.text(&display_label, label_x, label_y, [1.0, 1.0, 1.0, 1.0]);
        } else if is_desktop {
            // Desktop dither needs a nameplate; window interiors do not.
            let plate = Rect::new(label_x - 3.0, label_y - 2.0, text_w + 6.0, 14.0);
            canvas.rect(plate, theme_menu());
            canvas.stroke(plate, theme_muted());
            canvas.text(&display_label, label_x, label_y, theme_ink());
        } else {
            canvas.text(&display_label, label_x, label_y, theme_ink());
        }
    }
}

fn draw_selection_highlight(canvas: &mut Canvas<'_>, rect: Rect) {
    let [r, g, b, a] = render_accent();
    let base = [r, g, b, a];
    // Lighter highlight for top/left edges
    let light = [
        (r + 0.25).min(1.0),
        (g + 0.25).min(1.0),
        (b + 0.25).min(1.0),
        a,
    ];
    // Darker shadow for bottom/right edges
    let dark = [r * 0.6, g * 0.6, b * 0.6, a];
    canvas.rect(rect, base);
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, rect.width - 2.0, 1.0),
        light,
    );
    canvas.rect(
        Rect::new(rect.x + 1.0, rect.y + 1.0, 1.0, rect.height - 2.0),
        light,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconStyle {
    Retro,
    Material,
}

pub fn current_icon_style() -> IconStyle {
    if let Ok(val) = std::env::var("SLOPOS_ICON_STYLE") {
        if val.eq_ignore_ascii_case("material") {
            return IconStyle::Material;
        }
    }
    IconStyle::Retro
}

fn draw_desktop_icon(canvas: &mut Canvas<'_>, item: &IconItem) {
    // Fixed 32×32 icon footprint centered in the cell — NEVER use full
    // item.rect width for the shadow (that painted the gray column bands).
    const ICON: f32 = 32.0;
    let x = item.rect.x + (item.rect.width - ICON) * 0.5;
    let y = item.rect.y + 2.0;
    canvas.rect(
        Rect::new(x + 2.0, y + 2.0, ICON, ICON),
        [0.0, 0.0, 0.0, 0.25],
    );

    // Prefer known desktop/app labels over generic icon kind tags.
    match item.label.as_str() {
        "Hard Disk" | "Home" | "Trash" | "Applications" | "App Store" | "Finder" | "Settings"
        | "Terminal" | "TextEdit" => {
            draw_labeled_icon(canvas, item.label.as_str(), x, y);
            return;
        }
        _ => {}
    }

    if let Some(kind) = item.icon.as_deref() {
        match kind {
            "folder" => {
                draw_folder_icon(canvas, x - 6.0, y - 4.0, rgb(226, 216, 142));
                return;
            }
            "document" => {
                draw_document_icon(canvas, x - 6.0, y - 4.0);
                return;
            }
            "image" => {
                draw_image_icon(canvas, x, y);
                return;
            }
            "audio" => {
                draw_audio_icon(canvas, x, y);
                return;
            }
            "video" => {
                draw_video_icon(canvas, x, y);
                return;
            }
            "code" => {
                draw_code_icon(canvas, x, y);
                return;
            }
            "archive" => {
                draw_archive_icon(canvas, x, y);
                return;
            }
            "network" => {
                draw_network_icon(canvas, x, y);
                return;
            }
            "user" => {
                draw_user_icon(canvas, x, y);
                return;
            }
            _ => {}
        }
    }
    draw_labeled_icon(canvas, item.label.as_str(), x, y);
}

/// Dispatch per-app icons by label with Retro or Material style support.
fn draw_labeled_icon(canvas: &mut Canvas<'_>, label: &str, x: f32, y: f32) {
    if current_icon_style() == IconStyle::Material {
        draw_material_icon(canvas, label, x, y);
        return;
    }
    match label {
        "Hard Disk" => draw_drive_icon(canvas, x - 6.0, y - 4.0),
        "Home" => draw_folder_icon(canvas, x - 6.0, y - 4.0, rgb(226, 216, 142)),
        "Trash" => draw_trash_icon(canvas, x - 4.0, y - 4.0),
        "Applications" => draw_applications_icon(canvas, x, y),
        "App Store" => draw_store_icon(canvas, x, y),
        "Finder" => draw_finder_icon(canvas, x, y),
        "Settings" => draw_settings_icon(canvas, x, y),
        "Terminal" => draw_terminal_icon(canvas, x, y),
        "TextEdit" => draw_textedit_icon(canvas, x, y),
        _ => draw_generic_app_icon(canvas, x, y),
    }
}

fn draw_material_icon(canvas: &mut Canvas<'_>, label: &str, x: f32, y: f32) {
    let card_color = match label {
        "Hard Disk" => rgb(66, 133, 244),    // Material Blue
        "Home" => rgb(251, 188, 4),          // Material Amber
        "Trash" => rgb(234, 67, 53),         // Material Red
        "Applications" => rgb(103, 58, 183), // Material Purple
        "App Store" => rgb(52, 168, 83),     // Material Green
        "Finder" => rgb(0, 172, 193),        // Material Cyan
        "Settings" => rgb(96, 125, 139),     // Material Blue Grey
        "Terminal" => rgb(38, 50, 56),       // Material Dark Slate
        "TextEdit" => rgb(255, 112, 67),     // Material Deep Orange
        _ => rgb(120, 144, 156),
    };

    // Rounded Material card base
    canvas.rect(Rect::new(x, y, 32.0, 32.0), card_color);
    canvas.rect(
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        [1.0, 1.0, 1.0, 0.15],
    );

    // Material inner glyph symbol
    match label {
        "Hard Disk" => {
            canvas.rect(
                Rect::new(x + 8.0, y + 10.0, 16.0, 12.0),
                [1.0, 1.0, 1.0, 0.9],
            );
            canvas.rect(Rect::new(x + 12.0, y + 14.0, 8.0, 4.0), card_color);
        }
        "Home" | "folder" => {
            canvas.rect(
                Rect::new(x + 6.0, y + 10.0, 20.0, 14.0),
                [1.0, 1.0, 1.0, 0.9],
            );
            canvas.rect(Rect::new(x + 6.0, y + 8.0, 8.0, 4.0), [1.0, 1.0, 1.0, 0.9]);
        }
        "Trash" => {
            canvas.rect(
                Rect::new(x + 10.0, y + 10.0, 12.0, 14.0),
                [1.0, 1.0, 1.0, 0.9],
            );
            canvas.rect(Rect::new(x + 8.0, y + 8.0, 16.0, 2.0), [1.0, 1.0, 1.0, 0.9]);
        }
        "Settings" => {
            canvas.rect(
                Rect::new(x + 10.0, y + 10.0, 12.0, 12.0),
                [1.0, 1.0, 1.0, 0.9],
            );
            canvas.rect(Rect::new(x + 14.0, y + 14.0, 4.0, 4.0), card_color);
        }
        "Terminal" => {
            canvas.rect(Rect::new(x + 8.0, y + 12.0, 6.0, 2.0), rgb(80, 220, 120));
            canvas.rect(Rect::new(x + 12.0, y + 14.0, 2.0, 4.0), rgb(80, 220, 120));
            canvas.rect(
                Rect::new(x + 14.0, y + 18.0, 10.0, 2.0),
                [1.0, 1.0, 1.0, 0.9],
            );
        }
        _ => {
            canvas.rect(
                Rect::new(x + 10.0, y + 10.0, 12.0, 12.0),
                [1.0, 1.0, 1.0, 0.8],
            );
        }
    }
}

fn draw_image_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(180, 130, 210),
        true,
    );
    canvas.rect(Rect::new(x + 6.0, y + 6.0, 20.0, 20.0), theme_paper());
    canvas.rect(Rect::new(x + 10.0, y + 14.0, 12.0, 8.0), rgb(100, 160, 220));
    canvas.rect(Rect::new(x + 18.0, y + 9.0, 4.0, 4.0), rgb(240, 200, 80));
}

fn draw_audio_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(240, 160, 80),
        true,
    );
    canvas.rect(Rect::new(x + 8.0, y + 16.0, 6.0, 8.0), theme_ink());
    canvas.rect(Rect::new(x + 18.0, y + 10.0, 6.0, 8.0), theme_ink());
    canvas.rect(Rect::new(x + 12.0, y + 10.0, 12.0, 3.0), theme_ink());
}

fn draw_video_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(220, 80, 80),
        true,
    );
    canvas.rect(Rect::new(x + 6.0, y + 8.0, 20.0, 16.0), theme_paper());
    canvas.rect(Rect::new(x + 13.0, y + 12.0, 6.0, 8.0), rgb(220, 80, 80));
}

fn draw_code_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(60, 80, 120),
        true,
    );
    canvas.rect(Rect::new(x + 7.0, y + 12.0, 4.0, 8.0), rgb(80, 220, 120));
    canvas.rect(Rect::new(x + 21.0, y + 12.0, 4.0, 8.0), rgb(80, 220, 120));
    canvas.rect(Rect::new(x + 13.0, y + 10.0, 6.0, 12.0), theme_paper());
}

fn draw_archive_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(200, 140, 60),
        true,
    );
    canvas.rect(Rect::new(x + 6.0, y + 8.0, 20.0, 6.0), theme_paper());
    canvas.rect(Rect::new(x + 8.0, y + 14.0, 16.0, 12.0), theme_paper());
    canvas.rect(Rect::new(x + 14.0, y + 16.0, 4.0, 4.0), theme_ink());
}

fn draw_network_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(60, 160, 200),
        true,
    );
    canvas.rect(Rect::new(x + 8.0, y + 8.0, 16.0, 16.0), theme_paper());
    canvas.rect(Rect::new(x + 10.0, y + 10.0, 12.0, 12.0), rgb(60, 160, 200));
}

fn draw_user_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 2.0, y + 2.0, 28.0, 28.0),
        rgb(0, 150, 136),
        true,
    );
    canvas.rect(Rect::new(x + 12.0, y + 8.0, 8.0, 8.0), theme_paper());
    canvas.rect(Rect::new(x + 8.0, y + 18.0, 16.0, 8.0), theme_paper());
}

fn draw_drive_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    // Disk casing — theme-aware Graphite/Platinum
    draw_beveled_rect(
        canvas,
        Rect::new(x, y + 8.0, 44.0, 28.0),
        theme_face(),
        true,
    );
    // Disc slot
    canvas.rect(Rect::new(x + 6.0, y + 14.0, 32.0, 3.0), theme_ink());
    // LED Dot
    canvas.rect(Rect::new(x + 34.0, y + 26.0, 4.0, 4.0), rgb(80, 220, 80));
}

fn draw_document_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 8.0, y + 4.0, 28.0, 36.0),
        theme_paper(),
        true,
    );
    canvas.rect(Rect::new(x + 13.0, y + 12.0, 1.0, 20.0), S7_LAVENDER300);
    canvas.rect(Rect::new(x + 16.0, y + 15.0, 14.0, 1.0), theme_muted());
    canvas.rect(Rect::new(x + 16.0, y + 21.0, 16.0, 1.0), theme_muted());
    canvas.rect(Rect::new(x + 16.0, y + 27.0, 12.0, 1.0), theme_muted());
    canvas.rect(Rect::new(x + 29.0, y + 4.0, 7.0, 7.0), theme_face());
    canvas.rect(Rect::new(x + 29.0, y + 11.0, 8.0, 1.0), theme_muted());
    canvas.rect(Rect::new(x + 28.0, y + 4.0, 1.0, 8.0), theme_muted());
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
    draw_generic_app_icon(canvas, x + 6.0, y + 6.0);
}

/// Generic app (fallback) — stamped rectangle, theme-aware.
fn draw_generic_app_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(canvas, Rect::new(x, y, 32.0, 32.0), theme_face(), true);
    canvas.rect(Rect::new(x + 6.0, y + 8.0, 20.0, 3.0), theme_muted());
    canvas.rect(Rect::new(x + 6.0, y + 14.0, 16.0, 3.0), theme_muted());
    canvas.rect(Rect::new(x + 6.0, y + 20.0, 12.0, 3.0), theme_muted());
}

/// Face/window finder metaphor — not Apple logo.
fn draw_finder_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(canvas, Rect::new(x, y, 32.0, 32.0), theme_face(), true);
    canvas.rect(Rect::new(x + 4.0, y + 4.0, 24.0, 6.0), theme_muted());
    canvas.rect(Rect::new(x + 6.0, y + 5.0, 4.0, 4.0), theme_paper());
    let pane = if render_dark_mode() {
        [0.35, 0.35, 0.45, 1.0]
    } else {
        S7_LAVENDER100
    };
    canvas.rect(Rect::new(x + 4.0, y + 12.0, 10.0, 14.0), pane);
    canvas.rect(Rect::new(x + 16.0, y + 12.0, 12.0, 14.0), theme_paper());
    canvas.stroke(Rect::new(x + 4.0, y + 12.0, 24.0, 14.0), theme_ink());
}

fn draw_settings_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(canvas, Rect::new(x, y, 32.0, 32.0), theme_face(), true);
    canvas.rect(Rect::new(x + 10.0, y + 6.0, 12.0, 4.0), theme_muted());
    canvas.rect(Rect::new(x + 10.0, y + 22.0, 12.0, 4.0), theme_muted());
    canvas.rect(Rect::new(x + 6.0, y + 10.0, 4.0, 12.0), theme_muted());
    canvas.rect(Rect::new(x + 22.0, y + 10.0, 4.0, 12.0), theme_muted());
    canvas.rect(Rect::new(x + 11.0, y + 11.0, 10.0, 10.0), theme_muted());
    canvas.rect(Rect::new(x + 14.0, y + 14.0, 4.0, 4.0), theme_paper());
}

fn draw_terminal_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    let screen = if render_dark_mode() {
        [0.08, 0.10, 0.10, 1.0]
    } else {
        rgb(40, 44, 48)
    };
    draw_beveled_rect(canvas, Rect::new(x, y, 32.0, 32.0), screen, true);
    canvas.rect(Rect::new(x + 6.0, y + 10.0, 8.0, 2.0), rgb(80, 220, 120));
    canvas.rect(Rect::new(x + 6.0, y + 14.0, 2.0, 8.0), rgb(80, 220, 120));
    canvas.rect(Rect::new(x + 10.0, y + 20.0, 14.0, 2.0), theme_muted());
}

fn draw_textedit_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(
        canvas,
        Rect::new(x + 4.0, y + 2.0, 24.0, 28.0),
        theme_paper(),
        true,
    );
    canvas.rect(Rect::new(x + 20.0, y + 2.0, 8.0, 8.0), theme_face());
    canvas.rect(Rect::new(x + 8.0, y + 12.0, 16.0, 2.0), theme_ink());
    canvas.rect(Rect::new(x + 8.0, y + 17.0, 14.0, 2.0), theme_muted());
    canvas.rect(Rect::new(x + 8.0, y + 22.0, 12.0, 2.0), theme_muted());
}

fn draw_store_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    let bag_bg = if render_dark_mode() {
        [0.30, 0.30, 0.40, 1.0]
    } else {
        S7_LAVENDER100
    };
    draw_beveled_rect(canvas, Rect::new(x, y, 32.0, 32.0), bag_bg, true);
    canvas.rect(Rect::new(x + 8.0, y + 12.0, 16.0, 14.0), theme_paper());
    canvas.stroke(Rect::new(x + 8.0, y + 12.0, 16.0, 14.0), theme_ink());
    canvas.rect(Rect::new(x + 12.0, y + 8.0, 8.0, 6.0), theme_muted());
    canvas.rect(Rect::new(x + 14.0, y + 16.0, 4.0, 6.0), S7_LAVENDER300);
}

fn draw_applications_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    draw_beveled_rect(canvas, Rect::new(x, y, 32.0, 32.0), theme_face(), true);
    let tile = theme_muted();
    for (dx, dy) in [(5.0, 5.0), (17.0, 5.0), (5.0, 17.0), (17.0, 17.0)] {
        canvas.rect(Rect::new(x + dx, y + dy, 10.0, 10.0), tile);
        canvas.rect(
            Rect::new(x + dx + 2.0, y + dy + 2.0, 6.0, 2.0),
            theme_paper(),
        );
    }
}

fn draw_trash_icon(canvas: &mut Canvas<'_>, x: f32, y: f32) {
    let lid_color = theme_face();
    let body_color = theme_muted();
    let shadow_color = theme_ink();

    // Handle
    canvas.rect(Rect::new(x + 18.0, y + 2.0, 8.0, 3.0), lid_color);
    canvas.rect(Rect::new(x + 19.0, y + 1.0, 6.0, 1.0), theme_paper());

    // Lid rim
    draw_beveled_rect(
        canvas,
        Rect::new(x + 6.0, y + 5.0, 32.0, 5.0),
        lid_color,
        true,
    );

    // Can body
    draw_beveled_rect(
        canvas,
        Rect::new(x + 9.0, y + 10.0, 26.0, 34.0),
        body_color,
        true,
    );

    // Rib highlights
    for offset in [14.0, 20.0, 26.0, 32.0] {
        canvas.rect(Rect::new(x + offset, y + 14.0, 1.0, 26.0), shadow_color);
        canvas.rect(
            Rect::new(x + offset + 1.0, y + 14.0, 1.0, 26.0),
            theme_face(),
        );
    }
}

fn draw_list(canvas: &mut Canvas<'_>, rect: Rect, list: &ListView) {
    let list_bg = if render_dark_mode() {
        [0.09, 0.10, 0.11, 1.0]
    } else {
        [1.0, 1.0, 0.99, 1.0]
    };
    canvas.rect(rect, list_bg);
    canvas.stroke(rect, theme_color("border"));
    for (index, item) in list.items.iter().enumerate() {
        let y = rect.y + 6.0 + index as f32 * 18.0;
        if list.selected_index == Some(index) {
            let selection_color = if render_dark_mode() {
                [0.32, 0.38, 0.49, 1.0]
            } else {
                [0.25, 0.43, 0.67, 1.0]
            };
            canvas.rect(
                Rect::new(rect.x + 3.0, y - 3.0, rect.width - 6.0, 16.0),
                selection_color,
            );
        }
        canvas.text(
            item,
            rect.x + 8.0,
            y,
            if list.selected_index == Some(index) {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                theme_color("text")
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

fn glyph_pattern(ch: char) -> [u8; 9] {
    match ch {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0, 0,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110, 0, 0,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111, 0, 0,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110, 0, 0,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111, 0, 0,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0, 0,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111, 0, 0,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001, 0, 0,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111, 0, 0,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100, 0, 0,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001, 0, 0,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111, 0, 0,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001, 0, 0,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001, 0, 0,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110, 0, 0,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000, 0, 0,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101, 0, 0,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001, 0, 0,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110, 0, 0,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110, 0, 0,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100, 0, 0,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010, 0, 0,
        ],
        'X' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001, 0, 0,
        ],
        'Y' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111, 0, 0,
        ],
        'a' => [
            0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111, 0, 0,
        ],
        'b' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110, 0, 0,
        ],
        'c' => [
            0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10000, 0b01110, 0, 0,
        ],
        'd' => [
            0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111, 0, 0,
        ],
        'e' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110, 0, 0,
        ],
        'f' => [
            0b00110, 0b01001, 0b01000, 0b11110, 0b01000, 0b01000, 0b01000, 0, 0,
        ],
        'g' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b01111, 0b00001, 0b01110, 0b00001, 0b01110,
        ],
        'h' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001, 0, 0,
        ],
        'i' => [
            0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110, 0, 0,
        ],
        'j' => [
            0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100, 0b10010, 0b01100,
        ],
        'k' => [
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0, 0,
        ],
        'l' => [
            0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110, 0, 0,
        ],
        'm' => [
            0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10101, 0b10101, 0, 0,
        ],
        'n' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001, 0, 0,
        ],
        'o' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110, 0, 0,
        ],
        'p' => [
            0b00000, 0b00000, 0b01100, 0b01010, 0b01100, 0b01000, 0b01000, 0b01000, 0b01000,
        ],
        'q' => [
            0b00000, 0b00000, 0b01100, 0b10100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'r' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000, 0, 0,
        ],
        's' => [
            0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110, 0, 0,
        ],
        't' => [
            0b00100, 0b00100, 0b11110, 0b00100, 0b00100, 0b00100, 0b00011, 0, 0,
        ],
        'u' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101, 0, 0,
        ],
        'v' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100, 0, 0,
        ],
        'w' => [
            0b00000, 0b00000, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010, 0, 0,
        ],
        'x' => [
            0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0, 0,
        ],
        'y' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110, 0b00001, 0b01110,
        ],
        'z' => [
            0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111, 0, 0,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110, 0, 0,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110, 0, 0,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111, 0, 0,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110, 0, 0,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010, 0, 0,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110, 0, 0,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110, 0, 0,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0, 0,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110, 0, 0,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110, 0, 0,
        ],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0, 0, 0],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b11111, 0, 0],
        '.' => [0, 0, 0, 0, 0, 0b01100, 0b01100, 0, 0],
        ':' => [0, 0b01100, 0b01100, 0, 0b01100, 0b01100, 0, 0, 0],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000, 0, 0,
        ],
        '\\' => [
            0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001, 0, 0,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010, 0, 0,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000, 0, 0,
        ],
        ',' => [0, 0, 0, 0, 0, 0b01100, 0b00100, 0b01100, 0b00100],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100, 0, 0, 0],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0, 0b00100, 0, 0,
        ],
        '=' => [0, 0, 0b11111, 0, 0b11111, 0, 0, 0, 0],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101, 0, 0,
        ],
        ' ' => [0, 0, 0, 0, 0, 0, 0, 0, 0],
        _ => [
            0b11111, 0b10001, 0b00010, 0b00100, 0b00000, 0b00100, 0b00100, 0, 0,
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
    use slopos_kit::event::KeyCode as RKey;
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
    use super::{format_clock_from_seconds, parse_theme_preference, theme_accents};

    #[test]
    fn parses_dark_appearance_preference() {
        assert!(parse_theme_preference("appearance=dark\n").0);
        assert!(parse_theme_preference("appearance=Dark\n").0);
    }

    #[test]
    fn ignores_non_dark_appearance_preferences() {
        assert!(!parse_theme_preference("appearance=light\n").0);
        assert!(!parse_theme_preference("appearance=system\n").0);
        assert!(!parse_theme_preference("other=dark\n").0);
    }

    #[test]
    fn parses_named_theme_grape() {
        let (is_dark, accent) = parse_theme_preference("theme=grape\n");
        assert!(is_dark);
        assert_eq!(accent, theme_accents::GRAPE);
    }

    #[test]
    fn parses_named_theme_strawberry() {
        let (is_dark, accent) = parse_theme_preference("theme=strawberry\n");
        assert!(!is_dark);
        assert_eq!(accent, theme_accents::STRAWBERRY);
    }

    #[test]
    fn theme_key_overrides_appearance_key() {
        let content = "appearance=dark\ntheme=classic\n";
        let (is_dark, accent) = parse_theme_preference(content);
        assert!(!is_dark);
        assert_eq!(accent, theme_accents::CLASSIC);
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
