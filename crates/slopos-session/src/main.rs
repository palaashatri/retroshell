//! SLOPOS-I session supervisor.
//!
//! Copyright (c) 2026 Palaash Atri
//! SPDX-License-Identifier: MIT
//!
//! This process is the stable parent for the compositor and shell.  It keeps
//! host and private Wayland sockets separate, waits for compositor readiness,
//! and tears down the entire client process group if either critical process
//! exits.

use std::env;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SESSION_ROOT: &str = "slopos-i";
const READINESS_FILE: &str = "readiness";
const CLIENT_DISPLAY_FILE: &str = "client-wayland-display";
const TOKEN_FILE: &str = "token";
const VISION_SOCKET_ENV: &str = "SLOPOS_VISION_SOCKET";
const VISION_MODELS_ENV: &str = "SLOPOS_VISION_MODELS_DIR";
const VISION_ARTIFACT_ENV: &str = "SLOPOS_VISION_ARTIFACT_DIR";
const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(12);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_shutdown_signal(_signal: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

fn install_signal_handlers() -> Result<(), String> {
    // The handler only flips an AtomicBool, which is async-signal-safe. The
    // supervisor performs all process-group teardown back in its normal loop.
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        let previous = unsafe {
            libc::signal(
                signal,
                handle_shutdown_signal as *const () as libc::sighandler_t,
            )
        };
        if previous == libc::SIG_ERR {
            return Err(format!("cannot install signal handler for signal {signal}"));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backend {
    Drm,
    Nested,
    Headless,
}

#[derive(Clone, Debug)]
struct PrivateDisplay {
    socket_name: String,
    output_width: Option<u32>,
    output_height: Option<u32>,
}

struct VisionService {
    child: Child,
    socket_path: PathBuf,
}

impl Backend {
    fn cli_value(self) -> &'static str {
        match self {
            Self::Drm => "drm",
            Self::Nested => "nested",
            Self::Headless => "headless",
        }
    }
}

fn default_backend_for_host(display: Option<&str>, _wayland_display: Option<&str>) -> Backend {
    // The nested implementation is Smithay's X11 backend. A host Wayland
    // socket is not an X11 transport and must never select this path.
    if display.is_some_and(|value| !value.is_empty()) {
        Backend::Nested
    } else {
        Backend::Drm
    }
}

fn validate_backend_transport(backend: Backend, display: Option<&str>) -> Result<(), String> {
    if backend == Backend::Nested && !display.is_some_and(|value| !value.is_empty()) {
        return Err(
            "nested backend requires a non-empty DISPLAY (nested transport is X11-only); use --backend drm or --backend headless"
                .to_owned(),
        );
    }
    Ok(())
}

fn parse_backend() -> Result<Backend, String> {
    let mut value: Option<String> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--backend" {
            value = args.next();
        } else if let Some(v) = arg.strip_prefix("--backend=") {
            value = Some(v.to_owned());
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: slopos-session [--backend drm|nested|x11|headless]");
            std::process::exit(0);
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }

    match value.as_deref() {
        Some("drm") => Ok(Backend::Drm),
        Some("nested") | Some("x11") | Some("winit") => Ok(Backend::Nested),
        Some("headless") => Ok(Backend::Headless),
        Some(other) => Err(format!("unsupported backend '{other}'")),
        None => Ok(default_backend_for_host(
            env::var("DISPLAY").ok().as_deref(),
            env::var("WAYLAND_DISPLAY").ok().as_deref(),
        )),
    }
}

fn runtime_dir() -> Result<PathBuf, String> {
    let path = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::geteuid() })));
    fs::create_dir_all(&path)
        .map_err(|e| format!("cannot create XDG_RUNTIME_DIR {}: {e}", path.display()))?;
    Ok(path)
}

struct SessionRuntime {
    path: PathBuf,
}

impl Drop for SessionRuntime {
    fn drop(&mut self) {
        // This directory was created by this supervisor under a private
        // 0700 runtime root. Never glob or clean unrelated Wayland resources.
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn private_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("cannot create runtime directory {}: {e}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("cannot restrict runtime directory {}: {e}", path.display()))
}

fn session_nonce() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}-{}", std::process::id())
}

fn write_private_file(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("cannot restrict {}: {e}", path.display()))
}

