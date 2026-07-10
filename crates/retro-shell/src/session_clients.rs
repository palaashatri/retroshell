//! Multi-client session tracking: first-party apps launched as **separate processes**
//! that own their own surfaces under the compositor/labwc (not shell paint-rects).

use std::collections::HashMap;
use std::process::{Child, Command};
use std::time::{SystemTime, UNIX_EPOCH};

/// One external application client the shell has launched.
#[derive(Debug)]
pub struct ExternalClient {
    pub bundle_id: String,
    pub binary_name: String,
    pub pid: u32,
    pub child: Option<Child>,
    pub launched_at_unix: u64,
}

/// Registry of compositor-side client processes started by the shell.
#[derive(Debug, Default)]
pub struct SessionClientRegistry {
    clients: HashMap<u32, ExternalClient>,
}

impl SessionClientRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    pub fn clients(&self) -> impl Iterator<Item = &ExternalClient> {
        self.clients.values()
    }

    pub fn pids(&self) -> Vec<u32> {
        self.clients.keys().copied().collect()
    }

    pub fn register(&mut self, client: ExternalClient) {
        self.clients.insert(client.pid, client);
    }

    /// Reap exited children; returns number removed.
    pub fn reap(&mut self) -> usize {
        let mut dead = Vec::new();
        for (pid, client) in self.clients.iter_mut() {
            if let Some(child) = client.child.as_mut() {
                match child.try_wait() {
                    Ok(Some(_)) => dead.push(*pid),
                    Ok(None) => {}
                    Err(_) => dead.push(*pid),
                }
            }
        }
        let n = dead.len();
        for pid in dead {
            self.clients.remove(&pid);
        }
        n
    }

    pub fn count_for_bundle(&self, bundle_id: &str) -> usize {
        self.clients
            .values()
            .filter(|c| c.bundle_id == bundle_id)
            .count()
    }

    /// Force-terminate a tracked client by pid (kill process + drop from registry).
    /// Returns true if a registry entry was removed.
    pub fn force_quit_pid(&mut self, pid: u32) -> bool {
        let Some(mut client) = self.clients.remove(&pid) else {
            // Still try kill in case the process is untracked.
            let _ = kill_process(pid);
            return false;
        };
        if let Some(mut child) = client.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        } else {
            let _ = kill_process(pid);
        }
        true
    }
}

/// Best-effort process kill (used for force-quit of multi-client apps).
pub fn kill_process(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Parsed Force Quit list entry (must match formatting in shell Force Quit UI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForceQuitTarget {
    /// Shell-managed painted window, matched by title.
    WindowTitle(String),
    /// External multi-client process.
    ClientPid(u32),
}

/// Parse list labels produced by Force Quit UI:
/// - `window: {title}`
/// - `client: {binary} (pid {n})`
pub fn parse_force_quit_entry(entry: &str) -> Option<ForceQuitTarget> {
    let entry = entry.trim();
    if let Some(title) = entry.strip_prefix("window:") {
        let title = title.trim();
        if title.is_empty() {
            return None;
        }
        return Some(ForceQuitTarget::WindowTitle(title.to_string()));
    }
    if let Some(rest) = entry.strip_prefix("client:") {
        let rest = rest.trim();
        // "... (pid 123)"
        let pid_marker = "(pid ";
        let start = rest.rfind(pid_marker)?;
        let after = &rest[start + pid_marker.len()..];
        let digits = after.strip_suffix(')')?.trim();
        let pid: u32 = digits.parse().ok()?;
        return Some(ForceQuitTarget::ClientPid(pid));
    }
    // Legacy bare title (pre multi-client list format)
    if !entry.is_empty() {
        return Some(ForceQuitTarget::WindowTitle(entry.to_string()));
    }
    None
}

/// Map bundle id → executable name (shipped first-party apps).
pub fn binary_name_for_bundle(bundle_id: &str) -> Option<&'static str> {
    match bundle_id {
        "com.retro.finder" => Some("finder"),
        "com.retro.settings" => Some("settings"),
        "com.retro.textedit" => Some("textedit"),
        "com.retro.terminal" => Some("terminal"),
        "com.retro.appstore" => Some("appstore"),
        _ => None,
    }
}

/// Ordered filesystem candidates for a first-party binary (real search path).
pub fn binary_candidates(binary: &str) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join(binary));
        }
    }
    out.push(std::path::PathBuf::from(format!("target/debug/{binary}")));
    out.push(std::path::PathBuf::from(format!("target/release/{binary}")));
    out.push(std::path::PathBuf::from(format!("/usr/local/bin/{binary}")));
    out.push(std::path::PathBuf::from(binary));
    out
}

