pub mod application_registry;
pub mod desktop_manager;
pub mod dock;
pub mod launch_services;
pub mod menu_server;
pub mod notification_center;
pub mod session_manager;
pub mod theme_manager;
pub mod window_manager;
pub mod workspace_manager;

pub use application_registry::ApplicationRegistry;
pub use desktop_manager::DesktopManager;
pub use dock::Dock;
pub use launch_services::LaunchServices;
pub use menu_server::MenuServer;
pub use notification_center::NotificationCenter;
pub use session_manager::SessionManager;
pub use theme_manager::ThemeManager;
pub use window_manager::WindowManager;
pub use workspace_manager::WorkspaceManager;

use parking_lot::RwLock;
use retro_kit::event::MouseButton;
use retro_kit::icon_view::{IconItem, IconView};
use retro_kit::menu_bar::MenuBar;
use retro_kit::theme::ThemeContext;
use retro_kit::window::Window;
use retro_kit::{Event, EventResult, LayoutConstraint, Point, Rect, Size, Widget, WidgetState};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, ShellError>;

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("service error: {0}")]
    Service(String),
    #[error("window error: {0}")]
    Window(String),
    #[error("launch error: {0}")]
    Launch(String),
    #[error("theme error: {0}")]
    Theme(String),
    #[error("menu error: {0}")]
    Menu(String),
}

pub struct RetroShell {
    pub menu_server: Arc<RwLock<MenuServer>>,
    pub window_manager: Arc<RwLock<WindowManager>>,
    pub desktop_manager: Arc<RwLock<DesktopManager>>,
    pub dock: Arc<RwLock<Dock>>,
    pub notification_center: Arc<RwLock<NotificationCenter>>,
    pub workspace_manager: Arc<RwLock<WorkspaceManager>>,
    pub launch_services: Arc<RwLock<LaunchServices>>,
    pub session_manager: Arc<RwLock<SessionManager>>,
    pub theme_manager: Arc<RwLock<ThemeManager>>,
    pub application_registry: Arc<RwLock<ApplicationRegistry>>,
}

impl Default for RetroShell {
    fn default() -> Self {
        Self::new()
    }
}

impl RetroShell {
    pub fn new() -> Self {
        Self {
            menu_server: Arc::new(RwLock::new(MenuServer::new())),
            window_manager: Arc::new(RwLock::new(WindowManager::new())),
            desktop_manager: Arc::new(RwLock::new(DesktopManager::new())),
            dock: Arc::new(RwLock::new(Dock::new())),
            notification_center: Arc::new(RwLock::new(NotificationCenter::new())),
            workspace_manager: Arc::new(RwLock::new(WorkspaceManager::new())),
            launch_services: Arc::new(RwLock::new(LaunchServices::new())),
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
            theme_manager: Arc::new(RwLock::new(ThemeManager::new())),
            application_registry: Arc::new(RwLock::new(ApplicationRegistry::new())),
        }
    }

    pub fn theme_context(&self) -> ThemeContext {
        self.theme_manager.read().current_context()
    }

    pub fn startup() -> Result<Self> {
        let shell = Self::new();
        shell.launch_services.write().scan_applications();
        shell.theme_manager.write().load_default();
        Ok(shell)
    }

    pub fn run(&self) -> Result<()> {
        let mut app = retro_sdk::Application::new("RetroShell", "com.retro.shell");

        let desktop_view = ShellDesktop::new(
            self.menu_server.clone(),
            self.launch_services.clone(),
            self.window_manager.clone(),
        );

        let mut window = Window::new("RetroShell Desktop");
        window.set_content(Box::new(desktop_view));
        app.set_main_window(window);
        app.run();
        Ok(())
    }
}

struct ShellDesktop {
    state: WidgetState,
    menu_bar: MenuBar,
    desktop: IconView,
    windows: Vec<ShellWindow>,
    window_interaction: Option<WindowInteraction>,
    menu_server: Arc<RwLock<MenuServer>>,
    launch_services: Arc<RwLock<LaunchServices>>,
    window_manager: Arc<RwLock<WindowManager>>,
    bundle_ids: Vec<String>,
}

