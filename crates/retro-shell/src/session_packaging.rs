//! Session packaging health checks (greeter files, start script) — pure paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Expected packaging artifacts for a greeter-capable install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionPackagingLayout {
    pub wayland_session_desktop: PathBuf,
    pub xsession_desktop: PathBuf,
    pub start_script: PathBuf,
    pub user_service: PathBuf,
}

impl SessionPackagingLayout {
    /// Default FHS layout under a prefix (e.g. `/usr`).
    pub fn under_prefix(prefix: impl AsRef<Path>) -> Self {
        let p = prefix.as_ref();
        Self {
            wayland_session_desktop: p.join("share/wayland-sessions/retroshell.desktop"),
            xsession_desktop: p.join("share/xsessions/retroshell.desktop"),
            start_script: p.join("bin/start-retroshell"),
            user_service: p.join("lib/systemd/user/retroshell.service"),
        }
    }

    /// Repo-local packaging tree (development).
    pub fn from_repo_packaging(repo_root: impl AsRef<Path>) -> Self {
        let r = repo_root.as_ref();
        Self {
            wayland_session_desktop: r.join("packaging/retroshell-wayland.desktop"),
            xsession_desktop: r.join("packaging/retroshell.desktop"),
            start_script: r.join("scripts/start-retroshell"),
            user_service: r.join("packaging/retroshell.service"),
        }
    }
}

/// Result of checking packaging presence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagingHealth {
    pub wayland_session_ok: bool,
    pub xsession_ok: bool,
    pub start_script_ok: bool,
    pub user_service_ok: bool,
}

impl PackagingHealth {
    pub fn all_ok(&self) -> bool {
        self.wayland_session_ok
            && self.xsession_ok
            && self.start_script_ok
            && self.user_service_ok
    }

    pub fn score_points(&self) -> u8 {
        let mut n = 0u8;
        if self.wayland_session_ok {
            n += 25;
        }
        if self.xsession_ok {
            n += 25;
        }
        if self.start_script_ok {
            n += 25;
        }
        if self.user_service_ok {
            n += 25;
        }
        n
    }
}

/// Check which packaging files exist (pure filesystem probe).
pub fn check_packaging_health(layout: &SessionPackagingLayout) -> PackagingHealth {
    PackagingHealth {
        wayland_session_ok: layout.wayland_session_desktop.is_file(),
        xsession_ok: layout.xsession_desktop.is_file(),
        start_script_ok: layout.start_script.is_file(),
        user_service_ok: layout.user_service.is_file(),
    }
}

/// Greeter → session readiness checklist (pure content validation + files).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreeterSessionReadiness {
    pub packaging: PackagingHealth,
    pub wayland_desktop_valid: bool,
    pub xsession_desktop_valid: bool,
    pub start_script_executable_bit: bool,
    pub desktop_names_ok: bool,
    pub notes: Vec<String>,
}

impl GreeterSessionReadiness {
    /// True when files exist, desktops validate, and start script is present.
    /// Does **not** claim a live display manager was exercised.
    pub fn install_ready(&self) -> bool {
        self.packaging.all_ok()
            && self.wayland_desktop_valid
            && self.xsession_desktop_valid
            && self.desktop_names_ok
    }

    pub fn score_points(&self) -> u8 {
        let mut n = 0u8;
        if self.packaging.wayland_session_ok {
            n += 15;
        }
        if self.packaging.xsession_ok {
            n += 15;
        }
        if self.packaging.start_script_ok {
            n += 15;
        }
        if self.packaging.user_service_ok {
            n += 10;
        }
        if self.wayland_desktop_valid {
            n += 15;
        }
        if self.xsession_desktop_valid {
            n += 15;
        }
        if self.desktop_names_ok {
            n += 10;
        }
        if self.start_script_executable_bit {
            n += 5;
        }
        n
    }
}

