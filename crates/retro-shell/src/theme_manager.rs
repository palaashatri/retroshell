use crate::a11y_prefs::{apply_a11y_prefs_to_theme_name, A11yPrefs};
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
    /// Solarized color scheme (dark mode, blue accent).
    Solarized,
    /// Dracula color scheme (dark mode, purple accent).
    Dracula,
    /// High contrast theme (pure black/white with yellow accent).
    HighContrast,
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
            Self::Solarized => [0.15, 0.55, 0.82, 1.0], // #268bd2
            Self::Dracula => [0.74, 0.58, 0.98, 1.0],   // #bd93f9
            Self::HighContrast => [1.0, 1.0, 0.0, 1.0], // Yellow accent
        }
    }

    /// Whether this theme uses dark mode rendering.
    pub fn is_dark(self) -> bool {
        matches!(
            self,
            Self::Dark | Self::Grape | Self::Blueberry | Self::Solarized | Self::Dracula
        )
    }

    /// The settings.conf key value for this theme.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Dark => "dark",
            Self::Grape => "grape",
            Self::Blueberry => "blueberry",
            Self::Strawberry => "strawberry",
            Self::Solarized => "solarized",
            Self::Dracula => "dracula",
            Self::HighContrast => "highcontrast",
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
            "solarized" => Some(Self::Solarized),
            "dracula" => Some(Self::Dracula),
            "highcontrast" => Some(Self::HighContrast),
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
    /// Accessibility prefs loaded from settings.conf (contrast / reduced motion).
    pub a11y_prefs: A11yPrefs,
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
            a11y_prefs: A11yPrefs::default(),
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
        // Window background: softer white with depth
        tokens.insert(
            ThemeToken::WindowBackground,
            ThemeValue::new(Color::new(0.96, 0.96, 0.98, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.14, 1.0)),
        );
        // Window border: subtle shadow for depth
        tokens.insert(
            ThemeToken::WindowBorder,
            ThemeValue::new(Color::new(0.7, 0.7, 0.75, 1.0))
                .with_dark(Color::new(0.2, 0.2, 0.22, 1.0)),
        );
        // Window title: contrast text
        tokens.insert(
            ThemeToken::WindowTitle,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(0.95, 0.95, 0.95, 1.0)),
        );
        // Menu background: slightly off-white
        tokens.insert(
            ThemeToken::MenuBackground,
            ThemeValue::new(Color::new(0.98, 0.98, 0.99, 1.0))
                .with_dark(Color::new(0.11, 0.11, 0.13, 1.0)),
        );
        // Menu highlight: vibrant blue (improved from dull blue)
        tokens.insert(
            ThemeToken::MenuHighlight,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuText,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(0.95, 0.95, 0.95, 1.0)),
        );
        // Button background: lighter, more refined
        tokens.insert(
            ThemeToken::ButtonBackground,
            ThemeValue::new(Color::new(0.91, 0.91, 0.93, 1.0))
                .with_dark(Color::new(0.18, 0.18, 0.2, 1.0)),
        );
        // Button highlight: same vibrant blue
        tokens.insert(
            ThemeToken::ButtonHighlight,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        // Button shadow: more pronounced for depth
        tokens.insert(
            ThemeToken::ButtonShadow,
            ThemeValue::new(Color::new(0.7, 0.7, 0.75, 1.0))
                .with_dark(Color::new(0.08, 0.08, 0.1, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonText,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(0.95, 0.95, 0.95, 1.0)),
        );
        tokens.insert(
            ThemeToken::TextPrimary,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(0.95, 0.95, 0.95, 1.0)),
        );
        tokens.insert(
            ThemeToken::TextSecondary,
            ThemeValue::new(Color::new(0.35, 0.35, 0.35, 1.0))
                .with_dark(Color::new(0.65, 0.65, 0.65, 1.0)),
        );
        // Selection: same vibrant blue
        tokens.insert(
            ThemeToken::SelectionBackground,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionText,
            ThemeValue::new(Color::new(1.0, 1.0, 1.0, 1.0)),
        );
        // Desktop background: warmer, more visually interesting
        // Light: slightly warmed light purple/blue
        // Dark: deep space blue
        tokens.insert(
            ThemeToken::DesktopBackground,
            ThemeValue::new(Color::new(0.35, 0.35, 0.55, 1.0))
                .with_dark(Color::new(0.06, 0.06, 0.12, 1.0)),
        );
        // Dock: more translucent, better visual separation
        tokens.insert(
            ThemeToken::DockBackground,
            ThemeValue::new(Color::new(0.88, 0.88, 0.91, 0.85))
                .with_dark(Color::new(0.09, 0.09, 0.11, 0.92)),
        );
        // Scrollbar: refined gray with better contrast
        tokens.insert(
            ThemeToken::ScrollBar,
            ThemeValue::new(Color::new(0.65, 0.65, 0.7, 1.0))
                .with_dark(Color::new(0.35, 0.35, 0.38, 1.0)),
        );
        // Scrollbar hover: darker for interaction feedback
        tokens.insert(
            ThemeToken::ScrollBarHover,
            ThemeValue::new(Color::new(0.5, 0.5, 0.55, 1.0))
                .with_dark(Color::new(0.45, 0.45, 0.48, 1.0)),
        );
        // Separator: subtle, refined
        tokens.insert(
            ThemeToken::Separator,
            ThemeValue::new(Color::new(0.8, 0.8, 0.83, 1.0))
                .with_dark(Color::new(0.25, 0.25, 0.27, 1.0)),
        );
        // Focus ring: vibrant blue
        tokens.insert(
            ThemeToken::FocusRing,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        // Toolbar background: subtle, clean
        tokens.insert(
            ThemeToken::ToolbarBackground,
            ThemeValue::new(Color::new(0.93, 0.93, 0.95, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.14, 1.0)),
        );
        // Toolbar border: matches window border
        tokens.insert(
            ThemeToken::ToolbarBorder,
            ThemeValue::new(Color::new(0.7, 0.7, 0.75, 1.0))
                .with_dark(Color::new(0.2, 0.2, 0.22, 1.0)),
        );
        // Dialog background: same as window
        tokens.insert(
            ThemeToken::DialogBackground,
            ThemeValue::new(Color::new(0.96, 0.96, 0.98, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.14, 1.0)),
        );
        // Dialog border: subtle
        tokens.insert(
            ThemeToken::DialogBorder,
            ThemeValue::new(Color::new(0.7, 0.7, 0.75, 1.0))
                .with_dark(Color::new(0.2, 0.2, 0.22, 1.0)),
        );
        // Progress bar: vibrant blue gradient
        tokens.insert(
            ThemeToken::ProgressBarFill,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        tokens.insert(
            ThemeToken::ProgressBarTrack,
            ThemeValue::new(Color::new(0.9, 0.9, 0.92, 1.0))
                .with_dark(Color::new(0.15, 0.15, 0.17, 1.0)),
        );
        // Slider: blue accent
        tokens.insert(
            ThemeToken::SliderTrack,
            ThemeValue::new(Color::new(0.88, 0.88, 0.9, 1.0))
                .with_dark(Color::new(0.18, 0.18, 0.2, 1.0)),
        );
        tokens.insert(
            ThemeToken::SliderThumb,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        // Status bar: clean light gray
        tokens.insert(
            ThemeToken::StatusBarBackground,
            ThemeValue::new(Color::new(0.93, 0.93, 0.95, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.14, 1.0)),
        );
        // Icon background: subtle, can be highlighted
        tokens.insert(
            ThemeToken::IconBackground,
            ThemeValue::new(Color::new(0.91, 0.91, 0.93, 0.5))
                .with_dark(Color::new(0.18, 0.18, 0.2, 0.5)),
        );
        // Dock highlight: accent color
        tokens.insert(
            ThemeToken::DockHighlight,
            ThemeValue::new(Color::new(0.2, 0.42, 0.88, 1.0)),
        );
        // Notification styling
        tokens.insert(
            ThemeToken::NotificationBackground,
            ThemeValue::new(Color::new(0.96, 0.96, 0.98, 1.0))
                .with_dark(Color::new(0.12, 0.12, 0.14, 1.0)),
        );
        tokens.insert(
            ThemeToken::NotificationBorder,
            ThemeValue::new(Color::new(0.7, 0.7, 0.75, 1.0))
                .with_dark(Color::new(0.2, 0.2, 0.22, 1.0)),
        );
        // Disabled text: lighter gray
        tokens.insert(
            ThemeToken::DisabledText,
            ThemeValue::new(Color::new(0.6, 0.6, 0.6, 1.0))
                .with_dark(Color::new(0.5, 0.5, 0.5, 1.0)),
        );
        // Links: colored for visibility
        tokens.insert(
            ThemeToken::LinkText,
            ThemeValue::new(Color::new(0.1, 0.3, 0.8, 1.0))
                .with_dark(Color::new(0.5, 0.7, 1.0, 1.0)),
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
        // Graphite: refined grayscale with teal accent instead of blue
        tokens.insert(
            ThemeToken::WindowBackground,
            ThemeValue::new(Color::new(0.93, 0.93, 0.93, 1.0))
                .with_dark(Color::new(0.13, 0.13, 0.15, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowBorder,
            ThemeValue::new(Color::new(0.45, 0.45, 0.45, 1.0))
                .with_dark(Color::new(0.22, 0.22, 0.24, 1.0)),
        );
        tokens.insert(
            ThemeToken::WindowTitle,
            ThemeValue::new(Color::new(0.0, 0.0, 0.0, 1.0))
                .with_dark(Color::new(0.95, 0.95, 0.95, 1.0)),
        );
        tokens.insert(
            ThemeToken::MenuBackground,
            ThemeValue::new(Color::new(0.96, 0.96, 0.96, 1.0))
                .with_dark(Color::new(0.11, 0.11, 0.13, 1.0)),
        );
        // Graphite accent: teal/cyan
        tokens.insert(
            ThemeToken::MenuHighlight,
            ThemeValue::new(Color::new(0.2, 0.45, 0.5, 1.0))
                .with_dark(Color::new(0.3, 0.6, 0.65, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonBackground,
            ThemeValue::new(Color::new(0.85, 0.85, 0.85, 1.0))
                .with_dark(Color::new(0.18, 0.18, 0.2, 1.0)),
        );
        tokens.insert(
            ThemeToken::ButtonHighlight,
            ThemeValue::new(Color::new(0.2, 0.45, 0.5, 1.0))
                .with_dark(Color::new(0.3, 0.6, 0.65, 1.0)),
        );
        tokens.insert(
            ThemeToken::SelectionBackground,
            ThemeValue::new(Color::new(0.2, 0.45, 0.5, 1.0))
                .with_dark(Color::new(0.3, 0.6, 0.65, 1.0)),
        );
        tokens.insert(
            ThemeToken::DesktopBackground,
            ThemeValue::new(Color::new(0.15, 0.15, 0.15, 1.0))
                .with_dark(Color::new(0.04, 0.04, 0.06, 1.0)),
        );
        // Copy remaining from platinum with teal accent adjustments
        for (k, v) in &self.themes.get("platinum").unwrap().tokens {
            // Skip the ones we've already set
            if !matches!(
                k,
                ThemeToken::WindowBackground
                    | ThemeToken::WindowBorder
                    | ThemeToken::WindowTitle
                    | ThemeToken::MenuBackground
                    | ThemeToken::MenuHighlight
                    | ThemeToken::ButtonBackground
                    | ThemeToken::ButtonHighlight
                    | ThemeToken::SelectionBackground
                    | ThemeToken::DesktopBackground
            ) {
                tokens.entry(*k).or_insert_with(|| v.clone());
            }
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

    /// Load the theme and a11y prefs from settings.conf and apply them.
    ///
    /// High-contrast a11y preference overrides the named theme selection via
    /// [`apply_a11y_prefs_to_theme_name`]. Reduced motion is stored on
    /// [`Self::a11y_prefs`] for callers of [`crate::a11y_prefs::effective_animation_ms`].
    pub fn load_theme_from_settings(&mut self) {
        let path = settings_conf_path();
        let Ok(content) = std::fs::read_to_string(&path) else {
            return;
        };

        // Theme selection (named theme takes precedence over appearance).
        let mut found_theme = false;
        for line in content.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() == "theme" {
                if let Some(name) = ThemeName::parse(value) {
                    self.theme_name = name;
                    found_theme = true;
                    break;
                }
            }
        }
        if !found_theme {
            for line in content.lines() {
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                if key.trim() == "appearance" {
                    let is_dark = value.trim().eq_ignore_ascii_case("dark");
                    self.theme_name = if is_dark {
                        ThemeName::Dark
                    } else {
                        ThemeName::Classic
                    };
                    break;
                }
            }
        }

        // A11y prefs: high_contrast may force HighContrast theme; motion is stored.
        self.a11y_prefs = A11yPrefs::parse_from_conf(&content);
        self.theme_name = apply_a11y_prefs_to_theme_name(self.a11y_prefs, self.theme_name);
        self.is_dark = self.theme_name.is_dark();
        self.reload_themes();
    }

    /// Apply pure a11y prefs (e.g. from tests or an already-parsed conf).
    pub fn apply_a11y_prefs(&mut self, prefs: A11yPrefs) {
        self.a11y_prefs = prefs;
        self.theme_name = apply_a11y_prefs_to_theme_name(prefs, self.theme_name);
        self.is_dark = self.theme_name.is_dark();
        self.reload_themes();
    }

    fn save_theme_to_settings(&self) -> std::io::Result<()> {
        let path = settings_conf_path();
        // Read existing content, update or insert the `theme` key.
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let mut lines: Vec<String> = existing.lines().map(|l| l.to_string()).collect();
        let mut found = false;
        for line in &mut lines {
            if line.trim_start().starts_with("theme=") || line.trim_start().starts_with("theme =") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_round_trip() {
        let themes = [
            ThemeName::Classic,
            ThemeName::Dark,
            ThemeName::Grape,
            ThemeName::Blueberry,
            ThemeName::Strawberry,
            ThemeName::Solarized,
            ThemeName::Dracula,
            ThemeName::HighContrast,
        ];

        for theme in &themes {
            let as_str = theme.as_str();
            let parsed = ThemeName::parse(as_str);
            assert_eq!(
                parsed,
                Some(*theme),
                "Round-trip failed for theme: {:?}",
                theme
            );
        }
    }

    #[test]
    fn test_theme_string_parsing() {
        assert_eq!(ThemeName::parse("classic"), Some(ThemeName::Classic));
        assert_eq!(ThemeName::parse("dark"), Some(ThemeName::Dark));
        assert_eq!(ThemeName::parse("grape"), Some(ThemeName::Grape));
        assert_eq!(ThemeName::parse("blueberry"), Some(ThemeName::Blueberry));
        assert_eq!(ThemeName::parse("strawberry"), Some(ThemeName::Strawberry));
        assert_eq!(ThemeName::parse("solarized"), Some(ThemeName::Solarized));
        assert_eq!(ThemeName::parse("dracula"), Some(ThemeName::Dracula));
        assert_eq!(
            ThemeName::parse("highcontrast"),
            Some(ThemeName::HighContrast)
        );
        assert_eq!(ThemeName::parse("invalid"), None);
    }

    #[test]
    fn test_dark_mode_variants() {
        assert!(!ThemeName::Classic.is_dark());
        assert!(ThemeName::Dark.is_dark());
        assert!(ThemeName::Grape.is_dark());
        assert!(ThemeName::Blueberry.is_dark());
        assert!(!ThemeName::Strawberry.is_dark());
        assert!(ThemeName::Solarized.is_dark());
        assert!(ThemeName::Dracula.is_dark());
        assert!(!ThemeName::HighContrast.is_dark());
    }
}
