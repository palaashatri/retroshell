//! FreeDesktop portal-facing surface (xdg-desktop-portal Screenshot / Settings / OpenURI / ScreenCast).
//!
//! Pure request/result types and handlers are portable (macOS host tests). Linux session-bus
//! export lives in [`crate::portal_dbus`] and calls these pure functions.
//!
//! # D-Bus well-known names / paths (impl side)
//!
//! RetroShell registers a simplified portal backend third parties can call directly:
//!
//! | Role | Value |
//! |------|--------|
//! | Bus name | [`PORTAL_BUS_NAME`] (`org.retroshell.Portal`) |
//! | Object path | [`PORTAL_PATH`] (`/org/retroshell/portal`) |
//! | Screenshot iface | [`PORTAL_SCREENSHOT_INTERFACE`] (`org.freedesktop.impl.portal.Screenshot`) |
//! | Settings iface | [`PORTAL_SETTINGS_INTERFACE`] (`org.freedesktop.impl.portal.Settings`) |
//! | OpenURI iface | [`PORTAL_OPENURI_INTERFACE`] (`org.freedesktop.impl.portal.OpenURI`) |
//! | ScreenCast iface | [`PORTAL_SCREENCAST_INTERFACE`] (`org.freedesktop.impl.portal.ScreenCast`) |
//!
//! Local shell menus still use [`take_portal_style_screenshot`] (capture path) via
//! `shell.portal_screenshot`.
//!
//! # ScreenCast note
//!
//! [`ScreencastStream`] values are **protocol-level stubs**. `node_id` fields are placeholders
//! for a future PipeWire graph — this code does **not** create live PipeWire nodes or streams.
//! Sessions carry an honest [`PortalScreencastSession::backend_note`]
//! (`backend=portal_stub` or `backend=pipewire_socket_present`) from readiness probe.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::capture::{take_screenshot, CaptureError};

// ---------------------------------------------------------------------------
// D-Bus constants (session-bus portal backend)
// ---------------------------------------------------------------------------

/// Well-known session bus name for RetroShell portal handlers.
pub const PORTAL_BUS_NAME: &str = "org.retroshell.Portal";
/// Object path for portal interfaces.
pub const PORTAL_PATH: &str = "/org/retroshell/portal";
/// FreeDesktop portal Screenshot implementation interface.
pub const PORTAL_SCREENSHOT_INTERFACE: &str = "org.freedesktop.impl.portal.Screenshot";
/// FreeDesktop portal Settings implementation interface.
pub const PORTAL_SETTINGS_INTERFACE: &str = "org.freedesktop.impl.portal.Settings";
/// FreeDesktop portal OpenURI implementation interface.
pub const PORTAL_OPENURI_INTERFACE: &str = "org.freedesktop.impl.portal.OpenURI";
/// FreeDesktop portal FileChooser implementation interface.
pub const PORTAL_FILECHOOSER_INTERFACE: &str = "org.freedesktop.impl.portal.FileChooser";
/// FreeDesktop portal ScreenCast implementation interface.
pub const PORTAL_SCREENCAST_INTERFACE: &str = "org.freedesktop.impl.portal.ScreenCast";

/// ScreenCast source type bit: monitors / outputs.
pub const SCREENCAST_SOURCE_TYPE_MONITOR: u32 = 1;
/// ScreenCast source type bit: application windows.
pub const SCREENCAST_SOURCE_TYPE_WINDOW: u32 = 2;

/// Default placeholder PipeWire-style node id (not a live graph node).
pub const SCREENCAST_PLACEHOLDER_NODE_ID: u32 = 42;
/// Default stub stream width.
pub const SCREENCAST_DEFAULT_WIDTH: u32 = 1920;
/// Default stub stream height.
pub const SCREENCAST_DEFAULT_HEIGHT: u32 = 1080;

// ---------------------------------------------------------------------------
// Screenshot
// ---------------------------------------------------------------------------

/// Options corresponding to xdg-desktop-portal Screenshot request hints.
///
/// Kept pure so callers and tests can build requests without a session bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PortalScreenshotRequest {
    /// When true, the portal would show an interactive UI (region/window pick).
    pub interactive: bool,
    /// When true, the portal would include the pointer cursor in the image.
    pub include_cursor: bool,
}