/// Probe greeter session readiness from layout paths (reads desktop files).
pub fn check_greeter_session_readiness(layout: &SessionPackagingLayout) -> GreeterSessionReadiness {
    let packaging = check_packaging_health(layout);
    let mut notes = Vec::new();

    let wayland_desktop_valid = read_and_validate(&layout.wayland_session_desktop, &mut notes);
    let xsession_desktop_valid = read_and_validate(&layout.xsession_desktop, &mut notes);

    let desktop_names_ok = desktop_names_present(&layout.wayland_session_desktop)
        || desktop_names_present(&layout.xsession_desktop);
    if !desktop_names_ok {
        notes.push("DesktopNames not set on session .desktop files".into());
    }

    let start_script_executable_bit = is_executable(&layout.start_script);
    if packaging.start_script_ok && !start_script_executable_bit {
        notes.push("start-retroshell exists but executable bit not set".into());
    }

    if packaging.all_ok() && wayland_desktop_valid && xsession_desktop_valid {
        notes.push(
            "Install artifacts OK — live greeter login still requires DM + seat on target host"
                .into(),
        );
    }

    GreeterSessionReadiness {
        packaging,
        wayland_desktop_valid,
        xsession_desktop_valid,
        start_script_executable_bit,
        desktop_names_ok,
        notes,
    }
}

fn read_and_validate(path: &Path, notes: &mut Vec<String>) -> bool {
    match std::fs::read_to_string(path) {
        Ok(content) => match validate_session_desktop(&content) {
            Ok(()) => true,
            Err(errs) => {
                notes.push(format!("{}: {}", path.display(), errs.join("; ")));
                false
            }
        },
        Err(e) => {
            notes.push(format!("cannot read {}: {e}", path.display()));
            false
        }
    }
}

