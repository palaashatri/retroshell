//! Foreign-toplevel registry for task list / Force Quit.
//!
//! Mirrors compositor foreign-toplevel-list handles as pure session state so
//! shell Force Quit can list and kill external toplevels without compositor deps.

use crate::session_clients::kill_process;
use std::collections::HashMap;

/// One foreign toplevel handle known to the shell session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignToplevelEntry {
    pub handle_id: String,
    pub title: String,
    pub app_id: String,
    pub pid: Option<u32>,
}

/// Parsed `toplevel:` Force Quit list entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToplevelForceQuit {
    pub title: String,
    pub app_id: Option<String>,
    pub pid: Option<u32>,
}

/// Registry of foreign toplevels (compositor clients / external apps).
#[derive(Clone, Debug, Default)]
pub struct ForeignToplevelRegistry {
    /// Keyed by handle_id.
    entries: HashMap<String, ForeignToplevelEntry>,
}

impl ForeignToplevelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = &ForeignToplevelEntry> {
        self.entries.values()
    }

    /// Insert or replace by `handle_id`.
    pub fn add(&mut self, entry: ForeignToplevelEntry) {
        self.entries.insert(entry.handle_id.clone(), entry);
    }

    /// Update title/app_id/pid for an existing handle; returns false if missing.
    pub fn update(
        &mut self,
        handle_id: &str,
        title: Option<String>,
        app_id: Option<String>,
        pid: Option<Option<u32>>,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(handle_id) else {
            return false;
        };
        if let Some(t) = title {
            entry.title = t;
        }
        if let Some(a) = app_id {
            entry.app_id = a;
        }
        if let Some(p) = pid {
            entry.pid = p;
        }
        true
    }

    /// Remove a closed toplevel handle.
    pub fn close(&mut self, handle_id: &str) -> bool {
        self.entries.remove(handle_id).is_some()
    }

    /// Force Quit list labels.
    ///
    /// - with pid: `toplevel: {title} (pid {n})`
    /// - without:  `toplevel: {title} [{app_id}]`
    pub fn force_quit_labels(&self) -> Vec<String> {
        let mut labels: Vec<String> = self
            .entries
            .values()
            .map(|e| {
                if let Some(pid) = e.pid {
                    format!("toplevel: {} (pid {pid})", e.title)
                } else {
                    format!("toplevel: {} [{}]", e.title, e.app_id)
                }
            })
            .collect();
        labels.sort();
        labels
    }

    /// Find entry matching a parsed Force Quit target (by pid, else title/app_id).
    pub fn find_match(&self, target: &ToplevelForceQuit) -> Option<&ForeignToplevelEntry> {
        if let Some(pid) = target.pid {
            return self.entries.values().find(|e| e.pid == Some(pid));
        }
        self.entries.values().find(|e| {
            e.title == target.title
                && match &target.app_id {
                    Some(app) => e.app_id == *app,
                    None => true,
                }
        })
    }

    /// Remove entry matching target (does not kill).
    pub fn remove_match(&mut self, target: &ToplevelForceQuit) -> Option<ForeignToplevelEntry> {
        let handle = self.find_match(target)?.handle_id.clone();
        self.entries.remove(&handle)
    }
}

/// Parse Force Quit list labels:
/// - `toplevel: {title} (pid {n})`
/// - `toplevel: {title} [{app_id}]`
pub fn parse_toplevel_force_quit(entry: &str) -> Option<ToplevelForceQuit> {
    let entry = entry.trim();
    let rest = entry.strip_prefix("toplevel:")?.trim();
    if rest.is_empty() {
        return None;
    }

    // `title (pid N)`
    let pid_marker = " (pid ";
    if let Some(start) = rest.rfind(pid_marker) {
        let title = rest[..start].trim();
        if title.is_empty() {
            return None;
        }
        let after = &rest[start + pid_marker.len()..];
        let digits = after.strip_suffix(')')?.trim();
        let pid: u32 = digits.parse().ok()?;
        return Some(ToplevelForceQuit {
            title: title.to_string(),
            app_id: None,
            pid: Some(pid),
        });
    }

    // `title [app_id]`
    if let Some(open) = rest.rfind(" [") {
        if rest.ends_with(']') {
            let title = rest[..open].trim();
            let app_id = rest[open + 2..rest.len() - 1].trim();
            if title.is_empty() || app_id.is_empty() {
                return None;
            }
            return Some(ToplevelForceQuit {
                title: title.to_string(),
                app_id: Some(app_id.to_string()),
                pid: None,
            });
        }
    }

    None
}

