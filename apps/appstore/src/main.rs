use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers, MouseButton};
use retro_kit::label::Label;
use retro_kit::list_view::ListView;
use retro_kit::text_field::TextField;
use retro_kit::window::Window;
use retro_kit::{
    AccessibilityNode, Event, EventResult, LayoutConstraint, Point, Rect, Size, ThemeContext,
    Widget, WidgetState,
};
use retro_sdk::{build_menu, Application};
use std::process::Command;

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
}

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

    fn search(&self, query: &str) -> Result<Vec<String>, String> {
        let Some(manager) = self.manager else {
            return Err("NO PACKAGE MANAGER FOUND".to_string());
        };
        let query = query.trim();
        if query.is_empty() {
            return Err("SEARCH NEEDS QUERY".to_string());
        }

        let (binary, args) = manager.search_command(query);
        let output = Command::new(binary)
            .args(args)
            .output()
            .map_err(|err| format!("SEARCH FAILED {err}"))?;

        let text = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        let results = annotate_search_results(manager, parse_search_results(&text, 12));
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

    fn status_text(&self) -> String {
        match self.manager {
            Some(manager) => format!("BACKEND - {}", manager.display_name()),
            None => "BACKEND - NONE".to_string(),
        }
    }
}

fn command_exists(binary: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(binary).is_file())
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

struct AppStoreView {
    state: WidgetState,
    heading: Label,
    backend_label: Label,
    query: TextField,
    search_button: Button,
    refresh_button: Button,
    install_button: Button,
    remove_button: Button,
    update_button: Button,
    confirm_button: Button,
    results: ListView,
    status: Label,
    backend: PackageBackend,
    pending_transaction: Option<TransactionPlan>,
}

impl AppStoreView {
    fn new(backend: PackageBackend) -> Self {
        let mut query = TextField::new();
        query.set_text("doom");

        let mut view = Self {
            state: WidgetState::new(),
            heading: Label::new("SOFTWARE CATALOG"),
            backend_label: Label::new(backend.status_text()),
            query,
            search_button: Button::new("SEARCH"),
            refresh_button: Button::new("REFRESH"),
            install_button: Button::new("INSTALL"),
            remove_button: Button::new("REMOVE"),
            update_button: Button::new("UPDATE"),
            confirm_button: Button::new("CONFIRM"),
            results: ListView::new(),
            status: Label::new("READY"),
            backend,
            pending_transaction: None,
        };
        view.run_search();
        view
    }

    fn run_search(&mut self) -> bool {
        match self.backend.search(self.query.text()) {
            Ok(results) => {
                self.results.items = results;
                self.results.selected_index = (!self.results.items.is_empty()).then_some(0);
                self.pending_transaction = None;
                self.status.text = format!("{} RESULTS", self.results.items.len());
                true
            }
            Err(err) => {
                self.results.items = vec![err.clone()];
                self.results.selected_index = None;
                self.pending_transaction = None;
                self.status.text = err;
                false
            }
        }
    }

    fn refresh_backend(&mut self) -> bool {
        self.backend = PackageBackend::detect();
        self.backend_label.text = self.backend.status_text();
        self.run_search()
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
                self.results.items = parse_search_results(&output, 12);
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

    fn handle_button_click(&mut self, point: Point) -> bool {
        if self.search_button.rect().contains(point) {
            return self.run_search();
        }
        if self.refresh_button.rect().contains(point) {
            return self.refresh_backend();
        }
        if self.install_button.rect().contains(point) {
            return self.plan_transaction(PackageAction::Install);
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

        self.heading
            .set_rect(Rect::new(content_x, y, content_w, 24.0));
        let _ = self
            .heading
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));
        y += 30.0;

        self.backend_label
            .set_rect(Rect::new(content_x, y, content_w, 24.0));
        let _ = self
            .backend_label
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));
        y += 34.0;

        self.query
            .set_rect(Rect::new(content_x, y, content_w.min(260.0), 28.0));
        let _ = self.query.layout(LayoutConstraint::tight(Size::new(
            content_w.min(260.0),
            28.0,
        )));

        let button_y = y;
        let button_x = content_x + content_w.min(260.0) + 12.0;
        self.search_button
            .set_rect(Rect::new(button_x, button_y, 88.0, 28.0));
        let _ = self
            .search_button
            .layout(LayoutConstraint::tight(Size::new(88.0, 28.0)));

        self.refresh_button
            .set_rect(Rect::new(button_x + 96.0, button_y, 96.0, 28.0));
        let _ = self
            .refresh_button
            .layout(LayoutConstraint::tight(Size::new(96.0, 28.0)));
        y += 44.0;

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

        let status_h = 26.0;
        let list_h = (rect.height - (y - rect.y) - status_h - pad).max(0.0);
        self.results
            .set_rect(Rect::new(content_x, y, content_w, list_h));
        let _ = self
            .results
            .layout(LayoutConstraint::tight(Size::new(content_w, list_h)));
        y += list_h;

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
        self.results.draw(theme);
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

        let result = self.results.handle_event(event);
        if matches!(result, EventResult::Handled) {
            self.pending_transaction = None;
            if let Some(package) = self.selected_package() {
                self.status.text = format!("SELECTED - {package}");
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
        self.heading.update();
        self.backend_label.update();
        self.query.update();
        self.search_button.update();
        self.refresh_button.update();
        self.install_button.update();
        self.remove_button.update();
        self.update_button.update();
        self.confirm_button.update();
        self.results.update();
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
            &self.results,
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
            &mut self.results,
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

        assert!(view.status.text.contains("NO PACKAGE MANAGER"));
        assert_eq!(view.results.items, vec!["NO PACKAGE MANAGER FOUND"]);
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

        assert!(view.plan_transaction(PackageAction::Install));

        let plan = view.pending_transaction.as_ref().expect("transaction plan");
        assert_eq!(plan.package, "chocolate-doom");
        assert_eq!(plan.action, PackageAction::Install);
        assert!(view.results.items[0].contains("TRANSACTION PLAN"));
        assert!(view.status.text.contains("INSTALL READY"));
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
            command: vec![
                "brew".to_string(),
                "install".to_string(),
                "doom".to_string(),
            ],
        });

        assert!(!view.confirm_transaction());
        assert!(view.status.text.contains("CONFIRM BLOCKED"));
    }
}
