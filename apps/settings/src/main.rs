use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers, MouseButton};
use retro_kit::label::Label;
use retro_kit::tree_view::{TreeNode, TreeView};
use retro_kit::window::Window;
use retro_kit::{
    AccessibilityNode, Event, EventResult, LayoutConstraint, Point, Rect, Size, ThemeContext,
    Widget, WidgetState,
};
use retro_sdk::{build_menu, Application};
use std::fmt;
use std::fs;
use std::path::PathBuf;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut app = Application::new("Settings", "com.retro.settings");

    let mut file_menu = build_menu("File");
    file_menu.add_action("Close").with_shortcut(
        KeyCode::W,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    file_menu.add_separator();
    file_menu.add_action("Show All Settings");

    let mut edit_menu = build_menu("Edit");
    edit_menu.add_action("Undo");
    edit_menu.add_action("Redo");

    let mut view_menu = build_menu("View");
    view_menu.add_action("Show Search");

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("Settings Help");

    app.set_menus(vec![
        file_menu,
        edit_menu,
        view_menu,
        window_menu,
        help_menu,
    ]);

    let store = SettingsStore::default();
    let view = SettingsView::load(store);
    let mut window = Window::new("Settings");
    window.set_content(Box::new(view));
    app.set_main_window(window);
    app.run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppearanceMode {
    System,
    Light,
    Dark,
}

impl AppearanceMode {
    const ALL: [AppearanceMode; 3] = [
        AppearanceMode::Light,
        AppearanceMode::Dark,
        AppearanceMode::System,
    ];

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::System => "SYSTEM",
            Self::Light => "LIGHT",
            Self::Dark => "DARK",
        }
    }
}

impl fmt::Display for AppearanceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsState {
    appearance: AppearanceMode,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::System,
        }
    }
}

#[derive(Debug, Clone)]
struct SettingsStore {
    path: PathBuf,
}

impl Default for SettingsStore {
    fn default() -> Self {
        let config_dir = std::env::var_os("RETROSHELL_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|home| home.join(".config/retroshell"))
            })
            .unwrap_or_else(|| PathBuf::from("/tmp/retroshell"));
        Self {
            path: config_dir.join("settings.conf"),
        }
    }
}

impl SettingsStore {
    #[cfg(test)]
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load(&self) -> SettingsState {
        let Ok(content) = fs::read_to_string(&self.path) else {
            return SettingsState::default();
        };

        let mut state = SettingsState::default();
        for line in content.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == "appearance" {
                if let Some(mode) = AppearanceMode::parse(value) {
                    state.appearance = mode;
                }
            }
        }
        state
    }

    fn save(&self, state: &SettingsState) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            &self.path,
            format!("appearance={}\n", state.appearance.as_str()),
        )
    }
}

struct SettingsView {
    state: WidgetState,
    categories: TreeView,
    heading: Label,
    description: Label,
    status: Label,
    mode_buttons: Vec<Button>,
    settings: SettingsState,
    store: SettingsStore,
    last_error: Option<String>,
}

impl SettingsView {
    fn load(store: SettingsStore) -> Self {
        let settings = store.load();
        let mut categories = TreeView::new();
        categories.roots = vec![
            TreeNode::new("General"),
            TreeNode::new("Appearance"),
            TreeNode::new("Desktop & Dock"),
            TreeNode::new("Display"),
            TreeNode::new("Sound"),
            TreeNode::new("Network"),
            TreeNode::new("Keyboard"),
            TreeNode::new("Mouse"),
            TreeNode::new("Accessibility"),
            TreeNode::new("Privacy & Security"),
            TreeNode::new("Notifications"),
        ];
        categories.selected_path = Some(vec![1]);

        let mut view = Self {
            state: WidgetState::new(),
            categories,
            heading: Label::new("APPEARANCE"),
            description: Label::new("Choose how RetroShell draws native windows and apps."),
            status: Label::new(""),
            mode_buttons: AppearanceMode::ALL
                .iter()
                .map(|mode| Button::new(mode.label()))
                .collect(),
            settings,
            store,
            last_error: None,
        };
        view.refresh_labels();
        view
    }

    fn refresh_labels(&mut self) {
        for (button, mode) in self
            .mode_buttons
            .iter_mut()
            .zip(AppearanceMode::ALL.iter().copied())
        {
            button.checked = mode == self.settings.appearance;
            button.set_label(if button.checked {
                format!("{} ON", mode.label())
            } else {
                mode.label().to_string()
            });
        }

        let error = self
            .last_error
            .as_deref()
            .map(|error| format!(" - {error}"))
            .unwrap_or_default();
        self.status.text = format!("MODE - {}{}", self.settings.appearance.label(), error);
    }

