//! Session-scoped control messages shared by shell clients and the compositor.
//!
//! The shell owns global chrome, but it is not a window manager.  These messages
//! are the small control plane used when a shell action needs the compositor to
//! operate on the focused real client.  The endpoint lives inside the unique
//! session runtime directory created by `slopos-session`; it is never discovered
//! by scanning arbitrary Wayland sockets.

use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

use crate::spaces::SpacesControlCommand;

pub const SESSION_CONTROL_SOCKET: &str = "control.sock";
const APPLICATION_CONTROL_DIR: &str = "app-control";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WindowPresentationAction {
    ToggleZoom,
    SmartZoom,
    Fill,
    ToggleFullscreen,
    Fullscreen,
    Minimize,
    Restore,
    Close,
}

/// Input events accepted only by the explicitly enabled headless protocol
/// test harness. Coordinates are compositor-space logical pixels and button
/// codes use Linux input-event-codes values (for example, 0x110 for BTN_LEFT).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HeadlessInputEvent {
    Motion {
        x: i32,
        y: i32,
        time_msec: u32,
    },
    Button {
        button: u32,
        pressed: bool,
        time_msec: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SessionControlRequest {
    FocusedWindow {
        action: WindowPresentationAction,
    },
    /// Activate an existing compositor-owned application window from shell
    /// chrome such as the Dock. The compositor remains the sole owner of
    /// focus, stacking, and restore geometry.
    ActivateApplication {
        bundle_id: String,
    },
    /// Activate one of the current compositor's legacy indexed Spaces.
    /// Dynamic shell controls use the stable-ID [`Self::Spaces`] request.
    SwitchWorkspace {
        index: u8,
    },
    /// Apply a dynamic compositor-owned Spaces mutation.  The compositor
    /// publishes the resulting [`crate::SpacesSnapshot`] in the session
    /// runtime directory after a successful command.
    Spaces {
        command: SpacesControlCommand,
    },
    /// Atomically replace the compositor's logical output topology.
    /// The value uses `name:WIDTHxHEIGHT@x,y:sSCALE` entries separated by `;`.
    ReconfigureOutputs {
        layout: String,
    },
    /// Drive the nested/headless compositor's Smithay pointer path for a
    /// deterministic protocol test. Production nested and DRM sessions
    /// explicitly ignore this request.
    HeadlessTestInput {
        event: HeadlessInputEvent,
    },
    FocusedApplicationMenu {
        bundle_id: String,
        action_id: String,
    },
}

/// Name of the atomic compositor-to-shell Spaces projection in the session
/// runtime directory.
pub const SPACES_STATE_FILE: &str = "spaces-state.json";

/// Return the exact Spaces snapshot path for the current session.
pub fn session_spaces_state_path() -> Option<PathBuf> {
    std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR")
        .map(PathBuf::from)
        .map(|runtime| runtime.join(SPACES_STATE_FILE))
}

/// Publish one complete Spaces snapshot atomically.  The runtime directory is
/// session-scoped and owned by the session supervisor; readers never observe a
/// partially-written JSON document.
#[cfg(unix)]
pub fn write_spaces_snapshot(snapshot: &crate::SpacesSnapshot) -> io::Result<()> {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    let path = session_spaces_state_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "SLOPOS_SESSION_RUNTIME_DIR is not set",
        )
    })?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Spaces path has no parent"))?;
    fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(snapshot).map_err(io::Error::other)?;
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp = parent.join(format!(".{}.{}.tmp", SPACES_STATE_FILE, counter));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(not(unix))]
pub fn write_spaces_snapshot(_snapshot: &crate::SpacesSnapshot) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Spaces snapshots require the Unix session runtime",
    ))
}

/// Read the latest compositor snapshot, if the session has published one.
pub fn read_spaces_snapshot() -> io::Result<crate::SpacesSnapshot> {
    let path = session_spaces_state_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "SLOPOS_SESSION_RUNTIME_DIR is not set",
        )
    })?;
    let bytes = std::fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApplicationMenuRequest {
    pub bundle_id: String,
    pub action_id: String,
}

pub fn session_control_socket_path() -> Option<PathBuf> {
    std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR")
        .map(PathBuf::from)
        .map(|runtime| runtime.join(SESSION_CONTROL_SOCKET))
}

fn safe_socket_component(value: &str) -> String {
    let component: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if component.is_empty() {
        "application".to_string()
    } else {
        component
    }
}

pub fn application_control_socket_path(bundle_id: &str) -> Option<PathBuf> {
    std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR")
        .map(PathBuf::from)
        .map(|runtime| {
            runtime
                .join(APPLICATION_CONTROL_DIR)
                .join(format!("{}.sock", safe_socket_component(bundle_id)))
        })
}