/// Successful screenshot result (file path + recorded portal options).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalScreenshotResult {
    pub path: PathBuf,
    /// Request options as recorded by the pure portal handler.
    pub options: PortalScreenshotRequest,
}

/// Build the default portal-style screenshot filename (pure helper for tests).
///
/// Example: `RetroShell-Portal-Screenshot-1710000000.png`
pub fn portal_screenshot_filename(now_unix_secs: u64) -> String {
    format!("RetroShell-Portal-Screenshot-{now_unix_secs}.png")
}

/// Pure: screenshots directory under a base path (typically `$HOME`).
///
/// Example: `base/Pictures/Screenshots`
pub fn portal_screenshots_dir(base: &Path) -> PathBuf {
    base.join("Pictures").join("Screenshots")
}

/// Pure: `file://` URI for a screenshot path (portal results use this form).
pub fn portal_screenshot_uri_for(path: &Path) -> String {
    // Absolute paths: file:///abs/path ; relative: file://rel/path
    let s = path.to_string_lossy();
    if path.is_absolute() {
        format!("file://{s}")
    } else {
        format!("file://{s}")
    }
}

/// Pure portal Screenshot handler: plans path under `screenshots_dir`, records options.
///
/// Does **not** perform capture — D-Bus / shell code may capture separately and still
/// use this helper for deterministic filename/URI planning and option recording.
pub fn handle_portal_screenshot_request(
    request: PortalScreenshotRequest,
    screenshots_dir: &Path,
    now_unix_secs: u64,
) -> PortalScreenshotResult {
    let path = screenshots_dir.join(portal_screenshot_filename(now_unix_secs));
    PortalScreenshotResult {
        path,
        options: request,
    }
}

/// Take a screenshot through the portal-facing API surface.
///
/// Local capture via [`crate::capture::take_screenshot`]. Interactive/cursor options
/// are not yet honored by the capture backend.
pub fn take_portal_style_screenshot() -> Result<PathBuf, CaptureError> {
    take_screenshot()
}

/// Portal-style capture with explicit request options.
///
/// Options are recorded on the result; local capture still ignores interactive/cursor.
pub fn take_portal_style_screenshot_with(
    request: PortalScreenshotRequest,
) -> Result<PortalScreenshotResult, CaptureError> {
    let path = take_portal_style_screenshot()?;
    Ok(PortalScreenshotResult {
        path,
        options: request,
    })
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// FreeDesktop Settings portal namespaces RetroShell exposes in the pure map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortalSettingsNamespace {
    /// `org.freedesktop.appearance` — color-scheme, accent-color, etc.
    Appearance,
}

impl PortalSettingsNamespace {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Appearance => "org.freedesktop.appearance",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "org.freedesktop.appearance" => Some(Self::Appearance),
            _ => None,
        }
    }
}

/// Pure Settings.Read: look up a portal setting from the built-in map.
///
/// Known keys:
/// - `org.freedesktop.appearance` / `color-scheme` → `"0"` (no preference; FDO uint32 0)
/// - `org.freedesktop.appearance` / `accent-color` → `""` (unset)
pub fn read_portal_setting(namespace: &str, key: &str) -> Option<String> {
    match (namespace, key) {
        ("org.freedesktop.appearance", "color-scheme") => Some("0".to_string()),
        ("org.freedesktop.appearance", "accent-color") => Some(String::new()),
        _ => None,
    }
}

/// Pure Settings.ReadAll for a namespace.
pub fn read_all_portal_settings(namespace: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    match namespace {
        "org.freedesktop.appearance" => {
            if let Some(v) = read_portal_setting(namespace, "color-scheme") {
                map.insert("color-scheme".to_string(), v);
            }
            if let Some(v) = read_portal_setting(namespace, "accent-color") {
                map.insert("accent-color".to_string(), v);
            }
        }
        _ => {}
    }
    map
}

// ---------------------------------------------------------------------------
// FileChooser
// ---------------------------------------------------------------------------

