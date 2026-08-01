//! `slopos-fonts` — shared font service, discovery, font roles, and profiles for SLOPOS-I.
//!
//! Copyright (c) 2026 Palaash Atri
//! SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const MAX_FONT_FILE_BYTES: u64 = 64 * 1024 * 1024;
const DISABLED_MARKER_DIR: &str = ".disabled";

/// Smallest size emitted by the profile resolver, in logical points.
pub const MIN_FONT_SIZE: u32 = 8;
/// Largest size emitted by the profile resolver, in logical points.
pub const MAX_FONT_SIZE: u32 = 72;
/// Smallest accepted display scale for profile resolution.
pub const MIN_FONT_SCALE: f32 = 0.5;
/// Largest accepted display scale for profile resolution.
pub const MAX_FONT_SCALE: f32 = 4.0;

fn is_font_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "ttf" | "otf" | "ttc"
            )
        })
}

fn safe_file_name(file_name: &str) -> bool {
    !file_name.is_empty()
        && Path::new(file_name)
            .file_name()
            .and_then(|name| name.to_str())
            == Some(file_name)
        && !file_name.chars().any(|ch| ch.is_control() || ch == '\0')
}

fn sha256_file(path: &Path) -> Result<String, FontManagerError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut DigestWriter(&mut hasher))?;
    Ok(hex::encode(hasher.finalize()))
}

struct DigestWriter<'a>(&'a mut Sha256);

impl Write for DigestWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FontManagerError {
    #[error("font source is not a regular font file: {0}")]
    InvalidSource(String),
    #[error("font file is too large ({actual} bytes; maximum {maximum})")]
    TooLarge { actual: u64, maximum: u64 },
    #[error("font file name is unsafe: {0}")]
    UnsafeFileName(String),
    #[error("installed font was not found: {0}")]
    NotInstalled(String),
    #[error("font I/O failed: {0}")]
    Io(#[from] io::Error),
}

/// Metadata and enablement state for one installed font file.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstalledFont {
    pub file_name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub sha256: String,
    pub enabled: bool,
}

/// User-font installation and state manager.
///
/// This manager deliberately does not parse or render font tables. It owns the
/// filesystem lifecycle and exposes immutable file metadata to the future
/// font database/renderer, so a UI never has to perform unsafe path operations.
#[derive(Clone, Debug)]
pub struct FontManager {
    discovery: FontDiscoveryService,
    install_dir: PathBuf,
}

impl FontManager {
    pub fn new(install_dir: impl Into<PathBuf>) -> Self {
        Self {
            discovery: FontDiscoveryService::new(),
            install_dir: install_dir.into(),
        }
    }

