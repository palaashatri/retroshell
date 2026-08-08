//! Wire-safe control and snapshot types for compositor-authoritative Spaces.
//!
//! The compositor owns the mutable model.  The shell receives a compact
//! projection here and sends typed commands back through the session control
//! socket; it never edits compositor window state directly.

use serde::{Deserialize, Serialize};

/// Space-level fullscreen policy exposed across the compositor/session bus.
///
/// The wire enum intentionally lives in the bus crate so shell and compositor
/// clients can exchange the persisted policy without depending on each
/// other's implementation types.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceClassification {
    #[default]
    Normal,
    Fullscreen,
}

/// Multi-display policy for the compositor-owned Spaces set.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpacesDisplayPolicy {
    #[default]
    SharedSpan,
    IndependentPerDisplay,
}

/// A command that changes the compositor-owned Spaces model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SpacesControlCommand {
    Select {
        id: u64,
    },
    Create {
        name: String,
    },
    Rename {
        id: u64,
        name: String,
    },
    Reorder {
        id: u64,
        order: usize,
    },
    Remove {
        id: u64,
    },
    MoveWindow {
        window_id: String,
        target: SpaceTargetWire,
    },
    SetWallpaper {
        id: u64,
        wallpaper: Option<String>,
    },
    SetAppearance {
        id: u64,
        appearance: Option<String>,
    },
    SetClassification {
        id: u64,
        classification: SpaceClassification,
    },
    SetMultiMonitorPolicy {
        policy: SpacesDisplayPolicy,
    },
    AssignOutput {
        id: u64,
        output_id: Option<String>,
    },
    /// Assign an application ID to one Space, every Space, or the active
    /// Space default (`Current` clears a previously stored policy).
    SetApplicationPolicy {
        app_id: String,
        target: SpaceTargetWire,
    },
}

/// The wire form of a window membership target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceTargetWire {
    Current,
    Id { id: u64 },
    All,
}

/// Authoritative readback of an application-to-Space policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplicationSpacePolicySnapshot {
    pub app_id: String,
    pub target: SpaceTargetWire,
}

/// One compositor-owned Space row exposed to shell chrome.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpaceSnapshot {
    pub id: u64,
    pub order: usize,
    pub name: String,
    pub active: bool,
    pub window_count: usize,
    #[serde(default)]
    pub wallpaper: Option<String>,
    #[serde(default)]
    pub appearance: Option<String>,
    #[serde(default)]
    pub classification: SpaceClassification,
    #[serde(default)]
    pub output_id: Option<String>,
}

/// Monotonic compositor state projection used by shell reconciliation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpacesSnapshot {
    /// Changes whenever a compositor session starts.  A shell can therefore
    /// accept a lower revision after a compositor restart without treating it
    /// as stale state from the previous session.
    #[serde(default)]
    pub session_epoch: u64,
    pub revision: u64,
    pub active_space: u64,
    #[serde(default)]
    pub multi_monitor_policy: SpacesDisplayPolicy,
    #[serde(default)]
    pub application_policies: Vec<ApplicationSpacePolicySnapshot>,
    pub spaces: Vec<SpaceSnapshot>,
}
