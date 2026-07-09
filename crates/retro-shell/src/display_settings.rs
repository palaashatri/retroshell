//! Display settings (HDR, VRR, refresh rate, color space).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Display configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub hdr_enabled: bool,

    #[serde(default)]
    pub vrr_enabled: bool,

    #[serde(default = "default_refresh_rate")]
    pub refresh_rate: String, // "60hz", "120hz", "144hz", "165hz", "adaptive"

    #[serde(default = "default_color_space")]
    pub color_space: String, // "srgb", "rec2020", "scrgb"
}

fn default_refresh_rate() -> String {
    "60hz".to_string()
}

fn default_color_space() -> String {
    "srgb".to_string()
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            hdr_enabled: false,
            vrr_enabled: false,
            refresh_rate: default_refresh_rate(),
            color_space: default_color_space(),
        }
    }
}

impl DisplayConfig {
    /// Load display settings from config file.
    pub fn load(config_path: &PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(config) = toml::from_str::<std::collections::HashMap<String, Self>>(&content) {
                if let Some(display_config) = config.get("display") {
                    return display_config.clone();
                }
            }
        }
        Self::default()
    }

    /// Save display settings to config file.
    pub fn save(&self, config_path: &PathBuf) -> std::io::Result<()> {
        let mut config: std::collections::HashMap<String, Self> =
            if let Ok(content) = fs::read_to_string(config_path) {
                toml::from_str(&content).unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };

        config.insert("display".to_string(), self.clone());

        let toml_string = toml::to_string_pretty(&config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        fs::write(config_path, toml_string)
    }

    /// Validate settings (check for valid refresh rates and color spaces).
    pub fn validate(&self) -> bool {
        let valid_refresh_rates = vec!["60hz", "120hz", "144hz", "165hz", "adaptive"];
        let valid_color_spaces = vec!["srgb", "rec2020", "scrgb"];

        valid_refresh_rates.contains(&self.refresh_rate.as_str())
            && valid_color_spaces.contains(&self.color_space.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_config_default() {
        let config = DisplayConfig::default();
        assert!(!config.hdr_enabled);
        assert!(!config.vrr_enabled);
        assert_eq!(config.refresh_rate, "60hz");
        assert_eq!(config.color_space, "srgb");
    }

    #[test]
    fn test_display_config_validate() {
        let mut config = DisplayConfig::default();
        assert!(config.validate());

        config.refresh_rate = "invalid".to_string();
        assert!(!config.validate());

        config.refresh_rate = "120hz".to_string();
        config.color_space = "invalid".to_string();
        assert!(!config.validate());

        config.color_space = "rec2020".to_string();
        assert!(config.validate());
    }

    #[test]
    fn test_display_config_roundtrip() {
        let config = DisplayConfig {
            hdr_enabled: true,
            vrr_enabled: true,
            refresh_rate: "144hz".to_string(),
            color_space: "rec2020".to_string(),
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: DisplayConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.hdr_enabled, deserialized.hdr_enabled);
        assert_eq!(config.vrr_enabled, deserialized.vrr_enabled);
        assert_eq!(config.refresh_rate, deserialized.refresh_rate);
        assert_eq!(config.color_space, deserialized.color_space);
    }
}