/// Apply a toplevel Force Quit: remove from registry and kill pid if present.
/// Returns true if an entry was removed or a kill was attempted for a known pid.
pub fn apply_toplevel_force_quit(
    registry: &mut ForeignToplevelRegistry,
    target: &ToplevelForceQuit,
) -> bool {
    let removed = registry.remove_match(target);
    let pid = removed
        .as_ref()
        .and_then(|e| e.pid)
        .or(target.pid);
    if let Some(pid) = pid {
        let _ = kill_process(pid);
        return removed.is_some() || pid != 0;
    }
    removed.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_with_pid() -> ForeignToplevelEntry {
        ForeignToplevelEntry {
            handle_id: "ft-1".into(),
            title: "Finder".into(),
            app_id: "com.retro.finder".into(),
            pid: Some(4242),
        }
    }

    fn sample_no_pid() -> ForeignToplevelEntry {
        ForeignToplevelEntry {
            handle_id: "ft-2".into(),
            title: "Doom".into(),
            app_id: "org.idsoftware.doom".into(),
            pid: None,
        }
    }

    #[test]
    fn add_update_close_registry() {
        let mut reg = ForeignToplevelRegistry::new();
        reg.add(sample_with_pid());
        assert_eq!(reg.len(), 1);
        assert!(reg.update("ft-1", Some("Finder+".into()), None, None));
        assert_eq!(
            reg.entries().next().map(|e| e.title.as_str()),
            Some("Finder+")
        );
        assert!(reg.close("ft-1"));
        assert!(reg.is_empty());
        assert!(!reg.close("ft-1"));
    }

    #[test]
    fn force_quit_labels_format_with_and_without_pid() {
        let mut reg = ForeignToplevelRegistry::new();
        reg.add(sample_with_pid());
        reg.add(sample_no_pid());
        let labels = reg.force_quit_labels();
        assert!(labels.contains(&"toplevel: Finder (pid 4242)".to_string()));
        assert!(labels.contains(&"toplevel: Doom [org.idsoftware.doom]".to_string()));
    }

    #[test]
    fn parse_toplevel_force_quit_pid_and_app_id() {
        assert_eq!(
            parse_toplevel_force_quit("toplevel: Finder (pid 4242)"),
            Some(ToplevelForceQuit {
                title: "Finder".into(),
                app_id: None,
                pid: Some(4242),
            })
        );
        assert_eq!(
            parse_toplevel_force_quit("toplevel: Doom [org.idsoftware.doom]"),
            Some(ToplevelForceQuit {
                title: "Doom".into(),
                app_id: Some("org.idsoftware.doom".into()),
                pid: None,
            })
        );
        assert_eq!(parse_toplevel_force_quit("toplevel: "), None);
        assert_eq!(parse_toplevel_force_quit("window: Retro HD"), None);
        assert_eq!(parse_toplevel_force_quit("client: finder (pid 1)"), None);
    }

    #[test]
    fn apply_toplevel_force_quit_removes_entry() {
        let mut reg = ForeignToplevelRegistry::new();
        reg.add(sample_with_pid());
        reg.add(sample_no_pid());

        let target = parse_toplevel_force_quit("toplevel: Finder (pid 4242)").unwrap();
        assert!(apply_toplevel_force_quit(&mut reg, &target));
        assert_eq!(reg.len(), 1);
        assert!(reg.entries().all(|e| e.handle_id == "ft-2"));

        let target2 = parse_toplevel_force_quit("toplevel: Doom [org.idsoftware.doom]").unwrap();
        assert!(apply_toplevel_force_quit(&mut reg, &target2));
        assert!(reg.is_empty());
    }
}
