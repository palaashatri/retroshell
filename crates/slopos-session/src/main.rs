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
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const READINESS_FILE: &str = "slopos-client-wayland-display";
const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(12);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backend {
    Drm,
    Nested,
    Headless,
}

impl Backend {
    fn cli_value(self) -> &'static str {
        match self {
            Self::Drm => "drm",
            Self::Nested => "x11",
            Self::Headless => "headless",
        }
    }
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
        None => {
            if env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some() {
                Ok(Backend::Nested)
            } else {
                Ok(Backend::Drm)
            }
        }
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

fn read_timeout_from_env() -> Duration {
    env::var("SLOPOS_COMPOSITOR_WAIT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .filter(|d| !d.is_zero())
        .unwrap_or(DEFAULT_STARTUP_TIMEOUT)
}

fn remove_readiness_file(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "cannot remove stale readiness file {}: {e}",
            path.display()
        )),
    }
}

fn wait_for_private_socket(
    child: &mut Child,
    runtime: &Path,
    readiness: &Path,
    timeout: Duration,
) -> Result<String, String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("cannot inspect compositor process: {e}"))?
        {
            return Err(format!("slopos-compositor exited during startup: {status}"));
        }

        if let Ok(value) = fs::read_to_string(readiness) {
            let socket_name = value.trim();
            let safe_name = !socket_name.is_empty()
                && !socket_name.contains('/')
                && socket_name.starts_with("wayland-")
                && socket_name["wayland-".len()..]
                    .chars()
                    .all(|c| c.is_ascii_digit());
            if safe_name {
                let socket_path = runtime.join(socket_name);
                if let Ok(meta) = fs::symlink_metadata(&socket_path) {
                    if meta.file_type().is_socket() {
                        return Ok(socket_name.to_owned());
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
    let backend = parse_backend()?;
    let runtime = runtime_dir()?;
    let readiness = runtime.join(READINESS_FILE);
    remove_readiness_file(&readiness)?;

    let compositor_bin = sibling_or_path("slopos-compositor")?;
    let shell_bin = sibling_or_path("slopos-shell")?;

    let host_wayland: Option<OsString> = env::var_os("WAYLAND_DISPLAY");
    let mut compositor_cmd = child_command(&compositor_bin);
    compositor_cmd.arg("--backend").arg(backend.cli_value());
    compositor_cmd.env("XDG_RUNTIME_DIR", &runtime);
    if let Some(host) = host_wayland.as_ref() {
        compositor_cmd.env("SLOPOS_HOST_WAYLAND_DISPLAY", host);
    }
    // The compositor is the only SLOPOS process allowed to inherit the host
    // socket. Its clients are launched later with an explicit private socket.
    let mut compositor = compositor_cmd
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", compositor_bin.display()))?;

    let private_socket = match wait_for_private_socket(
        &mut compositor,
        &runtime,
        &readiness,
        read_timeout_from_env(),
    ) {
        Ok(socket) => socket,
        Err(error) => {
            terminate_group(&mut compositor);
            return Err(error);
        }
    };

    eprintln!(
        "[slopos-session] compositor pid={} backend={} client_socket={}",
        compositor.id(),
        backend.cli_value(),
        private_socket
    );

    let mut shell_cmd = child_command(&shell_bin);
    shell_cmd
        .env("XDG_RUNTIME_DIR", &runtime)
        .env("WAYLAND_DISPLAY", &private_socket)
        .env("SLOPOS_CLIENT_WAYLAND_DISPLAY", &private_socket)
        .env_remove("SLOPOS_HOST_WAYLAND_DISPLAY");
    if env::var_os("SLOPOS_KEEP_DISPLAY").is_none() {
        shell_cmd.env_remove("DISPLAY");
    }
    let mut shell = match shell_cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            terminate_group(&mut compositor);
            return Err(format!("cannot start {}: {error}", shell_bin.display()));
        }
    };

    eprintln!("[slopos-session] shell pid={}", shell.id());

    let result = loop {
        if let Some(status) = compositor
            .try_wait()
            .map_err(|e| format!("cannot wait for compositor: {e}"))?
        {
            terminate_group(&mut shell);
            break Err(format!("slopos-compositor exited: {status}"));
        }
        if let Some(status) = shell
            .try_wait()
            .map_err(|e| format!("cannot wait for shell: {e}"))?
        {
            terminate_group(&mut compositor);
            if status.success() {
                break Ok(());
            }
            break Err(format!("slopos-shell exited: {status}"));
        }
        thread::sleep(Duration::from_millis(100));
    };

    let _ = remove_readiness_file(&readiness);
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