struct ShellWindow {
    id: Uuid,
    window: Window,
    restore_rect: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
enum WindowInteraction {
    Move {
        window_id: Uuid,
        pointer_offset: Point,
    },
    Resize {
        window_id: Uuid,
        start_point: Point,
        start_rect: Rect,
    },
}

impl ShellDesktop {
    fn new(
        menu_server: Arc<RwLock<MenuServer>>,
        launch_services: Arc<RwLock<LaunchServices>>,
        window_manager: Arc<RwLock<WindowManager>>,
    ) -> Self {
        let mut desktop = IconView::new();
        desktop.icon_size = 56.0;
        desktop.spacing = 18.0;
        desktop.items = vec![
            IconItem {
                label: "Hard Disk".to_string(),
                icon: Some("drive".to_string()),
                selected: false,
                rect: Rect::ZERO,
            },
            IconItem {
                label: "Home".to_string(),
                icon: Some("home".to_string()),
                selected: false,
                rect: Rect::ZERO,
            },
            IconItem {
                label: "Applications".to_string(),
                icon: Some("applications".to_string()),
                selected: false,
                rect: Rect::ZERO,
            },
            IconItem {
                label: "Trash".to_string(),
                icon: Some("trash".to_string()),
                selected: false,
                rect: Rect::ZERO,
            },
        ];

        let mut bundle_ids = Vec::new();
        let mut bundles = launch_services
            .read()
            .bundles
            .values()
            .cloned()
            .collect::<Vec<_>>();
        bundles.sort_by(|left, right| left.name.cmp(&right.name));
        for bundle in bundles.iter().take(6) {
            bundle_ids.push(bundle.bundle_id.clone());
            desktop.items.push(IconItem {
                label: bundle.name.clone(),
                icon: Some(bundle.bundle_id.clone()),
                selected: false,
                rect: Rect::ZERO,
            });
        }

        let menus = menu_server.read().menus.clone();
        let mut shell = Self {
            state: WidgetState::new(),
            menu_bar: MenuBar::new(menus),
            desktop,
            windows: Vec::new(),
            window_interaction: None,
            menu_server,
            launch_services,
            window_manager,
            bundle_ids,
        };
        shell.open_finder_window();
        shell
    }

    fn launch_item(&self, index: usize) {
        let item = match self.desktop.items.get(index) {
            Some(item) => item,
            None => return,
        };

        if let Some(bundle_id) = item.icon.as_deref() {
            if self.bundle_ids.iter().any(|id| id == bundle_id) {
                launch_app_binary(bundle_id);
                return;
            }
        }

        match item.label.as_str() {
            "Applications" => {
                if let Some(bundle) = self
                    .launch_services
                    .read()
                    .bundle_for_id("com.retro.finder")
                {
                    launch_app_binary(&bundle.bundle_id);
                }
            }
            "Home" => tracing::info!("Opening home folder"),
            "Hard Disk" => tracing::info!("Opening hard disk"),
            "Trash" => tracing::info!("Opening trash"),
            _ => {}
        }
    }

    fn content_bounds(&self) -> Rect {
        Rect::new(
            self.rect().x,
            self.rect().y + 24.0,
            self.rect().width,
            (self.rect().height - 24.0).max(0.0),
        )
    }

    fn next_finder_rect(&self) -> Rect {
        let base = if self.rect().width > 0.0 && self.rect().height > 0.0 {
            default_finder_rect(self.rect())
        } else {
            Rect::new(66.0, 66.0, 520.0, 320.0)
        };
        let offset = (self.windows.len() as f32 * 22.0) % 132.0;
        clamp_window_rect(
            Rect::new(base.x + offset, base.y + offset, base.width, base.height),
            self.content_bounds(),
        )
    }