/// FileChooser portal request (OpenFile / SaveFile simplified).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PortalFileChooserRequest {
    pub title: String,
    pub accept_label: String,
    pub multiple: bool,
    pub directory: bool,
    /// Suggested filters as glob patterns (e.g. `*.png`).
    pub filters: Vec<String>,
    /// Current folder hint (absolute path string).
    pub current_folder: Option<String>,
    /// Suggested save name for SaveFile.
    pub current_name: Option<String>,
}

/// Pure FileChooser result (uris selected).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalFileChooserResult {
    pub uris: Vec<String>,
    pub choices: HashMap<String, String>,
}

/// Validate a FileChooser request (pure).
pub fn validate_file_chooser_request(req: &PortalFileChooserRequest) -> Result<(), String> {
    if req.title.trim().is_empty() {
        return Err("title empty".into());
    }
    if let Some(folder) = &req.current_folder {
        if folder.contains('\0') {
            return Err("current_folder contains null".into());
        }
    }
    Ok(())
}

/// Pure OpenFile handler: returns `file://` URIs under `current_folder` for
/// synthetic selection used by tests and non-interactive agents.
///
/// When `selected_names` is empty, returns Ok with empty uris (user cancelled).
pub fn handle_file_chooser_open(
    req: &PortalFileChooserRequest,
    selected_names: &[&str],
) -> Result<PortalFileChooserResult, String> {
    validate_file_chooser_request(req)?;
    if selected_names.is_empty() {
        return Ok(PortalFileChooserResult {
            uris: Vec::new(),
            choices: HashMap::new(),
        });
    }
    if !req.multiple && selected_names.len() > 1 {
        return Err("multiple selection not allowed".into());
    }
    let base = req
        .current_folder
        .clone()
        .unwrap_or_else(|| "/tmp".into());
    let uris: Vec<String> = selected_names
        .iter()
        .map(|name| {
            let path = Path::new(&base).join(name);
            portal_screenshot_uri_for(&path)
        })
        .collect();
    Ok(PortalFileChooserResult {
        uris,
        choices: HashMap::new(),
    })
}

/// Pure SaveFile handler: builds a single destination URI.
pub fn handle_file_chooser_save(
    req: &PortalFileChooserRequest,
    confirm: bool,
) -> Result<PortalFileChooserResult, String> {
    validate_file_chooser_request(req)?;
    if !confirm {
        return Ok(PortalFileChooserResult {
            uris: Vec::new(),
            choices: HashMap::new(),
        });
    }
    let name = req
        .current_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Untitled".into());
    let base = req
        .current_folder
        .clone()
        .unwrap_or_else(|| "/tmp".into());
    let path = Path::new(&base).join(name);
    Ok(PortalFileChooserResult {
        uris: vec![portal_screenshot_uri_for(&path)],
        choices: HashMap::new(),
    })
}

// ---------------------------------------------------------------------------
// ScreenCast
// ---------------------------------------------------------------------------

/// Options corresponding to xdg-desktop-portal ScreenCast SelectSources hints.
///
/// `types` is a bitfield of [`SCREENCAST_SOURCE_TYPE_MONITOR`] /
/// [`SCREENCAST_SOURCE_TYPE_WINDOW`]. `cursor_mode` follows the portal cursor
/// mode bits (Hidden=1, Embedded=2, Metadata=4) but is only recorded here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PortalScreencastRequest {
    /// Source type bitfield (monitor / window).
    pub types: u32,
    /// When true, more than one source may be selected.
    pub multiple: bool,
    /// Cursor mode bitfield (recorded only; not applied to a live capture).
    pub cursor_mode: u32,
}

/// One ScreenCast stream entry returned by Start.
///
/// **Stub:** `node_id` is a protocol-level placeholder, not a live PipeWire node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreencastStream {
    /// Placeholder node id for portal clients (not a live PipeWire graph node).
    pub node_id: u32,
    pub width: u32,
    pub height: u32,
    /// Source type bit (monitor or window).
    pub source_type: u32,
}

