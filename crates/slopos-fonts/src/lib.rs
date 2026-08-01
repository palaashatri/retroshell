//! `slopos-fonts` — shared font service, discovery, font roles, and profiles for SLOPOS-I.
//!
//! Copyright (c) 2026 Palaash Atri
//! SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Standard typography roles across the SLOPOS-I desktop environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontRole {
    SystemUi,
    Menu,
    WindowTitle,
    Body,
    Small,
    Monospace,
    DocumentDefault,
}

impl FontRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SystemUi => "system_ui",
            Self::Menu => "menu",
            Self::WindowTitle => "window_title",
            Self::Body => "body",
            Self::Small => "small",
            Self::Monospace => "monospace",
            Self::DocumentDefault => "document_default",
        }
    }

    pub fn default_size(self) -> f32 {
        match self {
            Self::SystemUi => 13.0,
            Self::Menu => 13.0,
            Self::WindowTitle => 13.0,
            Self::Body => 13.0,
            Self::Small => 11.0,
            Self::Monospace => 12.0,
            Self::DocumentDefault => 14.0,
        }
    }

    pub fn all() -> &'static [FontRole] {
        &[
            Self::SystemUi,
            Self::Menu,
            Self::WindowTitle,
            Self::Body,
            Self::Small,
            Self::Monospace,
            Self::DocumentDefault,
        ]
    }
}

/// Pre-configured appearance typography profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontProfile {
    Classic,
    #[default]
    Modern,
    Accessible,
    Custom,
}

impl FontProfile {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "classic" => Self::Classic,
            "modern" => Self::Modern,
            "accessible" => Self::Accessible,
            "custom" => Self::Custom,
            _ => Self::Modern,
        }
    }
}

/// Specification for a single font role (family name, size, weight).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FontRoleSpec {
    pub family: String,
    pub size: u32,
    pub weight: u16,
}

impl FontRoleSpec {
    pub fn new(family: impl Into<String>, size: u32, weight: u16) -> Self {
        Self {
            family: family.into(),
            size: size.clamp(8, 72),
            weight: weight.clamp(100, 900),
        }
    }
}

/// Active font profile configuration with per-role font specs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FontProfileConfig {
    pub profile: FontProfile,
    pub roles: HashMap<FontRole, FontRoleSpec>,
}

impl Default for FontProfileConfig {
    fn default() -> Self {
        Self::for_profile(FontProfile::Modern)
    }
}

impl FontProfileConfig {
    pub fn for_profile(profile: FontProfile) -> Self {
        let mut roles = HashMap::new();
        match profile {
            FontProfile::Classic => {
                roles.insert(FontRole::SystemUi, FontRoleSpec::new("Chicago", 12, 400));
                roles.insert(FontRole::Menu, FontRoleSpec::new("Chicago", 12, 400));
                roles.insert(FontRole::WindowTitle, FontRoleSpec::new("Chicago", 12, 700));
                roles.insert(FontRole::Body, FontRoleSpec::new("Geneva", 12, 400));
                roles.insert(FontRole::Small, FontRoleSpec::new("Geneva", 10, 400));
                roles.insert(FontRole::Monospace, FontRoleSpec::new("Monaco", 12, 400));
                roles.insert(
                    FontRole::DocumentDefault,
                    FontRoleSpec::new("Geneva", 13, 400),
                );
            }
            FontProfile::Modern => {
                roles.insert(FontRole::SystemUi, FontRoleSpec::new("Inter", 13, 400));
                roles.insert(FontRole::Menu, FontRoleSpec::new("Inter", 13, 400));
                roles.insert(FontRole::WindowTitle, FontRoleSpec::new("Inter", 13, 600));
                roles.insert(FontRole::Body, FontRoleSpec::new("Inter", 13, 400));
                roles.insert(FontRole::Small, FontRoleSpec::new("Inter", 11, 400));
                roles.insert(
                    FontRole::Monospace,
                    FontRoleSpec::new("JetBrains Mono", 12, 400),
                );
                roles.insert(
                    FontRole::DocumentDefault,
                    FontRoleSpec::new("Inter", 14, 400),
                );
            }
            FontProfile::Accessible => {
                roles.insert(
                    FontRole::SystemUi,
                    FontRoleSpec::new("Atkinson Hyperlegible", 15, 600),
                );
                roles.insert(
                    FontRole::Menu,
                    FontRoleSpec::new("Atkinson Hyperlegible", 15, 600),
                );
                roles.insert(
                    FontRole::WindowTitle,
                    FontRoleSpec::new("Atkinson Hyperlegible", 16, 700),
                );
                roles.insert(
                    FontRole::Body,
                    FontRoleSpec::new("Atkinson Hyperlegible", 15, 400),
                );
                roles.insert(
                    FontRole::Small,
                    FontRoleSpec::new("Atkinson Hyperlegible", 13, 400),
                );
                roles.insert(
                    FontRole::Monospace,
                    FontRoleSpec::new("JetBrains Mono", 14, 500),
                );
                roles.insert(
                    FontRole::DocumentDefault,
                    FontRoleSpec::new("Atkinson Hyperlegible", 16, 400),
                );
            }
            FontProfile::Custom => {
                return Self::for_profile(FontProfile::Modern);
            }
        }
        Self { profile, roles }
    }

