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
        let results = parse_search_results(&text, 12);
        if results.is_empty() {
            Ok(vec![format!(
                "NO RESULTS FOR {}",
                query.to_ascii_uppercase()
            )])
        } else {
            Ok(results)
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

struct AppStoreView {
    state: WidgetState,
    heading: Label,
    backend_label: Label,
    query: TextField,
    search_button: Button,
    refresh_button: Button,
    results: ListView,
    status: Label,
    backend: PackageBackend,
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
            results: ListView::new(),
            status: Label::new("READY"),
            backend,
        };
        view.run_search();
        view
    }

    fn run_search(&mut self) -> bool {
        match self.backend.search(self.query.text()) {
            Ok(results) => {
                self.results.items = results;
                self.status.text = format!("{} RESULTS", self.results.items.len());
                true
            }
            Err(err) => {
                self.results.items = vec![err.clone()];
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

    fn handle_button_click(&mut self, point: Point) -> bool {
        if self.search_button.rect().contains(point) {
            return self.run_search();
        }
        if self.refresh_button.rect().contains(point) {
            return self.refresh_backend();
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
    fn appstore_search_reports_missing_backend() {
        let view = AppStoreView::new(PackageBackend { manager: None });

        assert!(view.status.text.contains("NO PACKAGE MANAGER"));
        assert_eq!(view.results.items, vec!["NO PACKAGE MANAGER FOUND"]);
    }
}
