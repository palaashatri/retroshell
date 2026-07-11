//! Session packaging health checks (greeter files, start script) — pure paths.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_packaging_layout_is_complete() {
        // Crate is crates/retro-shell → repo root is ../..
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
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
}