#[cfg(unix)]
pub fn send_application_menu_action(bundle_id: &str, action_id: &str) -> io::Result<()> {
    use std::os::unix::net::UnixDatagram;

    let path = application_control_socket_path(bundle_id).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "SLOPOS_SESSION_RUNTIME_DIR is not set",
        )
    })?;
    let request = ApplicationMenuRequest {
        bundle_id: bundle_id.to_string(),
        action_id: action_id.to_string(),
    };
    let payload = serde_json::to_vec(&request).map_err(io::Error::other)?;
    UnixDatagram::unbound()?.send_to(&payload, path).map(|_| ())
}

#[cfg(not(unix))]
pub fn send_application_menu_action(_bundle_id: &str, _action_id: &str) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "application menu control requires a Unix-domain socket",
    ))
}

#[cfg(unix)]
pub fn send_session_control(request: &SessionControlRequest) -> io::Result<()> {
    use std::os::unix::net::UnixDatagram;

    let path = session_control_socket_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "SLOPOS_SESSION_RUNTIME_DIR is not set",
        )
    })?;
    let payload = serde_json::to_vec(request).map_err(io::Error::other)?;
    UnixDatagram::unbound()?.send_to(&payload, path).map(|_| ())
}

#[cfg(not(unix))]
pub fn send_session_control(_request: &SessionControlRequest) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "session control requires a Unix-domain socket",
    ))
}

#[cfg(unix)]
pub struct SessionControlListener {
    socket: std::os::unix::net::UnixDatagram,
    path: PathBuf,
}

#[cfg(unix)]
impl std::os::fd::AsFd for SessionControlListener {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.socket.as_fd()
    }
}

#[cfg(unix)]
impl SessionControlListener {
    /// Bind the exact socket owned by this session.  The runtime directory is
    /// already restricted to the session user by `slopos-session`.
    pub fn bind(runtime: &Path) -> io::Result<Self> {
        use std::os::unix::net::UnixDatagram;

        let path = runtime.join(SESSION_CONTROL_SOCKET);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let socket = UnixDatagram::bind(&path)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket, path })
    }

    pub fn drain(&self) -> Vec<SessionControlRequest> {
        let mut requests = Vec::new();
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            match self.socket.recv(&mut buffer) {
                Ok(size) => match serde_json::from_slice(&buffer[..size]) {
                    Ok(request) => requests.push(request),
                    Err(error) => {
                        tracing::warn!(%error, "discarding malformed session control request")
                    }
                },
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => {
                    tracing::warn!(%error, "session control socket read failed");
                    break;
                }
            }
        }
        requests
    }
}

#[cfg(unix)]
impl Drop for SessionControlListener {
    fn drop(&mut self) {
        // This is the exact socket created by this listener, never a glob or a
        // host Wayland socket.  Ignore a prior session supervisor cleanup.
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(unix)]
pub struct ApplicationControlListener {
    socket: std::os::unix::net::UnixDatagram,
    path: PathBuf,
}

#[cfg(unix)]
impl ApplicationControlListener {
    pub fn bind(bundle_id: &str) -> io::Result<Self> {
        let runtime = std::env::var_os("SLOPOS_SESSION_RUNTIME_DIR").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "SLOPOS_SESSION_RUNTIME_DIR is not set",
            )
        })?;
        Self::bind_at(bundle_id, Path::new(&runtime))
    }

    /// Bind an endpoint inside an explicit session runtime directory. The
    /// explicit form keeps tests and launchers from mutating process-global
    /// environment while exercising the application control plane.
    pub fn bind_at(bundle_id: &str, runtime: &Path) -> io::Result<Self> {
        use std::os::unix::net::UnixDatagram;

        let path = runtime
            .join(APPLICATION_CONTROL_DIR)
            .join(format!("{}.sock", safe_socket_component(bundle_id)));
        let directory = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "application control path has no parent",
            )
        })?;
        std::fs::create_dir_all(directory)?;
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                format!(
                    "application control endpoint already exists: {}",
                    path.display()
                ),
            ));
        }
        let socket = UnixDatagram::bind(&path)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket, path })
    }

    pub fn drain(&self) -> Vec<ApplicationMenuRequest> {
        let mut requests = Vec::new();
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            match self.socket.recv(&mut buffer) {
                Ok(size) => match serde_json::from_slice(&buffer[..size]) {
                    Ok(request) => requests.push(request),
                    Err(error) => {
                        tracing::warn!(%error, "discarding malformed application menu request")
                    }
                },
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => {
                    tracing::warn!(%error, "application control socket read failed");
                    break;
                }
            }
        }
        requests
    }

    /// Receive one application-menu request without polling.
    ///
    /// SDK clients normally sleep in their event loop while idle.  A blocking
    /// listener thread can therefore wait on the exact per-application socket
    /// and wake the UI event loop through its proxy, instead of making every
    /// client spin in `ControlFlow::Poll` or relying on an unrelated redraw.
    pub fn recv_blocking(&self) -> io::Result<ApplicationMenuRequest> {
        self.socket.set_nonblocking(false)?;
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let size = self.socket.recv(&mut buffer)?;
            match serde_json::from_slice(&buffer[..size]) {
                Ok(request) => return Ok(request),
                Err(error) => {
                    tracing::warn!(%error, "discarding malformed application menu request")
                }
            }
        }
    }
}