fn desktop_names_present(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .map(|c| {
            let keys = parse_desktop_keys(&c);
            keys.get("DesktopNames")
                .map(|v| !v.is_empty())
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// Parse `Key=Value` lines from a `.desktop` file into a map.
///
/// - Ignores blank lines and comments (`#...`).
/// - Ignores section headers (`[Desktop Entry]`).
/// - First occurrence of a key wins.
/// - Values keep surrounding whitespace after the first `=`.
pub fn parse_desktop_keys(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        map.entry(key.to_string())
            .or_insert_with(|| value.trim().to_string());
    }
    map
}

/// Validate a session `.desktop` entry for greeter use.
///
/// Requires:
/// - `Type=Application`
/// - `Exec` containing `start-retroshell`
/// - `Name` non-empty
///
/// Returns `Ok(())` on success, or `Err` with one message per failed rule.
pub fn validate_session_desktop(content: &str) -> Result<(), Vec<String>> {
    let keys = parse_desktop_keys(content);
    let mut errors = Vec::new();

    match keys.get("Type").map(String::as_str) {
        Some("Application") => {}
        Some(other) => errors.push(format!("Type must be Application (got '{other}')")),
        None => errors.push("missing required key: Type".to_string()),
    }

    match keys.get("Exec").map(String::as_str) {
        Some(exec) if exec.contains("start-retroshell") => {}
        Some(exec) => errors.push(format!(
            "Exec must contain start-retroshell (got '{exec}')"
        )),
        None => errors.push("missing required key: Exec".to_string()),
    }

    match keys.get("Name").map(String::as_str) {
        Some(name) if !name.is_empty() => {}
        Some(_) => errors.push("Name must be non-empty".to_string()),
        None => errors.push("missing required key: Name".to_string()),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repo_root() -> PathBuf {
        // Crate is crates/retro-shell → repo root is ../..
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn repo_packaging_layout_is_complete() {
        let root = repo_root();
        let layout = SessionPackagingLayout::from_repo_packaging(&root);
        let health = check_packaging_health(&layout);
        assert!(
            health.all_ok(),
            "expected full packaging tree in repo: {health:?} layout={layout:?}"
        );
        assert_eq!(health.score_points(), 100);
    }

    #[test]
    fn under_prefix_paths_are_fhs() {
        let l = SessionPackagingLayout::under_prefix("/usr");
        assert!(l
            .wayland_session_desktop
            .ends_with("share/wayland-sessions/retroshell.desktop"));
        assert!(l.start_script.ends_with("bin/start-retroshell"));
    }

    #[test]
    fn parse_desktop_keys_extracts_name_exec_type() {
        let content = "\
[Desktop Entry]
# a comment
Name=RetroShell
Exec=start-retroshell
Type=Application
DesktopNames=RetroShell
";
        let keys = parse_desktop_keys(content);
        assert_eq!(keys.get("Name").map(String::as_str), Some("RetroShell"));
        assert_eq!(
            keys.get("Exec").map(String::as_str),
            Some("start-retroshell")
        );
        assert_eq!(keys.get("Type").map(String::as_str), Some("Application"));
        assert_eq!(
            keys.get("DesktopNames").map(String::as_str),
            Some("RetroShell")
        );
        assert!(!keys.contains_key("# a comment"));
    }

    #[test]
    fn parse_desktop_keys_first_occurrence_wins() {
        let content = "Name=First\nName=Second\n";
        let keys = parse_desktop_keys(content);
        assert_eq!(keys.get("Name").map(String::as_str), Some("First"));
    }

    #[test]
    fn validate_session_desktop_accepts_valid() {
        let content = "\
[Desktop Entry]
Name=RetroShell
Exec=start-retroshell
Type=Application
";
        assert!(validate_session_desktop(content).is_ok());
    }

    #[test]
    fn validate_session_desktop_accepts_absolute_exec() {
        let content = "\
Name=RetroShell
Exec=/usr/local/bin/start-retroshell
Type=Application
";
        assert!(validate_session_desktop(content).is_ok());
    }

    #[test]
    fn validate_session_desktop_rejects_bad_type() {
        let content = "Name=X\nExec=start-retroshell\nType=Link\n";
        let err = validate_session_desktop(content).unwrap_err();
        assert!(err.iter().any(|e| e.contains("Type")));
    }

    #[test]
    fn validate_session_desktop_rejects_missing_and_empty() {
        let content = "Type=Application\nName=\n";
        let err = validate_session_desktop(content).unwrap_err();
        assert!(err.iter().any(|e| e.contains("Exec")));
        assert!(err.iter().any(|e| e.contains("Name")));
    }

    #[test]
    fn validate_session_desktop_rejects_wrong_exec() {
        let content = "Name=X\nExec=gnome-session\nType=Application\n";
        let err = validate_session_desktop(content).unwrap_err();
        assert!(err.iter().any(|e| e.contains("start-retroshell")));
    }

    #[test]
    fn packaging_retroshell_desktop_validates() {
        let path = repo_root().join("packaging/retroshell.desktop");
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let keys = parse_desktop_keys(&content);
        assert_eq!(keys.get("Name").map(String::as_str), Some("RetroShell"));
        assert!(
            keys.get("Exec")
                .is_some_and(|e| e.contains("start-retroshell")),
            "Exec keys={keys:?}"
        );
        assert_eq!(keys.get("Type").map(String::as_str), Some("Application"));
        validate_session_desktop(&content).expect("packaging/retroshell.desktop must validate");
    }

    #[test]
    fn packaging_retroshell_wayland_desktop_validates() {
        let path = repo_root().join("packaging/retroshell-wayland.desktop");
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        validate_session_desktop(&content)
            .expect("packaging/retroshell-wayland.desktop must validate");
    }

    #[test]
    fn repo_packaging_files_validate_via_layout() {
        let root = repo_root();
        let layout = SessionPackagingLayout::from_repo_packaging(&root);
        let health = check_packaging_health(&layout);
        assert!(health.all_ok(), "layout incomplete: {health:?}");

        for path in [
            &layout.wayland_session_desktop,
            &layout.xsession_desktop,
        ] {
            let content = fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            validate_session_desktop(&content).unwrap_or_else(|errs| {
                panic!("{} failed validate: {errs:?}", path.display());
            });
        }
    }

    #[test]
    fn greeter_session_readiness_repo() {
        let root = repo_root();
        let layout = SessionPackagingLayout::from_repo_packaging(&root);
        let ready = check_greeter_session_readiness(&layout);
        assert!(
            ready.install_ready(),
            "repo packaging should be greeter-install-ready: {ready:?}"
        );
        assert!(ready.score_points() >= 90, "score={}", ready.score_points());
    }
}
