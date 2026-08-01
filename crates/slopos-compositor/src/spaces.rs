use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stable, nonzero identity for a dynamic SLOPOS Space.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpaceId(NonZeroU64);

impl SpaceId {
    /// Construct an ID, rejecting the reserved zero value.
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    /// Return the stable numeric representation.
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for SpaceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(formatter)
    }
}

/// Whether a Space is a normal workspace or a fullscreen classification.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FullscreenClassification {
    #[default]
    Normal,
    Fullscreen,
}

/// How Spaces are associated with multiple displays.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiMonitorPolicy {
    /// One ordered Space set spans all displays.
    #[default]
    SharedSpan,
    /// Each display may own an independent ordered Space set.
    IndependentPerDisplay,
}

/// The membership scope used when assigning a window.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceTarget {
    Current,
    Named(String),
    All,
}

/// Errors returned by the pure Spaces state model and its persistence helpers.
#[derive(Debug)]
pub enum SpacesError {
    EmptySpaces,
    InvalidSpaceId(u64),
    DuplicateSpaceId(SpaceId),
    InvalidSpaceName(String),
    DuplicateSpaceName(String),
    ActiveSpaceMissing(SpaceId),
    InvalidNextSpaceId(u64),
    NextSpaceIdNotAfterExisting { next: u64, maximum_existing: u64 },
    SpaceNotFound(SpaceId),
    SpaceNameNotFound(String),
    CannotRemoveLastSpace,
    InvalidOrderIndex { index: usize, len: usize },
    InvalidWindowId(String),
    DuplicateWindowId(String),
    InvalidMetadata { field: &'static str, value: String },
    SpaceIdExhausted,
    InvalidPath(PathBuf),
    Io { path: PathBuf, source: io::Error },
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
}

impl fmt::Display for SpacesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySpaces => write!(formatter, "at least one Space is required"),
            Self::InvalidSpaceId(id) => {
                write!(formatter, "Space ID {id} is invalid; IDs are nonzero")
            }
            Self::DuplicateSpaceId(id) => write!(formatter, "duplicate Space ID {id}"),
            Self::InvalidSpaceName(name) => write!(formatter, "invalid Space name {name:?}"),
            Self::DuplicateSpaceName(name) => write!(formatter, "duplicate Space name {name:?}"),
            Self::ActiveSpaceMissing(id) => write!(formatter, "active Space {id} does not exist"),
            Self::InvalidNextSpaceId(id) => {
                write!(formatter, "next Space ID {id} is invalid; IDs are nonzero")
            }
            Self::NextSpaceIdNotAfterExisting {
                next,
                maximum_existing,
            } => write!(
                formatter,
                "next Space ID {next} must be greater than existing maximum {maximum_existing}"
            ),
            Self::SpaceNotFound(id) => write!(formatter, "Space {id} does not exist"),
            Self::SpaceNameNotFound(name) => {
                write!(formatter, "Space named {name:?} does not exist")
            }
            Self::CannotRemoveLastSpace => write!(formatter, "the last Space cannot be removed"),
            Self::InvalidOrderIndex { index, len } => {
                write!(formatter, "order index {index} is outside 0..{len}")
            }
            Self::InvalidWindowId(id) => write!(formatter, "invalid window ID {id:?}"),
            Self::DuplicateWindowId(id) => write!(formatter, "duplicate window ID {id:?}"),
            Self::InvalidMetadata { field, value } => {
                write!(formatter, "invalid {field} value {value:?}")
            }
            Self::SpaceIdExhausted => write!(formatter, "no stable Space IDs remain"),
            Self::InvalidPath(path) => {
                write!(formatter, "path has no file name: {}", path.display())
            }
            Self::Io { path, source } => {
                write!(formatter, "I/O error for {}: {source}", path.display())
            }
            Self::Serialize(source) => write!(formatter, "Space serialization failed: {source}"),
            Self::Deserialize(source) => {
                write!(formatter, "Space deserialization failed: {source}")
            }
        }
    }
}

impl Error for SpacesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Serialize(source) | Self::Deserialize(source) => Some(source),
            _ => None,
        }
    }
}

/// One ordered Space and its shell-facing metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawSpace")]
pub struct Space {
    id: SpaceId,
    name: String,
    wallpaper: Option<String>,
    appearance: Option<String>,
    classification: FullscreenClassification,
    windows: Vec<String>,
}

