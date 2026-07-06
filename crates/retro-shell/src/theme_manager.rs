use retro_kit::theme::{ThemeContext, ThemePalette, ThemeToken, ThemeValue};
use retro_kit::Color;
use std::collections::HashMap;
use std::path::PathBuf;

/// Named retro color themes available in RetroShell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeName {
    #[default]
    /// Mac OS 7 Platinum look (light mode, classic blue accent).
    Classic,
    /// Dark variant of the Classic theme.
    Dark,
    /// Purple-tinted dark theme.
    Grape,
    /// Deep blue dark theme.
    Blueberry,
    /// Warm red-orange tinted theme.
    Strawberry,
}

impl ThemeName {
    /// The accent color (RGBA f32) for this theme.
    pub fn accent_color(self) -> [f32; 4] {
        match self {
            Self::Classic => [0.36, 0.54, 0.85, 1.0],
            Self::Dark => [0.36, 0.54, 0.85, 1.0],
            Self::Grape => [0.55, 0.28, 0.72, 1.0],
            Self::Blueberry => [0.15, 0.25, 0.62, 1.0],
            Self::Strawberry => [0.82, 0.23, 0.28, 1.0],
        }
    }

    /// Whether this theme uses dark mode rendering.
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark | Self::Grape | Self::Blueberry)
    }

    /// The settings.conf key value for this theme.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Dark => "dark",
            Self::Grape => "grape",
            Self::Blueberry => "blueberry",
            Self::Strawberry => "strawberry",
        }
    }

    /// Parse a theme name from a settings.conf value.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "classic" => Some(Self::Classic),
            "dark" => Some(Self::Dark),
            "grape" => Some(Self::Grape),
            "blueberry" => Some(Self::Blueberry),
            "strawberry" => Some(Self::Strawberry),
            _ => None,
        }
    }
}

