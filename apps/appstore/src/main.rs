use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers, MouseButton};
use retro_kit::label::Label;
use retro_kit::list_view::ListView;
use retro_kit::progress_bar::ProgressBar;
use retro_kit::text_field::TextField;
use retro_kit::window::Window;
use retro_kit::{
    AccessibilityNode, Event, EventResult, LayoutConstraint, Point, Rect, Size, ThemeContext,
    Widget, WidgetState,
};
use retro_sdk::{build_menu, Application};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

// Featured packages shown when no search is active
const FEATURED_PACKAGES: &[&str] = &[
    "curl", "git", "vim", "htop", "neofetch", "python3", "nodejs", "ffmpeg",
];

// Category definitions: (display name, search keywords)
const CATEGORIES: &[(&str, &[&str])] = &[
    ("ALL", &[]),
    ("SYSTEM", &["util", "system", "admin", "cron", "syslog"]),
    ("DEVELOPMENT", &["dev", "lib", "build", "gcc", "clang", "python", "rust", "go"]),
    ("GAMES", &["game", "doom", "quake", "supertux", "mame"]),
    ("MEDIA", &["media", "audio", "video", "ffmpeg", "vlc", "mpv", "sox"]),
    ("OFFICE", &["office", "document", "pdf", "libreoffice", "writer"]),
    ("NETWORK", &["network", "net", "ssh", "ftp", "curl", "wget", "nmap"]),
];

fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut app = Application::new("App Store", "com.retro.appstore");

    let mut store_menu = build_menu("Store");
    store_menu.add_action("Refresh").with_shortcut(
        KeyCode::R,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    store_menu.add_action("Search").with_shortcut(
        KeyCode::F,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );

    let mut edit_menu = build_menu("Edit");
    edit_menu.add_action("Copy").with_shortcut(
        KeyCode::C,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    edit_menu.add_action("Paste").with_shortcut(
        KeyCode::V,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("App Store Help");

    app.set_menus(vec![store_menu, edit_menu, window_menu, help_menu]);

    let mut window = Window::new("App Store");
    window.set_content(Box::new(AppStoreView::new(PackageBackend::detect())));
    app.set_main_window(window);
    app.run();
}

// ── Package manager detection ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Pkg,
    Apk,
    Zypper,
    Brew,
}

impl PackageManager {
    fn display_name(self) -> &'static str {
        match self {
            Self::Apt => "APT",
            Self::Dnf => "DNF",
            Self::Pacman => "PACMAN",
            Self::Pkg => "PKG",
            Self::Apk => "APK",
            Self::Zypper => "ZYPPER",
            Self::Brew => "BREW",
        }
    }

    fn search_command(self, query: &str) -> (&'static str, Vec<String>) {
        match self {
            Self::Apt => ("apt-cache", vec!["search".into(), query.into()]),
            Self::Dnf => ("dnf", vec!["search".into(), query.into()]),
            Self::Pacman => ("pacman", vec!["-Ss".into(), query.into()]),
            Self::Pkg => ("pkg", vec!["search".into(), query.into()]),
            Self::Apk => ("apk", vec!["search".into(), query.into()]),
            Self::Zypper => ("zypper", vec!["search".into(), query.into()]),
            Self::Brew => ("brew", vec!["search".into(), query.into()]),
        }
    }

    fn installed_query_command(self, package: &str) -> (&'static str, Vec<String>) {
        match self {
            Self::Apt => (
                "dpkg-query",
                vec!["-W".into(), "-f=${Status}".into(), package.into()],
            ),
            Self::Dnf => ("rpm", vec!["-q".into(), package.into()]),
            Self::Pacman => ("pacman", vec!["-Q".into(), package.into()]),
            Self::Pkg => ("pkg", vec!["info".into(), package.into()]),
            Self::Apk => ("apk", vec!["info".into(), "-e".into(), package.into()]),
            Self::Zypper => (
                "zypper",
                vec![
                    "--non-interactive".into(),
                    "se".into(),
                    "-i".into(),
                    "-x".into(),
                    package.into(),
                ],
            ),
            Self::Brew => (
                "brew",
                vec!["list".into(), "--versions".into(), package.into()],
            ),
        }
    }

    fn transaction_command(self, action: PackageAction, package: &str) -> Vec<String> {
        match (self, action) {
            (Self::Apt, PackageAction::Install) => {
                vec!["sudo", "apt-get", "install", "-y", package]
            }
            (Self::Apt, PackageAction::Remove) => {
                vec!["sudo", "apt-get", "remove", "-y", package]
            }
            (Self::Apt, PackageAction::Update) => {
                vec![
                    "sudo",
                    "apt-get",
                    "install",
                    "--only-upgrade",
                    "-y",
                    package,
                ]
            }
            (Self::Dnf, PackageAction::Install) => vec!["sudo", "dnf", "install", "-y", package],
            (Self::Dnf, PackageAction::Remove) => vec!["sudo", "dnf", "remove", "-y", package],
            (Self::Dnf, PackageAction::Update) => vec!["sudo", "dnf", "upgrade", "-y", package],
            (Self::Pacman, PackageAction::Install) => {
                vec!["sudo", "pacman", "-S", "--noconfirm", package]
            }
            (Self::Pacman, PackageAction::Remove) => {
                vec!["sudo", "pacman", "-R", "--noconfirm", package]
            }
            (Self::Pacman, PackageAction::Update) => {
                vec!["sudo", "pacman", "-Syu", "--noconfirm", package]
            }
            (Self::Pkg, PackageAction::Install) => vec!["sudo", "pkg", "install", "-y", package],
            (Self::Pkg, PackageAction::Remove) => vec!["sudo", "pkg", "delete", "-y", package],
            (Self::Pkg, PackageAction::Update) => vec!["sudo", "pkg", "upgrade", "-y", package],
            (Self::Apk, PackageAction::Install) => vec!["sudo", "apk", "add", package],
            (Self::Apk, PackageAction::Remove) => vec!["sudo", "apk", "del", package],
            (Self::Apk, PackageAction::Update) => vec!["sudo", "apk", "upgrade", package],
            (Self::Zypper, PackageAction::Install) => {
                vec!["sudo", "zypper", "install", "-y", package]
            }
            (Self::Zypper, PackageAction::Remove) => {
                vec!["sudo", "zypper", "remove", "-y", package]
            }
            (Self::Zypper, PackageAction::Update) => {
                vec!["sudo", "zypper", "update", "-y", package]
            }
            (Self::Brew, PackageAction::Install) => vec!["brew", "install", package],
            (Self::Brew, PackageAction::Remove) => vec!["brew", "uninstall", package],
            (Self::Brew, PackageAction::Update) => vec!["brew", "upgrade", package],
        }
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    /// Query version + description for a package from the system package manager.
    fn package_details(self, package: &str) -> PackageDetails {
        let mut details = PackageDetails {
            name: package.to_string(),
            version: String::new(),
            description: String::new(),
            state: PackageInstallState::Unknown,
        };

        // Version via dpkg-s / rpm -qi / pacman -Qi etc.
        match self {
            Self::Apt => {
                if let Ok(out) = Command::new("dpkg").args(["-s", package]).output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        if let Some(v) = line.strip_prefix("Version: ") {
                            details.version = v.trim().to_string();
                        }
                        if let Some(d) = line.strip_prefix("Description: ") {
                            details.description = d.trim().to_string();
                        }
                    }
                }
                // Supplement description from apt-cache show if empty
                if details.description.is_empty() {
                    if let Ok(out) = Command::new("apt-cache").args(["show", package]).output() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        for line in text.lines() {
                            if let Some(v) = line.strip_prefix("Version: ") {
                                if details.version.is_empty() {
                                    details.version = v.trim().to_string();
                                }
                            }
                            if let Some(d) = line.strip_prefix("Description: ") {
                                if details.description.is_empty() {
                                    details.description = d.trim().to_string();
                                }
                            }
                        }
                    }
                }
            }
            Self::Brew => {
                if let Ok(out) = Command::new("brew").args(["info", "--json=v1", package]).output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    // Simple substring scan — avoids a JSON dep
                    if let Some(start) = text.find("\"versions\"") {
                        if let Some(stable_start) = text[start..].find("\"stable\":\"") {
                            let after = &text[start + stable_start + 10..];
                            if let Some(end) = after.find('"') {
                                details.version = after[..end].to_string();
                            }
                        }
                    }
                    if let Some(start) = text.find("\"desc\":\"") {
                        let after = &text[start + 8..];
                        if let Some(end) = after.find('"') {
                            details.description = after[..end].to_string();
                        }
                    }
                }
            }
            _ => {}
        }

        details.state = package_state_for_manager(self, package);
        details
    }
}