    fn set_appearance(&mut self, mode: AppearanceMode) -> bool {
        self.settings.appearance = mode;
        match self.store.save(&self.settings) {
            Ok(()) => {
                self.last_error = None;
                self.refresh_labels();
                true
            }
            Err(err) => {
                self.last_error = Some(format!("SAVE FAILED {err}"));
                self.refresh_labels();
                false
            }
        }
    }

    fn handle_mode_click(&mut self, point: Point) -> bool {
        let Some(index) = self
            .mode_buttons
            .iter()
            .position(|button| button.rect().contains(point))
        else {
            return false;
        };
        self.set_appearance(AppearanceMode::ALL[index])
    }
}

impl Widget for SettingsView {
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

        let sidebar_w = (rect.width * 0.28).clamp(170.0, 240.0);
        let divider_w = 4.0;
        self.categories
            .set_rect(Rect::new(rect.x, rect.y, sidebar_w, rect.height));
        let _ = self
            .categories
            .layout(LayoutConstraint::tight(Size::new(sidebar_w, rect.height)));

        let content_x = rect.x + sidebar_w + divider_w + 18.0;
        let content_w = (rect.width - sidebar_w - divider_w - 36.0).max(0.0);
        let mut y = rect.y + 20.0;

        self.heading
            .set_rect(Rect::new(content_x, y, content_w, 24.0));
        let _ = self
            .heading
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));
        y += 34.0;

        self.description
            .set_rect(Rect::new(content_x, y, content_w, 24.0));
        let _ = self
            .description
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));
        y += 42.0;

        for button in &mut self.mode_buttons {
            button.set_rect(Rect::new(content_x, y, 150.0, 30.0));
            let _ = button.layout(LayoutConstraint::tight(Size::new(150.0, 30.0)));
            y += 38.0;
        }

        y += 8.0;
        self.status
            .set_rect(Rect::new(content_x, y, content_w, 24.0));
        let _ = self
            .status
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.categories.draw(theme);
        self.heading.draw(theme);
        self.description.draw(theme);
        for button in &self.mode_buttons {
            button.draw(theme);
        }
        self.status.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::MouseDown {
            button: MouseButton::Left,
            point,
            ..
        } = event
        {
            if self.handle_mode_click(*point) {
                return EventResult::Handled;
            }
        }

        if matches!(self.categories.handle_event(event), EventResult::Handled) {
            return EventResult::Handled;
        }
        EventResult::Ignored
    }

    fn update(&mut self) {
        self.categories.update();
        self.heading.update();
        self.description.update();
        for button in &mut self.mode_buttons {
            button.update();
        }
        self.status.update();
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        let mut children: Vec<&dyn Widget> = vec![
            &self.categories,
            &self.heading,
            &self.description,
            &self.status,
        ];
        for button in &self.mode_buttons {
            children.push(button);
        }
        children
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        let mut children: Vec<&mut dyn Widget> = vec![
            &mut self.categories,
            &mut self.heading,
            &mut self.description,
            &mut self.status,
        ];
        for button in &mut self.mode_buttons {
            children.push(button);
        }
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_settings_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("retroshell_settings_{unique}_{sequence}"))
            .join("settings.conf")
    }

    fn click_mode(view: &mut SettingsView, index: usize) -> EventResult {
        let rect = view.mode_buttons[index].rect();
        view.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
            modifiers: Modifiers::NONE,
        })
    }

    #[test]
    fn settings_store_loads_default_when_missing() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());

        assert_eq!(store.load().appearance, AppearanceMode::System);

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn settings_store_persists_appearance_mode() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());
        let state = SettingsState {
            appearance: AppearanceMode::Dark,
        };

        store.save(&state).unwrap();

        assert_eq!(store.load(), state);
        assert_eq!(fs::read_to_string(&path).unwrap(), "appearance=dark\n");

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn settings_view_click_saves_dark_mode() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());
        let mut view = SettingsView::load(store);
        view.layout(LayoutConstraint::tight(Size::new(720.0, 420.0)));

        let result = click_mode(&mut view, 1);

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(view.settings.appearance, AppearanceMode::Dark);
        assert!(view.status.text.contains("DARK"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "appearance=dark\n");

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