/// Resolve first existing candidate path.
pub fn resolve_app_binary(bundle_id: &str) -> Result<std::path::PathBuf, String> {
    let binary = binary_name_for_bundle(bundle_id)
        .ok_or_else(|| format!("No binary registered for bundle '{bundle_id}'"))?;
    for candidate in binary_candidates(binary) {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    // Last resort: if `binary` is on PATH, Command can still run it by name.
    if which_exists(binary) {
        return Ok(std::path::PathBuf::from(binary));
    }
    Err(format!(
        "Could not find executable for '{bundle_id}' — checked PATH and target/{{debug,release}}/{binary}"
    ))
}

fn which_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Build a `Command` for launching a first-party app as a Wayland client process.
///
/// Inherits `WAYLAND_DISPLAY` / `XDG_RUNTIME_DIR` from the shell session so the
/// child attaches to the same compositor (labwc or retro-compositor).
pub fn build_app_command(path: &std::path::Path) -> Command {
    let mut command = Command::new(path);
    command.env("RETROSHELL_GLOBAL_MENU", "1");
    // Prefer Wayland when a session is available; do not force a wrong display.
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        command.env("WINIT_UNIX_BACKEND", "wayland");
    }
    command
}

/// Spawn an app client process and return an [`ExternalClient`] on success.
pub fn spawn_app_client(bundle_id: &str) -> Result<ExternalClient, String> {
    let binary = binary_name_for_bundle(bundle_id)
        .ok_or_else(|| format!("No binary registered for bundle '{bundle_id}'"))?
        .to_string();
    let path = resolve_app_binary(bundle_id)?;
    let mut command = build_app_command(&path);
    let child = command
        .spawn()
        .map_err(|err| format!("Failed to spawn '{}': {err}", path.display()))?;
    let pid = child.id();
    let launched_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(ExternalClient {
        bundle_id: bundle_id.to_string(),
        binary_name: binary,
        pid,
        child: Some(child),
        launched_at_unix,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_name_mapping_covers_first_party_suite() {
        assert_eq!(binary_name_for_bundle("com.retro.finder"), Some("finder"));
        assert_eq!(binary_name_for_bundle("com.retro.terminal"), Some("terminal"));
        assert_eq!(binary_name_for_bundle("com.retro.settings"), Some("settings"));
        assert_eq!(binary_name_for_bundle("unknown"), None);
    }

    #[test]
    fn binary_candidates_prefer_exe_dir_then_targets() {
        let c = binary_candidates("finder");
        assert!(c.iter().any(|p| p.ends_with("finder")));
        assert!(c.iter().any(|p| p.to_string_lossy().contains("target/debug")));
        assert!(c.iter().any(|p| p.to_string_lossy().contains("/usr/local/bin/")));
    }

    #[test]
    fn registry_tracks_two_clients_as_multi_client_session() {
        let mut reg = SessionClientRegistry::new();
        reg.register(ExternalClient {
            bundle_id: "com.retro.finder".into(),
            binary_name: "finder".into(),
            pid: 11,
            child: None,
            launched_at_unix: 1,
        });
        reg.register(ExternalClient {
            bundle_id: "com.retro.terminal".into(),
            binary_name: "terminal".into(),
            pid: 22,
            child: None,
            launched_at_unix: 2,
        });
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.count_for_bundle("com.retro.finder"), 1);
        let mut pids = reg.pids();
        pids.sort();
        assert_eq!(pids, vec![11, 22]);
    }

    #[test]
    fn resolve_or_fail_is_real_path_logic() {
        // Unknown bundle fails without inventing a binary.
        assert!(resolve_app_binary("com.not.real").is_err());
        // Known bundle either finds a path or fails with a clear message (host may lack bins).
        match resolve_app_binary("com.retro.finder") {
            Ok(p) => assert!(p.to_string_lossy().contains("finder")),
            Err(e) => assert!(e.contains("finder") || e.contains("Could not find")),
        }
    }

    #[test]
    fn parse_force_quit_entry_window_and_client() {
        assert_eq!(
            parse_force_quit_entry("window: Retro HD"),
            Some(ForceQuitTarget::WindowTitle("Retro HD".into()))
        );
        assert_eq!(
            parse_force_quit_entry("client: finder (pid 203)"),
            Some(ForceQuitTarget::ClientPid(203))
        );
        assert_eq!(
            parse_force_quit_entry("client: settings (pid 42)"),
            Some(ForceQuitTarget::ClientPid(42))
        );
        // Legacy bare title still resolves to a shell window target.
        assert_eq!(
            parse_force_quit_entry("About RetroShell"),
            Some(ForceQuitTarget::WindowTitle("About RetroShell".into()))
        );
        assert_eq!(parse_force_quit_entry("window: "), None);
        assert_eq!(parse_force_quit_entry("client: broken"), None);
    }

    #[test]
    fn force_quit_pid_removes_registry_entry() {
        let mut reg = SessionClientRegistry::new();
        reg.register(ExternalClient {
            bundle_id: "com.retro.finder".into(),
            binary_name: "finder".into(),
            pid: 999_001,
            child: None,
            launched_at_unix: 1,
        });
        assert!(reg.force_quit_pid(999_001));
        assert_eq!(reg.len(), 0);
        assert!(!reg.force_quit_pid(999_001));
    }
}