    fn open_finder_window(&mut self) -> Uuid {
        let rect = self.next_finder_rect();
        let mut window = build_desktop_finder_window();
        window.set_rect(rect);
        let id =
            self.window_manager
                .write()
                .create_window("com.retro.finder", window.title(), rect);
        self.windows.push(ShellWindow {
            id,
            window,
            restore_rect: None,
        });
        self.focus_window(id);
        self.layout_window(id);
        id
    }

    fn close_active_window(&mut self) {
        let Some(id) = self.active_window_id() else {
            return;
        };
        self.close_window(id);
    }

    fn close_window(&mut self, id: Uuid) {
        self.windows.retain(|window| window.id != id);
        self.window_manager.write().close_window(id);
        if matches!(
            self.window_interaction,
            Some(WindowInteraction::Move { window_id, .. } | WindowInteraction::Resize { window_id, .. })
                if window_id == id
        ) {
            self.window_interaction = None;
        }
    }

    fn toggle_window_zoom(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };

        if let Some(restore_rect) = self.windows[index].restore_rect.take() {
            let restore_rect = clamp_window_rect(restore_rect, self.content_bounds());
            self.windows[index].window.set_rect(restore_rect);
            self.window_manager.write().restore_window(id);
        } else {
            let current = self.windows[index].window.rect();
            let zoom_rect = zoomed_window_rect(self.content_bounds(), self.windows.len());
            self.windows[index].restore_rect = Some(current);
            self.windows[index].window.set_rect(zoom_rect);
            self.window_manager.write().maximize_window(id);
        }

        self.layout_window(id);
    }

    fn focus_window(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        let shell_window = self.windows.remove(index);
        self.windows.push(shell_window);
        self.window_manager.write().focus_window(id);
    }

    fn active_window_id(&self) -> Option<Uuid> {
        self.windows.last().map(|window| window.id)
    }

    fn window_index(&self, id: Uuid) -> Option<usize> {
        self.windows.iter().position(|window| window.id == id)
    }

    fn top_window_index_at(&self, point: Point) -> Option<usize> {
        self.windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, window)| window.window.rect().contains(point))
            .map(|(index, _)| index)
    }

    fn layout_window(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        let rect = self.windows[index].window.rect();
        let _ = self.windows[index]
            .window
            .layout(LayoutConstraint::tight(Size::new(rect.width, rect.height)));
        self.window_manager.write().move_window(id, rect);
    }

    fn layout_windows(&mut self) {
        let bounds = self.content_bounds();
        for index in 0..self.windows.len() {
            let rect = self.windows[index].window.rect();
            let rect = if self.windows[index].restore_rect.is_some() {
                zoomed_window_rect(bounds, self.windows.len())
            } else if rect.width <= 1.0 || rect.height <= 1.0 {
                let base = default_finder_rect(self.rect());
                let offset = (index as f32 * 22.0) % 132.0;
                Rect::new(base.x + offset, base.y + offset, base.width, base.height)
            } else {
                rect
            };
            let rect = clamp_window_rect(rect, bounds);
            let id = self.windows[index].id;
            self.windows[index].window.set_rect(rect);
            self.layout_window(id);
        }
    }

    fn handle_menu_action(&mut self, action: &str) {
        match action {
            "shell.new_finder_window" => {
                self.open_finder_window();
            }
            "shell.close_finder_window" => self.close_active_window(),
            "shell.open_finder" => launch_app_binary("com.retro.finder"),
            "shell.settings" => launch_app_binary("com.retro.settings"),
            "shell.software_catalog" => {
                tracing::info!(
                    "Software Catalog selected; package manager app is not implemented yet"
                );
            }
            "shell.about" => tracing::info!("About RetroShell selected"),
            _ => tracing::info!("Unhandled menu action: {action}"),
        }
    }

    fn move_window_to(&mut self, id: Uuid, point: Point, pointer_offset: Point) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        self.windows[index].restore_rect = None;
        self.window_manager.write().restore_window(id);
        let current = self.windows[index].window.rect();
        let moved = Rect::new(
            point.x - pointer_offset.x,
            point.y - pointer_offset.y,
            current.width,
            current.height,
        );
        let moved = clamp_window_rect(moved, self.content_bounds());
        self.windows[index].window.set_rect(moved);
        self.layout_window(id);
    }

    fn resize_window_to(&mut self, id: Uuid, point: Point, start_point: Point, start_rect: Rect) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        self.windows[index].restore_rect = None;
        self.window_manager.write().restore_window(id);
        let resized = Rect::new(
            start_rect.x,
            start_rect.y,
            (start_rect.width + point.x - start_point.x).max(320.0),
            (start_rect.height + point.y - start_point.y).max(220.0),
        );
        let resized = clamp_window_rect(resized, self.content_bounds());
        self.windows[index].window.set_rect(resized);
        self.layout_window(id);
    }
}

