use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers, MouseButton};
use retro_kit::label::Label;
use retro_kit::slider::Slider;
use retro_kit::window::Window;
use retro_kit::{
    AccessibilityNode, Event, EventResult, LayoutConstraint, Point, Rect, Size, ThemeContext,
    Widget, WidgetState,
};
use retro_sdk::{build_menu, Application};
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
enum Choice {
    AppearanceLight,
    AppearanceDark,
    AppearanceSystem,
    ThemeClassic,
    ThemeDark,
    ThemeGrape,
    ThemeBlueberry,
    ThemeStrawberry,
    DesktopIconsOn,
    DesktopIconsOff,
    DockBottom,
    DockRight,
    HdrOff,
    HdrOn,
    VrrOff,
    VrrAdaptive,
    SoundOff,
    SoundOn,
    NetworkOffline,
    NetworkDhcp,
    KeyboardSlow,
    KeyboardFast,
    MouseNaturalOff,
    MouseNaturalOn,
    AccessibilityOff,
    AccessibilityOn,
    PrivacyStandard,
    PrivacyStrict,
    NotificationsOff,
    NotificationsOn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    General,
    Appearance,
    DesktopDock,
    Display,
    Sound,
    Network,
    Keyboard,
    Mouse,
    Accessibility,
    Privacy,
    Notifications,
}

impl Category {
    const ALL: [Category; 11] = [
        Category::General,
        Category::Appearance,
        Category::DesktopDock,
        Category::Display,
        Category::Sound,
        Category::Network,
        Category::Keyboard,
        Category::Mouse,
        Category::Accessibility,
        Category::Privacy,
        Category::Notifications,
    ];