fn create_session_runtime(base: &Path) -> Result<(SessionRuntime, String), String> {
    let root = base.join(SESSION_ROOT);
    private_dir(&root)?;

    for attempt in 0..8u8 {
        let suffix = if attempt == 0 {
            String::new()
        } else {
            format!("-{attempt}")
        };
        let path = root.join(format!("session-{}{suffix}", session_nonce()));
        match fs::create_dir(&path) {
            Ok(()) => {
                private_dir(&path)?;
                let logs = path.join("logs");
                private_dir(&logs)?;
                let token = format!("{}-{}", session_nonce(), std::process::id());
                write_private_file(&path.join(TOKEN_FILE), &token)?;
                return Ok((SessionRuntime { path }, token));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "cannot create private session directory {}: {error}",
                    path.display()
                ));
            }
        }
    }

    Err("could not allocate a unique SLOPOS-I session directory".to_string())
}

fn sibling_or_path(name: &str) -> Result<PathBuf, String> {
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    let path = env::var_os("PATH").unwrap_or_default();
    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    let repo_candidates = [
        PathBuf::from(format!("target/release/{name}")),
        PathBuf::from(format!("target/debug/{name}")),
    ];
    for candidate in repo_candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(format!("required binary '{name}' was not found"))
}

fn configured_vision_models_dir_from_path(path: &Path) -> Result<PathBuf, String> {
    let path = fs::canonicalize(path)
        .map_err(|error| format!("cannot access the configured Vision model directory: {error}"))?;
    if !path.is_dir() {
        return Err("configured Vision model path is not a directory".to_string());
    }
    if !path.join("manifest.toml").is_file() {
        return Err(format!(
            "configured Vision model directory {} has no manifest.toml",
            path.display()
        ));
    }
    Ok(path)
}

fn configured_vision_models_dir() -> Result<PathBuf, String> {
    let value =
        env::var_os(VISION_MODELS_ENV).ok_or_else(|| format!("{VISION_MODELS_ENV} is not set"))?;
    if value.is_empty() {
        return Err(format!("{VISION_MODELS_ENV} is empty"));
    }
    configured_vision_models_dir_from_path(&PathBuf::from(value))
}

fn maybe_spawn_visiond(runtime: &Path, token: &str) -> Option<VisionService> {
    let visiond_bin = match sibling_or_path("slopos-visiond") {
        Ok(path) => path,
        Err(error) => {
            eprintln!("[slopos-session] Vision daemon not started: {error}");
            return None;
        }
    };
    let models_dir = match configured_vision_models_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("[slopos-session] Vision daemon not started: {error}");
            return None;
        }
    };

    let socket_path = runtime.join("vision.sock");
    let artifact_dir = runtime.join("vision-artifacts");
    let mut command = child_command(&visiond_bin);
    command
        .env("XDG_RUNTIME_DIR", runtime)
        .env("SLOPOS_SESSION_RUNTIME_DIR", runtime)
        .env("SLOPOS_SESSION_TOKEN", token)
        .env(VISION_SOCKET_ENV, &socket_path)
        .env(VISION_MODELS_ENV, &models_dir)
        .env(VISION_ARTIFACT_ENV, &artifact_dir)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("SLOPOS_HOST_WAYLAND_DISPLAY");

    match command.spawn() {
        Ok(child) => {
            eprintln!(
                "[slopos-session] visiond pid={} socket={} models={}",
                child.id(),
                socket_path.display(),
                models_dir.display()
            );
            Some(VisionService { child, socket_path })
        }
        Err(error) => {
            eprintln!(
                "[slopos-session] Vision daemon not started from {}: {error}",
                visiond_bin.display()
            );
            None
        }
    }
}

fn apply_vision_client_environment(command: &mut Command, service: Option<&VisionService>) {
    if let Some(service) = service {
        command.env(VISION_SOCKET_ENV, &service.socket_path);
    } else {
        // Do not leak a caller-provided socket into a session that did not
        // start its own Vision service.
        command.env_remove(VISION_SOCKET_ENV);
    }
}