fn default_finder_rect(shell_rect: Rect) -> Rect {
    let window_width = (shell_rect.width * 0.52).clamp(360.0, 560.0);
    let window_height = (shell_rect.height * 0.46).clamp(260.0, 380.0);
    Rect::new(
        shell_rect.x + 66.0,
        shell_rect.y + 66.0,
        window_width.min((shell_rect.width - 160.0).max(260.0)),
        window_height.min((shell_rect.height - 120.0).max(220.0)),
    )
}

fn titlebar_rect(window_rect: Rect) -> Rect {
    Rect::new(window_rect.x, window_rect.y, window_rect.width, 24.0)
}

fn close_box_rect(window_rect: Rect) -> Rect {
    Rect::new(window_rect.x + 8.0, window_rect.y + 7.0, 11.0, 11.0)
}

fn zoom_box_rect(window_rect: Rect) -> Rect {
    Rect::new(
        window_rect.x + window_rect.width - 19.0,
        window_rect.y + 7.0,
        11.0,
        11.0,
    )
}

fn resize_handle_rect(window_rect: Rect) -> Rect {
    Rect::new(
        window_rect.x + window_rect.width - 18.0,
        window_rect.y + window_rect.height - 18.0,
        18.0,
        18.0,
    )
}

fn zoomed_window_rect(bounds: Rect, window_count: usize) -> Rect {
    let margin = if window_count > 1 { 10.0 } else { 0.0 };
    Rect::new(
        bounds.x + margin,
        bounds.y + margin,
        (bounds.width - margin * 2.0).max(320.0),
        (bounds.height - margin * 2.0).max(220.0),
    )
}

fn clamp_window_rect(rect: Rect, bounds: Rect) -> Rect {
    let min_width = rect.width.min(bounds.width.max(1.0));
    let min_height = rect.height.min(bounds.height.max(1.0));
    let width = min_width.max(1.0);
    let height = min_height.max(1.0);
    let max_x = bounds.x + (bounds.width - width).max(0.0);
    let max_y = bounds.y + (bounds.height - height).max(0.0);

    Rect::new(
        rect.x.clamp(bounds.x, max_x),
        rect.y.clamp(bounds.y, max_y),
        width,
        height,
    )
}

fn build_desktop_finder_window() -> Window {
    let mut files = IconView::new();
    files.icon_size = 76.0;
    files.spacing = 10.0;
    files.items = vec![
        IconItem {
            label: "System".to_string(),
            icon: Some("folder".to_string()),
            selected: true,
            rect: Rect::ZERO,
        },
        IconItem {
            label: "Applications".to_string(),
            icon: Some("folder".to_string()),
            selected: false,
            rect: Rect::ZERO,
        },
        IconItem {
            label: "Documents".to_string(),
            icon: Some("folder".to_string()),
            selected: false,
            rect: Rect::ZERO,
        },
        IconItem {
            label: "Terminal".to_string(),
            icon: Some("com.retro.terminal".to_string()),
            selected: false,
            rect: Rect::ZERO,
        },
        IconItem {
            label: "TextEdit".to_string(),
            icon: Some("com.retro.textedit".to_string()),
            selected: false,
            rect: Rect::ZERO,
        },
        IconItem {
            label: "Read Me".to_string(),
            icon: Some("document".to_string()),
            selected: false,
            rect: Rect::ZERO,
        },
    ];

    let mut window = Window::new("Retro HD");
    window.set_content(Box::new(files));
    window
}

