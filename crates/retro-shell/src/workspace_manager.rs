//! Shell-side named desktops (menu bar / status chrome).
//!
//! Aligned with the compositor pure model (`WorkspaceId` / `WorkspaceState`):
//! fixed **8** virtual workspaces. Window menu exposes Desktop 1..8; cycling
//! and switch APIs mirror compositor (`next`/`previous` ≈ `cycle_next`/
//! `cycle_prev`, `switch_to` ≈ `activate`).

/// Number of shell UI desktops (Window menu `workspace.switch.0..7`).
pub const SHELL_DESKTOP_COUNT: usize = 8;

/// Compositor-backed workspace count (pure model in `retro-compositor`).
pub const COMPOSITOR_WORKSPACE_COUNT: usize = 8;

/// Pure bridge: shell active index ↔ compositor workspace id (0..7).
pub fn shell_index_to_compositor(index: usize) -> Option<u8> {
    if index < COMPOSITOR_WORKSPACE_COUNT {
        Some(index as u8)
    } else {
        None
    }
}

/// Whether a window on `window_workspace` is visible when shell active is `active`.
pub fn window_visible_on_active(active: usize, window_workspace: usize) -> bool {
    active == window_workspace && active < SHELL_DESKTOP_COUNT
}

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

    /// Compositor-aligned summary line for session logs.
    pub fn summary_line(&self) -> String {
        format!(
            "shell-workspace active={}/{} name={}",
            self.active,
            self.total,
            self.active_workspace()
                .map(|w| w.name.as_str())
                .unwrap_or("?")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_desktops_align_with_compositor() {
        assert_eq!(SHELL_DESKTOP_COUNT, COMPOSITOR_WORKSPACE_COUNT);
        assert_eq!(SHELL_DESKTOP_COUNT, 8);
        let wm = WorkspaceManager::new();
        assert_eq!(wm.total, 8);
        assert_eq!(wm.workspaces.len(), 8);
        assert_eq!(shell_index_to_compositor(7), Some(7));
        assert_eq!(shell_index_to_compositor(8), None);
        assert!(window_visible_on_active(2, 2));
        assert!(!window_visible_on_active(2, 3));
    }

    #[test]
    fn cycle_wraps_eight() {
        let mut wm = WorkspaceManager::new();
        for _ in 0..7 {
            wm.next();
        }
        assert_eq!(wm.active, 7);
        wm.next();
        assert_eq!(wm.active, 0);
        wm.previous();
        assert_eq!(wm.active, 7);
    }
}
