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
use retro_kit::theme::ThemeContext;
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
        let event_loop = retro_render::event_loop::RetroEventLoop::new();
        struct ShellHandler {
            window: Option<winit::window::Window>,
        }
        impl retro_render::event_loop::RetroAppHandler for ShellHandler {
            fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
                let attrs = winit::window::Window::default_attributes().with_title("RetroShell");
                match event_loop.create_window(attrs) {
                    Ok(window) => self.window = Some(window),
                    Err(err) => {
                        tracing::error!("failed to create shell window: {err}");
                        event_loop.exit();
                    }
                }
            }
            fn handle_window_event(
                &mut self,
                event_loop: &winit::event_loop::ActiveEventLoop,
                event: winit::event::WindowEvent,
            ) {
                if let winit::event::WindowEvent::CloseRequested = event {
                    event_loop.exit();
                }
            }
        }
        let mut handler = ShellHandler { window: None };
        let _ = event_loop.run(&mut handler);
        Ok(())
    }
}
