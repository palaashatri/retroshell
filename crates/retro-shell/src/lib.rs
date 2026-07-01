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

        let desktop_view =
            ShellDesktop::new(self.menu_server.clone(), self.launch_services.clone());

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
    finder_window: Window,
    finder_visible: bool,
    finder_placed: bool,
    window_interaction: Option<WindowInteraction>,
    menu_server: Arc<RwLock<MenuServer>>,
    launch_services: Arc<RwLock<LaunchServices>>,
    bundle_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum WindowInteraction {
    Move {
        pointer_offset: Point,
    },
    Resize {
        start_point: Point,
        start_rect: Rect,
    },
}

impl ShellDesktop {
    fn new(
        menu_server: Arc<RwLock<MenuServer>>,
        launch_services: Arc<RwLock<LaunchServices>>,
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
        Self {
            state: WidgetState::new(),
            menu_bar: MenuBar::new(menus),
            desktop,
            finder_window: build_desktop_finder_window(),
            finder_visible: true,
            finder_placed: false,
            window_interaction: None,
            menu_server,
            launch_services,
            bundle_ids,
        }
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

    fn layout_finder_window(&mut self) {
        let rect = self.finder_window.rect();
        let _ = self
            .finder_window
            .layout(LayoutConstraint::tight(Size::new(rect.width, rect.height)));
    }

    fn content_bounds(&self) -> Rect {
        Rect::new(
            self.rect().x,
            self.rect().y + 24.0,
            self.rect().width,
            (self.rect().height - 24.0).max(0.0),
        )
    }

    fn place_finder_window_if_needed(&mut self) {
        if self.finder_placed {
            return;
        }

        let rect = default_finder_rect(self.rect());
        self.finder_window.set_rect(rect);
        self.finder_placed = true;
    }

    fn show_finder_window(&mut self) {
        self.finder_visible = true;
        self.finder_placed = false;
        self.place_finder_window_if_needed();
        self.layout_finder_window();
    }

    fn handle_menu_action(&mut self, action: &str) {
        match action {
            "shell.new_finder_window" => self.show_finder_window(),
            "shell.close_finder_window" => {
                self.finder_visible = false;
                self.window_interaction = None;
            }
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

    fn move_finder_window_to(&mut self, point: Point, pointer_offset: Point) {
        let current = self.finder_window.rect();
        let moved = Rect::new(
            point.x - pointer_offset.x,
            point.y - pointer_offset.y,
            current.width,
            current.height,
        );
        self.finder_window
            .set_rect(clamp_window_rect(moved, self.content_bounds()));
        self.layout_finder_window();
    }

    fn resize_finder_window_to(&mut self, point: Point, start_point: Point, start_rect: Rect) {
        let resized = Rect::new(
            start_rect.x,
            start_rect.y,
            (start_rect.width + point.x - start_point.x).max(320.0),
            (start_rect.height + point.y - start_point.y).max(220.0),
        );
        self.finder_window
            .set_rect(clamp_window_rect(resized, self.content_bounds()));
        self.layout_finder_window();
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

fn resize_handle_rect(window_rect: Rect) -> Rect {
    Rect::new(
        window_rect.x + window_rect.width - 18.0,
        window_rect.y + window_rect.height - 18.0,
        18.0,
        18.0,
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

        if self.finder_visible {
            self.place_finder_window_if_needed();
            self.finder_window.set_rect(clamp_window_rect(
                self.finder_window.rect(),
                self.content_bounds(),
            ));
            self.layout_finder_window();
        }

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.desktop.draw(theme);
        if self.finder_visible {
            self.finder_window.draw(theme);
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
            } if self.finder_visible && self.finder_window.rect().contains(*point) => {
                let window_rect = self.finder_window.rect();
                if resize_handle_rect(window_rect).contains(*point) {
                    self.window_interaction = Some(WindowInteraction::Resize {
                        start_point: *point,
                        start_rect: window_rect,
                    });
                    return EventResult::Handled;
                }

                if titlebar_rect(window_rect).contains(*point) {
                    self.window_interaction = Some(WindowInteraction::Move {
                        pointer_offset: Point::new(
                            point.x - window_rect.x,
                            point.y - window_rect.y,
                        ),
                    });
                    return EventResult::Handled;
                }

                let result = self.finder_window.handle_event(event);
                if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
                    return result;
                }
            }
            Event::MouseMove { point, .. } => {
                if let Some(interaction) = self.window_interaction {
                    match interaction {
                        WindowInteraction::Move { pointer_offset } => {
                            self.move_finder_window_to(*point, pointer_offset);
                        }
                        WindowInteraction::Resize {
                            start_point,
                            start_rect,
                        } => self.resize_finder_window_to(*point, start_point, start_rect),
                    }
                    return EventResult::Handled;
                }

                if self.finder_visible && self.finder_window.rect().contains(*point) {
                    let result = self.finder_window.handle_event(event);
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
            Event::DoubleClick { point, .. }
                if self.finder_visible && self.finder_window.rect().contains(*point) =>
            {
                let result = self.finder_window.handle_event(event);
                if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
                    return result;
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
        vec![&self.desktop, &self.finder_window, &self.menu_bar]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![
            &mut self.desktop,
            &mut self.finder_window,
            &mut self.menu_bar,
        ]
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
    fn shell_menu_actions_hide_and_restore_finder_window() {
        let menu_server = Arc::new(RwLock::new(MenuServer::new()));
        let launch_services = Arc::new(RwLock::new(LaunchServices::new()));
        let mut desktop = ShellDesktop::new(menu_server, launch_services);
        desktop.layout(LayoutConstraint::tight(Size::new(960.0, 640.0)));

        assert!(desktop.finder_visible);
        let original_rect = desktop.finder_window.rect();

        desktop.handle_menu_action("shell.close_finder_window");
        assert!(!desktop.finder_visible);

        desktop.handle_menu_action("shell.new_finder_window");
        assert!(desktop.finder_visible);
        assert_eq!(desktop.finder_window.rect().x, original_rect.x);
        assert_eq!(desktop.finder_window.rect().y, original_rect.y);
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
