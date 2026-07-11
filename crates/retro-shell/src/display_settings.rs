//! Display settings (HDR, VRR, refresh rate, color space, multi-monitor arrange).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::display_arrange::{
    plan_display_apply, ArrangeMode, DisplayApplyPlan, DisplayArrangement, DisplayOutput,
};

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

    /// Multi-monitor arrange mode (`extend_right`, `mirror`, …).
    #[serde(default = "default_arrange_mode")]
    pub arrange_mode: String,

    /// Scale percent for primary (100 = 1×).
    #[serde(default = "default_scale_percent")]
    pub scale_percent: u32,
}

fn default_arrange_mode() -> String {
    "extend_right".into()
}

fn default_scale_percent() -> u32 {
    100
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
            arrange_mode: default_arrange_mode(),
            scale_percent: default_scale_percent(),
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
        let valid_refresh_rates = ["60hz", "120hz", "144hz", "165hz", "adaptive"];
        let valid_color_spaces = ["srgb", "rec2020", "scrgb"];

        valid_refresh_rates.contains(&self.refresh_rate.as_str())
            && valid_color_spaces.contains(&self.color_space.as_str())
            && ArrangeMode::parse(&self.arrange_mode).is_some()
            && (50..=400).contains(&self.scale_percent)
    }

    /// Pure: build a multi-monitor apply plan from this config + named outputs.
    pub fn plan_arrangement(&self, outputs: &[DisplayOutput]) -> Result<DisplayApplyPlan, String> {
        let mode = ArrangeMode::parse(&self.arrange_mode).unwrap_or_default();
        let mut outs: Vec<DisplayOutput> = outputs.to_vec();
        if outs.is_empty() {
            let mut primary = DisplayOutput::new("eDP-1", 1920, 1080)
                .with_scale(self.scale_percent);
            primary.is_primary = true;
            outs.push(primary);
        } else {
            for o in &mut outs {
                if o.is_primary {
                    o.scale_percent = self.scale_percent;
                }
            }
        }
        plan_display_apply(&DisplayArrangement {
            mode,
            outputs: outs,
        })
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
        let plan = config.plan_arrangement(&[]).unwrap();
        assert_eq!(plan.placed.len(), 1);

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
            arrange_mode: "mirror".into(),
            scale_percent: 200,
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: DisplayConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.hdr_enabled, deserialized.hdr_enabled);
        assert_eq!(config.vrr_enabled, deserialized.vrr_enabled);
        assert_eq!(config.refresh_rate, deserialized.refresh_rate);
        assert_eq!(config.color_space, deserialized.color_space);
        assert_eq!(config.arrange_mode, deserialized.arrange_mode);
        assert_eq!(config.scale_percent, deserialized.scale_percent);
    }
}