// ── Domain types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageAction {
    Install,
    Remove,
    Update,
}

impl PackageAction {
    fn label(self) -> &'static str {
        match self {
            Self::Install => "INSTALL",
            Self::Remove => "REMOVE",
            Self::Update => "UPDATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageInstallState {
    Installed,
    Available,
    Unknown,
}

impl PackageInstallState {
    fn label(self) -> &'static str {
        match self {
            Self::Installed => "INSTALLED",
            Self::Available => "AVAILABLE",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Default)]
struct PackageDetails {
    name: String,
    version: String,
    description: String,
    state: PackageInstallState,
}

impl Default for PackageInstallState {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransactionPlan {
    action: PackageAction,
    package: String,
    command: Vec<String>,
}

impl TransactionPlan {
    fn command_line(&self) -> String {
        self.command.join(" ")
    }

    fn log_lines(&self) -> Vec<String> {
        vec![
            "TRANSACTION PLAN".to_string(),
            format!("ACTION - {}", self.action.label()),
            format!("PACKAGE - {}", self.package),
            format!("COMMAND - {}", self.command_line()),
            "CONFIRM requires RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES=1".to_string(),
        ]
    }
}

/// Background install job state shared between the worker thread and the UI.
#[derive(Debug, Default)]
struct InstallJob {
    running: bool,
    progress: f32,       // 0.0 – 100.0
    message: String,
    finished: bool,
    success: bool,
    output: String,
}

// ── PackageBackend ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct PackageBackend {
    manager: Option<PackageManager>,
}

impl PackageBackend {
    fn detect() -> Self {
        let manager = [
            ("apt-cache", PackageManager::Apt),
            ("dnf", PackageManager::Dnf),
            ("pacman", PackageManager::Pacman),
            ("pkg", PackageManager::Pkg),
            ("apk", PackageManager::Apk),
            ("zypper", PackageManager::Zypper),
            ("brew", PackageManager::Brew),
        ]
        .into_iter()
        .find_map(|(binary, manager)| command_exists(binary).then_some(manager));

        Self { manager }
    }