/// ScreenCast session state held by pure handlers (and by the D-Bus session map).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalScreencastSession {
    pub session_id: String,
    /// Streams for Start; initially one monitor stub after create.
    pub streams: Vec<ScreencastStream>,
    /// Request options recorded at create.
    pub types: u32,
    pub multiple: bool,
    pub cursor_mode: u32,
    /// True after a successful [`select_screencast_sources`].
    pub sources_selected: bool,
    /// True after a successful [`start_screencast`].
    pub started: bool,
    /// Honest readiness note (`backend=portal_stub` or `backend=pipewire_socket_present`).
    ///
    /// Socket presence does **not** mean live PipeWire streams are exported.
    pub backend_note: String,
}

/// Portal Start / CreateSession `note` when only protocol stubs are available.
pub const SCREENCAST_NOTE_PORTAL_STUB: &str = "backend=portal_stub";
/// Portal note when the PipeWire socket exists (still not a live stream claim).
pub const SCREENCAST_NOTE_PIPEWIRE_SOCKET: &str = "backend=pipewire_socket_present";

/// Pure map from screencast readiness → honest backend note string.
///
/// `pipewire_socket_present` yields [`SCREENCAST_NOTE_PIPEWIRE_SOCKET`]; otherwise stub.
/// Never claims live PipeWire graph export.
pub fn screencast_backend_note(ready: &crate::screencast_pw::ScreencastReadiness) -> String {
    if ready.pipewire_socket_present {
        SCREENCAST_NOTE_PIPEWIRE_SOCKET.to_string()
    } else {
        SCREENCAST_NOTE_PORTAL_STUB.to_string()
    }
}

/// Pure map from socket presence alone (tests without full readiness struct).
pub fn screencast_backend_note_from_socket(socket_present: bool) -> String {
    if socket_present {
        SCREENCAST_NOTE_PIPEWIRE_SOCKET.to_string()
    } else {
        SCREENCAST_NOTE_PORTAL_STUB.to_string()
    }
}

static NEXT_SCREENCAST_SESSION: AtomicU64 = AtomicU64::new(1);

fn default_source_type(types: u32) -> u32 {
    if types & SCREENCAST_SOURCE_TYPE_MONITOR != 0 {
        SCREENCAST_SOURCE_TYPE_MONITOR
    } else if types & SCREENCAST_SOURCE_TYPE_WINDOW != 0 {
        SCREENCAST_SOURCE_TYPE_WINDOW
    } else {
        // Portal clients that pass types=0 still get a monitor stub.
        SCREENCAST_SOURCE_TYPE_MONITOR
    }
}

fn placeholder_stream(node_id: u32, source_type: u32) -> ScreencastStream {
    ScreencastStream {
        node_id,
        width: SCREENCAST_DEFAULT_WIDTH,
        height: SCREENCAST_DEFAULT_HEIGHT,
        source_type,
    }
}

/// Pure ScreenCast CreateSession: assigns an incremental session id and one monitor stream stub.
///
/// The stream's `node_id` is a placeholder — PipeWire is not started or connected.
/// Default backend note is [`SCREENCAST_NOTE_PORTAL_STUB`]; use
/// [`create_screencast_session_with_backend_note`] when a readiness probe is available.
pub fn create_screencast_session(req: PortalScreencastRequest) -> PortalScreencastSession {
    create_screencast_session_with_backend_note(req, SCREENCAST_NOTE_PORTAL_STUB)
}

/// CreateSession with an explicit honesty note from readiness probe (pure).
pub fn create_screencast_session_with_backend_note(
    req: PortalScreencastRequest,
    backend_note: impl Into<String>,
) -> PortalScreencastSession {
    let n = NEXT_SCREENCAST_SESSION.fetch_add(1, Ordering::Relaxed);
    let source_type = default_source_type(req.types);
    PortalScreencastSession {
        session_id: format!("screencast-{n}"),
        streams: vec![placeholder_stream(SCREENCAST_PLACEHOLDER_NODE_ID, source_type)],
        types: req.types,
        multiple: req.multiple,
        cursor_mode: req.cursor_mode,
        sources_selected: false,
        started: false,
        backend_note: backend_note.into(),
    }
}

/// Apply readiness probe result onto a session's backend note (pure; no I/O).
pub fn apply_screencast_readiness(
    session: &mut PortalScreencastSession,
    ready: &crate::screencast_pw::ScreencastReadiness,
) {
    session.backend_note = screencast_backend_note(ready);
}

