//! Spawn first-party RetroShell binaries as Wayland clients of this compositor.

use std::path::{Path, PathBuf};

/// Resolve a first-party binary from `~/retroshell/target/release`, PATH, or
/// `/usr/local/bin`.
pub fn resolve_client_bin(bin: &str) -> PathBuf {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    if let Ok(home) = std::env::var("HOME") {
        let candidate = PathBuf::from(home)
            .join("retroshell/target/release")
            .join(bin);
        if candidate.is_file() {
            return candidate;
        }
    }
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':').filter(|d| !d.is_empty()) {
            let candidate = Path::new(dir).join(bin);
            if candidate.is_file() {
                #[cfg(unix)]
                if candidate
                    .metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
                {
                    return candidate;
                }
                #[cfg(not(unix))]
                {
                    return candidate;
                }
            }
        }
    }
    let system = PathBuf::from(format!("/usr/local/bin/{bin}"));
    if system.is_file() {
        return system;
    }
    PathBuf::from(bin)
}

/// Spawn `bin` as a Wayland client on `wayland_socket_name`.
pub fn spawn_client(wayland_socket_name: &str, bin: &str) {
    let path = resolve_client_bin(bin);
    let mut cmd = std::process::Command::new(&path);
    cmd.env("WAYLAND_DISPLAY", wayland_socket_name)
        .env("WINIT_UNIX_BACKEND", "wayland")
        .env("RETROSHELL_GLOBAL_MENU", "1");
    if let Ok(w) = std::env::var("RETROSHELL_COMPOSITOR_WIDTH") {
        cmd.env("RETROSHELL_COMPOSITOR_WIDTH", w);
    }
    if let Ok(h) = std::env::var("RETROSHELL_COMPOSITOR_HEIGHT") {
        cmd.env("RETROSHELL_COMPOSITOR_HEIGHT", h);
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        cmd.env("XDG_RUNTIME_DIR", &runtime);
        let menu_dir = PathBuf::from(&runtime).join("retroshell").join("menus");
        let _ = std::fs::create_dir_all(&menu_dir);
        cmd.env("RETROSHELL_MENU_MANIFEST_DIR", menu_dir);
    }
    if bin == "retro-lock" {
        if let Ok(pw) = std::env::var("RETROSHELL_LOCK_PASSWORD") {
            cmd.env("RETROSHELL_LOCK_PASSWORD", pw);
        }
    } else {
        cmd.env_remove("RETROSHELL_LOCK_PASSWORD");
    }
    match cmd.spawn() {
        Ok(child) => {
            tracing::info!(bin, pid = child.id(), path = %path.display(), "spawned client");
        }
        Err(err) => {
            tracing::warn!(error = %err, bin, path = %path.display(), "spawn_client failed");
        }
    }
}