    fn label(self) -> &'static str {
        match self {
            Category::General => "General",
            Category::Appearance => "Appearance",
            Category::DesktopDock => "Desktop & Dock",
            Category::Display => "Display",
            Category::Sound => "Sound",
            Category::Network => "Network",
            Category::Keyboard => "Keyboard",
            Category::Mouse => "Mouse",
            Category::Accessibility => "Accessibility",
            Category::Privacy => "Privacy & Security",
            Category::Notifications => "Notifications",
        }
    }

    fn title(self) -> String {
        match self {
            Category::DesktopDock => "DESKTOP & DOCK".to_string(),
            Category::Privacy => "PRIVACY & SECURITY".to_string(),
            _ => self.label().to_ascii_uppercase(),
        }
    }

    fn description(self) -> &'static str {
        match self {
            Category::General => "Choose system defaults used by first-party RetroShell apps.",
            Category::Appearance => "Choose how RetroShell draws native windows and apps.",
            Category::DesktopDock => "Control desktop icons and the shell launcher position.",
            Category::Display => "Configure advertised display capabilities for the shell session.",
            Category::Sound => "Control desktop sound effects for native RetroShell apps.",
            Category::Network => "Set the network profile exposed to shell status surfaces.",
            Category::Keyboard => "Tune keyboard repeat behavior for native controls.",
            Category::Mouse => "Tune pointer and scrolling behavior.",
            Category::Accessibility => "Enable high-visibility affordances across native apps.",
            Category::Privacy => "Control privacy defaults used by app services.",
            Category::Notifications => "Control notification delivery for native apps.",
        }
    }

    fn choices(self) -> &'static [(Choice, &'static str)] {
        match self {
            Category::General => &[
                (Choice::AppearanceSystem, "System Appearance"),
                (Choice::NotificationsOn, "Notifications On"),
                (Choice::SoundOn, "Sound Effects On"),
            ],
            Category::Appearance => &[
                (Choice::AppearanceLight, "Light"),
                (Choice::AppearanceDark, "Dark"),
                (Choice::AppearanceSystem, "System"),
                (Choice::ThemeClassic, "Classic"),
                (Choice::ThemeDark, "Dark Theme"),
                (Choice::ThemeGrape, "Grape"),
                (Choice::ThemeBlueberry, "Blueberry"),
                (Choice::ThemeStrawberry, "Strawberry"),
            ],
            Category::DesktopDock => &[
                (Choice::DesktopIconsOn, "Desktop Icons On"),
                (Choice::DesktopIconsOff, "Desktop Icons Off"),
                (Choice::DockBottom, "Dock Bottom"),
                (Choice::DockRight, "Dock Right"),
            ],
            Category::Display => &[
                (Choice::HdrOff, "HDR Off"),
                (Choice::HdrOn, "HDR Requested"),
                (Choice::VrrOff, "VRR Off"),
                (Choice::VrrAdaptive, "VRR Adaptive"),
            ],
            Category::Sound => &[
                (Choice::SoundOff, "Sound Off"),
                (Choice::SoundOn, "Sound On"),
            ],
            Category::Network => &[
                (Choice::NetworkOffline, "Offline"),
                (Choice::NetworkDhcp, "DHCP"),
            ],
            Category::Keyboard => &[
                (Choice::KeyboardSlow, "Slow Repeat"),
                (Choice::KeyboardFast, "Fast Repeat"),
            ],
            Category::Mouse => &[
                (Choice::MouseNaturalOff, "Natural Scroll Off"),
                (Choice::MouseNaturalOn, "Natural Scroll On"),
            ],
            Category::Accessibility => &[
                (Choice::AccessibilityOff, "Assistive UI Off"),
                (Choice::AccessibilityOn, "Assistive UI On"),
            ],
            Category::Privacy => &[
                (Choice::PrivacyStandard, "Standard"),
                (Choice::PrivacyStrict, "Strict"),
            ],
            Category::Notifications => &[
                (Choice::NotificationsOff, "Notifications Off"),
                (Choice::NotificationsOn, "Notifications On"),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppearanceMode {
    System,
    Light,
    Dark,
}

impl AppearanceMode {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsState {
    appearance: AppearanceMode,
    theme: String,
    desktop_icons: bool,
    dock_position: String,
    hdr_requested: bool,
    vrr_adaptive: bool,
    sound_effects: bool,
    volume_percent: u8,
    network_profile: String,
    keyboard_repeat: String,
    natural_scroll: bool,
    pointer_speed: u8,
    assistive_ui: bool,
    privacy_mode: String,
    notifications: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::System,
            theme: "classic".to_string(),
            desktop_icons: true,
            dock_position: "bottom".to_string(),
            hdr_requested: false,
            vrr_adaptive: false,
            sound_effects: true,
            volume_percent: 75,
            network_profile: "dhcp".to_string(),
            keyboard_repeat: "fast".to_string(),
            natural_scroll: false,
            pointer_speed: 50,
            assistive_ui: false,
            privacy_mode: "standard".to_string(),
            notifications: true,
        }
    }
}

impl SettingsState {
    fn choice_enabled(&self, choice: Choice) -> bool {
        match choice {
            Choice::AppearanceLight => self.appearance == AppearanceMode::Light,
            Choice::AppearanceDark => self.appearance == AppearanceMode::Dark,
            Choice::AppearanceSystem => self.appearance == AppearanceMode::System,
            Choice::ThemeClassic => self.theme == "classic",
            Choice::ThemeDark => self.theme == "dark",
            Choice::ThemeGrape => self.theme == "grape",
            Choice::ThemeBlueberry => self.theme == "blueberry",
            Choice::ThemeStrawberry => self.theme == "strawberry",
            Choice::DesktopIconsOn => self.desktop_icons,
            Choice::DesktopIconsOff => !self.desktop_icons,
            Choice::DockBottom => self.dock_position == "bottom",
            Choice::DockRight => self.dock_position == "right",
            Choice::HdrOff => !self.hdr_requested,
            Choice::HdrOn => self.hdr_requested,
            Choice::VrrOff => !self.vrr_adaptive,
            Choice::VrrAdaptive => self.vrr_adaptive,
            Choice::SoundOff => !self.sound_effects,
            Choice::SoundOn => self.sound_effects,
            Choice::NetworkOffline => self.network_profile == "offline",
            Choice::NetworkDhcp => self.network_profile == "dhcp",
            Choice::KeyboardSlow => self.keyboard_repeat == "slow",
            Choice::KeyboardFast => self.keyboard_repeat == "fast",
            Choice::MouseNaturalOff => !self.natural_scroll,
            Choice::MouseNaturalOn => self.natural_scroll,
            Choice::AccessibilityOff => !self.assistive_ui,
            Choice::AccessibilityOn => self.assistive_ui,
            Choice::PrivacyStandard => self.privacy_mode == "standard",
            Choice::PrivacyStrict => self.privacy_mode == "strict",
            Choice::NotificationsOff => !self.notifications,
            Choice::NotificationsOn => self.notifications,
        }
    }

    fn apply_choice(&mut self, choice: Choice) {
        match choice {
            Choice::AppearanceLight => self.appearance = AppearanceMode::Light,
            Choice::AppearanceDark => self.appearance = AppearanceMode::Dark,
            Choice::AppearanceSystem => self.appearance = AppearanceMode::System,
            Choice::ThemeClassic => self.theme = "classic".to_string(),
            Choice::ThemeDark => self.theme = "dark".to_string(),
            Choice::ThemeGrape => self.theme = "grape".to_string(),
            Choice::ThemeBlueberry => self.theme = "blueberry".to_string(),
            Choice::ThemeStrawberry => self.theme = "strawberry".to_string(),
            Choice::DesktopIconsOn => self.desktop_icons = true,
            Choice::DesktopIconsOff => self.desktop_icons = false,
            Choice::DockBottom => self.dock_position = "bottom".to_string(),
            Choice::DockRight => self.dock_position = "right".to_string(),
            Choice::HdrOff => self.hdr_requested = false,
            Choice::HdrOn => self.hdr_requested = true,
            Choice::VrrOff => self.vrr_adaptive = false,
            Choice::VrrAdaptive => self.vrr_adaptive = true,
            Choice::SoundOff => self.sound_effects = false,
            Choice::SoundOn => self.sound_effects = true,
            Choice::NetworkOffline => self.network_profile = "offline".to_string(),
            Choice::NetworkDhcp => self.network_profile = "dhcp".to_string(),
            Choice::KeyboardSlow => self.keyboard_repeat = "slow".to_string(),
            Choice::KeyboardFast => self.keyboard_repeat = "fast".to_string(),
            Choice::MouseNaturalOff => self.natural_scroll = false,
            Choice::MouseNaturalOn => self.natural_scroll = true,
            Choice::AccessibilityOff => self.assistive_ui = false,
            Choice::AccessibilityOn => self.assistive_ui = true,
            Choice::PrivacyStandard => self.privacy_mode = "standard".to_string(),
            Choice::PrivacyStrict => self.privacy_mode = "strict".to_string(),
            Choice::NotificationsOff => self.notifications = false,
            Choice::NotificationsOn => self.notifications = true,
        }
    }

    fn status_line(&self, category: Category) -> String {
        match category {
            Category::General => format!(
                "GENERAL - {} / {} / {}",
                self.appearance.label(),
                if self.notifications {
                    "NOTIFY ON"
                } else {
                    "NOTIFY OFF"
                },
                if self.sound_effects {
                    "SOUND ON"
                } else {
                    "SOUND OFF"
                }
            ),
            Category::Appearance => format!(
                "MODE - {} / THEME - {}",
                self.appearance.label(),
                self.theme.to_ascii_uppercase()
            ),
            Category::DesktopDock => format!(
                "DESKTOP - ICONS {} / DOCK {}",
                if self.desktop_icons { "ON" } else { "OFF" },
                self.dock_position.to_ascii_uppercase()
            ),
            Category::Display => format!(
                "DISPLAY - HDR {} / VRR {}",
                if self.hdr_requested {
                    "REQUESTED"
                } else {
                    "OFF"
                },
                if self.vrr_adaptive { "ADAPTIVE" } else { "OFF" }
            ),
            Category::Sound => format!(
                "SOUND - EFFECTS {} / VOLUME {}%",
                if self.sound_effects { "ON" } else { "OFF" },
                self.volume_percent
            ),
            Category::Network => {
                format!("NETWORK - {}", self.network_profile.to_ascii_uppercase())
            }
            Category::Keyboard => {
                format!(
                    "KEYBOARD - {} REPEAT",
                    self.keyboard_repeat.to_ascii_uppercase()
                )
            }
            Category::Mouse => format!(
                "MOUSE - NATURAL SCROLL {} / SPEED {}%",
                if self.natural_scroll { "ON" } else { "OFF" },
                self.pointer_speed
            ),
            Category::Accessibility => format!(
                "ACCESSIBILITY - ASSISTIVE UI {}",
                if self.assistive_ui { "ON" } else { "OFF" }
            ),
            Category::Privacy => format!("PRIVACY - {}", self.privacy_mode.to_ascii_uppercase()),
            Category::Notifications => format!(
                "NOTIFICATIONS - {}",
                if self.notifications { "ON" } else { "OFF" }
            ),
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
            let value = value.trim();
            match key.trim() {
                "appearance" => {
                    if let Some(mode) = AppearanceMode::parse(value) {
                        state.appearance = mode;
                    }
                }
                "theme" if matches!(
                    value,
                    "classic" | "dark" | "grape" | "blueberry" | "strawberry"
                ) =>
                {
                    state.theme = value.to_string();
                }
                "desktop_icons" => state.desktop_icons = parse_bool(value, state.desktop_icons),
                "dock_position" if matches!(value, "bottom" | "right") => {
                    state.dock_position = value.to_string();
                }
                "hdr_requested" => state.hdr_requested = parse_bool(value, state.hdr_requested),
                "vrr_adaptive" => state.vrr_adaptive = parse_bool(value, state.vrr_adaptive),
                "sound_effects" => state.sound_effects = parse_bool(value, state.sound_effects),
                "volume_percent" => {
                    state.volume_percent = parse_percent(value, state.volume_percent)
                }
                "network_profile" if matches!(value, "offline" | "dhcp") => {
                    state.network_profile = value.to_string();
                }
                "keyboard_repeat" if matches!(value, "slow" | "fast") => {
                    state.keyboard_repeat = value.to_string();
                }
                "natural_scroll" => state.natural_scroll = parse_bool(value, state.natural_scroll),
                "pointer_speed" => state.pointer_speed = parse_percent(value, state.pointer_speed),
                "assistive_ui" => state.assistive_ui = parse_bool(value, state.assistive_ui),
                "privacy_mode" if matches!(value, "standard" | "strict") => {
                    state.privacy_mode = value.to_string();
                }
                "notifications" => state.notifications = parse_bool(value, state.notifications),
                _ => {}
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
            format!(
                concat!(
                    "appearance={}\n",
                    "theme={}\n",
                    "desktop_icons={}\n",
                    "dock_position={}\n",
                    "hdr_requested={}\n",
                    "vrr_adaptive={}\n",
                    "sound_effects={}\n",
                    "volume_percent={}\n",
                    "network_profile={}\n",
                    "keyboard_repeat={}\n",
                    "natural_scroll={}\n",
                    "pointer_speed={}\n",
                    "assistive_ui={}\n",
                    "privacy_mode={}\n",
                    "notifications={}\n"
                ),
                state.appearance.as_str(),
                state.theme,
                state.desktop_icons,
                state.dock_position,
                state.hdr_requested,
                state.vrr_adaptive,
                state.sound_effects,
                state.volume_percent,
                state.network_profile,
                state.keyboard_repeat,
                state.natural_scroll,
                state.pointer_speed,
                state.assistive_ui,
                state.privacy_mode,
                state.notifications
            ),
        )
    }
}

fn parse_bool(value: &str, fallback: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => fallback,
    }
}

fn parse_percent(value: &str, fallback: u8) -> u8 {
    value
        .trim()
        .parse::<u8>()
        .map(|value| value.min(100))
        .unwrap_or(fallback)
}

struct SettingsView {
    state: WidgetState,
    category_buttons: Vec<Button>,
    heading: Label,
    description: Label,
    status: Label,
    option_buttons: Vec<Button>,
    volume_label: Label,
    volume_slider: Slider,
    pointer_speed_label: Label,
    pointer_speed_slider: Slider,
    selected_category: Category,
    settings: SettingsState,
    store: SettingsStore,
    last_error: Option<String>,
}

impl SettingsView {
    fn load(store: SettingsStore) -> Self {
        let settings = store.load();
        let mut view = Self {
            state: WidgetState::new(),
            category_buttons: Category::ALL
                .iter()
                .map(|category| Button::new(category.label()))
                .collect(),
            heading: Label::new("APPEARANCE"),
            description: Label::new("Choose how RetroShell draws native windows and apps."),
            status: Label::new(""),
            option_buttons: Vec::new(),
            volume_label: Label::new("VOLUME"),
            volume_slider: Slider::new(),
            pointer_speed_label: Label::new("POINTER SPEED"),
            pointer_speed_slider: Slider::new(),
            selected_category: Category::Appearance,
            settings,
            store,
            last_error: None,
        };
        view.refresh_labels();
        view
    }

    fn refresh_labels(&mut self) {
        self.heading.text = self.selected_category.title();
        self.description.text = self.selected_category.description().to_string();

        self.option_buttons = self
            .selected_category
            .choices()
            .iter()
            .map(|(choice, label)| {
                let mut button = Button::new(if self.settings.choice_enabled(*choice) {
                    format!("{label} *")
                } else {
                    (*label).to_string()
                });
                button.checked = self.settings.choice_enabled(*choice);
                button
            })
            .collect();

        self.volume_label.text = format!("VOLUME {}%", self.settings.volume_percent);
        self.volume_slider.min = 0.0;
        self.volume_slider.max = 100.0;
        self.volume_slider.step = 5.0;
        self.volume_slider
            .set_value(self.settings.volume_percent as f32);

        self.pointer_speed_label.text = format!("POINTER SPEED {}%", self.settings.pointer_speed);
        self.pointer_speed_slider.min = 0.0;
        self.pointer_speed_slider.max = 100.0;
        self.pointer_speed_slider.step = 5.0;
        self.pointer_speed_slider
            .set_value(self.settings.pointer_speed as f32);

        for (button, category) in self
            .category_buttons
            .iter_mut()
            .zip(Category::ALL.iter().copied())
        {
            button.checked = category == self.selected_category;
            button.set_label(if button.checked {
                format!("{} *", category.label())
            } else {
                category.label().to_string()
            });
        }

        let error = self
            .last_error
            .as_deref()
            .map(|error| format!(" - {error}"))
            .unwrap_or_default();
        self.status.text = format!(
            "{}{}",
            self.settings.status_line(self.selected_category),
            error
        );
    }

    fn select_category(&mut self, category: Category) {
        self.selected_category = category;
        self.last_error = None;
        self.refresh_labels();
        self.relayout_if_visible();
    }

    fn apply_choice(&mut self, choice: Choice) -> bool {
        self.settings.apply_choice(choice);
        match self.store.save(&self.settings) {
            Ok(()) => {
                self.last_error = None;
                self.refresh_labels();
                self.relayout_if_visible();
                true
            }
            Err(err) => {
                self.last_error = Some(format!("SAVE FAILED {err}"));
                self.refresh_labels();
                self.relayout_if_visible();
                false
            }
        }
    }

    fn save_slider_value(&mut self) -> bool {
        match self.selected_category {
            Category::Sound => {
                self.settings.volume_percent = self.volume_slider.value.round() as u8
            }
            Category::Mouse => {
                self.settings.pointer_speed = self.pointer_speed_slider.value.round() as u8
            }
            _ => return false,
        }

        match self.store.save(&self.settings) {
            Ok(()) => {
                self.last_error = None;
                self.refresh_labels();
                self.relayout_if_visible();
                true
            }
            Err(err) => {
                self.last_error = Some(format!("SAVE FAILED {err}"));
                self.refresh_labels();
                self.relayout_if_visible();
                false
            }
        }
    }

    fn relayout_if_visible(&mut self) {
        let rect = self.rect();
        if rect.width > 0.0 && rect.height > 0.0 {
            let _ = self.layout(LayoutConstraint::tight(Size::new(rect.width, rect.height)));
        }
    }

    fn handle_category_click(&mut self, point: Point) -> bool {
        let Some(index) = self
            .category_buttons
            .iter()
            .position(|button| button.rect().contains(point))
        else {
            return false;
        };
        self.select_category(Category::ALL[index]);
        true
    }

    fn handle_option_click(&mut self, point: Point) -> bool {
        let Some(index) = self
            .option_buttons
            .iter()
            .position(|button| button.rect().contains(point))
        else {
            return false;
        };
        let choice = self.selected_category.choices()[index].0;
        self.apply_choice(choice)
    }

    fn handle_slider_event(&mut self, event: &Event) -> bool {
        let handled = match self.selected_category {
            Category::Sound => self.volume_slider.handle_event(event),
            Category::Mouse => self.pointer_speed_slider.handle_event(event),
            _ => EventResult::Ignored,
        };

        if matches!(handled, EventResult::Handled) {
            return self.save_slider_value();
        }

        false
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
        let mut y = rect.y + 12.0;
        for button in &mut self.category_buttons {
            button.set_rect(Rect::new(rect.x + 10.0, y, sidebar_w - 20.0, 24.0));
            let _ = button.layout(LayoutConstraint::tight(Size::new(sidebar_w - 20.0, 24.0)));
            y += 28.0;
        }

        let content_x = rect.x + sidebar_w + 18.0;
        let content_w = (rect.width - sidebar_w - 36.0).max(0.0);
        let mut content_y = rect.y + 20.0;

        self.heading
            .set_rect(Rect::new(content_x, content_y, content_w, 24.0));
        let _ = self
            .heading
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));
        content_y += 32.0;

        self.description
            .set_rect(Rect::new(content_x, content_y, content_w, 44.0));
        let _ = self
            .description
            .layout(LayoutConstraint::tight(Size::new(content_w, 44.0)));
        content_y += 56.0;

        let button_w = (content_w / 2.0 - 8.0).clamp(132.0, 220.0);
        for (index, button) in self.option_buttons.iter_mut().enumerate() {
            let col = index % 2;
            let row = index / 2;
            let x = content_x + col as f32 * (button_w + 12.0);
            let y = content_y + row as f32 * 38.0;
            button.set_rect(Rect::new(x, y, button_w, 28.0));
            let _ = button.layout(LayoutConstraint::tight(Size::new(button_w, 28.0)));
        }

        let slider_y = content_y + ((self.option_buttons.len() + 1) / 2) as f32 * 38.0 + 12.0;
        match self.selected_category {
            Category::Sound => {
                self.volume_label
                    .set_rect(Rect::new(content_x, slider_y, 180.0, 24.0));
                let _ = self
                    .volume_label
                    .layout(LayoutConstraint::tight(Size::new(180.0, 24.0)));
                self.volume_slider
                    .set_rect(Rect::new(content_x + 190.0, slider_y, 190.0, 24.0));
                let _ = self
                    .volume_slider
                    .layout(LayoutConstraint::tight(Size::new(190.0, 24.0)));
            }
            Category::Mouse => {
                self.pointer_speed_label
                    .set_rect(Rect::new(content_x, slider_y, 180.0, 24.0));
                let _ = self
                    .pointer_speed_label
                    .layout(LayoutConstraint::tight(Size::new(180.0, 24.0)));
                self.pointer_speed_slider.set_rect(Rect::new(
                    content_x + 190.0,
                    slider_y,
                    190.0,
                    24.0,
                ));
                let _ = self
                    .pointer_speed_slider
                    .layout(LayoutConstraint::tight(Size::new(190.0, 24.0)));
            }
            _ => {}
        }

        self.status.set_rect(Rect::new(
            content_x,
            rect.y + rect.height - 36.0,
            content_w,
            24.0,
        ));
        let _ = self
            .status
            .layout(LayoutConstraint::tight(Size::new(content_w, 24.0)));

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        for button in &self.category_buttons {
            button.draw(theme);
        }
        self.heading.draw(theme);
        self.description.draw(theme);
        for button in &self.option_buttons {
            button.draw(theme);
        }
        match self.selected_category {
            Category::Sound => {
                self.volume_label.draw(theme);
                self.volume_slider.draw(theme);
            }
            Category::Mouse => {
                self.pointer_speed_label.draw(theme);
                self.pointer_speed_slider.draw(theme);
            }
            _ => {}
        }
        self.status.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if self.handle_slider_event(event) {
            return EventResult::Handled;
        }

        if let Event::MouseDown {
            button: MouseButton::Left,
            point,
            ..
        } = event
        {
            if self.handle_category_click(*point) || self.handle_option_click(*point) {
                return EventResult::Handled;
            }
        }
        EventResult::Ignored
    }

    fn update(&mut self) {
        for button in &mut self.category_buttons {
            button.update();
        }
        self.heading.update();
        self.description.update();
        for button in &mut self.option_buttons {
            button.update();
        }
        self.volume_label.update();
        self.volume_slider.update();
        self.pointer_speed_label.update();
        self.pointer_speed_slider.update();
        self.status.update();
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        let mut children: Vec<&dyn Widget> = Vec::new();
        for button in &self.category_buttons {
            children.push(button);
        }
        children.push(&self.heading);
        children.push(&self.description);
        for button in &self.option_buttons {
            children.push(button);
        }
        match self.selected_category {
            Category::Sound => {
                children.push(&self.volume_label);
                children.push(&self.volume_slider);
            }
            Category::Mouse => {
                children.push(&self.pointer_speed_label);
                children.push(&self.pointer_speed_slider);
            }
            _ => {}
        }
        children.push(&self.status);
        children
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        let mut children: Vec<&mut dyn Widget> = Vec::new();
        for button in &mut self.category_buttons {
            children.push(button);
        }
        children.push(&mut self.heading);
        children.push(&mut self.description);
        for button in &mut self.option_buttons {
            children.push(button);
        }
        match self.selected_category {
            Category::Sound => {
                children.push(&mut self.volume_label);
                children.push(&mut self.volume_slider);
            }
            Category::Mouse => {
                children.push(&mut self.pointer_speed_label);
                children.push(&mut self.pointer_speed_slider);
            }
            _ => {}
        }
        children.push(&mut self.status);
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

    fn click_button(button: &Button) -> Event {
        let rect = button.rect();
        Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(rect.x + 2.0, rect.y + 2.0),
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: false,
            },
        }
    }

    fn assert_handled(result: EventResult) {
        assert!(matches!(result, EventResult::Handled));
    }

    #[test]
    fn settings_store_persists_all_supported_values() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path);
        let state = SettingsState {
            appearance: AppearanceMode::Dark,
            theme: "grape".to_string(),
            desktop_icons: false,
            dock_position: "right".to_string(),
            hdr_requested: true,
            vrr_adaptive: true,
            sound_effects: false,
            volume_percent: 35,
            network_profile: "offline".to_string(),
            keyboard_repeat: "slow".to_string(),
            natural_scroll: true,
            pointer_speed: 85,
            assistive_ui: true,
            privacy_mode: "strict".to_string(),
            notifications: false,
        };

        store.save(&state).unwrap();
        assert_eq!(store.load(), state);
    }

    #[test]
    fn settings_category_click_rebuilds_visible_options() {
        let store = SettingsStore::new(temp_settings_path());
        let mut view = SettingsView::load(store);
        view.set_rect(Rect::new(0.0, 0.0, 640.0, 420.0));
        view.layout(LayoutConstraint::tight(Size::new(640.0, 420.0)));

        let display_button = &view.category_buttons[3];
        assert_handled(view.handle_event(&click_button(display_button)));

        assert_eq!(view.selected_category, Category::Display);
        assert!(view.heading.text.contains("DISPLAY"));
        assert!(view
            .option_buttons
            .iter()
            .any(|button| button.label.contains("HDR")));
        assert!(view
            .option_buttons
            .iter()
            .any(|button| button.label.contains("VRR")));
        assert!(view
            .option_buttons
            .iter()
            .all(|button| button.rect().width > 0.0));
    }

    #[test]
    fn settings_option_click_updates_and_saves_state() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());
        let mut view = SettingsView::load(store);
        view.select_category(Category::Display);
        view.set_rect(Rect::new(0.0, 0.0, 640.0, 420.0));
        view.layout(LayoutConstraint::tight(Size::new(640.0, 420.0)));

        let hdr_button = &view.option_buttons[1];
        assert_handled(view.handle_event(&click_button(hdr_button)));

        let loaded = SettingsStore::new(path).load();
        assert!(loaded.hdr_requested);
        assert!(view.status.text.contains("HDR REQUESTED"));
    }

    #[test]
    fn settings_sound_slider_updates_and_saves_volume() {
        let path = temp_settings_path();
        let store = SettingsStore::new(path.clone());
        let mut view = SettingsView::load(store);
        view.select_category(Category::Sound);
        view.set_rect(Rect::new(0.0, 0.0, 640.0, 420.0));
        view.layout(LayoutConstraint::tight(Size::new(640.0, 420.0)));

        let slider = view.volume_slider.rect();
        assert_handled(view.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(slider.x + slider.width - 9.0, slider.y + 12.0),
            modifiers: Modifiers::NONE,
        }));

        let loaded = SettingsStore::new(path).load();
        assert_eq!(loaded.volume_percent, 100);
        assert!(view.status.text.contains("VOLUME 100%"));
    }
}
