//! Display settings (HDR, VRR, refresh rate, color space, multi-monitor arrange).
//!
//! # Settings UI contract
//!
//! The Settings app Display pane persists `arrange_mode` and `scale_percent` as
//! flat `key=value` lines in `settings.conf`. On every save of those fields the
//! UI **must** call [`DisplayConfig::apply_arrangement_env`] (which runs
//! [`plan_arrangement`] + [`crate::display_arrange::apply_display_plan_env`])
//! so nested compositor children immediately see `RETROSHELL_OUTPUTS_LAYOUT`.
//! Shell startup re-applies the same path via `apply_display_config_from_settings`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::display_arrange::{
    apply_display_plan_env, plan_display_apply, ArrangeMode, DisplayApplyPlan, DisplayArrangement,
    DisplayOutput,
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

    /// Build from Settings-app Display fields (flat conf / UI state).
    pub fn from_settings_fields(
        hdr_enabled: bool,
        vrr_enabled: bool,
        refresh_rate: impl Into<String>,
        color_space: impl Into<String>,
        arrange_mode: impl Into<String>,
        scale_percent: u32,
    ) -> Self {
        Self {
            hdr_enabled,
            vrr_enabled,
            refresh_rate: refresh_rate.into(),
            color_space: color_space.into(),
            arrange_mode: arrange_mode.into(),
            scale_percent: scale_percent.clamp(50, 400),
        }
    }

    /// Merge flat `key=value` settings.conf lines into arrange/scale (and HDR-class) fields.
    ///
    /// Pure: no I/O. Unknown keys are ignored. Used by shell startup and Settings save.
    pub fn merge_flat_settings_conf(&mut self, conf: &str) {
        for line in conf.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "arrange_mode" => {
                    if ArrangeMode::parse(value).is_some() {
                        self.arrange_mode = value.to_string();
                    }
                }
                "scale_percent" => {
                    if let Ok(n) = value.parse::<u32>() {
                        if (50..=400).contains(&n) {
                            self.scale_percent = n;
                        }
                    }
                }
                "hdr_requested" | "hdr_enabled" => {
                    self.hdr_enabled = parse_conf_bool(value, self.hdr_enabled);
                }
                "vrr_adaptive" | "vrr_enabled" => {
                    self.vrr_enabled = parse_conf_bool(value, self.vrr_enabled);
                }
                "refresh_rate"
                    if matches!(value, "60hz" | "120hz" | "144hz" | "165hz" | "adaptive") =>
                {
                    self.refresh_rate = value.to_string();
                }
                "color_space" if matches!(value, "srgb" | "rec2020" | "scrgb") => {
                    self.color_space = value.to_string();
                }
                _ => {}
            }
        }
    }

    /// Flat settings.conf pairs for Display fields (Settings app write path).
    pub fn flat_conf_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("hdr_requested".into(), self.hdr_enabled.to_string()),
            ("vrr_adaptive".into(), self.vrr_enabled.to_string()),
            ("refresh_rate".into(), self.refresh_rate.clone()),
            ("color_space".into(), self.color_space.clone()),
            ("arrange_mode".into(), self.arrange_mode.clone()),
            ("scale_percent".into(), self.scale_percent.to_string()),
        ]
    }

    /// Settings save hook: plan arrangement then live-apply `EmitLayoutEnv` to process env.
    ///
    /// Call after writing `arrange_mode` / `scale_percent` to settings.conf.
    pub fn apply_arrangement_env(
        &self,
        outputs: &[DisplayOutput],
    ) -> Result<Vec<(String, String)>, String> {
        let plan = self.plan_arrangement(outputs)?;
        Ok(apply_display_plan_env(&plan))
    }
}

fn parse_conf_bool(value: &str, fallback: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => fallback,
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

    #[test]
    fn settings_conf_round_trips_arrange_mode_and_scale() {
        // Settings app writes flat key=value; shell/Settings merge back into DisplayConfig.
        let original = DisplayConfig::from_settings_fields(
            true,
            false,
            "120hz",
            "srgb",
            "mirror",
            200,
        );
        assert!(original.validate());

        let conf: String = original
            .flat_conf_pairs()
            .into_iter()
            .map(|(k, v)| format!("{k}={v}\n"))
            .collect();
        assert!(conf.contains("arrange_mode=mirror"));
        assert!(conf.contains("scale_percent=200"));

        let mut loaded = DisplayConfig::default();
        loaded.merge_flat_settings_conf(&conf);
        assert_eq!(loaded.arrange_mode, "mirror");
        assert_eq!(loaded.scale_percent, 200);
        assert!(loaded.hdr_enabled);
        assert!(!loaded.vrr_enabled);
        assert_eq!(loaded.refresh_rate, "120hz");
        assert_eq!(loaded.color_space, "srgb");

        // Apply path used by Settings UI on save.
        let applied = loaded.apply_arrangement_env(&[]).unwrap();
        assert!(
            applied
                .iter()
                .any(|(k, v)| k == "RETROSHELL_OUTPUTS_LAYOUT" && v.contains(":s200")),
            "expected layout env with scale 200, got {applied:?}"
        );
        std::env::remove_var("RETROSHELL_OUTPUTS_LAYOUT");
    }

    #[test]
    fn merge_flat_rejects_invalid_arrange_mode() {
        let mut config = DisplayConfig::default();
        config.arrange_mode = "extend_right".into();
        config.merge_flat_settings_conf("arrange_mode=not_a_mode\nscale_percent=12\n");
        assert_eq!(config.arrange_mode, "extend_right");
        assert_eq!(config.scale_percent, 100);
    }
}