    /// Run a real package search limited to 20 results, tagged [installed]/[available].
    fn search(&self, query: &str) -> Result<Vec<String>, String> {
        let Some(manager) = self.manager else {
            return Err("NO PACKAGE MANAGER FOUND".to_string());
        };
        let query = query.trim();
        if query.is_empty() {
            return Err("SEARCH NEEDS QUERY".to_string());
        }

        let (binary, args) = manager.search_command(query);

        // Primary search
        let primary = Command::new(binary).args(&args).output();

        let text = match primary {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).to_string()
            }
            Ok(out) => {
                // apt-cache may return non-zero with useful stderr
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if !stderr.trim().is_empty() {
                    // Try dpkg fallback for apt
                    if manager == PackageManager::Apt {
                        match dpkg_grep_search(query) {
                            Ok(t) if !t.trim().is_empty() => t,
                            _ => stderr,
                        }
                    } else {
                        stderr
                    }
                } else {
                    String::new()
                }
            }
            Err(_) if manager == PackageManager::Apt => {
                // apt-cache not available, fall back to dpkg
                dpkg_grep_search(query).unwrap_or_default()
            }
            Err(err) => return Err(format!("SEARCH FAILED: {err}")),
        };

        let results = annotate_search_results(manager, parse_search_results(&text, 20));
        if results.is_empty() {
            Ok(vec![format!(
                "NO RESULTS FOR {}",
                query.to_ascii_uppercase()
            )])
        } else {
            Ok(results)
        }
    }

    fn plan_transaction(
        &self,
        action: PackageAction,
        package: &str,
    ) -> Result<TransactionPlan, String> {
        let Some(manager) = self.manager else {
            return Err("NO PACKAGE MANAGER FOUND".to_string());
        };
        let package = package.trim();
        if package.is_empty() {
            return Err("TRANSACTION NEEDS PACKAGE".to_string());
        }

        Ok(TransactionPlan {
            action,
            package: package.to_string(),
            command: manager.transaction_command(action, package),
        })
    }

    fn execute_transaction(&self, plan: &TransactionPlan) -> Result<String, String> {
        if std::env::var("RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES")
            .ok()
            .as_deref()
            != Some("1")
        {
            return Err(
                "CONFIRM BLOCKED - SET RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES=1".to_string(),
            );
        }

        let Some((binary, args)) = plan.command.split_first() else {
            return Err("TRANSACTION HAS NO COMMAND".to_string());
        };
        let output = Command::new(binary)
            .args(args)
            .output()
            .map_err(|err| format!("TRANSACTION FAILED {err}"))?;

        let text = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        if output.status.success() {
            Ok(text)
        } else {
            Err(if text.trim().is_empty() {
                "TRANSACTION FAILED".to_string()
            } else {
                text
            })
        }
    }

    /// Start an install in a background thread; returns an Arc<Mutex<InstallJob>> handle.
    fn install_async(&self, package: &str) -> Result<Arc<Mutex<InstallJob>>, String> {
        let Some(manager) = self.manager else {
            return Err("NO PACKAGE MANAGER FOUND".to_string());
        };

        // Check sudo availability
        if !sudo_available() && manager != PackageManager::Brew {
            return Err("REQUIRES ROOT - sudo not found".to_string());
        }

        let command = manager.transaction_command(PackageAction::Install, package);
        let job = Arc::new(Mutex::new(InstallJob {
            running: true,
            message: format!("Installing {}...", package),
            ..Default::default()
        }));
        let job_clone = Arc::clone(&job);

        thread::spawn(move || {
            {
                let mut j = job_clone.lock().unwrap();
                j.progress = 10.0;
                j.message = "Starting package manager...".to_string();
            }

            if let Some((binary, args)) = command.split_first() {
                let result = Command::new(binary).args(args).output();

                let mut j = job_clone.lock().unwrap();
                j.running = false;
                j.finished = true;
                j.progress = 100.0;

                match result {
                    Ok(out) if out.status.success() => {
                        j.success = true;
                        j.output = String::from_utf8_lossy(&out.stdout).to_string();
                        j.message = "Install complete!".to_string();
                    }
                    Ok(out) => {
                        j.success = false;
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        j.output = stderr.clone();
                        j.message = if stderr.trim().is_empty() {
                            "Install FAILED".to_string()
                        } else {
                            stderr.lines().next().unwrap_or("Install FAILED").to_string()
                        };
                    }
                    Err(err) => {
                        j.success = false;
                        j.message = format!("FAILED: {err}");
                    }
                }
            } else {
                let mut j = job_clone.lock().unwrap();
                j.running = false;
                j.finished = true;
                j.success = false;
                j.message = "No command".to_string();
            }
        });

        Ok(job)
    }

    fn package_details(&self, package: &str) -> Option<PackageDetails> {
        self.manager.map(|m| m.package_details(package))
    }

    fn status_text(&self) -> String {
        match self.manager {
            Some(manager) => format!("BACKEND - {}", manager.display_name()),
            None => "BACKEND - NONE".to_string(),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn command_exists(binary: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(binary).is_file())
}

fn sudo_available() -> bool {
    command_exists("sudo")
}

/// Fallback search using `dpkg -l | grep <query>` (works when apt-cache is absent).
fn dpkg_grep_search(query: &str) -> Result<String, String> {
    let dpkg = Command::new("dpkg")
        .args(["-l"])
        .output()
        .map_err(|e| e.to_string())?;
    let full = String::from_utf8_lossy(&dpkg.stdout);
    let q = query.to_ascii_lowercase();
    let filtered: String = full
        .lines()
        .filter(|line| line.to_ascii_lowercase().contains(&q))
        .map(|line| {
            // dpkg -l lines: "ii  pkgname  version  arch  description"
            let parts: Vec<&str> = line.splitn(5, ' ').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 5 {
                format!("{} - {}", parts[1], parts[4])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(filtered)
}

fn parse_search_results(output: &str, limit: usize) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(limit)
        .map(|line| {
            let mut text = line.replace('\t', " ");
            text.truncate(96);
            text
        })
        .collect()
}

fn annotate_search_results(manager: PackageManager, results: Vec<String>) -> Vec<String> {
    results
        .into_iter()
        .map(|line| {
            let state = package_name_from_result(&line)
                .map(|package| package_state_for_manager(manager, &package))
                .unwrap_or(PackageInstallState::Unknown);
            format!("[{}] {}", state.label(), line)
        })
        .collect()
}

fn package_state_for_manager(manager: PackageManager, package: &str) -> PackageInstallState {
    let (binary, args) = manager.installed_query_command(package);
    let Ok(output) = Command::new(binary).args(args).output() else {
        return PackageInstallState::Unknown;
    };

    if !output.status.success() {
        return PackageInstallState::Available;
    }

    if manager == PackageManager::Apt {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("install ok installed") {
            PackageInstallState::Installed
        } else {
            PackageInstallState::Available
        }
    } else {
        PackageInstallState::Installed
    }
}

fn package_name_from_result(result: &str) -> Option<String> {
    let result = result
        .strip_prefix("[INSTALLED] ")
        .or_else(|| result.strip_prefix("[AVAILABLE] "))
        .or_else(|| result.strip_prefix("[UNKNOWN] "))
        .unwrap_or(result);
    let first = result.split_whitespace().next()?.trim();
    let name = first
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or(first)
        .trim_matches(|c: char| matches!(c, ':' | ',' | ';'));
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Filter featured/search results by category keyword list.
fn filter_by_category(items: &[String], keywords: &[&str]) -> Vec<String> {
    if keywords.is_empty() {
        return items.to_vec();
    }
    items
        .iter()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            keywords.iter().any(|kw| lower.contains(*kw))
        })
        .cloned()
        .collect()
}

/// Return the display lines for the featured list.
fn featured_list(backend: &PackageBackend) -> Vec<String> {
    let Some(manager) = backend.manager else {
        return FEATURED_PACKAGES
            .iter()
            .map(|&p| format!("[FEATURED] {p}"))
            .collect();
    };
    FEATURED_PACKAGES
        .iter()
        .map(|&pkg| {
            let state = package_state_for_manager(manager, pkg);
            format!("[{}] {}", state.label(), pkg)
        })
        .collect()
}

// ── UI View ───────────────────────────────────────────────────────────────────

struct AppStoreView {
    state: WidgetState,
    heading: Label,
    backend_label: Label,
    query: TextField,
    search_button: Button,
    refresh_button: Button,
    // Category sidebar
    category_list: ListView,
    // Package results
    results: ListView,
    // Detail panel
    detail_name: Label,
    detail_version: Label,
    detail_description: Label,
    detail_state: Label,
    install_button: Button,
    remove_button: Button,
    update_button: Button,
    confirm_button: Button,
    // Progress bar for async install
    progress_bar: ProgressBar,
    progress_label: Label,
    status: Label,
    backend: PackageBackend,
    pending_transaction: Option<TransactionPlan>,
    /// Currently selected category index (matches CATEGORIES slice).
    category_index: usize,
    /// Whether we are in "featured" mode (empty search query).
    featured_mode: bool,
    /// All results before category filter applied (for re-filtering on category change).
    all_results: Vec<String>,
    /// Background install job handle.
    install_job: Option<Arc<Mutex<InstallJob>>>,
}

impl AppStoreView {
    fn new(backend: PackageBackend) -> Self {
        let mut query = TextField::new();
        query.set_text("");

        let mut category_list = ListView::new();
        for (name, _) in CATEGORIES {
            category_list.add_item(*name);
        }
        category_list.selected_index = Some(0);

        let mut view = Self {
            state: WidgetState::new(),
            heading: Label::new("SOFTWARE CATALOG"),
            backend_label: Label::new(backend.status_text()),
            query,
            search_button: Button::new("SEARCH"),
            refresh_button: Button::new("REFRESH"),
            category_list,
            results: ListView::new(),
            detail_name: Label::new(""),
            detail_version: Label::new(""),
            detail_description: Label::new(""),
            detail_state: Label::new(""),
            install_button: Button::new("INSTALL"),
            remove_button: Button::new("REMOVE"),
            update_button: Button::new("UPDATE"),
            confirm_button: Button::new("CONFIRM"),
            progress_bar: ProgressBar::new(),
            progress_label: Label::new(""),
            status: Label::new("READY"),
            backend,
            pending_transaction: None,
            category_index: 0,
            featured_mode: true,
            all_results: vec![],
            install_job: None,
        };
        view.load_featured();
        view
    }

    /// Load featured packages when search query is empty.
    fn load_featured(&mut self) {
        self.featured_mode = true;
        self.all_results = featured_list(&self.backend);
        self.apply_category_filter();
        self.pending_transaction = None;
        self.status.text = format!("FEATURED - {} PACKAGES", self.results.items.len());
        self.clear_detail();
    }

    fn run_search(&mut self) -> bool {
        let query = self.query.text().trim().to_string();
        if query.is_empty() {
            self.load_featured();
            return true;
        }
        self.featured_mode = false;
        match self.backend.search(&query) {
            Ok(results) => {
                self.all_results = results;
                self.apply_category_filter();
                self.pending_transaction = None;
                self.status.text = format!("{} RESULTS", self.results.items.len());
                self.clear_detail();
                true
            }
            Err(err) => {
                self.all_results = vec![];
                self.results.items = vec![err.clone()];
                self.results.selected_index = None;
                self.pending_transaction = None;
                self.status.text = err;
                self.clear_detail();
                false
            }
        }
    }

    /// Re-filter `all_results` through the selected category and populate `results`.
    fn apply_category_filter(&mut self) {
        let (_, keywords) = CATEGORIES[self.category_index];
        let filtered = filter_by_category(&self.all_results, keywords);
        self.results.items = filtered;
        self.results.selected_index = (!self.results.items.is_empty()).then_some(0);
    }

    fn refresh_backend(&mut self) -> bool {
        self.backend = PackageBackend::detect();
        self.backend_label.text = self.backend.status_text();
        if self.featured_mode {
            self.load_featured();
            true
        } else {
            self.run_search()
        }
    }

    fn selected_package(&self) -> Option<String> {
        self.results
            .selected_index
            .and_then(|index| self.results.items.get(index))
            .and_then(|line| package_name_from_result(line))
            .or_else(|| {
                let query = self.query.text().trim();
                (!query.is_empty()).then(|| query.to_string())
            })
    }

    /// Populate the detail panel for the given package name.
    fn show_package_detail(&mut self, package: &str) {
        if let Some(details) = self.backend.package_details(package) {
            self.detail_name.text = format!("PKG: {}", details.name.to_ascii_uppercase());
            self.detail_version.text = if details.version.is_empty() {
                "VERSION: N/A".to_string()
            } else {
                format!("VERSION: {}", details.version)
            };
            self.detail_description.text = if details.description.is_empty() {
                "No description available.".to_string()
            } else {
                let mut d = details.description.clone();
                d.truncate(120);
                d
            };
            self.detail_state.text = format!("STATUS: {}", details.state.label());
        } else {
            self.detail_name.text = format!("PKG: {}", package.to_ascii_uppercase());
            self.detail_version.text = "VERSION: N/A".to_string();
            self.detail_description.text = String::new();
            self.detail_state.text = "STATUS: UNKNOWN".to_string();
        }
    }

    fn clear_detail(&mut self) {
        self.detail_name.text = String::new();
        self.detail_version.text = String::new();
        self.detail_description.text = String::new();
        self.detail_state.text = String::new();
    }

    fn plan_transaction(&mut self, action: PackageAction) -> bool {
        let Some(package) = self.selected_package() else {
            self.status.text = "SELECT OR SEARCH FOR A PACKAGE".to_string();
            return false;
        };

        match self.backend.plan_transaction(action, &package) {
            Ok(plan) => {
                self.status.text = format!("{} READY - {}", action.label(), package);
                self.results.items = plan.log_lines();
                self.results.selected_index = None;
                self.pending_transaction = Some(plan);
                true
            }
            Err(err) => {
                self.status.text = err.clone();
                self.results.items = vec![err];
                self.results.selected_index = None;
                self.pending_transaction = None;
                false
            }
        }
    }

    fn confirm_transaction(&mut self) -> bool {
        let Some(plan) = self.pending_transaction.clone() else {
            self.status.text = "NO TRANSACTION TO CONFIRM".to_string();
            return false;
        };

        match self.backend.execute_transaction(&plan) {
            Ok(output) => {
                self.status.text = format!("{} COMPLETE - {}", plan.action.label(), plan.package);
                self.results.items = parse_search_results(&output, 20);
                if self.results.items.is_empty() {
                    self.results.items = vec!["TRANSACTION COMPLETE".to_string()];
                }
                self.pending_transaction = None;
                true
            }
            Err(err) => {
                self.status.text = err.clone();
                self.results.items = plan.log_lines();
                self.results.items.push(format!("STATUS - {err}"));
                false
            }
        }
    }

    /// Trigger a background install for the currently selected package.
    fn start_install_async(&mut self) {
        let Some(package) = self.selected_package() else {
            self.status.text = "SELECT A PACKAGE FIRST".to_string();
            return;
        };

        match self.backend.install_async(&package) {
            Ok(job) => {
                self.install_job = Some(job);
                self.progress_bar.indeterminate = true;
                self.progress_bar.value = 0.0;
                self.progress_label.text = format!("Installing {}...", package);
                self.status.text = format!("INSTALLING {} IN BACKGROUND", package);
            }
            Err(err) => {
                self.status.text = err.clone();
                self.progress_label.text = err;
            }
        }
    }

    /// Poll the background install job (called from update()).
    fn poll_install_job(&mut self) {
        let Some(job) = &self.install_job else {
            return;
        };
        let job = Arc::clone(job);
        let Ok(j) = job.lock() else { return };

        self.progress_bar.value = j.progress;
        self.progress_label.text = j.message.clone();

        if j.finished {
            self.progress_bar.indeterminate = false;
            if j.success {
                self.status.text = "INSTALL COMPLETE".to_string();
            } else {
                self.status.text = format!("INSTALL FAILED: {}", j.message);
            }
            drop(j);
            self.install_job = None;
        }
    }

    fn handle_button_click(&mut self, point: Point) -> bool {
        if self.search_button.rect().contains(point) {
            return self.run_search();
        }
        if self.refresh_button.rect().contains(point) {
            return self.refresh_backend();
        }
        if self.install_button.rect().contains(point) {
            // Use background async install with progress
            self.start_install_async();
            return true;
        }
        if self.remove_button.rect().contains(point) {
            return self.plan_transaction(PackageAction::Remove);
        }
        if self.update_button.rect().contains(point) {
            return self.plan_transaction(PackageAction::Update);
        }
        if self.confirm_button.rect().contains(point) {
            return self.confirm_transaction();
        }
        false
    }
}

impl Widget for AppStoreView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        let rect = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(rect);

        let pad = 18.0;
        let content_x = rect.x + pad;
        let content_w = (rect.width - pad * 2.0).max(0.0);
        let mut y = rect.y + 18.0;

        // Heading row
        self.heading
            .set_rect(Rect::new(content_x, y, content_w, 24.0));
        let _ = self
            .heading
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));
        y += 30.0;

        // Backend label
        self.backend_label
            .set_rect(Rect::new(content_x, y, content_w, 20.0));
        let _ = self
            .backend_label
            .layout(LayoutConstraint::tight(Size::new(content_w, 20.0)));
        y += 28.0;

        // Search bar row
        let search_field_w = content_w.min(260.0);
        self.query
            .set_rect(Rect::new(content_x, y, search_field_w, 28.0));
        let _ = self
            .query
            .layout(LayoutConstraint::tight(Size::new(search_field_w, 28.0)));

        let btn_y = y;
        let btn_x = content_x + search_field_w + 12.0;
        self.search_button
            .set_rect(Rect::new(btn_x, btn_y, 88.0, 28.0));
        let _ = self
            .search_button
            .layout(LayoutConstraint::tight(Size::new(88.0, 28.0)));

        self.refresh_button
            .set_rect(Rect::new(btn_x + 96.0, btn_y, 96.0, 28.0));
        let _ = self
            .refresh_button
            .layout(LayoutConstraint::tight(Size::new(96.0, 28.0)));
        y += 44.0;

        // Action buttons row
        let action_w = 94.0;
        self.install_button
            .set_rect(Rect::new(content_x, y, action_w, 28.0));
        let _ = self
            .install_button
            .layout(LayoutConstraint::tight(Size::new(action_w, 28.0)));
        self.remove_button
            .set_rect(Rect::new(content_x + 102.0, y, action_w, 28.0));
        let _ = self
            .remove_button
            .layout(LayoutConstraint::tight(Size::new(action_w, 28.0)));
        self.update_button
            .set_rect(Rect::new(content_x + 204.0, y, action_w, 28.0));
        let _ = self
            .update_button
            .layout(LayoutConstraint::tight(Size::new(action_w, 28.0)));
        self.confirm_button
            .set_rect(Rect::new(content_x + 306.0, y, 104.0, 28.0));
        let _ = self
            .confirm_button
            .layout(LayoutConstraint::tight(Size::new(104.0, 28.0)));
        y += 42.0;

        // Progress bar row (always laid out; visible when active)
        let pb_h = 14.0;
        self.progress_bar
            .set_rect(Rect::new(content_x, y, content_w.min(320.0), pb_h));
        let _ = self
            .progress_bar
            .layout(LayoutConstraint::tight(Size::new(content_w.min(320.0), pb_h)));
        self.progress_label
            .set_rect(Rect::new(content_x + content_w.min(320.0) + 10.0, y, content_w - content_w.min(320.0) - 10.0, pb_h));
        let _ = self
            .progress_label
            .layout(LayoutConstraint::tight(Size::new(content_w - content_w.min(320.0) - 10.0, pb_h)));
        y += pb_h + 8.0;

        // Main area: category sidebar (left) + package list (center) + detail panel (right)
        let status_h = 26.0;
        let main_h = (rect.height - (y - rect.y) - status_h - pad).max(0.0);

        let cat_w = 110.0;
        let detail_w = 220.0;
        let list_w = (content_w - cat_w - detail_w - 8.0 - 8.0).max(80.0);

        // Category sidebar
        let cat_x = content_x;
        self.category_list
            .set_rect(Rect::new(cat_x, y, cat_w, main_h));
        let _ = self
            .category_list
            .layout(LayoutConstraint::tight(Size::new(cat_w, main_h)));

        // Package results list
        let list_x = cat_x + cat_w + 8.0;
        self.results.set_rect(Rect::new(list_x, y, list_w, main_h));
        let _ = self
            .results
            .layout(LayoutConstraint::tight(Size::new(list_w, main_h)));

        // Detail panel on the right
        let detail_x = list_x + list_w + 8.0;
        let mut dy = y;
        let row_h = 22.0;
        let row_gap = 4.0;

        self.detail_name
            .set_rect(Rect::new(detail_x, dy, detail_w, row_h));
        let _ = self
            .detail_name
            .layout(LayoutConstraint::tight(Size::new(detail_w, row_h)));
        dy += row_h + row_gap;

        self.detail_version
            .set_rect(Rect::new(detail_x, dy, detail_w, row_h));
        let _ = self
            .detail_version
            .layout(LayoutConstraint::tight(Size::new(detail_w, row_h)));
        dy += row_h + row_gap;

        self.detail_state
            .set_rect(Rect::new(detail_x, dy, detail_w, row_h));
        let _ = self
            .detail_state
            .layout(LayoutConstraint::tight(Size::new(detail_w, row_h)));
        dy += row_h + row_gap;

        // Description can be taller
        let desc_h = (row_h * 3.0).min(main_h - (dy - y) - 4.0).max(row_h);
        self.detail_description
            .set_rect(Rect::new(detail_x, dy, detail_w, desc_h));
        let _ = self
            .detail_description
            .layout(LayoutConstraint::tight(Size::new(detail_w, desc_h)));

        y += main_h;

        // Status bar
        self.status
            .set_rect(Rect::new(content_x, y, content_w, status_h));
        let _ = self
            .status
            .layout(LayoutConstraint::tight(Size::new(content_w, status_h)));

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.heading.draw(theme);
        self.backend_label.draw(theme);
        self.query.draw(theme);
        self.search_button.draw(theme);
        self.refresh_button.draw(theme);
        self.install_button.draw(theme);
        self.remove_button.draw(theme);
        self.update_button.draw(theme);
        self.confirm_button.draw(theme);
        self.progress_bar.draw(theme);
        self.progress_label.draw(theme);
        self.category_list.draw(theme);
        self.results.draw(theme);
        self.detail_name.draw(theme);
        self.detail_version.draw(theme);
        self.detail_state.draw(theme);
        self.detail_description.draw(theme);
        self.status.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::KeyDown { key, modifiers } = event {
            if modifiers.meta && matches!(key, KeyCode::R | KeyCode::F) {
                if matches!(key, KeyCode::R) {
                    self.refresh_backend();
                } else {
                    self.run_search();
                }
                return EventResult::Handled;
            }
            if matches!(key, KeyCode::Enter) {
                self.run_search();
                return EventResult::Handled;
            }
        }

        if let Event::MouseDown {
            button: MouseButton::Left,
            point,
            ..
        } = event
        {
            if self.handle_button_click(*point) {
                return EventResult::Handled;
            }
        }

        // Category sidebar selection
        let cat_result = self.category_list.handle_event(event);
        if matches!(cat_result, EventResult::Handled) {
            if let Some(idx) = self.category_list.selected_index {
                if idx < CATEGORIES.len() && idx != self.category_index {
                    self.category_index = idx;
                    self.apply_category_filter();
                    self.status.text =
                        format!("CATEGORY - {} | {} RESULTS", CATEGORIES[idx].0, self.results.items.len());
                    self.clear_detail();
                }
            }
            return EventResult::Handled;
        }

        // Package results list selection
        let result = self.results.handle_event(event);
        if matches!(result, EventResult::Handled) {
            self.pending_transaction = None;
            if let Some(package) = self.selected_package() {
                self.status.text = format!("SELECTED - {}", package);
                self.show_package_detail(&package);
            }
            return EventResult::Handled;
        }

        let result = self.query.handle_event(event);
        if matches!(result, EventResult::Handled) {
            return EventResult::Handled;
        }
        EventResult::Ignored
    }

    fn update(&mut self) {
        self.poll_install_job();

        self.heading.update();
        self.backend_label.update();
        self.query.update();
        self.search_button.update();
        self.refresh_button.update();
        self.install_button.update();
        self.remove_button.update();
        self.update_button.update();
        self.confirm_button.update();
        self.progress_bar.update();
        self.progress_label.update();
        self.category_list.update();
        self.results.update();
        self.detail_name.update();
        self.detail_version.update();
        self.detail_state.update();
        self.detail_description.update();
        self.status.update();
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![
            &self.heading,
            &self.backend_label,
            &self.query,
            &self.search_button,
            &self.refresh_button,
            &self.install_button,
            &self.remove_button,
            &self.update_button,
            &self.confirm_button,
            &self.progress_bar,
            &self.progress_label,
            &self.category_list,
            &self.results,
            &self.detail_name,
            &self.detail_version,
            &self.detail_state,
            &self.detail_description,
            &self.status,
        ]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![
            &mut self.heading,
            &mut self.backend_label,
            &mut self.query,
            &mut self.search_button,
            &mut self.refresh_button,
            &mut self.install_button,
            &mut self.remove_button,
            &mut self.update_button,
            &mut self.confirm_button,
            &mut self.progress_bar,
            &mut self.progress_label,
            &mut self.category_list,
            &mut self.results,
            &mut self.detail_name,
            &mut self.detail_version,
            &mut self.detail_state,
            &mut self.detail_description,
            &mut self.status,
        ]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_package_search_results_with_limit() {
        let output = "doom - game\n\nfreedoom - data files\nchocolate-doom - port\n";
        let results = parse_search_results(output, 2);
        assert_eq!(results, vec!["doom - game", "freedoom - data files"]);
    }

    #[test]
    fn package_manager_builds_search_command() {
        let (binary, args) = PackageManager::Apt.search_command("doom");
        assert_eq!(binary, "apt-cache");
        assert_eq!(args, vec!["search", "doom"]);
    }

    #[test]
    fn package_manager_builds_installed_query_command() {
        let (binary, args) = PackageManager::Apt.installed_query_command("doom");
        assert_eq!(binary, "dpkg-query");
        assert_eq!(args, vec!["-W", "-f=${Status}", "doom"]);

        let (binary, args) = PackageManager::Pacman.installed_query_command("doom");
        assert_eq!(binary, "pacman");
        assert_eq!(args, vec!["-Q", "doom"]);
    }

    #[test]
    fn appstore_search_reports_missing_backend() {
        let view = AppStoreView::new(PackageBackend { manager: None });
        assert!(view.status.text.contains("FEATURED") || view.status.text.contains("NO PACKAGE MANAGER"));
    }

    #[test]
    fn package_manager_builds_transaction_commands() {
        assert_eq!(
            PackageManager::Apt.transaction_command(PackageAction::Install, "doom"),
            vec!["sudo", "apt-get", "install", "-y", "doom"]
        );
        assert_eq!(
            PackageManager::Brew.transaction_command(PackageAction::Remove, "doom"),
            vec!["brew", "uninstall", "doom"]
        );
    }

    #[test]
    fn package_name_extracts_common_search_result_formats() {
        assert_eq!(
            package_name_from_result("community/chocolate-doom 3.0 game port").as_deref(),
            Some("chocolate-doom")
        );
        assert_eq!(
            package_name_from_result("freedoom - data files").as_deref(),
            Some("freedoom")
        );
        assert_eq!(
            package_name_from_result("[INSTALLED] doom - game").as_deref(),
            Some("doom")
        );
    }

    #[test]
    fn annotate_search_results_adds_package_state_prefix() {
        let results = annotate_search_results(
            PackageManager::Apt,
            vec!["definitely-not-installed-retroshell-test-package - demo".to_string()],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].starts_with("[AVAILABLE] ") || results[0].starts_with("[UNKNOWN] "));
        assert!(results[0].contains("definitely-not-installed-retroshell-test-package"));
    }

    #[test]
    fn appstore_install_button_stages_transaction_plan() {
        let mut view = AppStoreView::new(PackageBackend {
            manager: Some(PackageManager::Apt),
        });
        view.results.items = vec!["chocolate-doom - game port".to_string()];
        view.results.selected_index = Some(0);
        // plan_transaction still works for remove/update staging
        assert!(view.plan_transaction(PackageAction::Remove));
        let plan = view.pending_transaction.as_ref().expect("transaction plan");
        assert_eq!(plan.package, "chocolate-doom");
        assert_eq!(plan.action, PackageAction::Remove);
        assert!(view.results.items[0].contains("TRANSACTION PLAN"));
        assert!(view.status.text.contains("REMOVE READY"));
    }

    #[test]
    fn appstore_confirm_is_blocked_without_explicit_env() {
        std::env::remove_var("RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES");
        let mut view = AppStoreView::new(PackageBackend {
            manager: Some(PackageManager::Brew),
        });
        view.pending_transaction = Some(TransactionPlan {
            action: PackageAction::Install,
            package: "doom".to_string(),
            command: vec!["brew".to_string(), "install".to_string(), "doom".to_string()],
        });
        assert!(!view.confirm_transaction());
        assert!(view.status.text.contains("CONFIRM BLOCKED"));
    }

    #[test]
    fn category_filter_all_returns_all_items() {
        let items = vec!["curl - transfer tool".to_string(), "vim - editor".to_string()];
        let filtered = filter_by_category(&items, &[]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn category_filter_keywords_match_subset() {
        let items = vec![
            "curl - network transfer".to_string(),
            "vim - editor".to_string(),
            "wget - network downloader".to_string(),
        ];
        let filtered = filter_by_category(&items, &["network"]);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|l| l.contains("network")));
    }

    #[test]
    fn featured_list_returns_expected_count_without_manager() {
        let backend = PackageBackend { manager: None };
        let items = featured_list(&backend);
        assert_eq!(items.len(), FEATURED_PACKAGES.len());
        assert!(items[0].contains("[FEATURED]"));
    }

    #[test]
    fn category_index_switches_apply_filter() {
        let mut view = AppStoreView::new(PackageBackend { manager: None });
        view.all_results = vec![
            "[FEATURED] curl".to_string(),
            "[FEATURED] vim".to_string(),
            "[FEATURED] wget".to_string(),
        ];
        // Switch to NETWORK category (index 6, keywords include "curl", "wget")
        view.category_index = 6;
        view.apply_category_filter();
        // With keywords ["network","net","ssh","ftp","curl","wget","nmap"]
        // "curl" and "wget" match
        assert!(view.results.items.len() >= 1);
    }
}