    pub fn get_spec(&self, role: FontRole) -> FontRoleSpec {
        self.roles
            .get(&role)
            .cloned()
            .unwrap_or_else(|| FontRoleSpec::new("Sans-Serif", role.default_size() as u32, 400))
    }
}

/// Font discovery service searching user and system directories.
#[derive(Clone, Debug)]
pub struct FontDiscoveryService {
    search_paths: Vec<PathBuf>,
}

impl Default for FontDiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

impl FontDiscoveryService {
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            let p1 = PathBuf::from(&data_home).join("fonts");
            let p2 = PathBuf::from(&data_home).join("slopos-i/fonts");
            search_paths.push(p1);
            search_paths.push(p2);
        } else if let Ok(home) = std::env::var("HOME") {
            search_paths.push(PathBuf::from(&home).join(".local/share/fonts"));
            search_paths.push(PathBuf::from(&home).join(".local/share/slopos-i/fonts"));
        }

        if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in data_dirs.split(':') {
                if !dir.is_empty() {
                    search_paths.push(PathBuf::from(dir).join("fonts"));
                }
            }
        }

        search_paths.push(PathBuf::from("/usr/local/share/fonts"));
        search_paths.push(PathBuf::from("/usr/share/fonts"));

        Self { search_paths }
    }

    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Discover available font files (`.ttf`, `.otf`, `.ttc`).
    pub fn discover_font_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for base_path in &self.search_paths {
            if base_path.exists() && base_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(base_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if ext_lower == "ttf" || ext_lower == "otf" || ext_lower == "ttc" {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_roles_and_defaults() {
        assert_eq!(FontRole::SystemUi.as_str(), "system_ui");
        assert_eq!(FontRole::all().len(), 7);
        assert_eq!(FontRole::SystemUi.default_size(), 13.0);
    }

    #[test]
    fn test_font_profile_config() {
        let classic = FontProfileConfig::for_profile(FontProfile::Classic);
        assert_eq!(classic.profile, FontProfile::Classic);
        let spec = classic.get_spec(FontRole::SystemUi);
        assert_eq!(spec.family, "Chicago");

        let modern = FontProfileConfig::for_profile(FontProfile::Modern);
        assert_eq!(modern.get_spec(FontRole::SystemUi).family, "Inter");

        let accessible = FontProfileConfig::for_profile(FontProfile::Accessible);
        assert_eq!(
            accessible.get_spec(FontRole::SystemUi).family,
            "Atkinson Hyperlegible"
        );
    }

    #[test]
    fn test_font_discovery_search_paths() {
        let service = FontDiscoveryService::new();
        assert!(!service.search_paths().is_empty());
        let has_system_fonts = service
            .search_paths()
            .iter()
            .any(|p| p.to_string_lossy().contains("fonts"));
        assert!(has_system_fonts);
    }
}