pub struct ThemeManager {
    pub themes: HashMap<String, ThemePalette>,
    pub current: String,
    pub is_dark: bool,
    pub is_hdr: bool,
    pub scale: f32,
    pub theme_name: ThemeName,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
            current: "platinum".to_string(),
            is_dark: false,
            is_hdr: false,
            scale: 1.0,
            theme_name: ThemeName::Classic,
        }
    }

    pub fn load_default(&mut self) {
        self.load_platinum();
        self.load_graphite();
        self.load_oled_graphite();
        self.load_high_contrast();
    }

    fn load_platinum(&mut self) {
        let mut tokens = HashMap::new();
        tokens.insert(
            ThemeToken::WindowBackground,
            ThemeValue::new(Color::new(0.95, 0.95, 0.95, 1.0))
                .with_dark(Color::new(0.15, 0.15, 0.15, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowBorder,
            ThemeValue::new(Color::new(0.5, 0.5, 0.5, 1.0))
                .with_dark(Color::new(0.3, 0.3, 0.3, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowTitle,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuBackground,
            ThemeValue::new(Color::new(0.98, 0.98, 0.98, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.12, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuHighlight,
            ThemeValue::new(Color::new(0.22, 0.44, 0.85, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuText,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonBackground,
            ThemeValue::new(Color::new(0.88, 0.88, 0.88, 1.0))
                .with_dark(Color::new(0.2, 0.2, 0.2, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonHighlight,
            ThemeValue::new(Color::new(0.22, 0.44, 0.85, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonShadow,
            ThemeValue::new(Color::new(0.6, 0.6, 0.6, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonText,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::TextPrimary,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::TextSecondary,
            ThemeValue::new(Color::new(0.4, 0.4, 0.4, 1.0))
                .with_dark(Color::new(0.7, 0.7, 0.7, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionBackground,
            ThemeValue::new(Color::new(0.22, 0.44, 0.85, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionText,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::DesktopBackground,
            ThemeValue::new(Color::new(0.25, 0.25, 0.45, 1.0))
                .with_dark(Color::new(0.08, 0.08, 0.15, 1.0)),
        );
        tokens.insert(
            ThemeToken::DockBackground,
            ThemeValue::new(Color::new(0.85, 0.85, 0.87, 0.8))
                .with_dark(Color::new(0.1, 0.1, 0.12, 0.9)),
        );
        tokens.insert(
            ThemeToken::ScrollBar,
            ThemeValue::new(Color::new(0.6, 0.6, 0.6, 1.0))
                .with_dark(Color::new(0.4, 0.4, 0.4, 1.0)),
        );
        tokens.insert(
            ThemeToken::Separator,
            ThemeValue::new(Color::new(0.75, 0.75, 0.75, 1.0))
                .with_dark(Color::new(0.3, 0.3, 0.3, 1.0)),
        );
        tokens.insert(
            ThemeToken::FocusRing,
            ThemeValue::new(Color::new(0.22, 0.44, 0.85, 1.0)),
        );
        tokens.insert(
            ThemeToken::ToolbarBackground,
            ThemeValue::new(Color::new(0.92, 0.92, 0.93, 1.0))
                .with_dark(Color::new(0.13, 0.13, 0.14, 1.0)),
        );

        self.themes.insert(
            "platinum".into(),
            ThemePalette {
                name: "Platinum".into(),
                is_dark: self.is_dark,
                tokens,
            },
        );
    }

    fn load_graphite(&mut self) {
        let mut tokens = HashMap::new();
        tokens.insert(
            ThemeToken::WindowBackground,
            ThemeValue::new(Color::new(0.93, 0.93, 0.93, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.12, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowBorder,
            ThemeValue::new(Color::new(0.4, 0.4, 0.4, 1.0))
                .with_dark(Color::new(0.25, 0.25, 0.25, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowTitle,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuBackground,
            ThemeValue::new(Color::new(0.96, 0.96, 0.96, 1.0))
                .with_dark(Color::new(0.1, 0.1, 0.1, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuHighlight,
            ThemeValue::new(Color::new(0.4, 0.4, 0.4, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonBackground,
            ThemeValue::new(Color::new(0.85, 0.85, 0.85, 1.0))
                .with_dark(Color::new(0.18, 0.18, 0.18, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonHighlight,
            ThemeValue::new(Color::new(0.4, 0.4, 0.4, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionBackground,
            ThemeValue::new(Color::new(0.4, 0.4, 0.4, 1.0)),
        );
        tokens.insert(
            ThemeToken::DesktopBackground,
            ThemeValue::new(Color::new(0.2, 0.2, 0.2, 1.0))
                .with_dark(Color::new(0.05, 0.05, 0.05, 1.0)),
        );
        // Copy remaining from platinum with grayscale adjustments
        for (k, v) in &self.themes.get("platinum").unwrap().tokens {
            tokens.entry(*k).or_insert_with(|| v.clone());
        }
        self.themes.insert(
            "graphite".into(),
            ThemePalette {
                name: "Graphite".into(),
                is_dark: self.is_dark,
                tokens,
            },
        );
    }

    fn load_oled_graphite(&mut self) {
        let mut tokens = HashMap::new();
        tokens.insert(
            ThemeToken::WindowBackground,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowBorder,
            ThemeValue::new(Color::new(0.15, 0.15, 0.15, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowTitle,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 0.9)),
        );
        tokens.insert(
            ThemeToken::MenuBackground,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuHighlight,
            ThemeValue::new(Color::new(0.3, 0.3, 0.3, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuText,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 0.9)),
        );
        tokens.insert(
            ThemeToken::ButtonBackground,
            ThemeValue::new(Color::new(0.08, 0.08, 0.08, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonHighlight,
            ThemeValue::new(Color::new(0.25, 0.25, 0.25, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonText,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 0.9)),
        );
        tokens.insert(
            ThemeToken::TextPrimary,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 0.95)),
        );
        tokens.insert(
            ThemeToken::TextSecondary,
            ThemeValue::new(Color::new(0.6, 0.6, 0.6, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionBackground,
            ThemeValue::new(Color::new(0.3, 0.3, 0.3, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionText,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::DesktopBackground,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::DockBackground,
            ThemeValue::new(Color::new(0.03, 0.03, 0.03, 0.95)),
        );
        tokens.insert(
            ThemeToken::ScrollBar,
            ThemeValue::new(Color::new(0.2, 0.2, 0.2, 1.0)),
        );
        tokens.insert(
            ThemeToken::Separator,
            ThemeValue::new(Color::new(0.15, 0.15, 0.15, 1.0)),
        );
        tokens.insert(
            ThemeToken::FocusRing,
            ThemeValue::new(Color::new(0.4, 0.4, 0.4, 1.0)),
        );
        self.themes.insert(
            "oled-graphite".into(),
            ThemePalette {
                name: "OLED Graphite".into(),
                is_dark: true,
                tokens,
            },
        );
    }

    fn load_high_contrast(&mut self) {
        let mut tokens = HashMap::new();
        tokens.insert(
            ThemeToken::WindowBackground,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0))
                .with_dark(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowBorder,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowTitle,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0))
                .with_dark(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuBackground,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0))
                .with_dark(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuHighlight,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonBackground,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0))
                .with_dark(Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonHighlight,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonText,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::TextPrimary,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::DesktopBackground,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        tokens.insert(
            ThemeToken::FocusRing,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        // Fill remaining
        for (k, v) in &self.themes.get("platinum").unwrap().tokens {
            tokens.entry(*k).or_insert_with(|| v.clone());
        }
        self.themes.insert(
            "high-contrast".into(),
            ThemePalette {
                name: "High Contrast".into(),
                is_dark: self.is_dark,
                tokens,
            },
        );
    }

    pub fn set_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current = name.to_string();
            true
        } else {
            false
        }
    }

    pub fn set_dark_mode(&mut self, dark: bool) {
        self.is_dark = dark;
        self.reload_themes();
    }

    pub fn reload_themes(&mut self) {
        self.themes.clear();
        self.load_default();
    }

    pub fn current_context(&self) -> ThemeContext {
        let mut ctx =
            ThemeContext::new(self.themes.get(&self.current).cloned().unwrap_or_else(|| {
                ThemePalette {
                    name: "Default".into(),
                    is_dark: self.is_dark,
                    tokens: HashMap::new(),
                }
            }));
        ctx.scale = self.scale;
        ctx.is_hdr = self.is_hdr;
        ctx
    }

    /// Set the active named theme, updating dark mode and saving to settings.conf.
    pub fn set_named_theme(&mut self, name: ThemeName) {
        self.theme_name = name;
        self.is_dark = name.is_dark();
        self.reload_themes();
        let _ = self.save_theme_to_settings();
    }

    /// Return the current named theme.
    pub fn current_theme(&self) -> ThemeName {
        self.theme_name
    }

    /// Load the theme from settings.conf and apply it.
    pub fn load_theme_from_settings(&mut self) {
        let path = settings_conf_path();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };
        for line in content.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == "theme" {
                if let Some(name) = ThemeName::parse(value) {
                    self.theme_name = name;
                    self.is_dark = name.is_dark();
                    self.reload_themes();
                    return;
                }
            }
        }
        // Fall back to appearance key
        for line in content.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == "appearance" {
                let is_dark = value.trim().eq_ignore_ascii_case("dark");
                self.is_dark = is_dark;
                self.theme_name = if is_dark { ThemeName::Dark } else { ThemeName::Classic };
                self.reload_themes();
                return;
            }
        }
    }

    fn save_theme_to_settings(&self) -> std::io::Result<()> {
        let path = settings_conf_path();
        // Read existing content, update or insert the `theme` key.
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let mut lines: Vec<String> = existing.lines().map(|l| l.to_string()).collect();
        let mut found = false;
        for line in &mut lines {
            if line.trim_start().starts_with("theme=")
                || line.trim_start().starts_with("theme =")
            {
                *line = format!("theme={}", self.theme_name.as_str());
                found = true;
                break;
            }
        }
        if !found {
            lines.push(format!("theme={}", self.theme_name.as_str()));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, lines.join("\n") + "\n")
    }
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
