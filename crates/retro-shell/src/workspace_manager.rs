//! Shell-side named desktops (menu bar / status chrome).
//!
//! The compositor pure model lives in `retro_compositor::{WorkspaceId,
//! WorkspaceState}` with a fixed **8** virtual workspaces and window→workspace
//! mapping. This shell manager keeps **4** classic "Desktop N" entries for the
//! Window menu shortcuts and status UI until chrome is unified with the
//! compositor-backed policy. Cycling / switch APIs mirror the pure model
//! (`next`/`previous` ≈ `cycle_next`/`cycle_prev`, `switch_to` ≈ `activate`).

/// Number of shell UI desktops (Window menu `workspace.switch.0..3`).
pub const SHELL_DESKTOP_COUNT: usize = 4;

/// Compositor-backed workspace count (pure model in `retro-compositor`).
/// Kept here for shell/compositor alignment docs and future bridge code.
pub const COMPOSITOR_WORKSPACE_COUNT: usize = 8;

pub struct WorkspaceManager {
    pub workspaces: Vec<Workspace>,
    pub active: usize,
    pub total: usize,
}

pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub background: Option<String>,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let workspaces = (0..SHELL_DESKTOP_COUNT)
            .map(|i| Workspace {
                id: i,
                name: format!("Desktop {}", i + 1),
                background: None,
            })
            .collect();
        Self {
            workspaces,
            active: 0,
            total: SHELL_DESKTOP_COUNT,
        }
    }

    /// Activate desktop by index (shell `0..total`). Mirrors compositor `activate`.
    pub fn switch_to(&mut self, index: usize) -> bool {
        if index < self.total {
            self.active = index;
            true
        } else {
            false
        }
    }

    /// Alias for [`Self::switch_to`] (compositor naming).
    pub fn activate(&mut self, index: usize) -> bool {
        self.switch_to(index)
    }

    /// Cycle forward, wrapping. Mirrors compositor `cycle_next`.
    pub fn next(&mut self) {
        if self.total == 0 {
            return;
        }
        self.active = (self.active + 1) % self.total;
    }

    /// Alias for [`Self::next`].
    pub fn cycle_next(&mut self) {
        self.next();
    }

    /// Cycle backward, wrapping. Mirrors compositor `cycle_prev`.
    pub fn previous(&mut self) {
        if self.total == 0 {
            return;
        }
        self.active = if self.active == 0 {
            self.total - 1
        } else {
            self.active - 1
        };
    }

    /// Alias for [`Self::previous`].
    pub fn cycle_prev(&mut self) {
        self.previous();
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.active)
    }

    pub fn add_workspace(&mut self, name: &str) {
        let id = self.total;
        self.workspaces.push(Workspace {
            id,
            name: name.to_string(),
            background: None,
        });
        self.total += 1;
    }
}