impl Space {
    pub fn new(id: SpaceId, name: impl Into<String>) -> Result<Self, SpacesError> {
        let space = Self {
            id,
            name: name.into(),
            wallpaper: None,
            appearance: None,
            classification: FullscreenClassification::Normal,
            windows: Vec::new(),
        };
        space.validate()?;
        Ok(space)
    }

    pub const fn id(&self) -> SpaceId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn wallpaper(&self) -> Option<&str> {
        self.wallpaper.as_deref()
    }

    pub fn appearance(&self) -> Option<&str> {
        self.appearance.as_deref()
    }

    pub const fn classification(&self) -> FullscreenClassification {
        self.classification
    }

    pub fn windows(&self) -> &[String] {
        &self.windows
    }

    fn validate(&self) -> Result<(), SpacesError> {
        if self.id.get() == 0 {
            return Err(SpacesError::InvalidSpaceId(0));
        }
        validate_space_name(&self.name)?;
        validate_metadata("wallpaper", self.wallpaper.as_deref())?;
        validate_metadata("appearance", self.appearance.as_deref())?;

        let mut seen = BTreeSet::new();
        for window in &self.windows {
            validate_window_id(window)?;
            if !seen.insert(window) {
                return Err(SpacesError::DuplicateWindowId(window.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct RawSpace {
    id: u64,
    name: String,
    #[serde(default)]
    wallpaper: Option<String>,
    #[serde(default)]
    appearance: Option<String>,
    #[serde(default)]
    classification: FullscreenClassification,
    #[serde(default)]
    windows: Vec<String>,
}

impl TryFrom<RawSpace> for Space {
    type Error = SpacesError;

    fn try_from(raw: RawSpace) -> Result<Self, Self::Error> {
        let id = SpaceId::new(raw.id).ok_or(SpacesError::InvalidSpaceId(raw.id))?;
        let space = Self {
            id,
            name: raw.name,
            wallpaper: raw.wallpaper,
            appearance: raw.appearance,
            classification: raw.classification,
            windows: raw.windows,
        };
        space.validate()?;
        Ok(space)
    }
}

/// Pure, serializable dynamic Spaces state. Window IDs are opaque strings; the
/// compositor remains the owner of actual window geometry and protocol state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawSpacesModel")]
pub struct SpacesModel {
    spaces: Vec<Space>,
    active_space: SpaceId,
    next_space_id: NonZeroU64,
    multi_monitor_policy: MultiMonitorPolicy,
}

impl Default for SpacesModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SpacesModel {
    pub fn new() -> Self {
        let first = SpaceId::new(1).expect("literal one is a valid Space ID");
        Self {
            spaces: vec![Space::new(first, "Space 1").expect("default Space name is valid")],
            active_space: first,
            next_space_id: NonZeroU64::new(2).expect("literal two is a valid Space ID"),
            multi_monitor_policy: MultiMonitorPolicy::SharedSpan,
        }
    }

    pub fn with_initial_name(name: impl Into<String>) -> Result<Self, SpacesError> {
        let mut model = Self::new();
        let first = model.active_space;
        model.spaces[0] = Space::new(first, name)?;
        Ok(model)
    }

    pub fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    pub fn space_ids(&self) -> Vec<SpaceId> {
        self.spaces.iter().map(Space::id).collect()
    }

    pub const fn active_space(&self) -> SpaceId {
        self.active_space
    }

    pub fn active(&self) -> &Space {
        self.space(self.active_space)
            .expect("validated model always has an active Space")
    }

    pub fn space(&self, id: SpaceId) -> Option<&Space> {
        self.spaces.iter().find(|space| space.id == id)
    }

    pub fn space_by_name(&self, name: &str) -> Option<&Space> {
        self.spaces.iter().find(|space| space.name == name)
    }

    pub fn position_of(&self, id: SpaceId) -> Option<usize> {
        self.spaces.iter().position(|space| space.id == id)
    }

    pub const fn multi_monitor_policy(&self) -> MultiMonitorPolicy {
        self.multi_monitor_policy
    }

    pub fn set_multi_monitor_policy(&mut self, policy: MultiMonitorPolicy) {
        self.multi_monitor_policy = policy;
    }

    pub fn create_space(&mut self, name: impl Into<String>) -> Result<SpaceId, SpacesError> {
        let name = name.into();
        validate_space_name(&name)?;
        self.ensure_unique_name(&name, None)?;

        let id = SpaceId(self.next_space_id);
        let next = id
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(SpacesError::SpaceIdExhausted)?;
        self.next_space_id = next;
        self.spaces.push(Space::new(id, name)?);
        Ok(id)
    }

    pub fn rename_space(
        &mut self,
        id: SpaceId,
        name: impl Into<String>,
    ) -> Result<(), SpacesError> {
        let name = name.into();
        validate_space_name(&name)?;
        self.space(id).ok_or(SpacesError::SpaceNotFound(id))?;
        self.ensure_unique_name(&name, Some(id))?;
        self.space_mut(id)?.name = name;
        Ok(())
    }

    pub fn activate_space(&mut self, id: SpaceId) -> Result<(), SpacesError> {
        self.space(id).ok_or(SpacesError::SpaceNotFound(id))?;
        self.active_space = id;
        Ok(())
    }

    pub fn reorder_space(&mut self, id: SpaceId, new_index: usize) -> Result<(), SpacesError> {
        if new_index >= self.spaces.len() {
            return Err(SpacesError::InvalidOrderIndex {
                index: new_index,
                len: self.spaces.len(),
            });
        }
        let old_index = self.position_of(id).ok_or(SpacesError::SpaceNotFound(id))?;
        let space = self.spaces.remove(old_index);
        self.spaces.insert(new_index, space);
        Ok(())
    }

    /// Remove a Space and return the active fallback. The following ordered
    /// Space is preferred for an active removal, then the preceding one. Any
    /// window that would otherwise lose all membership is moved to that
    /// fallback so the model never strands a window.
    pub fn remove_space(&mut self, id: SpaceId) -> Result<SpaceId, SpacesError> {
        let index = self.position_of(id).ok_or(SpacesError::SpaceNotFound(id))?;
        if self.spaces.len() == 1 {
            return Err(SpacesError::CannotRemoveLastSpace);
        }
        let fallback = if id == self.active_space {
            let fallback_index = if index + 1 < self.spaces.len() {
                index + 1
            } else {
                index - 1
            };
            self.spaces[fallback_index].id
        } else {
            self.active_space
        };
        let removed = self.spaces.remove(index);
        if id == self.active_space {
            self.active_space = fallback;
        }

        for window in removed.windows {
            if !self
                .spaces
                .iter()
                .any(|space| space.windows.iter().any(|candidate| candidate == &window))
            {
                self.space_mut(fallback)?.windows.push(window);
            }
        }
        Ok(fallback)
    }

    pub fn set_wallpaper(
        &mut self,
        id: SpaceId,
        wallpaper: Option<String>,
    ) -> Result<(), SpacesError> {
        validate_metadata("wallpaper", wallpaper.as_deref())?;
        self.space_mut(id)?.wallpaper = wallpaper;
        Ok(())
    }

    pub fn set_appearance(
        &mut self,
        id: SpaceId,
        appearance: Option<String>,
    ) -> Result<(), SpacesError> {
        validate_metadata("appearance", appearance.as_deref())?;
        self.space_mut(id)?.appearance = appearance;
        Ok(())
    }

    pub fn set_classification(
        &mut self,
        id: SpaceId,
        classification: FullscreenClassification,
    ) -> Result<(), SpacesError> {
        self.space_mut(id)?.classification = classification;
        Ok(())
    }

    /// Assign a window to the current Space, a uniquely named Space, or every
    /// Space. Current and named assignments are exclusive; All is inclusive.
    pub fn assign_window(
        &mut self,
        window: impl Into<String>,
        target: SpaceTarget,
    ) -> Result<(), SpacesError> {
        let window = window.into();
        validate_window_id(&window)?;
        let target_ids = self.target_ids(&target)?;

        for space in &mut self.spaces {
            space.windows.retain(|candidate| candidate != &window);
        }
        for id in target_ids {
            self.space_mut(id)?.windows.push(window.clone());
        }
        Ok(())
    }

    pub fn move_window(
        &mut self,
        window: impl Into<String>,
        target: SpaceTarget,
    ) -> Result<(), SpacesError> {
        self.assign_window(window, target)
    }

    pub fn assign_window_to_current(
        &mut self,
        window: impl Into<String>,
    ) -> Result<(), SpacesError> {
        self.assign_window(window, SpaceTarget::Current)
    }

    pub fn assign_window_to_named(
        &mut self,
        window: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(), SpacesError> {
        self.assign_window(window, SpaceTarget::Named(name.into()))
    }

    pub fn assign_window_to_all(&mut self, window: impl Into<String>) -> Result<(), SpacesError> {
        self.assign_window(window, SpaceTarget::All)
    }

    pub fn windows_in_space(&self, id: SpaceId) -> Option<&[String]> {
        self.space(id).map(Space::windows)
    }

    pub fn window_spaces(&self, window: &str) -> Vec<SpaceId> {
        self.spaces
            .iter()
            .filter(|space| space.windows.iter().any(|candidate| candidate == window))
            .map(Space::id)
            .collect()
    }

    pub fn remove_window(&mut self, window: &str) -> bool {
        let mut removed = false;
        for space in &mut self.spaces {
            let old_len = space.windows.len();
            space.windows.retain(|candidate| candidate != window);
            removed |= old_len != space.windows.len();
        }
        removed
    }

    /// Validate invariants before persistence or after a caller deserializes a
    /// value through another serde format.
    pub fn validate(&self) -> Result<(), SpacesError> {
        if self.spaces.is_empty() {
            return Err(SpacesError::EmptySpaces);
        }

        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut maximum_existing = 0;
        for space in &self.spaces {
            space.validate()?;
            if !ids.insert(space.id) {
                return Err(SpacesError::DuplicateSpaceId(space.id));
            }
            let normalized = normalize_name(space.name());
            if !names.insert(normalized) {
                return Err(SpacesError::DuplicateSpaceName(space.name.clone()));
            }
            maximum_existing = maximum_existing.max(space.id.get());
        }

        if !ids.contains(&self.active_space) {
            return Err(SpacesError::ActiveSpaceMissing(self.active_space));
        }
        if self.next_space_id.get() <= maximum_existing {
            return Err(SpacesError::NextSpaceIdNotAfterExisting {
                next: self.next_space_id.get(),
                maximum_existing,
            });
        }
        Ok(())
    }

    pub fn save_atomic(&self, path: impl AsRef<Path>) -> Result<(), SpacesError> {
        self.validate()?;
        let path = path.as_ref();
        let encoded = serde_json::to_vec_pretty(self).map_err(SpacesError::Serialize)?;
        atomic_write(path, &encoded)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, SpacesError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| SpacesError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(SpacesError::Deserialize)
    }

    fn ensure_unique_name(&self, name: &str, except: Option<SpaceId>) -> Result<(), SpacesError> {
        let normalized = normalize_name(name);
        if self
            .spaces
            .iter()
            .any(|space| Some(space.id) != except && normalize_name(space.name()) == normalized)
        {
            return Err(SpacesError::DuplicateSpaceName(name.to_owned()));
        }
        Ok(())
    }

    fn target_ids(&self, target: &SpaceTarget) -> Result<Vec<SpaceId>, SpacesError> {
        match target {
            SpaceTarget::Current => Ok(vec![self.active_space]),
            SpaceTarget::All => Ok(self.space_ids()),
            SpaceTarget::Named(name) => self
                .space_by_name(name)
                .map(|space| vec![space.id])
                .ok_or_else(|| SpacesError::SpaceNameNotFound(name.clone())),
        }
    }

    fn space_mut(&mut self, id: SpaceId) -> Result<&mut Space, SpacesError> {
        self.spaces
            .iter_mut()
            .find(|space| space.id == id)
            .ok_or(SpacesError::SpaceNotFound(id))
    }
}

#[derive(Deserialize)]
struct RawSpacesModel {
    spaces: Vec<Space>,
    active_space: u64,
    next_space_id: u64,
    #[serde(default)]
    multi_monitor_policy: MultiMonitorPolicy,
}

impl TryFrom<RawSpacesModel> for SpacesModel {
    type Error = SpacesError;

    fn try_from(raw: RawSpacesModel) -> Result<Self, Self::Error> {
        let active_space =
            SpaceId::new(raw.active_space).ok_or(SpacesError::InvalidSpaceId(raw.active_space))?;
        let next_space_id = NonZeroU64::new(raw.next_space_id)
            .ok_or(SpacesError::InvalidNextSpaceId(raw.next_space_id))?;
        let model = Self {
            spaces: raw.spaces,
            active_space,
            next_space_id,
            multi_monitor_policy: raw.multi_monitor_policy,
        };
        model.validate()?;
        Ok(model)
    }
}

fn validate_space_name(name: &str) -> Result<(), SpacesError> {
    if name.is_empty()
        || name.trim() != name
        || name
            .chars()
            .any(|character| character == '\0' || character.is_control())
    {
        return Err(SpacesError::InvalidSpaceName(name.to_owned()));
    }
    Ok(())
}

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
}

fn validate_window_id(window: &str) -> Result<(), SpacesError> {
    if window.is_empty()
        || window
            .chars()
            .any(|character| character == '\0' || character.is_control())
    {
        return Err(SpacesError::InvalidWindowId(window.to_owned()));
    }
    Ok(())
}

fn validate_metadata(field: &'static str, value: Option<&str>) -> Result<(), SpacesError> {
    if let Some(value) = value {
        if value.is_empty()
            || value
                .chars()
                .any(|character| character == '\0' || character.is_control())
        {
            return Err(SpacesError::InvalidMetadata {
                field,
                value: value.to_owned(),
            });
        }
    }
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), SpacesError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| SpacesError::InvalidPath(path.to_path_buf()))?
        .to_string_lossy();

    let mut temp_path = None;
    let mut temp_file = None;
    for _ in 0..100 {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{file_name}.{counter}.tmp"));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&candidate)
        {
            Ok(file) => {
                temp_path = Some(candidate);
                temp_file = Some(file);
                break;
            }
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(SpacesError::Io {
                    path: candidate,
                    source,
                });
            }
        }
    }

    let temp_path = temp_path.ok_or_else(|| SpacesError::Io {
        path: path.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate temporary path",
        ),
    })?;
    let mut temp_file = temp_file.expect("temporary path and file are created together");

    let result = (|| {
        temp_file
            .write_all(bytes)
            .map_err(|source| SpacesError::Io {
                path: temp_path.clone(),
                source,
            })?;
        temp_file.sync_all().map_err(|source| SpacesError::Io {
            path: temp_path.clone(),
            source,
        })?;
        drop(temp_file);
        fs::rename(&temp_path, path).map_err(|source| SpacesError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn save_spaces_atomic(model: &SpacesModel, path: impl AsRef<Path>) -> Result<(), SpacesError> {
    model.save_atomic(path)
}

pub fn load_spaces(path: impl AsRef<Path>) -> Result<SpacesModel, SpacesError> {
    SpacesModel::load(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path(label: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "slopos-spaces-{label}-{}-{id}.json",
            std::process::id()
        ))
    }

    #[test]
    fn lifecycle_keeps_stable_nonzero_ids_and_unique_names() {
        let mut model = SpacesModel::new();
        let first = model.active_space();
        let second = model.create_space("Work").expect("create work space");

        assert_ne!(first.get(), 0);
        assert_ne!(first, second);
        assert_eq!(model.space_ids(), &[first, second]);
        assert!(model.rename_space(second, "Projects").is_ok());
        assert_eq!(model.space(second).expect("projects").name(), "Projects");
        assert!(matches!(
            model.rename_space(first, "projects"),
            Err(SpacesError::DuplicateSpaceName(_))
        ));

        model.remove_space(second).expect("remove projects");
        let recreated = model.create_space("Recreated").expect("recreate space");
        assert!(recreated.get() > second.get());
    }

    #[test]
    fn ordering_and_active_space_are_independent() {
        let mut model = SpacesModel::new();
        let first = model.active_space();
        let work = model.create_space("Work").expect("work");
        let play = model.create_space("Play").expect("play");

        model.activate_space(play).expect("activate play");
        model.reorder_space(play, 0).expect("move play first");

        assert_eq!(model.space_ids(), &[play, first, work]);
        assert_eq!(model.active_space(), play);
        assert_eq!(model.position_of(work), Some(2));
    }

    #[test]
    fn membership_targets_current_named_and_all() {
        let mut model = SpacesModel::new();
        let work = model.create_space("Work").expect("work");
        let play = model.create_space("Play").expect("play");

        model
            .assign_window("finder", SpaceTarget::Current)
            .expect("current assignment");
        model
            .assign_window("editor", SpaceTarget::Named("Work".into()))
            .expect("named assignment");
        model
            .assign_window("terminal", SpaceTarget::All)
            .expect("all assignment");

        assert_eq!(model.window_spaces("finder"), vec![model.space_ids()[0]]);
        assert_eq!(model.window_spaces("editor"), vec![work]);
        assert_eq!(model.window_spaces("terminal"), model.space_ids());
        assert!(model
            .windows_in_space(play)
            .expect("play space")
            .contains(&"terminal".to_string()));

        model
            .move_window("finder", SpaceTarget::Named("Play".into()))
            .expect("move finder");
        assert_eq!(model.window_spaces("finder"), vec![play]);
    }

    #[test]
    fn removing_active_or_last_space_has_safe_fallback() {
        let mut model = SpacesModel::new();
        let first = model.active_space();
        let second = model.create_space("Second").expect("second");
        let third = model.create_space("Third").expect("third");
        model
            .assign_window("only-second", SpaceTarget::Named("Second".into()))
            .expect("assign orphan candidate");
        model.activate_space(second).expect("activate second");

        let fallback = model.remove_space(second).expect("remove active");
        assert_eq!(fallback, third);
        assert_eq!(model.active_space(), third);
        assert_eq!(model.window_spaces("only-second"), vec![third]);

        model.activate_space(first).expect("activate first");
        model.remove_space(third).expect("remove third");
        assert_eq!(model.active_space(), first);
        assert!(matches!(
            model.remove_space(first),
            Err(SpacesError::CannotRemoveLastSpace)
        ));
        assert_eq!(model.space_ids(), &[first]);
    }

    #[test]
    fn policy_and_per_space_presentation_metadata_round_trip() {
        let mut model = SpacesModel::new();
        let fullscreen = model.create_space("Video").expect("video");
        model
            .set_wallpaper(fullscreen, Some("wallpapers/video.png".into()))
            .expect("wallpaper");
        model
            .set_appearance(fullscreen, Some("graphite".into()))
            .expect("appearance");
        model
            .set_classification(fullscreen, FullscreenClassification::Fullscreen)
            .expect("classification");
        model.set_multi_monitor_policy(MultiMonitorPolicy::IndependentPerDisplay);

        assert_eq!(
            model.space(fullscreen).expect("video").wallpaper(),
            Some("wallpapers/video.png")
        );
        assert_eq!(
            model.space(fullscreen).expect("video").appearance(),
            Some("graphite")
        );
        assert_eq!(
            model.space(fullscreen).expect("video").classification(),
            FullscreenClassification::Fullscreen
        );
        assert_eq!(
            model.multi_monitor_policy(),
            MultiMonitorPolicy::IndependentPerDisplay
        );

        let encoded = serde_json::to_string(&model).expect("serialize metadata");
        let decoded: SpacesModel = serde_json::from_str(&encoded).expect("deserialize metadata");
        assert_eq!(decoded, model);
    }

    #[test]
    fn serde_rejects_zero_or_duplicate_ids_and_duplicate_names() {
        let zero_id = r#"{
            "spaces": [{"id": 0, "name": "Main", "windows": []}],
            "active_space": 0,
            "next_space_id": 2,
            "multi_monitor_policy": "shared_span"
        }"#;
        assert!(serde_json::from_str::<SpacesModel>(zero_id).is_err());

        let duplicate_id = r#"{
            "spaces": [
                {"id": 1, "name": "Main", "windows": []},
                {"id": 1, "name": "Other", "windows": []}
            ],
            "active_space": 1,
            "next_space_id": 2,
            "multi_monitor_policy": "shared_span"
        }"#;
        assert!(serde_json::from_str::<SpacesModel>(duplicate_id).is_err());

        let duplicate_name = r#"{
            "spaces": [
                {"id": 1, "name": "Main", "windows": []},
                {"id": 2, "name": "main", "windows": []}
            ],
            "active_space": 1,
            "next_space_id": 3,
            "multi_monitor_policy": "shared_span"
        }"#;
        assert!(serde_json::from_str::<SpacesModel>(duplicate_name).is_err());
    }

    #[test]
    fn atomic_persistence_round_trips_and_leaves_no_temp_file() {
        let path = temp_path("atomic");
        let _ = fs::remove_file(&path);

        let mut model = SpacesModel::new();
        model.create_space("Work").expect("work");
        model.save_atomic(&path).expect("save spaces atomically");

        let loaded = SpacesModel::load(&path).expect("load spaces");
        assert_eq!(loaded, model);
        let directory = path.parent().expect("temp directory");
        let prefix = format!(".{}.", path.file_name().unwrap().to_string_lossy());
        assert!(!fs::read_dir(directory)
            .expect("read temp directory")
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().starts_with(&prefix)));

        let sentinel = temp_path("sentinel");
        let target_directory = temp_path("target-directory");
        fs::write(&sentinel, b"keep this file").expect("write sentinel");
        fs::create_dir(&target_directory).expect("create directory target");
        assert!(model.save_atomic(&target_directory).is_err());
        assert_eq!(
            fs::read(&sentinel).expect("read sentinel"),
            b"keep this file"
        );

        fs::remove_file(path).expect("remove test state");
        fs::remove_file(sentinel).expect("remove sentinel");
        fs::remove_dir(target_directory).expect("remove directory target");
    }
}