    pub fn with_discovery(
        install_dir: impl Into<PathBuf>,
        discovery: FontDiscoveryService,
    ) -> Self {
        Self {
            discovery,
            install_dir: install_dir.into(),
        }
    }

    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }

    pub fn discover_system_files(&self) -> Vec<PathBuf> {
        self.discovery.discover_font_files()
    }

    pub fn installed_fonts(&self) -> Result<Vec<InstalledFont>, FontManagerError> {
        let mut fonts = Vec::new();
        if !self.install_dir.exists() {
            return Ok(fonts);
        }
        for entry in fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || !is_font_extension(&path) {
                continue;
            }
            fonts.push(self.describe_installed(&path)?);
        }
        fonts.sort_by(|left, right| left.file_name.cmp(&right.file_name));
        Ok(fonts)
    }

    pub fn install(&self, source: &Path) -> Result<InstalledFont, FontManagerError> {
        let (source_name, _source_size) = validate_font_source(source)?;
        fs::create_dir_all(&self.install_dir)?;
        let source_hash = sha256_file(source)?;

        for installed in self.installed_fonts()? {
            if installed.sha256 == source_hash {
                return Ok(installed);
            }
        }

        let mut file_name = source_name;
        let mut destination = self.install_dir.join(&file_name);
        if destination.exists() {
            let stem = Path::new(&file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| FontManagerError::UnsafeFileName(file_name.clone()))?;
            let extension = Path::new(&file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .ok_or_else(|| FontManagerError::UnsafeFileName(file_name.clone()))?;
            file_name = format!("{stem}-{}.{extension}", &source_hash[..8]);
            destination = self.install_dir.join(&file_name);
        }

        let temporary = self
            .install_dir
            .join(format!(".{file_name}.{}.tmp", std::process::id()));
        let mut input = fs::File::open(source)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        drop(output);
        if let Err(error) = fs::rename(&temporary, &destination) {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        self.describe_installed(&destination)
    }

    pub fn set_enabled(
        &self,
        file_name: &str,
        enabled: bool,
    ) -> Result<InstalledFont, FontManagerError> {
        let font = self.require_installed(file_name)?;
        let marker_dir = self.install_dir.join(DISABLED_MARKER_DIR);
        let marker = marker_dir.join(file_name);
        if enabled {
            if marker.exists() {
                fs::remove_file(marker)?;
            }
        } else {
            fs::create_dir_all(&marker_dir)?;
            let temporary = marker_dir.join(format!(".{file_name}.{}.tmp", std::process::id()));
            let mut marker_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            marker_file.write_all(b"disabled\n")?;
            marker_file.sync_all()?;
            drop(marker_file);
            if let Err(error) = fs::rename(&temporary, &marker) {
                let _ = fs::remove_file(&temporary);
                return Err(error.into());
            }
        }
        self.describe_installed(&font.path)
    }

    pub fn remove(&self, file_name: &str) -> Result<(), FontManagerError> {
        let font = self.require_installed(file_name)?;
        fs::remove_file(font.path)?;
        let marker = self.install_dir.join(DISABLED_MARKER_DIR).join(file_name);
        if marker.exists() {
            fs::remove_file(marker)?;
        }
        Ok(())
    }

    fn require_installed(&self, file_name: &str) -> Result<InstalledFont, FontManagerError> {
        if !safe_file_name(file_name) || !is_font_extension(Path::new(file_name)) {
            return Err(FontManagerError::UnsafeFileName(file_name.to_string()));
        }
        let path = self.install_dir.join(file_name);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| FontManagerError::NotInstalled(file_name.to_string()))?;
        if !metadata.file_type().is_file() {
            return Err(FontManagerError::NotInstalled(file_name.to_string()));
        }
        self.describe_installed(&path)
    }

    fn describe_installed(&self, path: &Path) -> Result<InstalledFont, FontManagerError> {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.file_type().is_file() || !is_font_extension(path) {
            return Err(FontManagerError::InvalidSource(path.display().to_string()));
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| FontManagerError::UnsafeFileName(path.display().to_string()))?;
        Ok(InstalledFont {
            file_name: file_name.to_string(),
            path: path.to_path_buf(),
            bytes: metadata.len(),
            sha256: sha256_file(path)?,
            enabled: !self
                .install_dir
                .join(DISABLED_MARKER_DIR)
                .join(file_name)
                .exists(),
        })
    }
}

fn validate_font_source(source: &Path) -> Result<(String, u64), FontManagerError> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|_| FontManagerError::InvalidSource(source.display().to_string()))?;
    if !metadata.file_type().is_file() || !is_font_extension(source) {
        return Err(FontManagerError::InvalidSource(
            source.display().to_string(),
        ));
    }
    if metadata.len() == 0 {
        return Err(FontManagerError::InvalidSource(
            source.display().to_string(),
        ));
    }
    if metadata.len() > MAX_FONT_FILE_BYTES {
        return Err(FontManagerError::TooLarge {
            actual: metadata.len(),
            maximum: MAX_FONT_FILE_BYTES,
        });
    }
    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| FontManagerError::UnsafeFileName(source.display().to_string()))?;
    if !safe_file_name(file_name) {
        return Err(FontManagerError::UnsafeFileName(file_name.to_string()));
    }
    Ok((file_name.to_string(), metadata.len()))
}

