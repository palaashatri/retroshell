//! FreeDesktop portal-facing screenshot surface (xdg-desktop-portal Screenshot).
//!
//! Pure request/result types mirror the future D-Bus portal API. Until an
//! xdg-desktop-portal bus backend is wired, [`take_portal_style_screenshot`]
//! is the local equivalent used by shell menus: it delegates to
//! [`crate::capture::take_screenshot`] (ImageMagick/ffmpeg/xwd path).

use std::path::PathBuf;

use crate::capture::{take_screenshot, CaptureError};

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

/// Successful screenshot result (file URI path side of the portal response).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalScreenshotResult {
    pub path: PathBuf,
}

/// Build the default portal-style screenshot filename (pure helper for tests).
///
/// Example: `RetroShell-Portal-Screenshot-1710000000.png`
pub fn portal_screenshot_filename(now_unix_secs: u64) -> String {
    format!("RetroShell-Portal-Screenshot-{now_unix_secs}.png")
}

/// Take a screenshot through the portal-facing API surface.
///
/// **Note:** Until an xdg-desktop-portal D-Bus backend is available, this is
/// the local equivalent used by shell menus. It delegates to
/// [`crate::capture::take_screenshot`] and ignores interactive/cursor options
/// (those apply only once a real portal implementation lands).
pub fn take_portal_style_screenshot() -> Result<PathBuf, CaptureError> {
    take_screenshot()
}

/// Portal-style capture with explicit request options.
///
/// Options are accepted for API symmetry with the FreeDesktop portal; they are
/// not yet honored by the local capture backend.
pub fn take_portal_style_screenshot_with(
    request: PortalScreenshotRequest,
) -> Result<PortalScreenshotResult, CaptureError> {
    let _ = request;
    let path = take_portal_style_screenshot()?;
    Ok(PortalScreenshotResult { path })
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
}