/// Pure ScreenCast SelectSources: bind `source_ids` as stream node_id placeholders.
///
/// Empty `source_ids` is an error (cancelled / nothing selected). When `multiple` is
/// false, more than one id is rejected. Does not talk to PipeWire.
pub fn select_screencast_sources(
    session: &mut PortalScreencastSession,
    source_ids: &[u32],
) -> Result<(), String> {
    if session.started {
        return Err("session already started".into());
    }
    if source_ids.is_empty() {
        return Err("no sources selected".into());
    }
    if !session.multiple && source_ids.len() > 1 {
        return Err("multiple sources not allowed".into());
    }
    let source_type = default_source_type(session.types);
    session.streams = source_ids
        .iter()
        .map(|&node_id| placeholder_stream(node_id, source_type))
        .collect();
    session.sources_selected = true;
    Ok(())
}

/// Pure ScreenCast Start: requires at least one stream (protocol-level stubs only).
///
/// On success marks the session started. Streams remain placeholders — no live PipeWire.
pub fn start_screencast(session: &mut PortalScreencastSession) -> Result<(), String> {
    if session.streams.is_empty() {
        return Err("no streams to start".into());
    }
    if session.started {
        return Err("session already started".into());
    }
    // CreateSession already supplies a default stream; SelectSources is optional for the
    // simplified path so Start can succeed after create alone.
    session.started = true;
    Ok(())
}

// ---------------------------------------------------------------------------
// OpenURI
// ---------------------------------------------------------------------------

