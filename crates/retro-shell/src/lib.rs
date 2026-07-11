pub mod a11y_actions;
pub mod a11y_prefs;
pub mod application_registry;
pub mod atspi_bus;
pub mod audio;
pub mod capture;
pub mod chrome_protocol;
pub mod desktop_manager;
pub mod display_arrange;
pub mod display_settings;
pub mod dock;
pub mod fdo_notifications;
pub mod foreign_toplevel;
pub mod foreign_toplevel_client;
pub mod i18n;
pub mod idle_policy;
pub mod layer_shell_client;
pub mod keyboard_nav;
pub mod launch_services;
pub mod menu_server;
pub mod mime_open;
pub mod network_connect;
pub mod network_manager;
pub mod notification_center;
pub mod polkit_agent;
pub mod portal;
pub mod portal_dbus;
pub mod portal_extra;
pub mod power;
pub mod screencast_pw;
pub mod session_actions;
pub mod session_clients;
pub mod session_manager;
pub mod session_packaging;
pub mod session_recovery;
pub mod shell_scale;
pub mod startup_budget;
pub mod theme_manager;
pub mod window_manager;
pub mod window_rules;
pub mod workspace_manager;

pub use a11y_actions::{
    a11y_invoke_is_live, actions_for_chrome, chrome_target_for_atspi_path, classify_a11y_invoke,
    invoke_id_for_object_name, plan_invoke, primary_invoke_for_chrome, resolve_pending_invoke,
    session_action_for_invoke, session_root_actions, summarize_actions, A11yDispatchTarget,
    AccessibleAction, ActionInterfaceSummary, InvokePlan,
};
pub use a11y_prefs::{
    apply_a11y_prefs_to_theme_name, effective_animation_ms, A11yPrefs, ContrastPreference,
    MotionPreference,
};
pub use application_registry::ApplicationRegistry;
pub use atspi_bus::{
    atspi_dbus_connection_available, chrome_focus_atspi_path, drain_in_process_events,
    emit_chrome_focus, in_process_event_count, serialize_chrome_focus_for_dbus,
    try_emit_accessible_event, EmitAccessibleResult,
};
pub use audio::{get_volume, set_volume};
pub use capture::{start_recording, stop_recording, take_screenshot};
pub use chrome_protocol::{
    chrome_focus_order, next_chrome_focus, should_paint_kit_chrome, ChromeFocusTarget, ChromeRole,
    ChromeSession, ProtocolChromeSurface,
};
pub use desktop_manager::DesktopManager;
pub use dock::Dock;
pub use foreign_toplevel::{
    apply_toplevel_force_quit, parse_toplevel_force_quit, ForeignToplevelEntry,
    ForeignToplevelRegistry, ToplevelForceQuit,
};
pub use foreign_toplevel_client::{
    apply_foreign_toplevel_list_event, apply_foreign_toplevel_list_events,
    try_sync_foreign_toplevels, ForeignToplevelListEvent,
};
pub use keyboard_nav::{
    apply_chrome_nav, is_dismissable_window_title, keyboard_nav_intent, KeyboardNavIntent,
};
pub use layer_shell_client::{
    chrome_to_layer_shell_requests, layer_shell_bind_summary, try_map_layer_shell_chrome,
    LayerShellBindResult, LayerShellChromeRequest,
};
pub use launch_services::LaunchServices;
pub use menu_server::MenuServer;
pub use mime_open::{
    mime_from_path, open_plan, parse_desktop_exec, seed_retroshell_defaults, DesktopAppEntry,
    MimeOpenRegistry, OpenPlan,
};
pub use network_connect::{
    nm_connect_plan, nm_connect_plan_validated, validate_nm_connect_request, NmConnectRequest,
};
pub use network_manager::{get_network_status, NetworkStatus};
pub use fdo_notifications::{
    try_register_session_bus as try_register_fdo_notifications, NotificationDaemon,
    NotificationPayload, NotificationServerState, NotifySendStyle, ServerInformation, Urgency,
    FDO_NOTIFICATIONS_BUS_NAME, FDO_NOTIFICATIONS_INTERFACE, FDO_NOTIFICATIONS_PATH,
};
pub use notification_center::{NotificationCenter, NotificationPriority};
pub use portal::{
    apply_screencast_readiness, create_screencast_session,
    create_screencast_session_with_backend_note, handle_file_chooser_open, handle_file_chooser_save,
    handle_open_uri, handle_portal_screenshot_request, portal_screenshot_filename,
    portal_screenshot_uri_for, portal_screenshots_dir, read_all_portal_settings,
    read_portal_setting, screencast_backend_note, screencast_backend_note_from_socket,
    select_screencast_sources, start_screencast, take_portal_style_screenshot,
    take_portal_style_screenshot_with, validate_file_chooser_request, PortalFileChooserRequest,
    PortalFileChooserResult, PortalScreencastRequest, PortalScreencastSession,
    PortalScreenshotRequest, PortalScreenshotResult, PortalSettingsNamespace, ScreencastStream,
    PORTAL_BUS_NAME, PORTAL_FILECHOOSER_INTERFACE, PORTAL_OPENURI_INTERFACE, PORTAL_PATH,
    PORTAL_SCREENCAST_INTERFACE, PORTAL_SCREENSHOT_INTERFACE, PORTAL_SETTINGS_INTERFACE,
    SCREENCAST_DEFAULT_HEIGHT, SCREENCAST_DEFAULT_WIDTH, SCREENCAST_NOTE_PIPEWIRE_SOCKET,
    SCREENCAST_NOTE_PORTAL_STUB, SCREENCAST_PLACEHOLDER_NODE_ID, SCREENCAST_SOURCE_TYPE_MONITOR,
    SCREENCAST_SOURCE_TYPE_WINDOW,
};
pub use polkit_agent::{
    handle_polkit_auth, try_register_polkit_agent, validate_polkit_request, PolkitAgentState,
    PolkitAuthDecision, PolkitAuthRequest, POLKIT_AGENT_BUS_NAME, POLKIT_AGENT_INTERFACE,
    POLKIT_AGENT_PATH,
};
pub use portal_dbus::try_register_portal_session_bus;
pub use portal_extra::{
    active_idle_inhibit_state, active_inhibits, clear_inhibit_store_for_tests, handle_inhibit,
    handle_inhibit_and_register, handle_print_request, handle_secret_retrieve, inhibit_blocks_idle,
    inhibit_to_idle_reason, portal_blocks_idle, register_inhibit_cookie, release_inhibit_cookie,
    InhibitFlag, PortalInhibitCookie, PortalInhibitRequest, PortalPrintRequest, PortalPrintResult,
    PortalSecretRequest, PortalSecretResult,
};
pub use power::{battery_info, BatteryInfo};
pub use display_arrange::{
    apply_display_plan_env, arrange_mode_from_env_value, arrangement_bounds, normalize_arrangement,
    place_outputs, plan_display_apply, ArrangeMode, DisplayApplyPlan, DisplayApplyStep,
    DisplayArrangement, DisplayOutput, PlacedOutput,
};
pub use display_settings::DisplayConfig;
pub use i18n::{
    format_message, is_rtl_language, text_direction_for_locale, tr, LocaleId, LocalePrefs,
    MessageCatalog, TextDirection,
};
pub use idle_policy::{
    idle_phase, recommended_action, secs_until_next_phase, IdleConfig, IdleInhibitState,
    IdlePhase, IdleRecommendedAction, InhibitReason,
};
pub use screencast_pw::{
    can_claim_live_streams, default_pipewire_socket, plan_list_pipewire_nodes,
    probe_screencast_readiness, probe_screencast_readiness_host, source_ids_for_portal,
    sources_from_outputs, sources_from_windows, PwListNodesPlan, ScreencastBackend,
    ScreencastReadiness, ScreencastSource, ScreencastSourceType,
};
pub use session_actions::{
    confirm_prompt, confirm_prompt_i18n_key, describe_plan, plan_requires_privileges,
    plan_session_action, plan_session_action_with, requires_confirmation, shell_delta_for_plan,
    PowerBackend, SessionAction, SessionActionPlan, ShellSessionDelta, LOGIND_BUS,
    LOGIND_MANAGER_IFACE, LOGIND_PATH,
};
pub use window_rules::{
    default_session_rules, evaluate_rules, field_matches, parse_rules_simple, rule_matches,
    MatchField, MatchKind, WindowInfo, WindowMatch, WindowRule, WindowRuleActions,
};
pub use session_clients::{
    binary_name_for_bundle, parse_force_quit_entry, resolve_app_binary, spawn_app_client,
    ForceQuitTarget, SessionClientRegistry,
};
pub use session_manager::SessionManager;
pub use session_packaging::{
    check_greeter_session_readiness, check_packaging_health, parse_desktop_keys,
    validate_session_desktop, GreeterSessionReadiness, PackagingHealth, SessionPackagingLayout,
};
pub use session_recovery::{
    recovery_plan, should_attempt_recovery, CheckpointClient, RecoveryStep, SessionCheckpoint,
};
pub use shell_scale::{
    detect_shell_scale_from_env, parse_shell_scale, scale_layout_dim, scaled_chrome_insets,
    ShellScale,
};
pub use startup_budget::{
    default_desktop_budget, overall_ok, record_phase, total_elapsed_ms, PhaseResult, StartupBudget,
    StartupPhase,
};
pub use theme_manager::ThemeManager;
pub use window_manager::WindowManager;
pub use workspace_manager::{
    WorkspaceManager, COMPOSITOR_WORKSPACE_COUNT, SHELL_DESKTOP_COUNT,
};

