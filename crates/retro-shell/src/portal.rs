//! FreeDesktop portal-facing surface (xdg-desktop-portal Screenshot / Settings / OpenURI).
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
//!
//! Local shell menus still use [`take_portal_style_screenshot`] (capture path) via
//! `shell.portal_screenshot`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    }
}