fn launch_app_binary(bundle_id: &str) {
    let binary = match bundle_id {
        "com.retro.finder" => "finder",
        "com.retro.settings" => "settings",
        "com.retro.textedit" => "textedit",
        "com.retro.terminal" => "terminal",
        _ => return,
    };

    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|dir| dir.join(binary))),
        Some(PathBuf::from(format!("target/debug/{binary}"))),
        Some(PathBuf::from(format!("target/release/{binary}"))),
        Some(PathBuf::from(binary)),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            match Command::new(&candidate).spawn() {
                Ok(_) => tracing::info!("Launched {}", candidate.display()),
                Err(err) => tracing::error!("Failed to launch {}: {err}", candidate.display()),
            }
            return;
        }
    }

    tracing::warn!("Could not find executable for {bundle_id}");
}

impl Widget for ShellDesktop {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));

        self.menu_bar
            .set_rect(Rect::new(self.rect().x, self.rect().y, size.width, 24.0));
        let _ = self
            .menu_bar
            .layout(LayoutConstraint::tight(Size::new(size.width, 24.0)));

        self.desktop.set_rect(Rect::new(
            self.rect().x,
            self.rect().y + 24.0,
            size.width,
            (size.height - 24.0).max(0.0),
        ));
        let _ = self.desktop.layout(LayoutConstraint::tight(Size::new(
            size.width,
            (size.height - 24.0).max(0.0),
        )));

        self.layout_windows();

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.desktop.draw(theme);
        for shell_window in &self.windows {
            shell_window.window.draw(theme);
        }
        self.menu_bar.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        let result = self.menu_bar.handle_event(event);
        if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
            return result;
        }

        match event {
            Event::MouseDown {
                button: MouseButton::Left,
                point,
                ..
            } => {
                let Some(index) = self.top_window_index_at(*point) else {
                    return self.desktop.handle_event(event);
                };
                let window_id = self.windows[index].id;
                self.focus_window(window_id);
                let Some(index) = self.window_index(window_id) else {
                    return EventResult::Ignored;
                };
                let window_rect = self.windows[index].window.rect();
                if close_box_rect(window_rect).contains(*point) {
                    self.close_window(window_id);
                    return EventResult::Handled;
                }

                if zoom_box_rect(window_rect).contains(*point) {
                    self.toggle_window_zoom(window_id);
                    return EventResult::Handled;
                }

                if resize_handle_rect(window_rect).contains(*point) {
                    self.window_interaction = Some(WindowInteraction::Resize {
                        window_id,
                        start_point: *point,
                        start_rect: window_rect,
                    });
                    return EventResult::Handled;
                }

                if titlebar_rect(window_rect).contains(*point) {
                    self.window_interaction = Some(WindowInteraction::Move {
                        window_id,
                        pointer_offset: Point::new(
                            point.x - window_rect.x,
                            point.y - window_rect.y,
                        ),
                    });
                    return EventResult::Handled;
                }

                let result = self.windows[index].window.handle_event(event);
                if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
                    return result;
                }
            }
            Event::MouseMove { point, .. } => {
                if let Some(interaction) = self.window_interaction {
                    match interaction {
                        WindowInteraction::Move {
                            window_id,
                            pointer_offset,
                        } => {
                            self.move_window_to(window_id, *point, pointer_offset);
                        }
                        WindowInteraction::Resize {
                            window_id,
                            start_point,
                            start_rect,
                        } => self.resize_window_to(window_id, *point, start_point, start_rect),
                    }
                    return EventResult::Handled;
                }

                if let Some(index) = self.top_window_index_at(*point) {
                    let result = self.windows[index].window.handle_event(event);
                    if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
                        return result;
                    }
                }
            }
            Event::MouseUp {
                button: MouseButton::Left,
                ..
            } => {
                if self.window_interaction.take().is_some() {
                    return EventResult::Handled;
                }
            }
            Event::DoubleClick { point, .. } => {
                if let Some(index) = self.top_window_index_at(*point) {
                    let result = self.windows[index].window.handle_event(event);
                    if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
                        return result;
                    }
                }
            }
            _ => {}
        }

        if let Event::DoubleClick {
            button: MouseButton::Left,
            point,
            ..
        } = event
        {
            for (index, item) in self.desktop.items.iter().enumerate() {
                if item.rect.contains(*point) {
                    self.launch_item(index);
                    return EventResult::Handled;
                }
            }
        }

        self.desktop.handle_event(event)
    }

    fn update(&mut self) {
        self.menu_bar.menus = self.menu_server.read().menus.clone();

        if let Some(action) = self.menu_bar.last_action.take() {
            tracing::info!("Menu action: {action}");
            self.handle_menu_action(&action);
        }
    }

    fn children(&self) -> Vec<&dyn Widget> {
        let mut children: Vec<&dyn Widget> = Vec::with_capacity(self.windows.len() + 2);
        children.push(&self.desktop);
        for shell_window in &self.windows {
            children.push(&shell_window.window);
        }
        children.push(&self.menu_bar);
        children
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        let capacity = self.windows.len() + 2;
        let mut children: Vec<&mut dyn Widget> = Vec::with_capacity(capacity);
        children.push(&mut self.desktop);
        for shell_window in &mut self.windows {
            children.push(&mut shell_window.window);
        }
        children.push(&mut self.menu_bar);
        children
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_desktop() -> (ShellDesktop, Arc<RwLock<WindowManager>>) {
        let menu_server = Arc::new(RwLock::new(MenuServer::new()));
        let launch_services = Arc::new(RwLock::new(LaunchServices::new()));
        let window_manager = Arc::new(RwLock::new(WindowManager::new()));
        let mut desktop = ShellDesktop::new(menu_server, launch_services, window_manager.clone());
        desktop.layout(LayoutConstraint::tight(Size::new(960.0, 640.0)));
        (desktop, window_manager)
    }

    fn assert_rect_eq(actual: Rect, expected: Rect) {
        assert_eq!(actual.x, expected.x);
        assert_eq!(actual.y, expected.y);
        assert_eq!(actual.width, expected.width);
        assert_eq!(actual.height, expected.height);
    }

    #[test]
    fn default_finder_rect_stays_inside_shell() {
        let shell = Rect::new(0.0, 0.0, 960.0, 640.0);
        let rect = default_finder_rect(shell);

        assert!(rect.x >= shell.x);
        assert!(rect.y >= shell.y);
        assert!(rect.x + rect.width <= shell.x + shell.width);
        assert!(rect.y + rect.height <= shell.y + shell.height);
        assert!(rect.width >= 360.0);
        assert!(rect.height >= 260.0);
    }

    #[test]
    fn clamp_window_rect_keeps_window_visible() {
        let bounds = Rect::new(0.0, 24.0, 960.0, 616.0);
        let rect = Rect::new(-200.0, 900.0, 420.0, 280.0);
        let clamped = clamp_window_rect(rect, bounds);

        assert_eq!(clamped.x, bounds.x);
        assert_eq!(clamped.y, bounds.y + bounds.height - clamped.height);
        assert_eq!(clamped.width, rect.width);
        assert_eq!(clamped.height, rect.height);
    }

    #[test]
    fn resize_handle_tracks_bottom_right_corner() {
        let window = Rect::new(66.0, 66.0, 500.0, 300.0);
        let handle = resize_handle_rect(window);

        assert!(handle.contains(Point::new(565.0, 365.0)));
        assert!(!handle.contains(Point::new(540.0, 340.0)));
    }

    #[test]
    fn classic_titlebar_controls_match_drawn_chrome() {
        let window = Rect::new(66.0, 66.0, 500.0, 300.0);

        assert!(close_box_rect(window).contains(Point::new(78.0, 78.0)));
        assert!(zoom_box_rect(window).contains(Point::new(554.0, 78.0)));
        assert!(!titlebar_rect(window).contains(Point::new(554.0, 96.0)));
    }

    #[test]
    fn shell_menu_actions_create_and_close_managed_windows() {
        let (mut desktop, window_manager) = test_desktop();

        assert_eq!(desktop.windows.len(), 1);
        let first_id = desktop.windows[0].id;
        assert_eq!(window_manager.read().active_window, Some(first_id));

        desktop.handle_menu_action("shell.new_finder_window");
        assert_eq!(desktop.windows.len(), 2);
        let second_id = desktop.windows[1].id;
        assert_ne!(first_id, second_id);
        assert_eq!(window_manager.read().active_window, Some(second_id));

        desktop.handle_menu_action("shell.close_finder_window");
        assert_eq!(desktop.windows.len(), 1);
        assert_eq!(desktop.windows[0].id, first_id);
        assert_eq!(window_manager.read().active_window, Some(first_id));
    }

    #[test]
    fn focusing_window_raises_it_to_front() {
        let (mut desktop, window_manager) = test_desktop();
        let first_id = desktop.windows[0].id;
        let second_id = desktop.open_finder_window();

        desktop.focus_window(first_id);

        assert_eq!(desktop.active_window_id(), Some(first_id));
        assert_eq!(
            desktop.windows.last().map(|window| window.id),
            Some(first_id)
        );
        assert_eq!(window_manager.read().active_window, Some(first_id));
        assert_ne!(
            desktop.windows.last().map(|window| window.id),
            Some(second_id)
        );
    }

    #[test]
    fn close_box_closes_the_clicked_window() {
        let (mut desktop, window_manager) = test_desktop();
        let first_id = desktop.windows[0].id;
        let point = Point::new(78.0, 78.0);

        let result = desktop.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point,
            modifiers: retro_kit::event::Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert!(desktop.windows.is_empty());
        assert!(!window_manager.read().windows.contains_key(&first_id));
    }

    #[test]
    fn zoom_box_toggles_managed_window_between_zoomed_and_restored() {
        let (mut desktop, window_manager) = test_desktop();
        let id = desktop.windows[0].id;
        let original = desktop.windows[0].window.rect();
        let point = Point::new(original.x + original.width - 14.0, original.y + 12.0);

        let result = desktop.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point,
            modifiers: retro_kit::event::Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert!(desktop.windows[0].restore_rect.is_some());
        assert_rect_eq(desktop.windows[0].restore_rect.unwrap(), original);
        assert_eq!(
            window_manager.read().windows[&id].state,
            window_manager::WindowState::Maximized
        );
        assert!(desktop.windows[0].window.rect().width > original.width);

        let zoomed = desktop.windows[0].window.rect();
        let restore_point = Point::new(zoomed.x + zoomed.width - 14.0, zoomed.y + 12.0);
        let result = desktop.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: restore_point,
            modifiers: retro_kit::event::Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert!(desktop.windows[0].restore_rect.is_none());
        assert_eq!(
            window_manager.read().windows[&id].state,
            window_manager::WindowState::Normal
        );
        assert_rect_eq(desktop.windows[0].window.rect(), original);
    }

    #[test]
    fn default_shell_menus_have_routable_action_ids() {
        let server = MenuServer::new();
        let file = server
            .menus
            .iter()
            .find(|menu| menu.title == "File")
            .expect("file menu exists");

        assert_eq!(file.items[0].action_id, "shell.new_finder_window");
        assert_eq!(file.items[1].action_id, "shell.open_finder");
        assert_eq!(file.items[2].action_id, "shell.close_finder_window");
    }
}