use parking_lot::RwLock;
use retro_kit::event::MouseButton;
use retro_kit::icon_view::{IconItem, IconView};
use retro_kit::button::Button;
use retro_kit::list_view::ListView;
use retro_kit::workspace_grid_view::WorkspaceGridView;
use retro_kit::label::Label;
use retro_kit::layout::LayoutView;
use retro_kit::menu::{Menu, MenuItemKind};
use retro_kit::menu_bar::MenuBar;
use retro_kit::text_field::TextField;
use retro_kit::theme::ThemeContext;
use retro_kit::window::Window;
use retro_kit::{
    Event, EventResult, Layout, LayoutConstraint, Point, Rect, Size, Widget, WidgetState, DockView,
};
use std::fs;
use std::path::PathBuf;
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
        {
            let mut tm = shell.theme_manager.write();
            tm.load_default();
            // Load named theme + a11y prefs (high_contrast / reduced_motion) from settings.conf.
            tm.load_theme_from_settings();
        }
        // Locale from LANG + optional settings.conf; drive system menu chrome strings.
        {
            let conf_text = read_settings_conf_text();
            let prefs = if conf_text.is_empty() {
                LocalePrefs::parse_from_env_lang(std::env::var("LANG").ok().as_deref())
            } else {
                let mut p = LocalePrefs::parse_from_conf(&conf_text);
                // Env LANG still wins when conf has no locale key.
                if p.locale.language == "en"
                    && p.locale.region.as_deref() == Some("US")
                    && !conf_text.lines().any(|l| {
                        let t = l.trim();
                        t.starts_with("locale=")
                            || t.starts_with("lang=")
                            || t.starts_with("language=")
                    })
                {
                    p = LocalePrefs::parse_from_env_lang(std::env::var("LANG").ok().as_deref());
                }
                p
            };
            shell.menu_server.write().apply_locale_labels(&prefs);
            tracing::info!(locale = %prefs.locale.tag(), "shell menu locale applied");
        }
        // Multi-monitor arrange: plan + live EmitLayoutEnv (RETROSHELL_OUTPUTS_LAYOUT).
        apply_display_config_from_settings();
        // Best-effort FreeDesktop Notifications on the session bus (Linux).
        // Failure is non-fatal: pure NotificationCenter still works in-process.
        let _ = fdo_notifications::try_register_session_bus(shell.notification_center.clone());
        // Best-effort portal Screenshot/Settings/OpenURI on the session bus (Linux).
        let _ = portal_dbus::try_register_portal_session_bus();
        // Best-effort polkit authentication agent (Linux).
        let _ = polkit_agent::try_register_polkit_agent();
        Ok(shell)
    }

    /// Bind protocol session chrome (layer-shell) and sync foreign-toplevel list.
    ///
    /// Call after `WAYLAND_DISPLAY` is set (compositor/labwc running). Non-fatal.
    pub(crate) fn attach_wayland_session_protocols(desktop: &mut ShellDesktop) {
        if let Some(summary) = layer_shell_client::try_map_layer_shell_chrome(&desktop.chrome) {
            tracing::info!(
                surfaces = ?summary.mapped_namespaces,
                "shell mapped layer-shell chrome"
            );
            desktop.layer_shell_bound = true;
        }
        if let Some(n) =
            foreign_toplevel_client::try_sync_foreign_toplevels(&mut desktop.foreign_toplevels)
        {
            tracing::info!(toplevels = n, "shell synced foreign-toplevel-list");
            desktop.foreign_toplevel_synced = true;
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut app = retro_sdk::Application::new("RetroShell", "com.retro.shell");
        app.set_initial_size(Size::new(1280.0, 800.0));

        let desktop_view = ShellDesktop::new(
            self.menu_server.clone(),
            self.launch_services.clone(),
            self.window_manager.clone(),
            self.notification_center.clone(),
            self.workspace_manager.clone(),
            self.dock.clone(),
            self.session_manager.clone(),
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
    notification_center: Arc<RwLock<NotificationCenter>>,
    workspace_manager: Arc<RwLock<WorkspaceManager>>,
    dock: Arc<RwLock<Dock>>,
    session_manager: Arc<RwLock<SessionManager>>,
    dock_view: DockView,
    bundle_ids: Vec<String>,
    /// Notification banner pop-up windows, rebuilt each update() from visible notifications.
    notification_popup_windows: Vec<Window>,
    /// Last application-launch error, if any. Set by `launch_external_app` on failure.
    /// Intended for display in the status bar (rendering integration pending).
    last_error: Option<String>,
    /// Whether the screen is currently locked.
    locked: bool,
    /// Lock screen overlay widget, shown when `locked` is true.
    lock_screen_widget: Window,
    /// Password field for the lock screen.
    lock_password_field: TextField,
    /// Error message to display on lock screen (e.g., "Incorrect password").
    lock_error_message: Option<String>,
    /// The expected lock password (from env or config).
    expected_lock_password: Option<String>,
    /// Independent first-party app processes (compositor/labwc clients).
    session_clients: SessionClientRegistry,
    /// Protocol-backed session chrome (layer-shell menu bar / dock roles).
    chrome: ChromeSession,
    /// Foreign-toplevel registry for task list / Force Quit (compositor-synced when possible).
    foreign_toplevels: ForeignToplevelRegistry,
    /// True after a successful `zwlr_layer_shell_v1` chrome bind.
    layer_shell_bound: bool,
    /// True after a successful `ext_foreign_toplevel_list_v1` sync.
    foreign_toplevel_synced: bool,
    /// Keyboard-only chrome focus region (Tab cycle).
    chrome_focus: ChromeFocusTarget,
    /// Monotonic instant of last user input (for idle auto-lock).
    last_input_at: std::time::Instant,
    /// Idle lock/suspend policy (from defaults / settings.conf).
    idle_config: IdleConfig,
    /// Portal / media idle inhibit tokens.
    idle_inhibit: IdleInhibitState,
}

struct ShellWindow {
    id: Uuid,
    window: Window,
    folder_path: Option<PathBuf>,
    restore_rect: Option<Rect>,
    mode: ShellWindowMode,
    workspace: usize,
}

struct FolderOpenTarget {
    title: String,
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellWindowMode {
    Normal,
    Minimized,
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
        notification_center: Arc<RwLock<NotificationCenter>>,
        workspace_manager: Arc<RwLock<WorkspaceManager>>,
        dock: Arc<RwLock<Dock>>,
        session_manager: Arc<RwLock<SessionManager>>,
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
        let lock_screen_widget = build_lock_screen_window();
        let expected_lock_password = get_lock_password();
        let mut lock_password_field = TextField::new()
            .with_placeholder("Enter password");
        lock_password_field.is_password = true;
        let mut shell = Self {
            state: WidgetState::new(),
            menu_bar: MenuBar::new(menus),
            desktop,
            windows: Vec::new(),
            window_interaction: None,
            menu_server,
            launch_services,
            window_manager,
            notification_center,
            workspace_manager,
            dock: dock.clone(),
            session_manager,
            dock_view: DockView::new(),
            bundle_ids,
            notification_popup_windows: Vec::new(),
            last_error: None,
            locked: false,
            lock_screen_widget,
            lock_password_field,
            lock_error_message: None,
            expected_lock_password,
            session_clients: SessionClientRegistry::new(),
            // Protocol chrome: menu bar (24) + dock (64) matching content_bounds insets.
            chrome: ChromeSession::bootstrap_default(1280, 800, 24, 64),
            foreign_toplevels: ForeignToplevelRegistry::new(),
            layer_shell_bound: false,
            foreign_toplevel_synced: false,
            chrome_focus: ChromeFocusTarget::MenuBar,
            last_input_at: std::time::Instant::now(),
            // settings.conf flat keys (idle_lock_secs, …) when present; else defaults.
            idle_config: IdleConfig::parse_from_conf(&read_settings_conf_text()),
            idle_inhibit: IdleInhibitState::new(),
        };
        // Map layer-shell chrome + sync foreign-toplevel list when a compositor is live.
        RetroShell::attach_wayland_session_protocols(&mut shell);
        shell.open_finder_window();
        shell
    }

    /// Refresh foreign-toplevel list from the compositor before showing Force Quit.
    fn refresh_foreign_toplevels_from_compositor(&mut self) {
        if let Some(n) =
            foreign_toplevel_client::try_sync_foreign_toplevels(&mut self.foreign_toplevels)
        {
            self.foreign_toplevel_synced = true;
            tracing::debug!(toplevels = n, "Force Quit: foreign-toplevel-list refreshed");
            self.apply_foreign_rule_workspaces_to_shell_windows();
        }
    }

    /// When foreign-toplevel window rules assign a workspace, move matching
    /// [`ShellWindow`]s to that workspace index (clamped to 0..7).
    fn apply_foreign_rule_workspaces_to_shell_windows(&mut self) {
        let assignments: Vec<(String, String, usize)> = self
            .foreign_toplevels
            .entries()
            .filter_map(|e| {
                e.workspace.map(|ws| {
                    (
                        e.title.clone(),
                        e.app_id.clone(),
                        (ws as usize).min(SHELL_DESKTOP_COUNT.saturating_sub(1)),
                    )
                })
            })
            .collect();
        if assignments.is_empty() {
            return;
        }
        let mut moved = 0usize;
        for (title, app_id, ws) in assignments {
            for shell_window in &mut self.windows {
                let title_match = shell_window.window.title() == title;
                // Session-client foreign entries use binary name as title and
                // bundle_id as app_id; also match title substring for internal windows.
                let app_title_match = !app_id.is_empty()
                    && shell_window
                        .window
                        .title()
                        .to_ascii_lowercase()
                        .contains(&app_id.to_ascii_lowercase());
                if title_match || app_title_match {
                    if shell_window.workspace != ws {
                        shell_window.workspace = ws;
                        self.window_manager
                            .write()
                            .assign_workspace(shell_window.id, ws);
                        moved += 1;
                    }
                }
            }
        }
        if moved > 0 {
            tracing::debug!(moved, "window rules: moved ShellWindows to rule workspaces");
        }
    }

    /// Drain kit AT-SPI pending DoAction queue into real shell handlers.
    fn drain_a11y_pending_actions(&mut self) {
        let pending = retro_kit::drain_pending_actions();
        for action in pending {
            let plan = resolve_pending_invoke(
                &action.path,
                &action.object_name,
                action.action_index,
                &action.action_name,
            );
            if !plan.valid {
                tracing::debug!(
                    path = %action.path,
                    name = %action.object_name,
                    "a11y DoAction: no shell invoke mapping"
                );
                continue;
            }
            tracing::info!(
                invoke_id = %plan.invoke_id,
                path = %action.path,
                "a11y DoAction → shell handler"
            );
            self.dispatch_a11y_invoke(&plan.invoke_id);
        }
    }

    /// Handle chrome.* and shell.* invoke ids from a11y / keyboard Activate.
    ///
    /// Routing is pure-classified via [`classify_a11y_invoke`]; side effects run here.
    fn dispatch_a11y_invoke(&mut self, invoke_id: &str) {
        match classify_a11y_invoke(invoke_id) {
            A11yDispatchTarget::ChromeMenuActivate => {
                // Open the system/Retro menu (index 0). Prefer title "Retro" when present.
                if let Some(idx) = self
                    .menu_bar
                    .menus
                    .iter()
                    .position(|m| m.title == "Retro")
                {
                    let _ = self.menu_bar.open_menu_at(idx);
                } else {
                    let _ = self.menu_bar.open_first_menu();
                }
            }
            A11yDispatchTarget::ChromeDockActivate => {
                if let Some(bundle) = self.bundle_ids.first().cloned() {
                    self.launch_external_app(&bundle);
                }
            }
            A11yDispatchTarget::ChromeDesktopOpen => {
                if let Some(item) = self.desktop.items.iter().position(|i| i.selected) {
                    self.launch_item(item);
                } else if !self.desktop.items.is_empty() {
                    self.launch_item(0);
                }
            }
            A11yDispatchTarget::ChromeWindowActivateNext => {
                // Focus next non-minimized window on the active workspace (Cmd+Tab parity).
                self.focus_next_window();
            }
            A11yDispatchTarget::ChromeWindowClose => self.close_active_window(),
            A11yDispatchTarget::ChromeWindowMinimize => {
                if let Some(id) = self.active_window_id() {
                    self.toggle_window_minimized(id);
                }
            }
            A11yDispatchTarget::ChromeDockMenu | A11yDispatchTarget::ChromeDesktopMenu => {
                tracing::debug!(invoke_id, "a11y context menu invoke (log-only: no popup yet)");
            }
            A11yDispatchTarget::MenuAction(action) => {
                // shell.lock / shell.log_out / shell.notification_center /
                // shell.force_quit / workspace.next / workspace.previous / …
                self.handle_menu_action(action);
            }
            A11yDispatchTarget::MenuActionOwned(action) => {
                self.handle_menu_action(&action);
            }
            A11yDispatchTarget::Unknown => {
                // Fall through: keep prior behaviour for any unmapped but menu-like ids.
                self.handle_menu_action(invoke_id);
            }
        }
    }

    /// Cycle focus to the next non-minimized window on the active workspace.
    fn focus_next_window(&mut self) {
        let active_workspace = self.active_workspace();
        let workspace_window_ids: Vec<Uuid> = self
            .windows
            .iter()
            .filter(|w| w.workspace == active_workspace && w.mode != ShellWindowMode::Minimized)
            .map(|w| w.id)
            .collect();
        if workspace_window_ids.is_empty() {
            return;
        }
        let next_id = if let Some(current_id) = self.active_window_id() {
            let pos = workspace_window_ids
                .iter()
                .position(|&id| id == current_id)
                .unwrap_or(0);
            workspace_window_ids[(pos + 1) % workspace_window_ids.len()]
        } else {
            workspace_window_ids[0]
        };
        self.focus_window(next_id);
    }

    fn launch_item(&mut self, index: usize) {
        let item = match self.desktop.items.get(index) {
            Some(item) => item,
            None => return,
        };

        if let Some(bundle_id) = item.icon.as_deref() {
            if self.bundle_ids.iter().any(|id| id == bundle_id) {
                let bundle_id = bundle_id.to_string();
                self.launch_external_app(&bundle_id);
                return;
            }
        }

        match item.label.as_str() {
            "Applications" => {
                let bundle_id = self
                    .launch_services
                    .read()
                    .bundle_for_id("com.retro.finder")
                    .map(|bundle| bundle.bundle_id.clone());
                if let Some(bundle_id) = bundle_id {
                    self.launch_external_app(&bundle_id);
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
        // Prefer protocol chrome exclusive zones (menu bar top / dock bottom).
        // HiDPI: scale insets when RETROSHELL_OUTPUT_SCALE / SHELL_SCALE > 1.
        let scale = detect_shell_scale_from_env();
        let menu_height = self
            .chrome
            .surfaces()
            .iter()
            .find(|s| s.role == ChromeRole::MenuBar && s.mapped)
            .map(|s| s.height as f64)
            .unwrap_or(24.0);
        let dock_height = self
            .chrome
            .surfaces()
            .iter()
            .find(|s| s.role == ChromeRole::Dock && s.mapped)
            .map(|s| s.height as f64)
            .unwrap_or(64.0);
        let (menu_height, dock_height) =
            scaled_chrome_insets(scale, menu_height, dock_height);
        let menu_height = menu_height as f32;
        let dock_height = dock_height as f32;
        Rect::new(
            self.rect().x,
            self.rect().y + menu_height,
            self.rect().width,
            (self.rect().height - menu_height - dock_height).max(0.0),
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

    fn active_workspace(&self) -> usize {
        self.workspace_manager.read().active
    }

    fn open_folder_window<S: Into<String>>(&mut self, title: S, path: PathBuf) -> Uuid {
        let rect = self.next_finder_rect();
        let title = title.into();
        let mut window = build_folder_window(&title, &path);
        window.set_rect(rect);
        let workspace = self.active_workspace();
        let id =
            self.window_manager
                .write()
                .create_window("com.retro.finder", window.title(), rect);
        self.window_manager.write().assign_workspace(id, workspace);
        self.windows.push(ShellWindow {
            id,
            window,
            folder_path: Some(path),
            restore_rect: None,
            mode: ShellWindowMode::Normal,
            workspace,
        });
        self.focus_window(id);
        self.layout_window(id);
        id
    }

    fn open_message_window<S: Into<String>>(
        &mut self,
        title: S,
        lines: impl IntoIterator<Item = String>,
    ) -> Uuid {
        let title = title.into();
        let rect = clamp_window_rect(
            Rect::new(
                self.content_bounds().x + 112.0,
                self.content_bounds().y + 72.0,
                540.0,
                240.0,
            ),
            self.content_bounds(),
        );
        let mut window = build_message_window(&title, lines);
        window.set_rect(rect);
        let workspace = self.active_workspace();
        let id = self
            .window_manager
            .write()
            .create_window("com.retro.shell", window.title(), rect);
        self.window_manager.write().assign_workspace(id, workspace);
        self.windows.push(ShellWindow {
            id,
            window,
            folder_path: None,
            restore_rect: None,
            mode: ShellWindowMode::Normal,
            workspace,
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

        if self.windows[index].mode == ShellWindowMode::Minimized {
            self.restore_minimized_window(id);
            return;
        }

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

    fn toggle_window_minimized(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };

        if self.windows[index].mode == ShellWindowMode::Minimized {
            self.restore_minimized_window(id);
            return;
        }

        let current = self.windows[index].window.rect();
        let minimized_rect = minimized_window_rect(self.content_bounds(), index);
        self.windows[index].restore_rect = Some(current);
        self.windows[index].mode = ShellWindowMode::Minimized;
        self.windows[index].window.set_rect(minimized_rect);
        self.window_manager.write().minimize_window(id);
        self.layout_window(id);
    }

    fn restore_minimized_window(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };
        let restore_rect = self.windows[index]
            .restore_rect
            .take()
            .unwrap_or_else(|| default_finder_rect(self.rect()));
        let restore_rect = clamp_window_rect(restore_rect, self.content_bounds());
        self.windows[index].window.set_rect(restore_rect);
        self.windows[index].mode = ShellWindowMode::Normal;
        self.window_manager.write().restore_window(id);
        self.layout_window(id);
    }

    fn toggle_window_fullscreen(&mut self, id: Uuid) {
        let Some(index) = self.window_index(id) else {
            return;
        };

        if self.windows[index].mode == ShellWindowMode::Minimized {
            self.restore_minimized_window(id);
            return;
        }

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
            self.refresh_menu_manifests();
            self.menu_server.write().set_active_app_menus(&app_id);
        } else {
            self.menu_server.write().reset_to_shell_menus();
        }
        self.menu_bar.menus = self.menu_server.read().menus.clone();
    }

    fn activate_app_menu(&mut self, bundle_id: &str) {
        self.refresh_menu_manifests();
        self.menu_server.write().set_active_app_menus(bundle_id);
        self.menu_bar.menus = self.menu_server.read().menus.clone();
    }

    fn refresh_menu_manifests(&mut self) {
        let Some(dir) = retro_sdk::menu_manifest_dir() else {
            return;
        };
        if let Err(err) = self.menu_server.write().load_menu_manifests_from_dir(dir) {
            tracing::warn!("failed to load menu manifests: {err}");
        }
    }

    fn switch_workspace(&mut self, workspace: usize) -> bool {
        if !self.workspace_manager.write().switch_to(workspace) {
            return false;
        }
        let active_workspace = self.active_workspace();
        for shell_window in &mut self.windows {
            shell_window.window.is_active = false;
        }
        let active_id = self
            .windows
            .iter()
            .rev()
            .find(|window| window.workspace == active_workspace)
            .map(|window| window.id);
        if let Some(id) = active_id {
            if let Some(index) = self.window_index(id) {
                self.windows[index].window.is_active = true;
            }
            self.window_manager.write().focus_window(id);
        } else {
            self.window_manager.write().active_window = None;
        }
        self.open_workspace_status_window();
        true
    }

    fn switch_to_next_workspace(&mut self) {
        self.workspace_manager.write().next();
        let active = self.active_workspace();
        let _ = self.switch_workspace(active);
    }

    fn switch_to_previous_workspace(&mut self) {
        self.workspace_manager.write().previous();
        let active = self.active_workspace();
        let _ = self.switch_workspace(active);
    }

    fn open_workspace_status_window(&mut self) {
        for window in &self.windows {
            if window.window.title() == "Workspace" {
                self.focus_window(window.id);
                return;
            }
        }

        let manager = self.workspace_manager.read();
        let active = manager.active;
        let name = manager
            .active_workspace()
            .map(|workspace| workspace.name.clone())
            .unwrap_or_else(|| format!("Desktop {}", active + 1));
        drop(manager);

        let visible_count = self
            .windows
            .iter()
            .filter(|window| window.workspace == active)
            .count();

        let rect = clamp_window_rect(
            Rect::new(
                self.content_bounds().x + 180.0,
                self.content_bounds().y + 120.0,
                300.0,
                260.0,
            ),
            self.content_bounds(),
        );

        let mut layout = Layout::vertical(12.0);
        layout.add(Box::new(Label::new("Select/Switch Workspace:")));

        let mut grid = WorkspaceGridView::new();
        grid.active_index = active;
        grid.items = (1..=SHELL_DESKTOP_COUNT)
            .map(|n| format!("Desktop {n}"))
            .collect();
        layout.add(Box::new(grid));

        let desc = format!("Active: {} ({} windows)", name, visible_count);
        layout.add(Box::new(Label::new(desc)));

        let mut window = Window::new("Workspace");
        window.set_content(Box::new(LayoutView::new(layout)));
        window.set_rect(rect);

        let workspace = self.active_workspace();
        let id = self
            .window_manager
            .write()
            .create_window("com.retro.shell", window.title(), rect);
        self.window_manager.write().assign_workspace(id, workspace);
        self.windows.push(ShellWindow {
            id,
            window,
            folder_path: None,
            restore_rect: None,
            mode: ShellWindowMode::Normal,
            workspace,
        });
        self.focus_window(id);
        self.layout_window(id);
    }

    fn launch_external_app(&mut self, bundle_id: &str) {
        // Reap exited clients first so the registry reflects the live multi-client set.
        let _ = self.session_clients.reap();
        match session_clients::spawn_app_client(bundle_id) {
            Ok(client) => {
                let pid = client.pid;
                let binary_name = client.binary_name.clone();
                tracing::info!(
                    "Launched multi-client app {bundle_id} as pid {pid} (compositor-managed surface)"
                );
                // Foreign-toplevel mirror for Force Quit / task list (with pid).
                self.foreign_toplevels.add(ForeignToplevelEntry::new(
                    format!("session-client-{pid}"),
                    binary_name,
                    bundle_id,
                    Some(pid),
                ));
                self.apply_foreign_rule_workspaces_to_shell_windows();
                self.session_clients.register(client);
                self.last_error = None;
                self.activate_app_menu(bundle_id);
                self.record_notification(
                    bundle_id,
                    "Application Launched",
                    &format!(
                        "Started process pid={pid} ({} client(s) active).",
                        self.session_clients.len()
                    ),
                    NotificationPriority::Normal,
                );
            }
            Err(msg) => {
                tracing::error!("launch_external_app failed for {bundle_id}: {msg}");
                self.last_error = Some(msg.clone());
                self.record_notification(
                    bundle_id,
                    "Launch Failed",
                    &msg,
                    NotificationPriority::Normal,
                );
            }
        }
    }

    /// Apply a Force Quit list selection (window title, external client, or foreign toplevel).
    /// Returns true if a shell window closed or a client/toplevel was force-quit.
    fn apply_force_quit_entry(&mut self, entry: &str) -> bool {
        if let Some(target) = parse_toplevel_force_quit(entry) {
            let ok = apply_toplevel_force_quit(&mut self.foreign_toplevels, &target);
            if let Some(pid) = target.pid {
                // Keep session client registry in sync when quitting by pid.
                let client_ok = self.session_clients.force_quit_pid(pid);
                return ok || client_ok;
            }
            return ok;
        }
        match parse_force_quit_entry(entry) {
            Some(ForceQuitTarget::WindowTitle(title)) => {
                let target_id = self
                    .windows
                    .iter()
                    .find(|w| w.window.title() == title)
                    .map(|w| w.id);
                if let Some(tid) = target_id {
                    self.close_window(tid);
                    true
                } else {
                    false
                }
            }
            Some(ForceQuitTarget::ClientPid(pid)) => {
                // Drop matching foreign-toplevel entry if present (match by pid).
                let _ = self.foreign_toplevels.remove_match(&ToplevelForceQuit {
                    title: String::new(),
                    app_id: None,
                    pid: Some(pid),
                });
                self.session_clients.force_quit_pid(pid)
            }
            None => false,
        }
    }

    fn active_window_id(&self) -> Option<Uuid> {
        let active_workspace = self.active_workspace();
        self.windows
            .iter()
            .rev()
            .find(|window| window.workspace == active_workspace)
            .map(|window| window.id)
    }

    fn window_index(&self, id: Uuid) -> Option<usize> {
        self.windows.iter().position(|window| window.id == id)
    }

    fn top_window_index_at(&self, point: Point) -> Option<usize> {
        let active_workspace = self.active_workspace();
        self.windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, window)| {
                window.workspace == active_workspace && window.window.rect().contains(point)
            })
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
            } else if self.windows[index].mode == ShellWindowMode::Minimized {
                minimized_window_rect(bounds, index)
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
            "shell.open_finder" => self.launch_external_app("com.retro.finder"),
            "shell.settings" => self.launch_external_app("com.retro.settings"),
            "shell.software_catalog" => self.launch_external_app("com.retro.appstore"),
            "shell.about" => {
                self.open_about_window();
            }
            "shell.notification_center" => self.open_notification_center_window(),
            "shell.clear_notifications" => self.clear_notifications(),
            "shell.recent_items" => self.open_shell_status_window(
                "Recent Items",
                [
                    "Recent item tracking is not populated yet.".to_string(),
                    "Finder and app launches will be recorded here once session history is wired."
                        .to_string(),
                ],
            ),
            "shell.force_quit" => self.open_force_quit_window(),
            "shell.lock" => self.handle_session_action(session_actions::SessionAction::Lock),
            "shell.log_out" | "shell.logout" => {
                self.handle_session_action(session_actions::SessionAction::Logout)
            }
            "shell.suspend" | "shell.sleep" => {
                self.handle_session_action(session_actions::SessionAction::Suspend)
            }
            "shell.reboot" | "shell.restart" => {
                self.handle_session_action(session_actions::SessionAction::Reboot)
            }
            "shell.power_off" | "shell.shutdown" | "shell.poweroff" => {
                self.handle_session_action(session_actions::SessionAction::PowerOff)
            }
            "shell.save" => self.open_shell_status_window(
                "Save",
                ["The active shell window has no document to save.".to_string()],
            ),
            "shell.print" => self.open_shell_status_window(
                "Print",
                ["Printing is not connected to a system print service yet.".to_string()],
            ),
            "shell.screenshot" | "shell.portal_screenshot" => {
                // shell.portal_screenshot is the FreeDesktop portal-facing path;
                // until xdg-desktop-portal is wired it uses the same local capture.
                let result = if action == "shell.portal_screenshot" {
                    portal::take_portal_style_screenshot()
                } else {
                    capture::take_screenshot()
                };
                match result {
                    Ok(path) => {
                        self.record_notification(
                            "com.retro.shell",
                            "Screenshot Saved",
                            &format!("Saved to {}", path.display()),
                            NotificationPriority::Normal,
                        );
                    }
                    Err(err) => {
                        self.record_notification(
                            "com.retro.shell",
                            "Screenshot Failed",
                            &err.to_string(),
                            NotificationPriority::High,
                        );
                    }
                }
            }
            "shell.start_recording" => match capture::start_recording() {
                Ok(path) => {
                    self.record_notification(
                        "com.retro.shell",
                        "Screen Recording",
                        &format!("Recording to {}", path.display()),
                        NotificationPriority::Normal,
                    );
                }
                Err(err) => {
                    self.record_notification(
                        "com.retro.shell",
                        "Recording Failed",
                        &err.to_string(),
                        NotificationPriority::High,
                    );
                }
            },
            "shell.stop_recording" => match capture::stop_recording() {
                Ok(path) => {
                    self.record_notification(
                        "com.retro.shell",
                        "Recording Saved",
                        &format!("Saved to {}", path.display()),
                        NotificationPriority::Normal,
                    );
                }
                Err(err) => {
                    self.record_notification(
                        "com.retro.shell",
                        "Stop Recording Failed",
                        &err.to_string(),
                        NotificationPriority::High,
                    );
                }
            },
            "shell.undo" | "shell.redo" | "shell.cut" | "shell.copy" | "shell.paste"
            | "shell.select_all" => self.open_shell_status_window(
                "Edit",
                ["This edit command is only available inside document-aware apps.".to_string()],
            ),
            "shell.show_toolbar" => self.open_shell_status_window(
                "Toolbar",
                [
                    "Finder toolbar controls are already visible in shell folder windows."
                        .to_string(),
                ],
            ),
            "shell.show_sidebar" => self.open_shell_status_window(
                "Sidebar",
                ["The internal shell Finder view does not have a sidebar yet.".to_string()],
            ),
            "shell.help_search" => self.open_shell_status_window(
                "Help",
                [
                    "Help search is not indexed yet.".to_string(),
                    "Use the README and docs/implementation_plan.md for current status."
                        .to_string(),
                ],
            ),
            "workspace.previous" => self.switch_to_previous_workspace(),
            "workspace.next" => self.switch_to_next_workspace(),
            action if action.starts_with("workspace.switch.") => {
                if let Some(index) = action
                    .strip_prefix("workspace.switch.")
                    .and_then(|value| value.parse::<usize>().ok())
                {
                    let _ = self.switch_workspace(index);
                }
            }
            "shell.quit" => {
                std::process::exit(0);
            }
            "finder.new_folder" => self.handle_new_folder(),
            "finder.get_info" => self.handle_get_info(),
            "finder.rename" => self.handle_rename(),
            "finder.move_to_trash" => self.handle_move_to_trash(),
            _ if self.handle_sdk_app_menu_action(action) => {}
            _ => tracing::info!("Unhandled menu action: {action}"),
        }
    }

    fn handle_sdk_app_menu_action(&mut self, action: &str) -> bool {
        let active_app = self.menu_server.read().active_app.clone();
        let Some(active_app) = active_app else {
            return false;
        };
        if !action.starts_with(&format!("{active_app}.")) {
            return false;
        }

        let action_label = menu_action_label(&self.menu_bar.menus, action).unwrap_or_else(|| {
            action
                .rsplit('.')
                .next()
                .unwrap_or(action)
                .replace('_', " ")
        });
        self.open_shell_status_window(
            "Application Menu Action",
            [
                format!("Application: {active_app}"),
                format!("Action: {action_label}"),
                format!("Identifier: {action}"),
                "Cross-process dispatch requires session/compositor IPC.".to_string(),
            ],
        );
        true
    }

    /// Execute a session power/logout action via pure plan + shell side effects.
    fn handle_session_action(&mut self, action: session_actions::SessionAction) {
        use session_actions::{
            confirm_prompt, describe_plan, plan_session_action, plan_requires_privileges,
            shell_delta_for_plan, SessionActionPlan,
        };

        let plan = plan_session_action(action);

        // Destructive actions: show confirm UI first (status window). User can
        // re-trigger from Power menu after reading; shell.quit remains immediate.
        if let Some(prompt) = confirm_prompt(action) {
            if !matches!(
                action,
                session_actions::SessionAction::Logout
                    | session_actions::SessionAction::Reboot
                    | session_actions::SessionAction::PowerOff
            ) {
                // unreachable for current confirm_prompt set
            } else {
                // For logout we still proceed (session exit is the product intent).
                // Reboot/PowerOff stay gated: show plan + require explicit systemctl spawn
                // only after confirm status — we present the plan and run system commands
                // when privileges path is available; otherwise notify.
                if matches!(
                    action,
                    session_actions::SessionAction::Reboot
                        | session_actions::SessionAction::PowerOff
                ) {
                    self.open_shell_status_window(
                        match action {
                            session_actions::SessionAction::Reboot => "Restart",
                            _ => "Shut Down",
                        },
                        [
                            prompt.to_string(),
                            format!("Plan: {}", describe_plan(&plan)),
                            if plan_requires_privileges(&plan) {
                                "This will invoke system power management (systemctl/logind)."
                                    .to_string()
                            } else {
                                String::new()
                            },
                            "Executing now…".to_string(),
                        ],
                    );
                }
            }
        }

        let delta = shell_delta_for_plan(&plan);
        if delta.lock {
            if self.expected_lock_password.is_some() {
                self.session_manager.write().lock_screen();
                self.locked = true;
                self.lock_password_field.set_text("");
                self.lock_error_message = None;
            } else {
                self.notification_center.write().post(
                    "com.retro.shell",
                    "Lock Password Not Set",
                    "Configure RETROSHELL_LOCK_PASSWORD env var or lock_password in ~/.config/retroshell/settings.conf",
                    NotificationPriority::High,
                );
            }
            return;
        }

        match plan {
            SessionActionPlan::ShellExit { code } => {
                tracing::info!(code, "session logout: shell exit");
                self.session_manager.write().logout_without_exit();
                std::process::exit(code);
            }
            SessionActionPlan::SystemCommand { argv } => {
                tracing::info!(?argv, "session power action");
                match std::process::Command::new(&argv[0]).args(&argv[1..]).spawn() {
                    Ok(_) => {
                        self.record_notification(
                            "com.retro.shell",
                            "Session",
                            &format!("Started: {}", argv.join(" ")),
                            NotificationPriority::Normal,
                        );
                    }
                    Err(err) => {
                        self.record_notification(
                            "com.retro.shell",
                            "Session Action Failed",
                            &format!("{} ({err})", describe_plan(&SessionActionPlan::SystemCommand { argv: argv.clone() })),
                            NotificationPriority::High,
                        );
                        self.open_shell_status_window(
                            "Session Action",
                            [
                                describe_plan(&SessionActionPlan::SystemCommand { argv }),
                                format!("Could not spawn: {err}"),
                                "Install systemd/logind or run on a real session host.".to_string(),
                            ],
                        );
                    }
                }
            }
            SessionActionPlan::LogindMethod { method, interactive } => {
                self.open_shell_status_window(
                    "Session Action",
                    [
                        format!("logind method: {method} (interactive={interactive})"),
                        format!("bus: {}", session_actions::LOGIND_BUS),
                        "D-Bus logind invoke is planned; use systemctl backend in this build."
                            .to_string(),
                    ],
                );
            }
            SessionActionPlan::ShellLock => {}
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
        let title = self.windows[index].window.title().to_string();
        // Try to get info for the selected file first; fall back to the folder window itself.
        let selected_name = self.selected_file_name(index);
        let lines = if let Some(ref sel) = selected_name {
            if let Some(ref folder_path) = self.windows[index].folder_path.clone() {
                folder_info_lines(sel, &folder_path.join(sel))
            } else {
                vec![
                    format!("Name: {sel}"),
                    "Kind: RetroShell window".to_string(),
                    "Location: Internal shell workspace".to_string(),
                ]
            }
        } else if let Some(ref path) = self.windows[index].folder_path.clone() {
            folder_info_lines(&title, path)
        } else {
            vec![
                format!("Name: {title}"),
                "Kind: RetroShell window".to_string(),
                "Location: Internal shell workspace".to_string(),
            ]
        };
        let info_title = selected_name.unwrap_or(title);
        self.open_message_window(format!("{info_title} Info"), lines);
    }

    /// Returns the label of the currently selected icon item in the active folder window, if any.
    fn selected_file_name(&self, window_index: usize) -> Option<String> {
        let shell_window = self.windows.get(window_index)?;
        let icon_view = shell_window
            .window
            .content
            .as_ref()
            .and_then(|content| content.as_any().downcast_ref::<IconView>())?;
        icon_view
            .items
            .iter()
            .find(|item| item.selected)
            .map(|item| item.label.clone())
    }

    fn handle_rename(&mut self) {
        let Some(id) = self.active_window_id() else {
            return;
        };
        let Some(index) = self.window_index(id) else {
            return;
        };
        let folder_path_opt = self.windows[index].folder_path.clone();
        let Some(folder_path) = folder_path_opt else {
            self.open_shell_status_window(
                "Rename",
                ["Select a file in a folder window first.".to_string()],
            );
            return;
        };
        let Some(old_name) = self.selected_file_name(index) else {
            self.open_shell_status_window(
                "Rename",
                ["No file selected. Click a file icon to select it, then choose Rename.".to_string()],
            );
            return;
        };

        // Derive a new name: append " copy" or increment a counter if "copy" already present.
        let new_name = derive_rename_suggestion(&old_name);
        let old_path = folder_path.join(&old_name);
        let new_path = folder_path.join(&new_name);

        match fs::rename(&old_path, &new_path) {
            Ok(()) => {
                tracing::info!("Renamed '{}' -> '{}'", old_path.display(), new_path.display());
                self.refresh_active_folder_window();
                self.open_shell_status_window(
                    "Rename",
                    [
                        format!("Renamed: {old_name}"),
                        format!("New name: {new_name}"),
                        "Note: a text-input prompt is not yet available; a suggested name was applied automatically.".to_string(),
                    ],
                );
            }
            Err(err) => {
                tracing::error!("Rename failed: {err}");
                self.open_shell_status_window(
                    "Rename Failed",
                    [
                        format!("Could not rename '{old_name}'."),
                        format!("Error: {err}"),
                    ],
                );
            }
        }
    }

    fn handle_move_to_trash(&mut self) {
        let Some(id) = self.active_window_id() else {
            return;
        };
        let Some(index) = self.window_index(id) else {
            return;
        };
        let folder_path_opt = self.windows[index].folder_path.clone();
        let Some(folder_path) = folder_path_opt else {
            self.open_shell_status_window(
                "Move to Trash",
                ["Select a file in a folder window first.".to_string()],
            );
            return;
        };
        let Some(file_name) = self.selected_file_name(index) else {
            self.open_shell_status_window(
                "Move to Trash",
                ["No file selected. Click a file icon to select it, then choose Move to Trash.".to_string()],
            );
            return;
        };

        let trash = trash_dir();
        if let Err(err) = fs::create_dir_all(&trash) {
            tracing::error!("Could not create trash directory: {err}");
            self.open_shell_status_window(
                "Move to Trash",
                [
                    format!("Could not create Trash directory: {err}"),
                ],
            );
            return;
        }

        let src = folder_path.join(&file_name);
        // Avoid overwriting existing trash items with the same name.
        let mut dest = trash.join(&file_name);
        let mut counter = 1u32;
        while dest.exists() {
            let stem = std::path::Path::new(&file_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&file_name);
            let ext = std::path::Path::new(&file_name)
                .extension()
                .and_then(|s| s.to_str());
            let candidate = if let Some(ext) = ext {
                format!("{stem} {counter}.{ext}")
            } else {
                format!("{stem} {counter}")
            };
            dest = trash.join(&candidate);
            counter += 1;
        }

        match fs::rename(&src, &dest) {
            Ok(()) => {
                tracing::info!("Moved '{}' to trash ('{}')", src.display(), dest.display());
                self.refresh_active_folder_window();
                self.open_shell_status_window(
                    "Move to Trash",
                    [format!("'{file_name}' moved to Trash.")],
                );
            }
            Err(err) => {
                tracing::error!("Move to trash failed: {err}");
                self.open_shell_status_window(
                    "Move to Trash Failed",
                    [
                        format!("Could not move '{file_name}' to Trash."),
                        format!("Error: {err}"),
                    ],
                );
            }
        }
    }

    fn open_about_window(&mut self) {
        for window in &self.windows {
            if window.window.title() == "About RetroShell" {
                self.focus_window(window.id);
                return;
            }
        }

        let rect = clamp_window_rect(
            Rect::new(
                self.content_bounds().x + 180.0,
                self.content_bounds().y + 120.0,
                400.0,
                320.0,
            ),
            self.content_bounds(),
        );

        // Gather live system info
        let host = session_manager::hostname();
        let uptime = format_uptime(session_manager::uptime_seconds());
        let (used_kb, total_kb) = session_manager::memory_usage();
        let mem_line = if total_kb > 0 {
            format!(
                "Memory: {} / {}",
                format_mem_gb(used_kb),
                format_mem_gb(total_kb)
            )
        } else {
            "Memory: Not available".to_string()
        };
        let battery_line = power::battery_info().summary_line();
        let network_line = network_manager::get_network_status().summary_line();

        let mut layout = Layout::vertical(12.0);
        layout.add(Box::new(Label::new("          RetroShell   ")));
        layout.add(Box::new(Label::new("----------------------------------------")));
        layout.add(Box::new(Label::new("    Classic Desktop Environment")));
        layout.add(Box::new(Label::new("    Built in Rust with wgpu")));
        layout.add(Box::new(Label::new("    Version 1.0.0 (Production)")));
        layout.add(Box::new(Label::new("----------------------------------------")));
        layout.add(Box::new(Label::new(format!("Hostname: {host}"))));
        layout.add(Box::new(Label::new(format!("Uptime: {uptime}"))));
        layout.add(Box::new(Label::new(mem_line)));
        layout.add(Box::new(Label::new(battery_line)));
        layout.add(Box::new(Label::new(network_line)));
        let _ = self.session_clients.reap();
        layout.add(Box::new(Label::new(format!(
            "External clients: {}",
            self.session_clients.len()
        ))));

        let mut btn_layout = Layout::horizontal(10.0);
        btn_layout.add(Box::new(Button::new("OK")));
        layout.add(Box::new(LayoutView::new(btn_layout)));

        let mut window = Window::new("About RetroShell");
        window.set_content(Box::new(LayoutView::new(layout)));
        window.set_rect(rect);

        let workspace = self.active_workspace();
        let id = self
            .window_manager
            .write()
            .create_window("com.retro.shell", window.title(), rect);
        self.window_manager.write().assign_workspace(id, workspace);
        self.windows.push(ShellWindow {
            id,
            window,
            folder_path: None,
            restore_rect: None,
            mode: ShellWindowMode::Normal,
            workspace,
        });
        self.focus_window(id);
        self.layout_window(id);
    }

    fn record_notification(
        &mut self,
        app_id: &str,
        title: &str,
        message: &str,
        priority: NotificationPriority,
    ) -> String {
        self.notification_center
            .write()
            .post(app_id, title, message, priority)
    }

    fn open_notification_center_window(&mut self) {
        let visible = self
            .notification_center
            .read()
            .visible()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut lines = vec!["Notification Center".to_string()];
        if visible.is_empty() {
            lines.push("No active notifications.".to_string());
        } else {
            for notification in visible {
                lines.push(format!(
                    "{} - {} ({:?})",
                    notification.id, notification.title, notification.priority
                ));
                lines.push(format!("  App: {}", notification.app_id));
                lines.push(format!("  {}", notification.message));
            }
        }
        self.open_message_window("Notification Center", lines);
    }

    fn clear_notifications(&mut self) {
        self.notification_center.write().dismiss_all();
        self.open_message_window(
            "Notification Center",
            ["All active notifications dismissed.".to_string()],
        );
    }

    fn open_force_quit_window(&mut self) {
        // Pull live compositor-owned toplevels into Force Quit list.
        self.refresh_foreign_toplevels_from_compositor();
        for window in &self.windows {
            if window.window.title() == "Force Quit" {
                self.focus_window(window.id);
                return;
            }
        }

        let rect = clamp_window_rect(
            Rect::new(
                self.content_bounds().x + 150.0,
                self.content_bounds().y + 100.0,
                400.0,
                300.0,
            ),
            self.content_bounds(),
        );

        let mut layout = Layout::vertical(10.0);
        layout.add(Box::new(Label::new(
            "Shell windows, session clients, and compositor foreign-toplevels:",
        )));

        let mut items = Vec::new();
        for w in &self.windows {
            if w.window.title() != "RetroShell Desktop" && w.window.title() != "Force Quit" {
                items.push(format!("window: {}", w.window.title()));
            }
        }
        let _ = self.session_clients.reap();
        for client in self.session_clients.clients() {
            items.push(format!(
                "client: {} (pid {})",
                client.binary_name, client.pid
            ));
        }
        items.extend(self.foreign_toplevels.force_quit_labels());

        let mut list_view = ListView::new();
        list_view.items = items;
        list_view.selected_index = if list_view.items.is_empty() {
            None
        } else {
            Some(0)
        };
        layout.add(Box::new(list_view));

        let mut btn_layout = Layout::horizontal(10.0);
        btn_layout.add(Box::new(Button::new("Cancel")));
        btn_layout.add(Box::new(Button::new("Force Quit")));
        layout.add(Box::new(LayoutView::new(btn_layout)));

        let mut window = Window::new("Force Quit");
        window.set_content(Box::new(LayoutView::new(layout)));
        window.set_rect(rect);

        let workspace = self.active_workspace();
        let id = self
            .window_manager
            .write()
            .create_window("com.retro.shell", window.title(), rect);
        self.window_manager.write().assign_workspace(id, workspace);
        self.windows.push(ShellWindow {
            id,
            window,
            folder_path: None,
            restore_rect: None,
            mode: ShellWindowMode::Normal,
            workspace,
        });
        self.focus_window(id);
        self.layout_window(id);
    }

    fn open_shell_status_window<S: Into<String>>(
        &mut self,
        title: S,
        lines: impl IntoIterator<Item = String>,
    ) {
        self.open_message_window(title, lines);
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
        if self.windows[index].mode == ShellWindowMode::Minimized {
            return;
        }
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
        if self.windows[index].mode == ShellWindowMode::Minimized {
            return;
        }
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

fn minimized_window_rect(bounds: Rect, slot: usize) -> Rect {
    let width = bounds.width.clamp(220.0, 360.0);
    let height = 24.0;
    let gap = 8.0;
    let x = bounds.x + gap + (slot as f32 * (width + gap)) % (bounds.width - width - gap).max(1.0);
    let y = bounds.y + bounds.height - height - gap;
    Rect::new(x, y, width, height)
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

fn settings_conf_path() -> PathBuf {
    std::env::var_os("RETROSHELL_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config/retroshell"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/retroshell"))
        .join("settings.conf")
}

fn read_settings_conf_text() -> String {
    fs::read_to_string(settings_conf_path()).unwrap_or_default()
}

/// Load DisplayConfig and apply arrangement env (live nested layout bridge).
fn apply_display_config_from_settings() {
    let path = settings_conf_path();
    // DisplayConfig::load expects a TOML map with a `[display]` table; also
    // accept flat arrange_mode= in settings.conf via env override path.
    let mut config = DisplayConfig::load(&path);
    // Flat settings.conf keys (theme manager style) for arrange_mode / scale.
    let conf = read_settings_conf_text();
    for line in conf.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("arrange_mode=") {
            config.arrange_mode = v.trim().to_string();
        }
        if let Some(v) = line.strip_prefix("scale_percent=") {
            if let Ok(n) = v.trim().parse::<u32>() {
                config.scale_percent = n;
            }
        }
    }
    match config.plan_arrangement(&[]) {
        Ok(plan) => {
            let applied = apply_display_plan_env(&plan);
            if !applied.is_empty() {
                tracing::info!(
                    mode = %config.arrange_mode,
                    steps = plan.steps.len(),
                    logical = %(format!("{}x{}", plan.logical_width, plan.logical_height)),
                    env = ?applied,
                    "display arrange plan applied (EmitLayoutEnv)"
                );
            }
        }
        Err(err) => {
            tracing::warn!(%err, "display arrangement plan failed");
        }
    }
}

fn get_lock_password() -> Option<String> {
    // First, check environment variable
    if let Ok(password) = std::env::var("RETROSHELL_LOCK_PASSWORD") {
        let password = password.trim();
        if !password.is_empty() {
            return Some(password.to_string());
        }
    }

    // Then, check config file
    if let Ok(contents) = fs::read_to_string(settings_conf_path()) {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("lock_password=") {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

/// Pure password check used by the lock screen (and unit tests).
/// Empty entered password never unlocks. Unlock only on exact match.
pub fn verify_lock_password(entered: &str, expected: &str) -> bool {
    !entered.is_empty() && entered == expected
}

fn shell_locale_prefs() -> LocalePrefs {
    LocalePrefs::parse_from_env_lang(std::env::var("LANG").ok().as_deref())
}

fn build_lock_screen_window() -> Window {
    let locale = shell_locale_prefs();
    let mut layout = Layout::vertical(24.0);
    layout.add(Box::new(Label::new("RetroShell")));
    layout.add(Box::new(Label::new(tr("lock.prompt", &locale.locale))));
    let mut window = Window::new(tr("menu.lock_screen", &locale.locale));
    window.set_content(Box::new(LayoutView::new(layout)));
    window
}

fn build_message_window(title: &str, lines: impl IntoIterator<Item = String>) -> Window {
    let mut layout = Layout::vertical(8.0);
    for line in lines {
        layout.add(Box::new(Label::new(line)));
    }

    let mut window = Window::new(title);
    window.set_content(Box::new(LayoutView::new(layout)));
    window
}

fn folder_info_lines(title: &str, path: &PathBuf) -> Vec<String> {
    let metadata = fs::metadata(path).ok();
    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);

    let item_count = if is_dir {
        fs::read_dir(path)
            .map(|entries| entries.filter_map(|entry| entry.ok()).count())
            .ok()
    } else {
        None
    };

    let kind = metadata
        .as_ref()
        .map(|m| {
            if m.is_dir() {
                "Folder"
            } else if m.is_file() {
                "Document"
            } else {
                "Filesystem item"
            }
        })
        .unwrap_or("Unavailable");

    let writable = metadata
        .as_ref()
        .map(|m| {
            if m.permissions().readonly() {
                "No"
            } else {
                "Yes"
            }
        })
        .unwrap_or("Unknown");

    let file_size = metadata
        .as_ref()
        .filter(|m| m.is_file())
        .map(|m| human_readable_size(m.len()));

    let mut lines = vec![
        format!("Name: {title}"),
        format!("Kind: {kind}"),
        format!("Location: {}", path.display()),
    ];

    if let Some(size) = file_size {
        lines.push(format!("Size: {size}"));
    }

    if let Some(count) = item_count {
        lines.push(format!("Items: {count}"));
    }

    lines.push(format!("Writable: {writable}"));
    lines
}

fn human_readable_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

fn derive_rename_suggestion(name: &str) -> String {
    let path = std::path::Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    let ext = path.extension().and_then(|s| s.to_str());

    // If the stem already ends with " copy N", increment N.
    // Otherwise append " copy".
    let new_stem = if let Some(idx) = stem.rfind(" copy") {
        let suffix = &stem[idx + 5..];
        if suffix.is_empty() {
            format!("{} copy 2", &stem[..idx])
        } else if let Ok(n) = suffix.trim().parse::<u32>() {
            format!("{} copy {}", &stem[..idx], n + 1)
        } else {
            format!("{stem} copy")
        }
    } else {
        format!("{stem} copy")
    };

    if let Some(ext) = ext {
        format!("{new_stem}.{ext}")
    } else {
        new_stem
    }
}

fn menu_action_label(menus: &[Menu], action_id: &str) -> Option<String> {
    for menu in menus {
        for item in &menu.items {
            if item.action_id == action_id {
                return Some(item.label.clone());
            }
            if matches!(item.kind, MenuItemKind::Submenu) {
                if let Some(submenu) = &item.submenu {
                    if let Some(label) = menu_action_label(std::slice::from_ref(submenu), action_id)
                    {
                        return Some(label);
                    }
                }
            }
        }
    }
    None
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

/// Format uptime seconds as a human-readable string like "2d 4h" or "1h 23m".
fn format_uptime(secs: u64) -> String {
    let minutes = secs / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else {
        format!("{}m", minutes)
    }
}

/// Format a kilobyte count as a GB string (e.g. "2.1 GB") or MB if small.
fn format_mem_gb(kb: u64) -> String {
    const MB: u64 = 1024;
    const GB: u64 = 1024 * MB;
    if kb >= GB {
        format!("{:.1} GB", kb as f64 / GB as f64)
    } else if kb >= MB {
        format!("{:.0} MB", kb as f64 / MB as f64)
    } else {
        format!("{} KB", kb)
    }
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

        // Always keep the lock screen widget sized to fill the desktop
        self.lock_screen_widget.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));
        let _ = self.lock_screen_widget.layout(LayoutConstraint::tight(Size::new(size.width, size.height)));

        if self.locked {
            return size;
        }

        self.menu_bar
            .set_rect(Rect::new(self.rect().x, self.rect().y, size.width, 24.0));
        let _ = self
            .menu_bar
            .layout(LayoutConstraint::tight(Size::new(size.width, 24.0)));

        self.desktop.set_rect(Rect::new(
            self.rect().x,
            self.rect().y + 24.0,
            size.width,
            (size.height - 24.0 - 64.0).max(0.0),
        ));
        let _ = self.desktop.layout(LayoutConstraint::tight(Size::new(
            size.width,
            (size.height - 24.0 - 64.0).max(0.0),
        )));

        self.dock_view.set_rect(Rect::new(
            self.rect().x,
            self.rect().y + size.height - 64.0,
            size.width,
            64.0,
        ));
        let _ = self.dock_view.layout(LayoutConstraint::tight(Size::new(size.width, 64.0)));

        self.layout_windows();

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        if self.locked {
            self.lock_screen_widget.draw(theme);
            return;
        }
        self.desktop.draw(theme);
        let active_workspace = self.active_workspace();
        // Draw non-active windows first
        for shell_window in self
            .windows
            .iter()
            .filter(|window| window.workspace == active_workspace)
            .rev()
            .skip(1)
        {
            shell_window.window.draw(theme);
        }
        // Draw active window last (on top)
        if let Some(active) = self
            .windows
            .iter()
            .rev()
            .find(|window| window.workspace == active_workspace)
        {
            active.window.draw(theme);
        }
        // When layer-shell chrome is bound, menu bar / dock are protocol surfaces —
        // skip kit dual-paint so chrome is not overdrawn in the shell canvas.
        if should_paint_kit_chrome(self.layer_shell_bound) {
            self.dock_view.draw(theme);
        }
        // Draw notification banners on top of windows and dock, below menu bar
        for popup in &self.notification_popup_windows {
            popup.draw(theme);
        }
        if should_paint_kit_chrome(self.layer_shell_bound) {
            self.menu_bar.draw(theme);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        // Any pointer/key activity resets idle timer (auto-lock policy).
        match event {
            Event::KeyDown { .. }
            | Event::KeyUp { .. }
            | Event::MouseDown { .. }
            | Event::MouseUp { .. }
            | Event::MouseMove { .. }
            | Event::Scroll { .. } => {
                self.last_input_at = std::time::Instant::now();
            }
            _ => {}
        }

        // When locked, handle password entry
        if self.locked {
            match event {
                Event::KeyDown { key: retro_kit::event::KeyCode::Escape, .. } => {
                    // Escape key: clear the field and error
                    self.lock_password_field.set_text("");
                    self.lock_error_message = None;
                    return EventResult::Handled;
                }
                Event::KeyDown { key: retro_kit::event::KeyCode::Enter, .. } => {
                    // Enter key: attempt to unlock (never unlock on empty / wrong / non-Enter keys)
                    let entered_password = self.lock_password_field.text().to_string();
                    if let Some(ref expected) = self.expected_lock_password {
                        if verify_lock_password(&entered_password, expected) {
                            self.session_manager.write().unlock();
                            self.locked = false;
                            self.lock_password_field.set_text("");
                            self.lock_error_message = None;
                            return EventResult::Handled;
                        } else {
                            self.lock_error_message = Some("Incorrect password".to_string());
                            self.lock_password_field.set_text("");
                            return EventResult::Handled;
                        }
                    }
                    return EventResult::Handled;
                }
                Event::Char { .. } | Event::KeyDown { key: retro_kit::event::KeyCode::Backspace, .. } => {
                    // Pass character/backspace events to the password field
                    self.lock_password_field.handle_event(event);
                    self.lock_error_message = None;
                    return EventResult::Handled;
                }
                _ => {
                    // Swallow all other events while locked
                    return EventResult::Handled;
                }
            }
        }

        let result = self.menu_bar.handle_event(event);
        if matches!(result, EventResult::Handled | EventResult::StopPropagation) {
            return result;
        }

        if let Event::KeyDown { key, modifiers } = event {
            // Unified keyboard-only nav policy (Tab / Shift+Tab / Escape / Enter / lock / workspaces).
            let key_name = match key {
                retro_kit::event::KeyCode::Tab => "tab",
                retro_kit::event::KeyCode::Escape => "escape",
                retro_kit::event::KeyCode::Enter => "enter",
                retro_kit::event::KeyCode::Space => "space",
                retro_kit::event::KeyCode::L => "l",
                retro_kit::event::KeyCode::Q => "q",
                retro_kit::event::KeyCode::LeftBracket => "[",
                retro_kit::event::KeyCode::RightBracket => "]",
                _ => "",
            };
            if !key_name.is_empty() {
                if let Some(intent) = keyboard_nav_intent(
                    key_name,
                    modifiers.shift,
                    modifiers.meta,
                    modifiers.control,
                    modifiers.alt,
                ) {
                    match intent {
                        KeyboardNavIntent::NextChromeRegion
                        | KeyboardNavIntent::PrevChromeRegion => {
                            self.chrome_focus = apply_chrome_nav(self.chrome_focus, intent);
                            // Best-effort AT-SPI Focus: in-process always; D-Bus if registered.
                            let emit = crate::atspi_bus::emit_chrome_focus(self.chrome_focus);
                            tracing::debug!(
                                focus = ?self.chrome_focus,
                                ?intent,
                                dbus = emit.dbus_emitted,
                                "chrome focus"
                            );
                            return EventResult::Handled;
                        }
                        KeyboardNavIntent::Dismiss => {
                            // Close topmost dismissable transient window.
                            if let Some(id) = self
                                .windows
                                .iter()
                                .rev()
                                .find(|w| is_dismissable_window_title(w.window.title()))
                                .map(|w| w.id)
                            {
                                self.close_window(id);
                                return EventResult::Handled;
                            }
                        }
                        KeyboardNavIntent::Activate => {
                            // When chrome has focus, drain primary invoke from a11y_actions.
                            let plan = primary_invoke_for_chrome(self.chrome_focus);
                            if plan.valid {
                                tracing::debug!(
                                    invoke_id = %plan.invoke_id,
                                    focus = ?self.chrome_focus,
                                    "chrome Activate → a11y invoke"
                                );
                                self.dispatch_a11y_invoke(&plan.invoke_id);
                                return EventResult::Handled;
                            }
                            // Enter on Force Quit list is handled by window widgets below.
                        }
                        KeyboardNavIntent::NextWindow => {
                            // fall through to Meta+Tab block
                        }
                        KeyboardNavIntent::LockScreen => {
                            self.handle_session_action(session_actions::SessionAction::Lock);
                            return EventResult::Handled;
                        }
                        KeyboardNavIntent::LogOut => {
                            self.handle_session_action(session_actions::SessionAction::Logout);
                            return EventResult::Handled;
                        }
                        KeyboardNavIntent::NextWorkspace => {
                            self.switch_to_next_workspace();
                            return EventResult::Handled;
                        }
                        KeyboardNavIntent::PrevWorkspace => {
                            self.switch_to_previous_workspace();
                            return EventResult::Handled;
                        }
                    }
                }
            }
            // Cmd+Tab: cycle focus through non-minimized windows on the active workspace
            if modifiers.meta && *key == retro_kit::event::KeyCode::Tab {
                self.focus_next_window();
                return EventResult::Handled;
            }

            // Cmd+W: close the front window on the active workspace
            if modifiers.meta && *key == retro_kit::event::KeyCode::W {
                if let Some(id) = self.active_window_id() {
                    self.close_window(id);
                    return EventResult::Handled;
                }
            }

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
                let dock_rect = self.dock_view.rect();
                if dock_rect.contains(*point) {
                    let item_size = 48.0;
                    let padding = 8.0;
                    let item_spacing = 6.0;
                    let items_count = self.dock_view.items.len();
                    let total_width = items_count as f32 * (item_size + item_spacing) - item_spacing + padding * 2.0;
                    let dock_x = dock_rect.x + (dock_rect.width - total_width) * 0.5;
                    let click_offset_x = point.x - dock_x - padding;
                    if click_offset_x >= 0.0 {
                        let item_idx = (click_offset_x / (item_size + item_spacing)) as usize;
                        if item_idx < items_count {
                            let app_id = self.dock.write().launch_app(item_idx);
                            if let Some(app_id) = app_id {
                                self.launch_external_app(&app_id);
                            }
                        }
                    }
                    return EventResult::Handled;
                }

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
                    self.toggle_window_minimized(window_id);
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

                if self.windows[index].window.title() == "Force Quit" {
                    // Collect click outcome without holding a borrow across mut methods.
                    enum FqClick {
                        Cancel,
                        Confirm { entry: Option<String> },
                    }
                    let fq_click = (|| {
                        let content = self.windows[index].window.content.as_deref()?;
                        let layout_view = content.as_any().downcast_ref::<LayoutView>()?;
                        let Layout::Vertical { children, .. } = &layout_view.layout else {
                            return None;
                        };
                        if children.len() < 3 {
                            return None;
                        }
                        let list_view = children[1].as_any().downcast_ref::<ListView>();
                        let btn_layout_widget =
                            children[2].as_any().downcast_ref::<LayoutView>()?;
                        let Layout::Horizontal {
                            children: btn_children,
                            ..
                        } = &btn_layout_widget.layout
                        else {
                            return None;
                        };
                        if btn_children.len() < 2 {
                            return None;
                        }
                        let cancel_btn = btn_children[0].as_any().downcast_ref::<Button>()?;
                        let force_quit_btn = btn_children[1].as_any().downcast_ref::<Button>()?;
                        if cancel_btn.rect().contains(*point) {
                            return Some(FqClick::Cancel);
                        }
                        if force_quit_btn.rect().contains(*point) {
                            let entry = list_view.and_then(|list| {
                                list.selected_index
                                    .and_then(|i| list.items.get(i).cloned())
                            });
                            return Some(FqClick::Confirm { entry });
                        }
                        None
                    })();

                    match fq_click {
                        Some(FqClick::Cancel) => {
                            self.close_window(window_id);
                            return EventResult::Handled;
                        }
                        Some(FqClick::Confirm { entry }) => {
                            if let Some(entry) = entry {
                                let _ = self.apply_force_quit_entry(&entry);
                            }
                            self.close_window(window_id);
                            return EventResult::Handled;
                        }
                        None => {}
                    }
                }

                if self.windows[index].window.title() == "Workspace" {
                    if let Some(content) = self.windows[index].window.content.as_deref() {
                        if let Some(layout_view) = content.as_any().downcast_ref::<LayoutView>() {
                            if let Layout::Vertical { children, .. } = &layout_view.layout {
                                if children.len() >= 2 {
                                    if let Some(grid) = children[1].as_any().downcast_ref::<WorkspaceGridView>() {
                                        let grid_rect = grid.rect();
                                        if grid_rect.contains(*point) {
                                            let local_x = point.x - grid_rect.x;
                                            let local_y = point.y - grid_rect.y;
                                            let col = if local_x < grid_rect.width * 0.5 { 0 } else { 1 };
                                            let row = if local_y < grid_rect.height * 0.5 { 0 } else { 1 };
                                            let clicked_idx = row * 2 + col;
                                            self.handle_menu_action(&format!("workspace.switch.{}", clicked_idx));
                                            self.close_window(window_id);
                                            return EventResult::Handled;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if self.windows[index].window.title() == "About RetroShell" {
                    if let Some(content) = self.windows[index].window.content.as_deref() {
                        if let Some(layout_view) = content.as_any().downcast_ref::<LayoutView>() {
                            if let Layout::Vertical { children, .. } = &layout_view.layout {
                                if children.len() >= 11 {
                                    if let Some(btn_layout_widget) = children[10].as_any().downcast_ref::<LayoutView>() {
                                        if let Layout::Horizontal { children: btn_children, .. } = &btn_layout_widget.layout {
                                            if !btn_children.is_empty() {
                                                if let Some(btn) = btn_children[0].as_any().downcast_ref::<Button>() {
                                                    if btn.rect().contains(*point) {
                                                        self.close_window(window_id);
                                                        return EventResult::Handled;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
        // Sync lock state from SessionManager
        if self.session_manager.read().state == session_manager::SessionState::Locked && !self.locked {
            self.locked = true;
        }

        // AT-SPI DoAction queue → real shell handlers (lock / log out / chrome.*).
        self.drain_a11y_pending_actions();

        // Idle auto-lock: pure policy → shell lock when password configured.
        // Merge process-wide portal Inhibit cookies (Idle/Suspend flags) without
        // permanently mutating shell-local idle_inhibit (so UnInhibit clears).
        if !self.locked {
            let idle_secs = self.last_input_at.elapsed().as_secs();
            let mut inhibit = self.idle_inhibit.clone();
            for reason in portal_extra::active_idle_inhibit_state().reasons() {
                inhibit.add(*reason);
            }
            let phase = idle_phase(
                &self.idle_config,
                idle_secs,
                self.locked,
                &inhibit,
            );
            if recommended_action(phase, self.locked) == IdleRecommendedAction::Lock {
                if self.expected_lock_password.is_some() {
                    tracing::info!(idle_secs, "idle policy: auto-lock");
                    self.handle_session_action(session_actions::SessionAction::Lock);
                }
            }
        }

        // Update lock screen widget with current password field state (i18n strings).
        if self.locked {
            let locale = shell_locale_prefs();
            let mut layout = Layout::vertical(12.0);
            layout.add(Box::new(Label::new("RetroShell")));
            layout.add(Box::new(Label::new("")));
            layout.add(Box::new(Label::new(tr("lock.prompt", &locale.locale))));

            // Add a copy of the password field for display
            let mut field = TextField::new()
                .with_placeholder(tr("lock.prompt", &locale.locale));
            field.is_password = true;
            field.set_text(self.lock_password_field.text());
            layout.add(Box::new(field));

            if let Some(ref error) = self.lock_error_message {
                let msg = if error.contains("Incorrect") {
                    tr("lock.error", &locale.locale)
                } else {
                    error.clone()
                };
                layout.add(Box::new(Label::new(msg)));
            }

            self.lock_screen_widget.set_content(Box::new(LayoutView::new(layout)));
        }

        self.menu_bar.menus = self.menu_server.read().menus.clone();

        if let Some(action) = self.menu_bar.last_action.take() {
            tracing::info!("Menu action: {action}");
            self.handle_menu_action(&action);
        }

        // Sync DockView items from shared Dock
        let dock_read = self.dock.read();
        let mut dock_view_items = Vec::new();
        for item in &dock_read.items {
            dock_view_items.push(retro_kit::dock_view::DockViewItem {
                label: item.label.clone(),
                icon: item.icon.clone().unwrap_or_default(),
                is_focused: item.state == crate::dock::AppState::Focused,
                is_running: item.state == crate::dock::AppState::Running || item.state == crate::dock::AppState::Focused,
            });
        }
        self.dock_view.items = dock_view_items;

        // Expire old notifications (older than 5 seconds)
        {
            self.notification_center
                .write()
                .clear_expired(std::time::Duration::from_secs(5));
        }

        // Rebuild notification popup windows from currently visible notifications
        let notifications: Vec<(String, String)> = self
            .notification_center
            .read()
            .visible()
            .into_iter()
            .map(|n| (n.title.clone(), n.message.clone()))
            .collect();

        self.notification_popup_windows.clear();
        let right_margin = 12.0;
        let popup_w = 280.0;
        let popup_h = 80.0;
        let menu_bar_h = 24.0;
        let gap = 8.0;
        let desktop_width = self.rect().width;

        for (i, (title, message)) in notifications.iter().enumerate() {
            let x = desktop_width - popup_w - right_margin;
            let y = menu_bar_h + gap + i as f32 * (popup_h + gap);
            let rect = Rect::new(x, y, popup_w, popup_h);

            let mut layout = Layout::vertical(4.0);
            layout.add(Box::new(Label::new(format!("[!] {title}"))));
            layout.add(Box::new(Label::new(message.clone())));

            let mut popup = Window::new(title.as_str());
            popup.set_content(Box::new(LayoutView::new(layout)));
            popup.set_rect(rect);
            let _ = popup.layout(LayoutConstraint::tight(Size::new(popup_w, popup_h)));

            self.notification_popup_windows.push(popup);
        }
    }

    fn children(&self) -> Vec<&dyn Widget> {
        if self.locked {
            return vec![&self.lock_screen_widget as &dyn Widget];
        }
        let capacity = self.windows.len() + 3 + self.notification_popup_windows.len();
        let mut children: Vec<&dyn Widget> = Vec::with_capacity(capacity);
        children.push(&self.desktop);
        let active_workspace = self.active_workspace();
        for shell_window in &self.windows {
            if shell_window.workspace == active_workspace {
                children.push(&shell_window.window);
            }
        }
        children.push(&self.dock_view);
        // Notification banners are drawn above dock but below menu bar
        for popup in &self.notification_popup_windows {
            children.push(popup as &dyn Widget);
        }
        children.push(&self.menu_bar);
        children
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        if self.locked {
            return vec![&mut self.lock_screen_widget as &mut dyn Widget];
        }
        let capacity = self.windows.len() + 3 + self.notification_popup_windows.len();
        let mut children: Vec<&mut dyn Widget> = Vec::with_capacity(capacity);
        children.push(&mut self.desktop);
        let active_workspace = self.workspace_manager.read().active;
        for shell_window in &mut self.windows {
            if shell_window.workspace == active_workspace {
                children.push(&mut shell_window.window);
            }
        }
        children.push(&mut self.dock_view);
        // Notification banners are drawn above dock but below menu bar
        for popup in &mut self.notification_popup_windows {
            children.push(popup as &mut dyn Widget);
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
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static MENU_MANIFEST_ENV_LOCK: Mutex<()> = Mutex::new(());
    static LOCK_PASSWORD_ENV_LOCK: Mutex<()> = Mutex::new(());

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
        let notification_center = Arc::new(RwLock::new(NotificationCenter::new()));
        let workspace_manager = Arc::new(RwLock::new(WorkspaceManager::new()));
        let dock = Arc::new(RwLock::new(Dock::new()));
        let session_manager = Arc::new(RwLock::new(SessionManager::new()));
        let mut desktop = ShellDesktop::new(
            menu_server,
            launch_services,
            window_manager.clone(),
            notification_center,
            workspace_manager,
            dock,
            session_manager,
        );
        desktop.layout(LayoutConstraint::tight(Size::new(960.0, 640.0)));
        (desktop, window_manager)
    }

    fn assert_rect_eq(actual: Rect, expected: Rect) {
        assert_eq!(actual.x, expected.x);
        assert_eq!(actual.y, expected.y);
        assert_eq!(actual.width, expected.width);
        assert_eq!(actual.height, expected.height);
    }

    fn rect_eq(left: Rect, right: Rect) -> bool {
        left.x == right.x
            && left.y == right.y
            && left.width == right.width
            && left.height == right.height
    }

    fn message_window_lines(window: &ShellWindow) -> Vec<String> {
        let layout_view = window
            .window
            .content
            .as_ref()
            .and_then(|content| content.as_any().downcast_ref::<LayoutView>())
            .expect("message window uses layout view");
        let Layout::Vertical { children, .. } = &layout_view.layout else {
            panic!("message window uses vertical layout");
        };
        children
            .iter()
            .filter_map(|child| {
                child
                    .as_any()
                    .downcast_ref::<Label>()
                    .map(|l| l.text.clone())
            })
            .collect()
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
    fn shell_global_menu_switches_to_launched_sdk_app() {
        let _guard = MENU_MANIFEST_ENV_LOCK.lock().unwrap();
        std::env::remove_var("RETROSHELL_MENU_MANIFEST_DIR");
        let (mut desktop, _) = test_desktop();

        desktop.activate_app_menu("com.retro.textedit");

        let titles = desktop
            .menu_bar
            .menus
            .iter()
            .map(|menu| menu.title.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            desktop.menu_server.read().active_app.as_deref(),
            Some("com.retro.textedit")
        );
        assert!(titles.contains(&"TextEdit"));
        assert!(titles.contains(&"File"));
        assert!(titles.contains(&"Edit"));
    }

    #[test]
    fn shell_global_menu_uses_loaded_sdk_manifest_for_active_app() {
        let _guard = MENU_MANIFEST_ENV_LOCK.lock().unwrap();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("retroshell_menu_manifest_shell_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("RETROSHELL_MENU_MANIFEST_DIR", &dir);

        let mut textedit_file = retro_kit::menu::Menu::new("File");
        textedit_file
            .add_action("Save As...")
            .with_action("com.retro.textedit.file.save_as");
        let manifest = retro_sdk::MenuManifest {
            app_name: "TextEdit".to_string(),
            bundle_id: "com.retro.textedit".to_string(),
            menus: vec![textedit_file],
            updated_at_millis: 1,
        };
        fs::write(
            dir.join("com_retro_textedit.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let (mut desktop, _) = test_desktop();
        desktop.activate_app_menu("com.retro.textedit");

        assert_eq!(
            desktop.menu_server.read().active_app.as_deref(),
            Some("com.retro.textedit")
        );
        assert_eq!(
            desktop
                .menu_bar
                .menus
                .iter()
                .find(|menu| menu.title == "File")
                .unwrap()
                .items[0]
                .action_id,
            "com.retro.textedit.file.save_as"
        );

        let _ = fs::remove_dir_all(&dir);
        std::env::remove_var("RETROSHELL_MENU_MANIFEST_DIR");
    }

    #[test]
    fn loaded_sdk_menu_action_opens_visible_dispatch_status() {
        let _guard = MENU_MANIFEST_ENV_LOCK.lock().unwrap();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("retroshell_menu_action_shell_{unique}"));
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("RETROSHELL_MENU_MANIFEST_DIR", &dir);

        let mut textedit_file = retro_kit::menu::Menu::new("File");
        textedit_file
            .add_action("Save As...")
            .with_action("com.retro.textedit.file.save_as");
        let manifest = retro_sdk::MenuManifest {
            app_name: "TextEdit".to_string(),
            bundle_id: "com.retro.textedit".to_string(),
            menus: vec![textedit_file],
            updated_at_millis: 1,
        };
        fs::write(
            dir.join("com_retro_textedit.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let (mut desktop, _) = test_desktop();
        desktop.activate_app_menu("com.retro.textedit");
        desktop.handle_menu_action("com.retro.textedit.file.save_as");

        let active = desktop.windows.last().expect("dispatch status window");
        assert_eq!(active.window.title(), "Application Menu Action");
        let lines = message_window_lines(active);
        assert!(lines.contains(&"Application: com.retro.textedit".to_string()));
        assert!(lines.contains(&"Action: Save As...".to_string()));
        assert!(lines.contains(&"Identifier: com.retro.textedit.file.save_as".to_string()));

        let _ = fs::remove_dir_all(&dir);
        std::env::remove_var("RETROSHELL_MENU_MANIFEST_DIR");
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
    fn workspace_switch_hides_windows_from_other_workspaces() {
        let (mut desktop, window_manager) = test_desktop();
        let first_id = desktop.windows[0].id;

        assert_eq!(desktop.active_workspace(), 0);
        assert_eq!(desktop.children().len(), 4);

        desktop.handle_menu_action("workspace.switch.1");
        assert_eq!(desktop.active_workspace(), 1);
        assert_ne!(desktop.active_window_id(), Some(first_id));
        assert_eq!(
            window_manager.read().active_window,
            Some(desktop.windows.last().unwrap().id)
        );
        assert_eq!(desktop.windows.last().unwrap().window.title(), "Workspace");
        assert_eq!(desktop.windows.last().unwrap().workspace, 1);
        assert!(desktop
            .children()
            .iter()
            .any(|child| rect_eq(child.rect(), desktop.windows.last().unwrap().window.rect())));

        desktop.handle_menu_action("workspace.switch.0");
        assert_eq!(desktop.active_workspace(), 0);
        assert!(desktop.windows.iter().any(|window| window.id == first_id));
        assert!(desktop
            .children()
            .iter()
            .any(|child| rect_eq(child.rect(), desktop.windows[0].window.rect())));
    }

    #[test]
    fn workspace_shortcut_actions_cycle_active_workspace() {
        let (mut desktop, _) = test_desktop();

        desktop.handle_menu_action("workspace.next");
        assert_eq!(desktop.active_workspace(), 1);

        desktop.handle_menu_action("workspace.previous");
        assert_eq!(desktop.active_workspace(), 0);
    }

    #[test]
    fn about_menu_opens_real_message_window() {
        let (mut desktop, window_manager) = test_desktop();

        desktop.handle_menu_action("shell.about");

        let active = desktop.windows.last().expect("about window");
        assert_eq!(active.window.title(), "About RetroShell");
        assert_eq!(active.folder_path, None);
        assert_eq!(window_manager.read().active_window, Some(active.id));
        let lines = message_window_lines(active);
        assert!(lines[0].contains("RetroShell"));
        assert!(lines.iter().any(|line| line.contains("Classic Desktop Environment")));
    }

    #[test]
    fn notification_center_lists_and_clears_active_notifications() {
        let (mut desktop, _) = test_desktop();

        let id = desktop.record_notification(
            "com.retro.textedit",
            "Document Saved",
            "note.txt was written to disk.",
            NotificationPriority::Normal,
        );

        assert_eq!(id, "notif-0");
        assert_eq!(desktop.notification_center.read().visible().len(), 1);

        desktop.handle_menu_action("shell.notification_center");
        let active = desktop.windows.last().expect("notification center window");
        assert_eq!(active.window.title(), "Notification Center");
        let lines = message_window_lines(active);
        assert!(lines
            .iter()
            .any(|line| line.contains("notif-0 - Document Saved")));
        assert!(lines
            .iter()
            .any(|line| line.contains("App: com.retro.textedit")));

        desktop.handle_menu_action("shell.clear_notifications");
        assert!(desktop.notification_center.read().visible().is_empty());
        let active = desktop.windows.last().expect("clear confirmation");
        assert_eq!(active.window.title(), "Notification Center");
        assert!(message_window_lines(active)
            .iter()
            .any(|line| line.contains("dismissed")));
    }

    #[test]
    fn get_info_menu_opens_folder_metadata_window() {
        let root = temp_shell_root();
        fs::write(root.join("note.txt"), "hello").unwrap();
        let (mut desktop, window_manager) = test_desktop();
        desktop.open_folder_window("Root", root.clone());

        desktop.handle_menu_action("finder.get_info");

        let active = desktop.windows.last().expect("info window");
        assert_eq!(active.window.title(), "Root Info");
        assert_eq!(active.folder_path, None);
        assert_eq!(window_manager.read().active_window, Some(active.id));
        let lines = message_window_lines(active);
        assert!(lines.contains(&"Name: Root".to_string()));
        assert!(lines.contains(&"Kind: Folder".to_string()));
        assert!(lines.contains(&format!("Location: {}", root.display())));
        assert!(lines.contains(&"Items: 1".to_string()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn force_quit_menu_opens_running_window_list() {
        let (mut desktop, window_manager) = test_desktop();

        desktop.handle_menu_action("shell.force_quit");

        let active = desktop.windows.last().expect("force quit window");
        assert_eq!(active.window.title(), "Force Quit");
        assert_eq!(active.folder_path, None);
        assert_eq!(window_manager.read().active_window, Some(active.id));
        
        let layout_view = active.window.content.as_deref()
            .and_then(|c| c.as_any().downcast_ref::<LayoutView>())
            .expect("uses layout view");
        if let Layout::Vertical { children, .. } = &layout_view.layout {
            let label = children[0].as_any().downcast_ref::<Label>().expect("label");
            assert_eq!(
                label.text,
                "Shell windows, session clients, and compositor foreign-toplevels:"
            );
            let list = children[1].as_any().downcast_ref::<ListView>().expect("list");
            assert!(list
                .items
                .iter()
                .any(|item| item == "window: Retro HD" || item.contains("Retro HD")));
        } else {
            panic!("not vertical layout");
        }
    }

    #[test]
    fn force_quit_apply_closes_listed_shell_window() {
        let (mut desktop, _) = test_desktop();
        // test_desktop opens a Finder-style "Retro HD" window among others.
        let before = desktop.windows.len();
        assert!(
            desktop
                .windows
                .iter()
                .any(|w| w.window.title() == "Retro HD"),
            "precondition: Retro HD window present"
        );

        // Drive the same path Force Quit button uses (shipped apply helper).
        assert!(desktop.apply_force_quit_entry("window: Retro HD"));
        assert!(
            !desktop
                .windows
                .iter()
                .any(|w| w.window.title() == "Retro HD"),
            "Retro HD must be closed after force quit"
        );
        assert!(desktop.windows.len() < before);
    }

    #[test]
    fn force_quit_apply_kills_registered_client_pid() {
        let (mut desktop, _) = test_desktop();
        desktop.session_clients.register(session_clients::ExternalClient {
            bundle_id: "com.retro.finder".into(),
            binary_name: "finder".into(),
            pid: 424_242,
            child: None,
            launched_at_unix: 1,
        });
        assert_eq!(desktop.session_clients.len(), 1);
        assert!(desktop.apply_force_quit_entry("client: finder (pid 424242)"));
        assert_eq!(desktop.session_clients.len(), 0);
    }

    #[test]
    fn help_search_menu_opens_status_window() {
        let (mut desktop, _) = test_desktop();

        desktop.handle_menu_action("shell.help_search");

        let active = desktop.windows.last().expect("help window");
        assert_eq!(active.window.title(), "Help");
        let lines = message_window_lines(active);
        assert!(lines.iter().any(|line| line.contains("not indexed yet")));
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
    fn minimize_box_collapses_and_restores_managed_window() {
        let (mut desktop, window_manager) = test_desktop();
        let id = desktop.windows[0].id;
        let original = desktop.windows[0].window.rect();
        let point = Point::new(original.x + 28.0, original.y + 12.0);

        let result = desktop.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point,
            modifiers: retro_kit::event::Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(desktop.windows[0].mode, ShellWindowMode::Minimized);
        assert_rect_eq(desktop.windows[0].restore_rect.unwrap(), original);
        assert_eq!(
            window_manager.read().windows[&id].state,
            window_manager::WindowState::Minimized
        );
        assert_eq!(desktop.windows[0].window.rect().height, 24.0);

        let minimized = desktop.windows[0].window.rect();
        let restore_point = Point::new(minimized.x + 28.0, minimized.y + 12.0);
        let result = desktop.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: restore_point,
            modifiers: retro_kit::event::Modifiers::NONE,
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(desktop.windows[0].mode, ShellWindowMode::Normal);
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
        for menu in &server.menus {
            for item in &menu.items {
                if matches!(item.kind, retro_kit::menu::MenuItemKind::Action) {
                    assert!(
                        !item.action_id.is_empty(),
                        "{} > {} has no action id",
                        menu.title,
                        item.label
                    );
                }
            }
        }

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

    #[test]
    fn lock_accepts_correct_password() {
        // Drive the shipped verify_lock_password used by Enter-to-unlock.
        assert!(verify_lock_password("test_password", "test_password"));
        assert!(verify_lock_password("s3cret!", "s3cret!"));
    }

    #[test]
    fn lock_rejects_wrong_password() {
        assert!(!verify_lock_password("wrong", "correct_password"));
        assert!(!verify_lock_password("", "correct_password"));
        assert!(!verify_lock_password("correct_password", ""));
        assert!(!verify_lock_password("Correct_password", "correct_password"));
    }

    #[test]
    fn lock_password_env_is_source_for_expected_secret() {
        let _lock = LOCK_PASSWORD_ENV_LOCK.lock().unwrap();
        std::env::remove_var("RETROSHELL_LOCK_PASSWORD");
        std::env::set_var("RETROSHELL_LOCK_PASSWORD", "env_secret");
        let expected = get_lock_password().expect("env secret");
        assert!(verify_lock_password("env_secret", &expected));
        assert!(!verify_lock_password("other", &expected));
        std::env::remove_var("RETROSHELL_LOCK_PASSWORD");
    }

    #[test]
    fn a11y_dispatch_shell_session_actions_are_live() {
        let (mut desktop, _) = test_desktop();

        desktop.dispatch_a11y_invoke("shell.notification_center");
        assert!(desktop
            .windows
            .iter()
            .any(|w| w.window.title() == "Notification Center"));

        desktop.dispatch_a11y_invoke("shell.force_quit");
        assert!(desktop
            .windows
            .iter()
            .any(|w| w.window.title() == "Force Quit"));

        desktop.expected_lock_password = Some("secret".into());
        desktop.dispatch_a11y_invoke("shell.lock");
        assert!(desktop.locked);
    }

    #[test]
    fn a11y_dispatch_chrome_window_close_and_activate_next() {
        let (mut desktop, window_manager) = test_desktop();
        // Start with the default Finder; open a second window so activate can cycle.
        desktop.handle_menu_action("shell.new_finder_window");
        assert!(desktop.windows.len() >= 2);

        let first = desktop.windows[0].id;
        let second = desktop.windows[1].id;
        desktop.focus_window(first);
        assert_eq!(desktop.active_window_id(), Some(first));

        desktop.dispatch_a11y_invoke("chrome.window.activate");
        let after_activate = desktop.active_window_id();
        assert_eq!(after_activate, Some(second));
        assert_eq!(window_manager.read().active_window, Some(second));

        let before_close = desktop.windows.len();
        desktop.dispatch_a11y_invoke("chrome.window.close");
        assert_eq!(desktop.windows.len(), before_close - 1);
        assert!(!desktop.windows.iter().any(|w| w.id == second));
    }

    #[test]
    fn a11y_dispatch_workspace_next_previous() {
        let (mut desktop, _) = test_desktop();
        assert_eq!(desktop.active_workspace(), 0);

        desktop.dispatch_a11y_invoke("workspace.next");
        assert_eq!(desktop.active_workspace(), 1);

        desktop.dispatch_a11y_invoke("workspace.previous");
        assert_eq!(desktop.active_workspace(), 0);
    }

    #[test]
    fn a11y_dispatch_chrome_menu_activate_opens_system_menu() {
        let (mut desktop, _) = test_desktop();
        assert!(desktop.menu_bar.open_menu.is_none());
        assert!(
            desktop
                .menu_bar
                .menus
                .iter()
                .any(|m| m.title == "Retro"),
            "precondition: system Retro menu present"
        );

        desktop.dispatch_a11y_invoke("chrome.menu.activate");
        let retro_idx = desktop
            .menu_bar
            .menus
            .iter()
            .position(|m| m.title == "Retro")
            .expect("Retro menu");
        assert_eq!(desktop.menu_bar.open_menu, Some(retro_idx));

        desktop.menu_bar.close();
        assert!(desktop.menu_bar.open_menu.is_none());

        desktop.dispatch_a11y_invoke("chrome.menu.system");
        assert_eq!(desktop.menu_bar.open_menu, Some(retro_idx));
    }

    #[test]
    fn a11y_chrome_activate_enter_dispatches_primary_invoke() {
        let (mut desktop, _) = test_desktop();
        desktop.chrome_focus = ChromeFocusTarget::Windows;
        desktop.handle_menu_action("shell.new_finder_window");
        let first = desktop.windows[0].id;
        desktop.focus_window(first);

        let result = desktop.handle_event(&Event::KeyDown {
            key: retro_kit::event::KeyCode::Enter,
            modifiers: Modifiers::NONE,
        });
        assert!(matches!(result, EventResult::Handled));
        // Windows primary invoke is chrome.window.activate → focus next.
        assert_ne!(desktop.active_window_id(), Some(first));
    }

    #[test]
    fn idle_config_parses_from_settings_conf_keys() {
        let cfg = IdleConfig::parse_from_conf(
            "# comment\nidle_warn_secs=15\nidle_lock_secs=45\nidle_suspend_secs=90\nlock_on_suspend=false\n",
        );
        assert_eq!(cfg.warn_after_secs, 15);
        assert_eq!(cfg.lock_after_secs, 45);
        assert_eq!(cfg.suspend_after_secs, 90);
        assert!(!cfg.lock_on_suspend);
        // Empty conf keeps defaults (used when settings.conf is absent).
        assert_eq!(IdleConfig::parse_from_conf(""), IdleConfig::default());
    }

    #[test]
    fn portal_idle_inhibit_merges_into_phase() {
        clear_inhibit_store_for_tests();
        let cfg = IdleConfig {
            warn_after_secs: 1,
            lock_after_secs: 2,
            suspend_after_secs: 0,
            lock_on_suspend: true,
            inhibited: false,
        };
        let base = IdleInhibitState::new();
        assert_eq!(
            idle_phase(&cfg, 10, false, &base),
            IdlePhase::ShouldLock
        );

        let _ = handle_inhibit_and_register(&PortalInhibitRequest {
            app_id: "player".into(),
            window: String::new(),
            flags: InhibitFlag::Idle as u32,
            reason: "playing".into(),
        });
        let mut merged = base.clone();
        for reason in active_idle_inhibit_state().reasons() {
            merged.add(*reason);
        }
        assert!(merged.is_inhibited());
        assert_eq!(idle_phase(&cfg, 10, false, &merged), IdlePhase::Active);
        clear_inhibit_store_for_tests();
    }
}