/// Standard typography roles across the SLOPOS-I desktop environment.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
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

    /// Generic family alias used when no requested or safe available family
    /// can be selected. These aliases are not bundled-font declarations.
    pub fn generic_fallback_family(self) -> &'static str {
        match self {
            Self::Monospace => "Monospace",
            _ => "Sans-Serif",
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
            size: size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE),
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
            FontProfile::Custom => {}
        }
        Self { profile, roles }
    }

    pub fn get_spec(&self, role: FontRole) -> FontRoleSpec {
        self.roles.get(&role).cloned().unwrap_or_else(|| {
            FontRoleSpec::new(
                role.generic_fallback_family(),
                role.default_size() as u32,
                400,
            )
        })
    }

    /// Resolve every role against an explicit set of available family names.
    ///
    /// Profile names are preferences only. In particular, Classic's
    /// historical Chicago/Geneva/Monaco names are never treated as bundled
    /// fonts: they are selected only when the caller reports a matching
    /// family in `available_families`.
    pub fn resolve<I, S>(&self, available_families: I, scale: f32) -> ResolvedFontProfile
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        FontProfileResolver::new(available_families).resolve(self, scale)
    }
}

/// A resolved role after availability, fallback, and display-scale policy is
/// applied.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResolvedFontRole {
    /// The normalized configured family name, before fallback selection.
    pub requested_family: String,
    /// The selected family, or a generic alias when no safe available family
    /// matched. This is not a claim that the family is bundled by SLOPOS-I.
    pub family: String,
    /// The clamped logical size after applying the display scale.
    pub size: u32,
    /// The clamped font weight.
    pub weight: u16,
    /// Whether the configured family was unavailable or malformed.
    pub used_fallback: bool,
}

impl ResolvedFontRole {
    pub fn as_spec(&self) -> FontRoleSpec {
        FontRoleSpec::new(self.family.clone(), self.size, self.weight)
    }
}

/// Fully resolved typography profile with deterministic role ordering.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResolvedFontProfile {
    pub profile: FontProfile,
    pub roles: BTreeMap<FontRole, ResolvedFontRole>,
}

impl ResolvedFontProfile {
    pub fn get_role(&self, role: FontRole) -> Option<&ResolvedFontRole> {
        self.roles.get(&role)
    }

    pub fn get_spec(&self, role: FontRole) -> FontRoleSpec {
        self.roles
            .get(&role)
            .map(ResolvedFontRole::as_spec)
            .unwrap_or_else(|| {
                FontRoleSpec::new(
                    role.generic_fallback_family(),
                    role.default_size() as u32,
                    400,
                )
            })
    }
}

/// Authoritative resolver for profile roles.
///
/// The available-family set is explicit and normalized once at construction,
/// so resolution never scans the filesystem or depends on `HashSet` order.
/// No family is considered bundled; a family is selected by name only when it
/// is present in this set, otherwise a safe generic alias is the final result.
#[derive(Clone, Debug, Default)]
pub struct FontProfileResolver {
    available_families: HashSet<String>,
}

impl FontProfileResolver {
    pub fn new<I, S>(available_families: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            available_families: available_families
                .into_iter()
                .map(|family| normalize_family_name(family.as_ref()))
                .filter(|family| !family.is_empty())
                .collect(),
        }
    }

    pub fn resolve(&self, config: &FontProfileConfig, scale: f32) -> ResolvedFontProfile {
        let roles = FontRole::all()
            .iter()
            .copied()
            .map(|role| {
                let configured = config.get_spec(role);
                let requested_family = canonical_family_name(&configured.family);
                let (family, used_fallback) = resolve_family(
                    config.profile,
                    role,
                    &requested_family,
                    &self.available_families,
                );
                let size = scaled_font_size(configured.size, scale);
                let weight = configured.weight.clamp(100, 900);

                (
                    role,
                    ResolvedFontRole {
                        requested_family,
                        family,
                        size,
                        weight,
                        used_fallback,
                    },
                )
            })
            .collect();

        ResolvedFontProfile {
            profile: config.profile,
            roles,
        }
    }
}

