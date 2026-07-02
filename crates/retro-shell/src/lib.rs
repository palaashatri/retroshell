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
use std::fs;
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
        app.set_initial_size(Size::new(1280.0, 800.0));

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
    folder_path: Option<PathBuf>,
    restore_rect: Option<Rect>,
    mode: ShellWindowMode,
}

struct FolderOpenTarget {
    title: String,
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellWindowMode {
    Normal,
    Zoomed,
    Fullscreen,
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

    fn launch_item(&mut self, index: usize) {
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
            "Home" => {
                self.open_folder_window("Home", home_dir());
            }
            "Hard Disk" => {
                self.open_folder_window("Hard Disk", PathBuf::from("/"));
            }
            "Trash" => {
                self.open_folder_window("Trash", trash_dir());
            }
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
        self.open_folder_window("Retro HD", PathBuf::from("/"))
    }

    fn open_folder_window<S: Into<String>>(&mut self, title: S, path: PathBuf) -> Uuid {
        let rect = self.next_finder_rect();
        let title = title.into();
        let mut window = build_folder_window(&title, &path);
        window.set_rect(rect);
        let id =
            self.window_manager
                .write()
                .create_window("com.retro.finder", window.title(), rect);
        self.windows.push(ShellWindow {
            id,
            window,
            folder_path: Some(path),
            restore_rect: None,
            mode: ShellWindowMode::Normal,
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
        if let Some(active) = self.windows.last_mut() {
            active.window.is_active = true;
        }
        if matches!(
            self.window_interaction,
            Some(WindowInteraction::Move { window_id, .. } | WindowInteraction::Resize { window_id, .. })
            if window_id == id
        ) {
            self.window_interaction = None;
        }
        self.sync_global_menu_to_active_window();
    }

    fn toggle_window_zoom(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };

        if self.windows[index].mode == ShellWindowMode::Zoomed {
            let Some(restore_rect) = self.windows[index].restore_rect.take() else {
                return;
            };
            let restore_rect = clamp_window_rect(restore_rect, self.content_bounds());
            self.windows[index].window.set_rect(restore_rect);
            self.windows[index].mode = ShellWindowMode::Normal;
            self.window_manager.write().restore_window(id);
        } else {
            let current = self.windows[index].window.rect();
            let zoom_rect = zoomed_window_rect(self.content_bounds(), self.windows.len());
            self.windows[index].restore_rect = Some(current);
            self.windows[index].mode = ShellWindowMode::Zoomed;
            self.windows[index].window.set_rect(zoom_rect);
            self.window_manager.write().maximize_window(id);
        }

        self.layout_window(id);
    }

    fn toggle_window_fullscreen(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };

        if self.windows[index].mode == ShellWindowMode::Fullscreen {
            let Some(restore_rect) = self.windows[index].restore_rect.take() else {
                return;
            };
            let restore_rect = clamp_window_rect(restore_rect, self.content_bounds());
            self.windows[index].window.set_rect(restore_rect);
            self.windows[index].mode = ShellWindowMode::Normal;
            self.window_manager.write().restore_window(id);
        } else {
            let current = self.windows[index].window.rect();
            let fullscreen_rect = fullscreen_window_rect(self.content_bounds());
            self.windows[index].restore_rect = Some(current);
            self.windows[index].mode = ShellWindowMode::Fullscreen;
            self.windows[index].window.set_rect(fullscreen_rect);
            self.window_manager.write().set_fullscreen(id);
        }

        self.window_interaction = None;
        self.focus_window(id);
        self.layout_window(id);
    }

    fn focus_window(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        let mut shell_window = self.windows.remove(index);
        shell_window.window.is_active = true;
        for w in &mut self.windows {
            w.window.is_active = false;
        }
        self.windows.push(shell_window);
        self.window_manager.write().focus_window(id);
        self.sync_global_menu_to_active_window();
    }

    fn sync_global_menu_to_active_window(&mut self) {
        let active_app = self.window_manager.read().active_window.and_then(|id| {
            self.window_manager
                .read()
                .windows
                .get(&id)
                .map(|window| window.app_id.clone())
        });

        if let Some(app_id) = active_app {
            self.menu_server.write().set_active_app_menus(&app_id);
        } else {
            self.menu_server.write().reset_to_shell_menus();
        }
        self.menu_bar.menus = self.menu_server.read().menus.clone();
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

    fn folder_open_target_at(&self, window_index: usize, point: Point) -> Option<FolderOpenTarget> {
        let shell_window = self.windows.get(window_index)?;
        let folder_path = shell_window.folder_path.as_ref()?;
        let icon_view = shell_window
            .window
            .content
            .as_ref()
            .and_then(|content| content.as_any().downcast_ref::<IconView>())?;
        let item = icon_view
            .items
            .iter()
            .find(|item| item.rect.contains(point))?;
        if item.icon.as_deref() != Some("folder") {
            return None;
        }

        let path = folder_path.join(&item.label);
        if !path.is_dir() {
            return None;
        }

        Some(FolderOpenTarget {
            title: item.label.clone(),
            path,
        })
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
            let rect = if self.windows[index].mode == ShellWindowMode::Fullscreen {
                fullscreen_window_rect(bounds)
            } else if self.windows[index].mode == ShellWindowMode::Zoomed {
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
            "shell.zoom_window" => {
                if let Some(id) = self.active_window_id() {
                    self.toggle_window_zoom(id);
                }
            }
            "shell.toggle_fullscreen" => {
                if let Some(id) = self.active_window_id() {
                    self.toggle_window_fullscreen(id);
                }
            }
            "shell.open_home" => {
                self.open_folder_window("Home", home_dir());
            }
            "shell.open_computer" => {
                self.open_folder_window("Hard Disk", PathBuf::from("/"));
            }
            "shell.open_finder" => launch_app_binary("com.retro.finder"),
            "shell.settings" => launch_app_binary("com.retro.settings"),
            "shell.software_catalog" => launch_app_binary("com.retro.appstore"),
            "shell.about" => {
                self.open_folder_window("About RetroShell", PathBuf::from("/"));
            }
            "shell.quit" => {
                std::process::exit(0);
            }
            "finder.new_folder" => self.handle_new_folder(),
            "finder.get_info" => self.handle_get_info(),
            _ => tracing::info!("Unhandled menu action: {action}"),
        }
    }

    fn handle_new_folder(&mut self) {
        let Some(id) = self.active_window_id() else {
            return;
        };
        let Some(index) = self.window_index(id) else {
            return;
        };
        let Some(folder_path) = self.windows[index].folder_path.clone() else {
            return;
        };
        let mut name = "untitled folder".to_string();
        let mut counter = 1;
        while folder_path.join(&name).exists() {
            name = format!("untitled folder {counter}");
            counter += 1;
        }
        if let Err(err) = fs::create_dir_all(folder_path.join(&name)) {
            tracing::error!("Failed to create folder: {err}");
            return;
        }
        self.refresh_active_folder_window();
    }

    fn handle_get_info(&mut self) {
        let Some(id) = self.active_window_id() else {
            return;
        };
        let Some(index) = self.window_index(id) else {
            return;
        };
        let info = if let Some(ref path) = self.windows[index].folder_path {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            format!("{} — Info", name)
        } else {
            self.windows[index].window.title().to_string()
        };
        let info_title = format!("{info} Info");
        self.open_folder_window(&info_title, PathBuf::from("/"));
    }

    fn refresh_active_folder_window(&mut self) {
        let Some(id) = self.active_window_id() else {
            return;
        };
        let Some(index) = self.window_index(id) else {
            return;
        };
        let Some(ref path) = self.windows[index].folder_path.clone() else {
            return;
        };
        let mut files = retro_kit::icon_view::IconView::new();
        files.icon_size = 76.0;
        files.spacing = 10.0;
        files.items = folder_items_for_path(path);
        self.windows[index].window.set_content(Box::new(files));
        self.layout_window(id);
    }

    fn move_window_to(&mut self, id: Uuid, point: Point, pointer_offset: Point) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        self.windows[index].restore_rect = None;
        self.windows[index].mode = ShellWindowMode::Normal;
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
        self.windows[index].mode = ShellWindowMode::Normal;
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

fn minimize_box_rect(window_rect: Rect) -> Rect {
    Rect::new(window_rect.x + 22.0, window_rect.y + 7.0, 11.0, 11.0)
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

fn fullscreen_window_rect(bounds: Rect) -> Rect {
    Rect::new(
        bounds.x,
        bounds.y,
        bounds.width.max(320.0),
        bounds.height.max(220.0),
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

fn build_folder_window(title: &str, path: &PathBuf) -> Window {
    let mut files = IconView::new();
    files.icon_size = 76.0;
    files.spacing = 10.0;
    files.items = folder_items_for_path(path);

    let mut window = Window::new(title);
    window.set_content(Box::new(files));
    window
}

fn folder_items_for_path(path: &PathBuf) -> Vec<IconItem> {
    let Ok(entries) = fs::read_dir(path) else {
        return vec![IconItem {
            label: format!("⚠ Unable to read: {}", path.display()),
            icon: Some("document".to_string()),
            selected: false,
            rect: Rect::ZERO,
        }];
    };

    let mut entries = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                return None;
            }
            let is_dir = entry.file_type().ok().is_some_and(|kind| kind.is_dir());
            Some((name, is_dir))
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.to_lowercase().cmp(&right.0.to_lowercase()))
    });

    let mut items: Vec<IconItem> = entries
        .into_iter()
        .map(|(label, is_dir)| IconItem {
            label,
            icon: Some(if is_dir { "folder" } else { "document" }.to_string()),
            selected: false,
            rect: Rect::ZERO,
        })
        .collect();

    if items.is_empty() {
        items.push(IconItem {
            label: "This folder is empty".to_string(),
            icon: Some("document".to_string()),
            selected: false,
            rect: Rect::ZERO,
        });
    }

    items
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn trash_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share"))
        .join("Trash/files")
}

fn launch_app_binary(bundle_id: &str) {
    let binary = match bundle_id {
        "com.retro.finder" => "finder",
        "com.retro.settings" => "settings",
        "com.retro.textedit" => "textedit",
        "com.retro.terminal" => "terminal",
        "com.retro.appstore" => "appstore",
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
        // Draw non-active windows first
        for shell_window in self.windows.iter().rev().skip(1) {
            shell_window.window.draw(theme);
        }
        // Draw active window last (on top)
        if let Some(active) = self.windows.last() {
            active.window.draw(theme);
        }
        self.menu_bar.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        let result = self.menu_bar.handle_event(event);
        if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
            return result;
        }

        if let Event::KeyDown { key, modifiers } = event {
            let action = self
                .menu_server
                .read()
                .action_for_shortcut(*key, *modifiers);
            if let Some(action) = action {
                self.handle_menu_action(&action);
                return EventResult::Handled;
            }
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

                if minimize_box_rect(window_rect).contains(*point) {
                    self.window_manager.write().minimize_window(window_id);
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
                    let window_id = self.windows[index].id;
                    self.focus_window(window_id);
                    let Some(index) = self.window_index(window_id) else {
                        return EventResult::Ignored;
                    };

                    if let Some(target) = self.folder_open_target_at(index, *point) {
                        self.open_folder_window(target.title, target.path);
                        return EventResult::Handled;
                    }

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
    use retro_kit::event::Modifiers;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_shell_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("retroshell_shell_folder_{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

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

    fn icon_item_center(window: &ShellWindow, label: &str) -> Point {
        let icon_view = window
            .window
            .content
            .as_ref()
            .and_then(|content| content.as_any().downcast_ref::<IconView>())
            .expect("shell folder window has icon content");
        let rect = icon_view
            .items
            .iter()
            .find(|item| item.label == label)
            .expect("folder item exists")
            .rect;
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
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
    fn folder_items_sort_directories_first_and_hide_dotfiles() {
        let root = temp_shell_root();
        fs::create_dir_all(root.join("Folder")).unwrap();
        fs::write(root.join("note.txt"), "hello").unwrap();
        fs::write(root.join(".hidden"), "secret").unwrap();

        let items = folder_items_for_path(&root);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "Folder");
        assert_eq!(items[0].icon.as_deref(), Some("folder"));
        assert_eq!(items[1].label, "note.txt");
        assert_eq!(items[1].icon.as_deref(), Some("document"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn desktop_home_icon_opens_managed_folder_window() {
        let (mut desktop, window_manager) = test_desktop();
        let initial_count = desktop.windows.len();
        let home_index = desktop
            .desktop
            .items
            .iter()
            .position(|item| item.label == "Home")
            .expect("home desktop icon exists");

        desktop.launch_item(home_index);

        assert_eq!(desktop.windows.len(), initial_count + 1);
        let active = desktop.windows.last().expect("active home window");
        assert_eq!(active.window.title(), "Home");
        assert_eq!(window_manager.read().active_window, Some(active.id));
    }

    #[test]
    fn shell_global_menu_switches_to_focused_finder_window() {
        let (mut desktop, _) = test_desktop();

        let titles = desktop
            .menu_bar
            .menus
            .iter()
            .map(|menu| menu.title.as_str())
            .collect::<Vec<_>>();

        assert!(titles.contains(&"Retro"));
        assert!(titles.contains(&"Finder"));
        assert_eq!(
            desktop.menu_server.read().active_app.as_deref(),
            Some("com.retro.finder")
        );

        let second_id = desktop.open_finder_window();
        desktop.focus_window(second_id);
        let titles = desktop
            .menu_bar
            .menus
            .iter()
            .map(|menu| menu.title.as_str())
            .collect::<Vec<_>>();
        assert!(titles.contains(&"Finder"));
        assert!(titles.contains(&"Go"));
    }

    #[test]
    fn shell_global_menu_resets_when_last_window_closes() {
        let (mut desktop, _) = test_desktop();
        let ids = desktop
            .windows
            .iter()
            .map(|window| window.id)
            .collect::<Vec<_>>();

        for id in ids {
            desktop.close_window(id);
        }

        assert!(desktop.windows.is_empty());
        assert_eq!(desktop.menu_server.read().active_app, None);
        assert!(!desktop
            .menu_bar
            .menus
            .iter()
            .any(|menu| menu.title == "Finder"));
    }

    #[test]
    fn shell_folder_window_double_click_opens_child_folder() {
        let root = temp_shell_root();
        fs::create_dir_all(root.join("Projects")).unwrap();
        fs::write(root.join("note.txt"), "hello").unwrap();
        let (mut desktop, window_manager) = test_desktop();
        let initial_count = desktop.windows.len();
        let root_id = desktop.open_folder_window("Root", root.clone());
        let index = desktop.window_index(root_id).unwrap();
        let point = icon_item_center(&desktop.windows[index], "Projects");

        let result = desktop.handle_event(&Event::DoubleClick {
            button: MouseButton::Left,
            point,
            modifiers: Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(desktop.windows.len(), initial_count + 2);
        let active = desktop.windows.last().expect("child folder window");
        assert_eq!(active.window.title(), "Projects");
        assert_eq!(
            active.folder_path.as_deref(),
            Some(root.join("Projects").as_path())
        );
        assert_eq!(window_manager.read().active_window, Some(active.id));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shell_folder_window_double_click_file_does_not_open_window() {
        let root = temp_shell_root();
        fs::write(root.join("note.txt"), "hello").unwrap();
        let (mut desktop, _) = test_desktop();
        let root_id = desktop.open_folder_window("Root", root.clone());
        let index = desktop.window_index(root_id).unwrap();
        let point = icon_item_center(&desktop.windows[index], "note.txt");
        let initial_count = desktop.windows.len();

        let result = desktop.handle_event(&Event::DoubleClick {
            button: MouseButton::Left,
            point,
            modifiers: Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(desktop.windows.len(), initial_count);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn classic_titlebar_controls_match_drawn_chrome() {
        let window = Rect::new(66.0, 66.0, 500.0, 300.0);

        assert!(close_box_rect(window).contains(Point::new(78.0, 78.0)));
        assert!(minimize_box_rect(window).contains(Point::new(92.0, 78.0)));
        assert!(!close_box_rect(window).contains(Point::new(92.0, 78.0)));
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
    fn fullscreen_menu_toggles_active_window_state() {
        let (mut desktop, window_manager) = test_desktop();
        let id = desktop.windows[0].id;
        let original = desktop.windows[0].window.rect();

        desktop.handle_menu_action("shell.toggle_fullscreen");

        assert_eq!(desktop.windows[0].mode, ShellWindowMode::Fullscreen);
        assert!(desktop.windows[0].restore_rect.is_some());
        assert_rect_eq(desktop.windows[0].restore_rect.unwrap(), original);
        assert_eq!(
            window_manager.read().windows[&id].state,
            window_manager::WindowState::Fullscreen
        );
        assert_rect_eq(desktop.windows[0].window.rect(), desktop.content_bounds());

        desktop.handle_menu_action("shell.toggle_fullscreen");

        assert_eq!(desktop.windows[0].mode, ShellWindowMode::Normal);
        assert!(desktop.windows[0].restore_rect.is_none());
        assert_eq!(
            window_manager.read().windows[&id].state,
            window_manager::WindowState::Normal
        );
        assert_rect_eq(desktop.windows[0].window.rect(), original);
    }

    #[test]
    fn global_menu_shortcut_opens_new_finder_window() {
        let (mut desktop, _) = test_desktop();
        let initial_count = desktop.windows.len();

        let result = desktop.handle_event(&Event::KeyDown {
            key: retro_kit::event::KeyCode::N,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(desktop.windows.len(), initial_count + 1);
        assert_eq!(
            desktop.menu_server.read().active_app.as_deref(),
            Some("com.retro.finder")
        );
    }

    #[test]
    fn global_menu_shortcut_closes_active_window() {
        let (mut desktop, _) = test_desktop();
        let initial_count = desktop.windows.len();

        let result = desktop.handle_event(&Event::KeyDown {
            key: retro_kit::event::KeyCode::W,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(desktop.windows.len(), initial_count.saturating_sub(1));
    }

    #[test]
    fn global_menu_go_home_action_opens_home_window() {
        let (mut desktop, _) = test_desktop();
        let initial_count = desktop.windows.len();

        desktop.handle_menu_action("shell.open_home");

        assert_eq!(desktop.windows.len(), initial_count + 1);
        assert_eq!(desktop.windows.last().unwrap().window.title(), "Home");
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

        let view = server
            .menus
            .iter()
            .find(|menu| menu.title == "View")
            .expect("view menu exists");
        assert_eq!(view.items[3].action_id, "shell.toggle_fullscreen");
    }
}