/// Pure OpenURI validation: only `http`, `https`, and `file` schemes are allowed.
///
/// Does not open a browser or file manager — D-Bus/impl may act after validation.
pub fn handle_open_uri(uri: &str) -> Result<(), String> {
    let uri = uri.trim();
    if uri.is_empty() {
        return Err("empty URI".to_string());
    }
    if uri.contains('\0') {
        return Err("URI contains null byte".to_string());
    }
    let scheme = match uri.split_once(':') {
        Some((s, rest)) if !s.is_empty() && !rest.is_empty() => s.to_ascii_lowercase(),
        Some((s, _)) if !s.is_empty() => {
            // scheme: with empty rest — still treat as scheme present
            s.to_ascii_lowercase()
        }
        _ => return Err("missing URI scheme".to_string()),
    };
    match scheme.as_str() {
        "http" | "https" | "file" => Ok(()),
        other => Err(format!("scheme not allowed: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_screenshot_filename_pure() {
        assert_eq!(
            portal_screenshot_filename(123),
            "RetroShell-Portal-Screenshot-123.png"
        );
        assert_eq!(
            portal_screenshot_filename(0),
            "RetroShell-Portal-Screenshot-0.png"
        );
        assert_eq!(
            portal_screenshot_filename(1_710_000_000),
            "RetroShell-Portal-Screenshot-1710000000.png"
        );
    }

    #[test]
    fn request_defaults_are_non_interactive() {
        let req = PortalScreenshotRequest::default();
        assert!(!req.interactive);
        assert!(!req.include_cursor);
    }

    #[test]
    fn request_fields_round_trip() {
        let req = PortalScreenshotRequest {
            interactive: true,
            include_cursor: true,
        };
        assert!(req.interactive);
        assert!(req.include_cursor);
    }

    #[test]
    fn portal_screenshots_dir_pure() {
        let dir = portal_screenshots_dir(Path::new("/home/user"));
        assert_eq!(dir, PathBuf::from("/home/user/Pictures/Screenshots"));
    }

    #[test]
    fn portal_screenshot_uri_for_absolute() {
        let uri = portal_screenshot_uri_for(Path::new("/tmp/shot.png"));
        assert_eq!(uri, "file:///tmp/shot.png");
    }

    #[test]
    fn handle_portal_screenshot_request_records_options_and_path() {
        let req = PortalScreenshotRequest {
            interactive: true,
            include_cursor: false,
        };
        let dir = Path::new("/tmp/Screenshots");
        let result = handle_portal_screenshot_request(req, dir, 42);
        assert_eq!(
            result.path,
            PathBuf::from("/tmp/Screenshots/RetroShell-Portal-Screenshot-42.png")
        );
        assert_eq!(result.options, req);
        assert_eq!(
            portal_screenshot_uri_for(&result.path),
            "file:///tmp/Screenshots/RetroShell-Portal-Screenshot-42.png"
        );
    }

    #[test]
    fn read_portal_setting_appearance_color_scheme() {
        assert_eq!(
            read_portal_setting(
                PortalSettingsNamespace::Appearance.as_str(),
                "color-scheme"
            ),
            Some("0".to_string())
        );
        assert_eq!(
            read_portal_setting("org.freedesktop.appearance", "accent-color"),
            Some(String::new())
        );
        assert_eq!(
            read_portal_setting("org.freedesktop.appearance", "nope"),
            None
        );
        assert_eq!(read_portal_setting("unknown.ns", "color-scheme"), None);
    }

    #[test]
    fn read_all_portal_settings_appearance() {
        let all = read_all_portal_settings("org.freedesktop.appearance");
        assert_eq!(all.get("color-scheme").map(String::as_str), Some("0"));
        assert!(all.contains_key("accent-color"));
        assert!(read_all_portal_settings("other").is_empty());
    }

    #[test]
    fn portal_settings_namespace_parse() {
        assert_eq!(
            PortalSettingsNamespace::parse("org.freedesktop.appearance"),
            Some(PortalSettingsNamespace::Appearance)
        );
        assert_eq!(PortalSettingsNamespace::parse("nope"), None);
    }

    #[test]
    fn handle_open_uri_allows_http_https_file() {
        assert!(handle_open_uri("https://example.com/path").is_ok());
        assert!(handle_open_uri("http://example.com").is_ok());
        assert!(handle_open_uri("file:///tmp/doc.pdf").is_ok());
        assert!(handle_open_uri("  https://x.test  ").is_ok());
    }

    #[test]
    fn handle_open_uri_rejects_other_schemes() {
        assert!(handle_open_uri("ftp://files.example").is_err());
        assert!(handle_open_uri("javascript:alert(1)").is_err());
        assert!(handle_open_uri("smb://share").is_err());
        assert!(handle_open_uri("").is_err());
        assert!(handle_open_uri("noscheme").is_err());
        assert!(handle_open_uri("http\0://bad").is_err());
    }

    #[test]
    fn bus_constants_match_documented_names() {
        assert_eq!(PORTAL_BUS_NAME, "org.retroshell.Portal");
        assert_eq!(PORTAL_PATH, "/org/retroshell/portal");
        assert_eq!(
            PORTAL_SCREENSHOT_INTERFACE,
            "org.freedesktop.impl.portal.Screenshot"
        );
        assert_eq!(
            PORTAL_SETTINGS_INTERFACE,
            "org.freedesktop.impl.portal.Settings"
        );
        assert_eq!(
            PORTAL_OPENURI_INTERFACE,
            "org.freedesktop.impl.portal.OpenURI"
        );
        assert_eq!(
            PORTAL_FILECHOOSER_INTERFACE,
            "org.freedesktop.impl.portal.FileChooser"
        );
        assert_eq!(
            PORTAL_SCREENCAST_INTERFACE,
            "org.freedesktop.impl.portal.ScreenCast"
        );
    }

    #[test]
    fn create_screencast_session_assigns_id_and_monitor_stream() {
        let req = PortalScreencastRequest {
            types: SCREENCAST_SOURCE_TYPE_MONITOR,
            multiple: false,
            cursor_mode: 1,
        };
        let a = create_screencast_session(req);
        let b = create_screencast_session(req);
        assert!(a.session_id.starts_with("screencast-"));
        assert_ne!(a.session_id, b.session_id);
        assert_eq!(a.streams.len(), 1);
        assert_eq!(a.streams[0].source_type, SCREENCAST_SOURCE_TYPE_MONITOR);
        assert_eq!(a.streams[0].node_id, SCREENCAST_PLACEHOLDER_NODE_ID);
        assert_eq!(a.streams[0].width, SCREENCAST_DEFAULT_WIDTH);
        assert_eq!(a.cursor_mode, 1);
        assert!(!a.sources_selected);
        assert!(!a.started);
    }

    #[test]
    fn select_screencast_sources_updates_streams() {
        let mut session = create_screencast_session(PortalScreencastRequest {
            types: SCREENCAST_SOURCE_TYPE_MONITOR | SCREENCAST_SOURCE_TYPE_WINDOW,
            multiple: true,
            cursor_mode: 0,
        });
        assert!(select_screencast_sources(&mut session, &[]).is_err());
        select_screencast_sources(&mut session, &[7, 9]).unwrap();
        assert!(session.sources_selected);
        assert_eq!(session.streams.len(), 2);
        assert_eq!(session.streams[0].node_id, 7);
        assert_eq!(session.streams[1].node_id, 9);
    }

    #[test]
    fn select_screencast_sources_rejects_multiple_when_not_allowed() {
        let mut session = create_screencast_session(PortalScreencastRequest {
            types: SCREENCAST_SOURCE_TYPE_MONITOR,
            multiple: false,
            cursor_mode: 0,
        });
        assert!(select_screencast_sources(&mut session, &[1, 2]).is_err());
    }

    #[test]
    fn start_screencast_requires_non_empty_streams() {
        let mut session = create_screencast_session(PortalScreencastRequest::default());
        start_screencast(&mut session).unwrap();
        assert!(session.started);
        assert!(!session.streams.is_empty());
        assert!(start_screencast(&mut session).is_err());

        let mut empty = create_screencast_session(PortalScreencastRequest::default());
        empty.streams.clear();
        assert!(start_screencast(&mut empty).is_err());
    }

    #[test]
    fn screencast_backend_note_honesty() {
        assert_eq!(
            screencast_backend_note_from_socket(false),
            SCREENCAST_NOTE_PORTAL_STUB
        );
        assert_eq!(
            screencast_backend_note_from_socket(true),
            SCREENCAST_NOTE_PIPEWIRE_SOCKET
        );
        let stub = create_screencast_session(PortalScreencastRequest::default());
        assert_eq!(stub.backend_note, SCREENCAST_NOTE_PORTAL_STUB);

        let ready = crate::screencast_pw::probe_screencast_readiness(
            Some("/run/user/1000"),
            true,
            false,
        );
        let mut session = create_screencast_session_with_backend_note(
            PortalScreencastRequest::default(),
            screencast_backend_note(&ready),
        );
        assert_eq!(session.backend_note, SCREENCAST_NOTE_PIPEWIRE_SOCKET);
        apply_screencast_readiness(
            &mut session,
            &crate::screencast_pw::probe_screencast_readiness(None, false, false),
        );
        assert_eq!(session.backend_note, SCREENCAST_NOTE_PORTAL_STUB);
        // Start still succeeds with stubs — note does not imply live streams.
        start_screencast(&mut session).unwrap();
        assert!(session.started);
    }

    #[test]
    fn handle_file_chooser_open_single() {
        let req = PortalFileChooserRequest {
            title: "Open".into(),
            multiple: false,
            current_folder: Some("/home/u".into()),
            ..Default::default()
        };
        let r = handle_file_chooser_open(&req, &["a.txt"]).unwrap();
        assert_eq!(r.uris.len(), 1);
        assert!(r.uris[0].contains("a.txt"));
        assert!(handle_file_chooser_open(&req, &["a", "b"]).is_err());
    }

    #[test]
    fn handle_file_chooser_save_confirm() {
        let req = PortalFileChooserRequest {
            title: "Save".into(),
            current_folder: Some("/tmp".into()),
            current_name: Some("out.md".into()),
            ..Default::default()
        };
        let cancelled = handle_file_chooser_save(&req, false).unwrap();
        assert!(cancelled.uris.is_empty());
        let saved = handle_file_chooser_save(&req, true).unwrap();
        assert_eq!(saved.uris.len(), 1);
        assert!(saved.uris[0].ends_with("out.md") || saved.uris[0].contains("out.md"));
    }

    #[test]
    fn validate_file_chooser_rejects_empty_title() {
        let req = PortalFileChooserRequest::default();
        assert!(validate_file_chooser_request(&req).is_err());
    }
}