const SAFE_SANS_FALLBACKS: &[&str] = &["Noto Sans", "DejaVu Sans", "Liberation Sans", "Sans-Serif"];

const SAFE_ACCESSIBLE_SANS_FALLBACKS: &[&str] = &[
    "Atkinson Hyperlegible",
    "Noto Sans",
    "DejaVu Sans",
    "Liberation Sans",
    "Sans-Serif",
];

const SAFE_MONOSPACE_FALLBACKS: &[&str] = &[
    "Noto Sans Mono",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Monospace",
];

fn fallback_families(profile: FontProfile, role: FontRole) -> &'static [&'static str] {
    if role == FontRole::Monospace {
        SAFE_MONOSPACE_FALLBACKS
    } else if profile == FontProfile::Accessible {
        SAFE_ACCESSIBLE_SANS_FALLBACKS
    } else {
        SAFE_SANS_FALLBACKS
    }
}

fn canonical_family_name(family: &str) -> String {
    family
        .chars()
        .map(|character| match character {
            '-' | '_' => ' ',
            _ => character,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalize family names for case-, separator-, and whitespace-insensitive
/// matching while retaining a stable display spelling separately.
pub fn normalize_family_name(family: &str) -> String {
    canonical_family_name(family).to_lowercase()
}

fn resolve_family(
    profile: FontProfile,
    role: FontRole,
    requested_family: &str,
    available_families: &HashSet<String>,
) -> (String, bool) {
    let requested_key = normalize_family_name(requested_family);
    if !requested_key.is_empty() && available_families.contains(&requested_key) {
        return (requested_family.to_string(), false);
    }

    for fallback in fallback_families(profile, role) {
        let fallback_name = canonical_family_name(fallback);
        if available_families.contains(&normalize_family_name(&fallback_name)) {
            return (fallback_name, true);
        }
    }

    (role.generic_fallback_family().to_string(), true)
}

fn scaled_font_size(size: u32, scale: f32) -> u32 {
    let scale = if scale.is_finite() {
        scale.clamp(MIN_FONT_SCALE, MAX_FONT_SCALE)
    } else {
        1.0
    };
    ((size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE) as f32 * scale).round() as u32)
        .clamp(MIN_FONT_SIZE, MAX_FONT_SIZE)
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
        let mut seen_paths = HashSet::new();
        for base_path in &self.search_paths {
            discover_font_files_in_dir(base_path, &mut files, &mut seen_paths);
        }
        files
    }
}

fn discover_font_files_in_dir(
    base_path: &Path,
    files: &mut Vec<PathBuf>,
    seen_paths: &mut HashSet<PathBuf>,
) {
    let Ok(metadata) = fs::symlink_metadata(base_path) else {
        return;
    };
    if !metadata.file_type().is_dir() {
        return;
    }

    let Ok(entries) = std::fs::read_dir(base_path) else {
        return;
    };

    let mut paths = Vec::new();
    for entry in entries.flatten() {
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            discover_font_files_in_dir(&path, files, seen_paths);
            continue;
        }

        if !file_type.is_file() || !is_font_extension(&path) {
            continue;
        }

        let Ok(canonical_path) = fs::canonicalize(&path) else {
            continue;
        };
        if seen_paths.insert(canonical_path) {
            files.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "slopos-fonts-{prefix}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock moved backwards")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_font_file(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directories");
        }
        fs::write(path, b"font").expect("write font file");
    }

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

        let custom = FontProfileConfig::for_profile(FontProfile::Custom);
        assert_eq!(custom.profile, FontProfile::Custom);
        assert!(custom.roles.is_empty());
    }

    #[test]
    fn font_profile_resolver_resolves_classic_modern_accessible_and_custom_profiles() {
        let classic = FontProfileConfig::for_profile(FontProfile::Classic)
            .resolve(["chicago", "geneva", "monaco"], 1.0);
        assert_eq!(classic.profile, FontProfile::Classic);
        assert_eq!(classic.get_spec(FontRole::SystemUi).family, "Chicago");
        assert_eq!(classic.get_spec(FontRole::Body).family, "Geneva");
        assert_eq!(classic.get_spec(FontRole::Monospace).family, "Monaco");

        let modern = FontProfileConfig::for_profile(FontProfile::Modern)
            .resolve([" INTER ", "jetbrains-mono"], 1.0);
        assert_eq!(modern.get_spec(FontRole::SystemUi).family, "Inter");
        assert_eq!(
            modern.get_spec(FontRole::Monospace).family,
            "JetBrains Mono"
        );

        let accessible = FontProfileConfig::for_profile(FontProfile::Accessible)
            .resolve(["atkinson hyperlegible", "JETBRAINS MONO"], 1.0);
        assert_eq!(
            accessible.get_spec(FontRole::SystemUi).family,
            "Atkinson Hyperlegible"
        );
        assert_eq!(
            accessible.get_spec(FontRole::Monospace).family,
            "JetBrains Mono"
        );

        let mut custom = FontProfileConfig::for_profile(FontProfile::Custom);
        custom.roles.insert(
            FontRole::SystemUi,
            FontRoleSpec::new("  Nimbus   Sans ", 20, 500),
        );
        custom
            .roles
            .insert(FontRole::Monospace, FontRoleSpec::new("My_Mono", 10, 300));
        let custom = custom.resolve(["nImBuS sAnS", "my mono"], 1.25);
        assert_eq!(custom.profile, FontProfile::Custom);
        assert_eq!(custom.get_spec(FontRole::SystemUi).family, "Nimbus Sans");
        assert_eq!(custom.get_spec(FontRole::SystemUi).size, 25);
        assert_eq!(custom.get_spec(FontRole::Monospace).family, "My Mono");
        assert_eq!(custom.get_spec(FontRole::Monospace).size, 13);
    }

    #[test]
    fn font_profile_resolver_uses_safe_fallbacks_for_unavailable_families() {
        let classic = FontProfileConfig::for_profile(FontProfile::Classic)
            .resolve(["Noto Sans", "Noto Sans Mono"], 1.0);

        assert_eq!(classic.get_spec(FontRole::SystemUi).family, "Noto Sans");
        assert_eq!(classic.get_spec(FontRole::Body).family, "Noto Sans");
        assert_eq!(
            classic.get_spec(FontRole::Monospace).family,
            "Noto Sans Mono"
        );
        assert!(
            classic
                .get_role(FontRole::SystemUi)
                .expect("system UI role")
                .used_fallback
        );
        assert!(!["Chicago", "Geneva", "Monaco"]
            .contains(&classic.get_spec(FontRole::SystemUi).family.as_str()));

        let mut custom = FontProfileConfig::for_profile(FontProfile::Custom);
        custom.roles.insert(
            FontRole::Body,
            FontRoleSpec::new("Unavailable Family", 13, 400),
        );
        let custom = custom.resolve(["DejaVu Sans"], 1.0);
        assert_eq!(custom.get_spec(FontRole::Body).family, "DejaVu Sans");
        assert!(
            custom
                .get_role(FontRole::Body)
                .expect("body role")
                .used_fallback
        );

        let empty = FontProfileConfig::for_profile(FontProfile::Classic)
            .resolve(std::iter::empty::<&str>(), 1.0);
        assert_eq!(empty.get_spec(FontRole::SystemUi).family, "Sans-Serif");
        assert_eq!(empty.get_spec(FontRole::Monospace).family, "Monospace");
    }

    #[test]
    fn font_profile_resolver_has_stable_fallback_order_and_matching() {
        let config = FontProfileConfig::for_profile(FontProfile::Modern);
        let first = config.resolve(
            [
                "Liberation Sans",
                "DejaVu Sans",
                "Noto Sans",
                "Liberation Mono",
                "DejaVu Sans Mono",
            ],
            1.0,
        );
        let second = config.resolve(
            [
                "DejaVu Sans Mono",
                "Noto Sans",
                "Liberation Mono",
                "DejaVu Sans",
                "Liberation Sans",
            ],
            1.0,
        );

        assert_eq!(first, second);
        assert_eq!(first.get_spec(FontRole::SystemUi).family, "Noto Sans");
        assert_eq!(
            first.get_spec(FontRole::Monospace).family,
            "DejaVu Sans Mono"
        );
        assert_eq!(
            normalize_family_name("  JetBrains-MONO  "),
            "jetbrains mono"
        );
    }

    #[test]
    fn font_profile_resolver_sanitizes_malformed_profile_values() {
        assert_eq!(FontProfile::parse("  ACCESSIBLE "), FontProfile::Accessible);
        assert_eq!(FontProfile::parse("unknown-profile"), FontProfile::Modern);
        assert_eq!(FontProfile::parse(""), FontProfile::Modern);

        let mut malformed = FontProfileConfig::for_profile(FontProfile::Custom);
        malformed.roles.insert(
            FontRole::SystemUi,
            FontRoleSpec {
                family: " \t".to_string(),
                size: u32::MAX,
                weight: u16::MAX,
            },
        );
        let resolved = malformed.resolve(std::iter::empty::<&str>(), f32::NAN);
        let system_ui = resolved
            .get_role(FontRole::SystemUi)
            .expect("system UI role");
        assert_eq!(system_ui.requested_family, "");
        assert_eq!(system_ui.family, "Sans-Serif");
        assert_eq!(system_ui.size, MAX_FONT_SIZE);
        assert_eq!(system_ui.weight, 900);
        assert!(system_ui.used_fallback);

        malformed.roles.insert(
            FontRole::Body,
            FontRoleSpec {
                family: "Inter".to_string(),
                size: 0,
                weight: 0,
            },
        );
        let body = malformed.resolve(["Inter"], 2.0).get_spec(FontRole::Body);
        assert_eq!(body.family, "Inter");
        assert_eq!(body.size, MIN_FONT_SIZE * 2);
        assert_eq!(body.weight, 100);
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

    #[test]
    fn test_font_discovery_recurses_into_nested_directories() {
        let temp_dir = make_temp_dir("recursive");
        let root_font = temp_dir.join("root.ttf");
        let nested_font = temp_dir.join("nested").join("family.otf");
        let deep_font = temp_dir.join("nested").join("deep").join("mono.ttc");
        let ignored = temp_dir.join("nested").join("notes.txt");

        write_font_file(&root_font);
        write_font_file(&nested_font);
        write_font_file(&deep_font);
        write_font_file(&ignored);

        let service = FontDiscoveryService {
            search_paths: vec![temp_dir.clone()],
        };
        let discovered = service.discover_font_files();

        assert!(discovered.contains(&root_font));
        assert!(discovered.contains(&nested_font));
        assert!(discovered.contains(&deep_font));
        assert!(!discovered.contains(&ignored));

        fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
    }

    #[test]
    fn font_discovery_preserves_root_precedence_and_deduplicates_overlapping_roots() {
        let temp_dir = make_temp_dir("overlap");
        let root_font = temp_dir.join("00-root.ttf");
        let nested_dir = temp_dir.join("01-nested");
        let shared_font = nested_dir.join("00-shared.otf");
        let later_font = nested_dir.join("01-later.ttc");

        write_font_file(&root_font);
        write_font_file(&shared_font);
        write_font_file(&later_font);

        let service = FontDiscoveryService {
            search_paths: vec![temp_dir.clone(), nested_dir.clone()],
        };
        let discovered = service.discover_font_files();

        assert_eq!(discovered, vec![root_font, shared_font, later_font]);

        fs::remove_dir_all(&temp_dir).expect("cleanup overlap temp dir");
    }

    #[cfg(unix)]
    #[test]
    fn font_discovery_skips_symlinked_fonts_and_directories() {
        use std::os::unix::fs::symlink;

        let temp_dir = make_temp_dir("symlinks");
        let real_font = temp_dir.join("real.ttf");
        let nested_dir = temp_dir.join("nested");
        let nested_font = nested_dir.join("nested.otf");
        let linked_font = temp_dir.join("linked.ttc");
        let linked_dir = temp_dir.join("linked-directory");
        let cycle = nested_dir.join("cycle");

        write_font_file(&real_font);
        write_font_file(&nested_font);
        symlink(&real_font, &linked_font).expect("create symlinked font");
        symlink(&nested_dir, &linked_dir).expect("create symlinked directory");
        symlink(&temp_dir, &cycle).expect("create directory cycle");

        let service = FontDiscoveryService {
            search_paths: vec![temp_dir.clone()],
        };
        let discovered = service.discover_font_files();

        assert_eq!(discovered, vec![nested_font, real_font]);
        assert!(!discovered.contains(&linked_font));
        assert!(!discovered.contains(&linked_dir));
        assert!(!discovered.iter().any(|path| path.starts_with(&cycle)));

        fs::remove_dir_all(&temp_dir).expect("cleanup symlink temp dir");
    }

    #[cfg(unix)]
    #[test]
    fn test_font_discovery_skips_unreadable_nested_directories() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = make_temp_dir("unreadable");
        let readable_font = temp_dir.join("readable").join("ok.ttf");
        let unreadable_dir = temp_dir.join("private");
        let unreadable_font = unreadable_dir.join("hidden.otf");

        write_font_file(&readable_font);
        write_font_file(&unreadable_font);

        let mut permissions = fs::metadata(&unreadable_dir)
            .expect("unreadable dir metadata")
            .permissions();
        permissions.set_mode(0o000);
        fs::set_permissions(&unreadable_dir, permissions).expect("make unreadable");

        let service = FontDiscoveryService {
            search_paths: vec![temp_dir.clone()],
        };
        let discovered = service.discover_font_files();

        assert!(discovered.contains(&readable_font));
        assert!(!discovered.contains(&unreadable_font));

        let mut permissions = fs::metadata(&unreadable_dir)
            .expect("restore dir metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&unreadable_dir, permissions).expect("restore permissions");
        fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
    }

    #[test]
    fn font_manager_installs_deduplicates_toggles_and_removes_atomically() {
        let temp_dir = make_temp_dir("manager");
        let source = temp_dir.join("Nested Family.ttf");
        let install_dir = temp_dir.join("fonts");
        fs::write(&source, b"valid font bytes").expect("write source font");
        let manager = FontManager::new(&install_dir);

        let installed = manager.install(&source).expect("install font");
        assert_eq!(installed.file_name, "Nested Family.ttf");
        assert!(installed.enabled);
        assert_eq!(manager.installed_fonts().unwrap().len(), 1);

        let duplicate = manager.install(&source).expect("deduplicate font");
        assert_eq!(duplicate.sha256, installed.sha256);
        assert_eq!(manager.installed_fonts().unwrap().len(), 1);

        let disabled = manager
            .set_enabled(&installed.file_name, false)
            .expect("disable font");
        assert!(!disabled.enabled);
        let enabled = manager
            .set_enabled(&installed.file_name, true)
            .expect("enable font");
        assert!(enabled.enabled);

        manager.remove(&installed.file_name).expect("remove font");
        assert!(manager.installed_fonts().unwrap().is_empty());
        fs::remove_dir_all(temp_dir).expect("cleanup manager temp dir");
    }

    #[test]
    fn font_manager_rejects_unsafe_or_non_font_sources() {
        let temp_dir = make_temp_dir("manager-validation");
        let install_dir = temp_dir.join("fonts");
        let bad_extension = temp_dir.join("notes.txt");
        fs::write(&bad_extension, b"not a font").expect("write invalid source");
        let manager = FontManager::new(&install_dir);

        assert!(matches!(
            manager.install(&bad_extension),
            Err(FontManagerError::InvalidSource(_))
        ));
        assert!(matches!(
            manager.remove("../escape.ttf"),
            Err(FontManagerError::UnsafeFileName(_))
        ));
        fs::remove_dir_all(temp_dir).expect("cleanup validation temp dir");
    }
}