fn read_timeout_from_env() -> Duration {
    env::var("SLOPOS_COMPOSITOR_WAIT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .filter(|d| !d.is_zero())
        .unwrap_or(DEFAULT_STARTUP_TIMEOUT)
}

fn wait_for_private_socket(
    child: &mut Child,
    runtime: &Path,
    readiness: &Path,
    token: &str,
    timeout: Duration,
) -> Result<PrivateDisplay, String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            return Err("shutdown requested during compositor startup".to_string());
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("cannot inspect compositor process: {e}"))?
        {
            return Err(format!("slopos-compositor exited during startup: {status}"));
        }

        if let Ok(value) = fs::read_to_string(readiness) {
            let mut lines = value.lines();
            let socket_name = lines.next().unwrap_or_default().trim();
            let fields: std::collections::HashMap<&str, &str> = lines
                .filter_map(|line| line.split_once('='))
                .map(|(key, value)| (key.trim(), value.trim()))
                .collect();
            let child_matches = fields
                .get("pid")
                .and_then(|pid| pid.parse::<u32>().ok())
                .is_some_and(|pid| pid == child.id());
            let token_matches = fields.get("token").is_some_and(|value| *value == token);
            let safe_name = !socket_name.is_empty()
                && !socket_name.contains('/')
                && socket_name.starts_with("wayland-")
                && socket_name["wayland-".len()..]
                    .chars()
                    .all(|c| c.is_ascii_digit());
            let client_display_matches = fs::read_to_string(runtime.join(CLIENT_DISPLAY_FILE))
                .map(|value| value.trim() == socket_name)
                .unwrap_or(false);
            if safe_name && child_matches && token_matches && client_display_matches {
                let socket_path = runtime.join(socket_name);
                if let Ok(meta) = fs::symlink_metadata(&socket_path) {
                    if meta.file_type().is_socket() {
                        let output_width = fields
                            .get("width")
                            .and_then(|value| value.parse::<u32>().ok())
                            .filter(|value| *value > 0);
                        let output_height = fields
                            .get("height")
                            .and_then(|value| value.parse::<u32>().ok())
                            .filter(|value| *value > 0);
                        return Ok(PrivateDisplay {
                            socket_name: socket_name.to_owned(),
                            output_width,
                            output_height,
                        });
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    Err(format!(
        "slopos-compositor did not publish a live private socket within {}s",
        timeout.as_secs()
    ))
}

fn terminate_group(child: &mut Child) {
    if child.try_wait().ok().flatten().is_some() {
        return;
    }
    let pid = child.id() as i32;
    unsafe {
        libc::kill(-pid, libc::SIGTERM);
    }
    let deadline = Instant::now() + SHUTDOWN_GRACE;
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(_) => return,
        }
    }
    unsafe {
        libc::kill(-pid, libc::SIGKILL);
    }
    let _ = child.wait();
}

fn child_command(path: &Path) -> Command {
    let mut command = Command::new(path);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    // A dedicated process group lets the supervisor terminate the shell and
    // every application it launches without globbing process names.
    command.process_group(0);
    command
}

fn run() -> Result<(), String> {
    install_signal_handlers()?;
    let backend = parse_backend()?;
    validate_backend_transport(backend, env::var("DISPLAY").ok().as_deref())?;
    let base_runtime = runtime_dir()?;
    let (session_runtime, token) = create_session_runtime(&base_runtime)?;
    let runtime = &session_runtime.path;
    let readiness = runtime.join(READINESS_FILE);

    let compositor_bin = sibling_or_path("slopos-compositor")?;
    let shell_bin = sibling_or_path("slopos-shell")?;
    let mut visiond = maybe_spawn_visiond(runtime, &token);

    let mut compositor_cmd = child_command(&compositor_bin);
    compositor_cmd.arg("--backend").arg(backend.cli_value());
    compositor_cmd.env("XDG_RUNTIME_DIR", runtime);
    compositor_cmd
        .env("SLOPOS_SESSION_RUNTIME_DIR", runtime)
        .env("SLOPOS_SESSION_TOKEN", &token)
        // There is no nested-Wayland transport. Do not pass a host Wayland
        // name that could be mistaken for a connection owned by SLOPOS.
        .env_remove("SLOPOS_HOST_WAYLAND_DISPLAY");
    apply_vision_client_environment(&mut compositor_cmd, visiond.as_ref());
    // The compositor is the only SLOPOS process allowed to inherit the host
    // X11 display in nested mode. Its clients are launched later with an
    // explicit private Wayland socket.
    let mut compositor = compositor_cmd.spawn().map_err(|e| {
        if let Some(service) = visiond.as_mut() {
            terminate_group(&mut service.child);
        }
        format!("cannot start {}: {e}", compositor_bin.display())
    })?;

    let private_display = match wait_for_private_socket(
        &mut compositor,
        runtime,
        &readiness,
        &token,
        read_timeout_from_env(),
    ) {
        Ok(socket) => socket,
        Err(error) => {
            terminate_group(&mut compositor);
            if let Some(service) = visiond.as_mut() {
                terminate_group(&mut service.child);
            }
            return Err(error);
        }
    };

    eprintln!(
        "[slopos-session] compositor pid={} backend={} client_socket={}",
        compositor.id(),
        backend.cli_value(),
        private_display.socket_name
    );

    let mut shell_cmd = child_command(&shell_bin);
    shell_cmd
        .env("XDG_RUNTIME_DIR", runtime)
        .env("SLOPOS_SESSION_RUNTIME_DIR", runtime)
        .env("WAYLAND_DISPLAY", &private_display.socket_name)
        .env(
            "SLOPOS_CLIENT_WAYLAND_DISPLAY",
            &private_display.socket_name,
        )
        // Linux production shell chrome is always layer-shell; keep this
        // explicit in the child environment for diagnostics and direct
        // consumers of the session contract.
        .env("SLOPOS_LAYER_SHELL_CHROME", "1")
        .env(
            "SLOPOS_ACTIVE_TOPLEVEL_FILE",
            runtime.join("active-toplevel"),
        )
        .env_remove("SLOPOS_HOST_WAYLAND_DISPLAY");
    apply_vision_client_environment(&mut shell_cmd, visiond.as_ref());
    if let (Some(width), Some(height)) =
        (private_display.output_width, private_display.output_height)
    {
        shell_cmd
            .env("SLOPOS_COMPOSITOR_WIDTH", width.to_string())
            .env("SLOPOS_COMPOSITOR_HEIGHT", height.to_string());
    }
    if env::var_os("SLOPOS_KEEP_DISPLAY").is_none() {
        shell_cmd.env_remove("DISPLAY");
    }
    let mut shell = match shell_cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            terminate_group(&mut compositor);
            if let Some(service) = visiond.as_mut() {
                terminate_group(&mut service.child);
            }
            return Err(format!("cannot start {}: {error}", shell_bin.display()));
        }
    };

    eprintln!("[slopos-session] shell pid={}", shell.id());

    let result = loop {
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            eprintln!("[slopos-session] shutdown signal received; stopping session");
            terminate_group(&mut shell);
            terminate_group(&mut compositor);
            break Ok(());
        }
        if let Some(status) = compositor
            .try_wait()
            .map_err(|e| format!("cannot wait for compositor: {e}"))?
        {
            terminate_group(&mut shell);
            if let Some(service) = visiond.as_mut() {
                terminate_group(&mut service.child);
            }
            break Err(format!("slopos-compositor exited: {status}"));
        }
        let visiond_status = match visiond.as_mut() {
            Some(service) => service
                .child
                .try_wait()
                .map_err(|e| format!("cannot wait for slopos-visiond: {e}"))?,
            None => None,
        };
        if let Some(status) = visiond_status {
            eprintln!(
                "[slopos-session] slopos-visiond exited: {status}; Vision is unavailable for this session"
            );
            visiond = None;
        }
        if let Some(status) = shell
            .try_wait()
            .map_err(|e| format!("cannot wait for shell: {e}"))?
        {
            terminate_group(&mut compositor);
            if let Some(service) = visiond.as_mut() {
                terminate_group(&mut service.child);
            }
            if status.success() {
                break Ok(());
            }
            break Err(format!("slopos-shell exited: {status}"));
        }
        thread::sleep(Duration::from_millis(100));
    };

    if let Some(service) = visiond.as_mut() {
        terminate_group(&mut service.child);
    }
    result
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("slopos-session: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_backend_requires_x11_display_for_nested() {
        assert_eq!(default_backend_for_host(Some(":99"), None), Backend::Nested);
        assert_eq!(
            default_backend_for_host(None, Some("wayland-0")),
            Backend::Drm,
            "a Wayland-only host must not select the X11 nested backend"
        );
        assert_eq!(default_backend_for_host(Some(""), None), Backend::Drm);
        assert_eq!(default_backend_for_host(None, None), Backend::Drm);
    }

    #[test]
    fn explicit_nested_backend_fails_without_x11_display() {
        let error = validate_backend_transport(Backend::Nested, None).unwrap_err();
        assert!(error.contains("DISPLAY"));
        assert!(validate_backend_transport(Backend::Nested, Some(":99")).is_ok());
        assert!(validate_backend_transport(Backend::Drm, None).is_ok());
        assert!(validate_backend_transport(Backend::Headless, None).is_ok());
    }

    #[test]
    fn vision_models_directory_requires_a_manifest() {
        let path = env::temp_dir().join(format!("slopos-session-vision-{}", session_nonce()));
        fs::create_dir(&path).unwrap();
        assert!(configured_vision_models_dir_from_path(&path).is_err());

        fs::write(path.join("manifest.toml"), b"models = []").unwrap();
        assert_eq!(
            configured_vision_models_dir_from_path(&path).unwrap(),
            fs::canonicalize(&path).unwrap()
        );

        fs::remove_dir_all(path).unwrap();
    }
}