#[cfg(unix)]
impl Drop for ApplicationControlListener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(not(unix))]
pub struct ApplicationControlListener;

#[cfg(not(unix))]
impl ApplicationControlListener {
    pub fn bind(_bundle_id: &str) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "application menu control requires a Unix-domain socket",
        ))
    }

    pub fn drain(&self) -> Vec<ApplicationMenuRequest> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_round_trips_through_json() {
        let request = SessionControlRequest::FocusedWindow {
            action: WindowPresentationAction::ToggleFullscreen,
        };
        let encoded = serde_json::to_vec(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<SessionControlRequest>(&encoded).unwrap(),
            request
        );
    }

    #[test]
    fn output_reconfiguration_request_round_trips_through_json() {
        let request = SessionControlRequest::ReconfigureOutputs {
            layout: "LEFT:800x600@0,0:s100;RIGHT:1024x768@800,0:s100".into(),
        };
        let encoded = serde_json::to_vec(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<SessionControlRequest>(&encoded).unwrap(),
            request
        );
    }

    #[test]
    fn activate_application_request_round_trips_through_json() {
        let request = SessionControlRequest::ActivateApplication {
            bundle_id: "com.slopos.settings".into(),
        };
        let encoded = serde_json::to_vec(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<SessionControlRequest>(&encoded).unwrap(),
            request
        );
    }

    #[test]
    fn switch_workspace_request_round_trips_through_json() {
        let request = SessionControlRequest::SwitchWorkspace { index: 3 };
        let encoded = serde_json::to_vec(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<SessionControlRequest>(&encoded).unwrap(),
            request
        );
    }

    #[test]
    fn headless_test_input_request_round_trips_through_json() {
        let request = SessionControlRequest::HeadlessTestInput {
            event: HeadlessInputEvent::Motion {
                x: 70,
                y: 70,
                time_msec: 100,
            },
        };
        let encoded = serde_json::to_vec(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<SessionControlRequest>(&encoded).unwrap(),
            request
        );
    }

    #[cfg(unix)]
    #[test]
    fn listener_drains_typed_requests() {
        use std::os::unix::net::UnixDatagram;
        // macOS limits Unix-domain socket paths to a small fixed byte budget.
        // Keep the per-process directory name short so this test also works
        // under long `$TMPDIR` paths used by the default macOS test runner.
        let runtime = std::env::temp_dir().join(format!("slo-{}", std::process::id()));
        std::fs::create_dir_all(&runtime).unwrap();
        let listener = SessionControlListener::bind(&runtime).unwrap();
        let sender = UnixDatagram::unbound().unwrap();
        let request = SessionControlRequest::FocusedWindow {
            action: WindowPresentationAction::Minimize,
        };
        sender
            .send_to(
                serde_json::to_vec(&request).unwrap().as_slice(),
                runtime.join(SESSION_CONTROL_SOCKET),
            )
            .unwrap();
        assert_eq!(listener.drain(), vec![request]);
        drop(listener);
        let _ = std::fs::remove_dir(runtime);
    }

    #[cfg(unix)]
    #[test]
    fn listener_drains_switch_workspace_request() {
        use std::os::unix::net::UnixDatagram;

        let runtime = std::env::temp_dir().join(format!("slo-ws-{}", std::process::id()));
        std::fs::create_dir_all(&runtime).unwrap();
        let listener = SessionControlListener::bind(&runtime).unwrap();
        let sender = UnixDatagram::unbound().unwrap();
        let request = SessionControlRequest::SwitchWorkspace { index: 6 };
        sender
            .send_to(
                serde_json::to_vec(&request).unwrap().as_slice(),
                runtime.join(SESSION_CONTROL_SOCKET),
            )
            .unwrap();
        assert_eq!(listener.drain(), vec![request]);
        drop(listener);
        let _ = std::fs::remove_dir(runtime);
    }

    #[cfg(unix)]
    #[test]
    fn application_listener_drains_typed_menu_requests() {
        use std::os::unix::net::UnixDatagram;

        let runtime = std::env::temp_dir().join(format!("slo-app-{}", std::process::id()));
        std::fs::create_dir_all(&runtime).unwrap();
        let bundle_id = "com.slopos.test";
        let listener = ApplicationControlListener::bind_at(bundle_id, &runtime).unwrap();
        let socket_path = runtime
            .join(APPLICATION_CONTROL_DIR)
            .join("com.slopos.test.sock");
        let sender = UnixDatagram::unbound().unwrap();
        let request = ApplicationMenuRequest {
            bundle_id: bundle_id.to_string(),
            action_id: "com.slopos.test.file.open".to_string(),
        };
        sender
            .send_to(
                serde_json::to_vec(&request).unwrap().as_slice(),
                socket_path,
            )
            .unwrap();
        assert_eq!(listener.drain(), vec![request]);
        drop(listener);
        let _ = std::fs::remove_dir(runtime.join(APPLICATION_CONTROL_DIR));
        let _ = std::fs::remove_dir(runtime);
    }
}
