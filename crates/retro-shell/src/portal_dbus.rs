//! FreeDesktop portal session-bus export (Linux only).
//!
//! Host tests never open D-Bus. On Linux, [`try_register_portal_session_bus`]
//! best-effort claims [`crate::portal::PORTAL_BUS_NAME`] and serves Screenshot,
//! Settings, and OpenURI interfaces that call pure handlers in [`crate::portal`].

use crate::portal::{PORTAL_BUS_NAME, PORTAL_PATH};

/// Best-effort register portal interfaces on the session bus.
///
/// On non-Linux hosts, or when the session bus is missing / name is taken,
/// logs and returns `false` — never fails shell startup.
pub fn try_register_portal_session_bus() -> bool {
    #[cfg(target_os = "linux")]
    {
        use crate::portal::{
            PORTAL_OPENURI_INTERFACE, PORTAL_SCREENSHOT_INTERFACE, PORTAL_SETTINGS_INTERFACE,
        };
        match linux::register() {
            Ok(()) => {
                tracing::info!(
                    bus = PORTAL_BUS_NAME,
                    path = PORTAL_PATH,
                    screenshot = PORTAL_SCREENSHOT_INTERFACE,
                    settings = PORTAL_SETTINGS_INTERFACE,
                    openuri = PORTAL_OPENURI_INTERFACE,
                    "RetroShell portal handlers registered on session bus"
                );
                true
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "RetroShell portal registration skipped"
                );
                false
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tracing::debug!("RetroShell portal registration skipped (non-Linux host)");
        false
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::portal::{
        handle_file_chooser_open, handle_file_chooser_save, handle_open_uri,
        handle_portal_screenshot_request, portal_screenshot_uri_for, portal_screenshots_dir,
        read_all_portal_settings, read_portal_setting, take_portal_style_screenshot_with,
        PortalFileChooserRequest, PortalScreenshotRequest, PORTAL_FILECHOOSER_INTERFACE,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex as StdMutex;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zbus::blocking::connection::Builder as ConnectionBuilder;
    use zbus::blocking::Connection;
    use zbus::interface;
    use zbus::zvariant::{OwnedValue, Value};

    /// Keeps the session bus connection alive for process lifetime.
    static REGISTRATION: StdMutex<Option<PortalRegistration>> = StdMutex::new(None);

    struct PortalRegistration {
        _connection: Connection,
    }

    struct PortalScreenshotIface;

    #[interface(name = "org.freedesktop.impl.portal.Screenshot")]
    impl PortalScreenshotIface {
        /// Portal Screenshot: pure plan + best-effort local capture.
        ///
        /// Returns `(response, results)` where response `0` = success, `2` = error/cancel.
        /// `results` includes `uri` (`file://...`) on success.
        fn screenshot(
            &self,
            _handle: zbus::zvariant::ObjectPath<'_>,
            _app_id: &str,
            _parent_window: &str,
            options: HashMap<String, OwnedValue>,
        ) -> (u32, HashMap<String, OwnedValue>) {
            let interactive = option_bool(&options, "interactive").unwrap_or(false);
            let include_cursor = option_bool(&options, "cursor").unwrap_or(false)
                || option_bool(&options, "include-cursor").unwrap_or(false);
            let request = PortalScreenshotRequest {
                interactive,
                include_cursor,
            };

            // Prefer real capture when tools are available; fall back to pure planned path.
            let result = match take_portal_style_screenshot_with(request) {
                Ok(r) => r,
                Err(_) => {
                    let base = std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("/tmp"));
                    let dir = portal_screenshots_dir(&base);
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    handle_portal_screenshot_request(request, &dir, now)
                }
            };

            let uri = portal_screenshot_uri_for(&result.path);
            let mut results: HashMap<String, OwnedValue> = HashMap::new();
            if let Ok(v) = OwnedValue::try_from(Value::from(uri)) {
                results.insert("uri".to_string(), v);
            }
            (0u32, results)
        }
    }

    struct PortalSettingsIface;

    #[interface(name = "org.freedesktop.impl.portal.Settings")]
    impl PortalSettingsIface {
        /// Settings.Read — pure map lookup; value as string variant.
        fn read(&self, namespace: &str, key: &str) -> zbus::fdo::Result<OwnedValue> {
            match read_portal_setting(namespace, key) {
                Some(v) => OwnedValue::try_from(Value::from(v)).map_err(|e| {
                    zbus::fdo::Error::Failed(format!("value conversion failed: {e}"))
                }),
                None => Err(zbus::fdo::Error::Failed(format!(
                    "setting not found: {namespace} / {key}"
                ))),
            }
        }

        /// Settings.ReadAll — pure map for the namespace.
        fn read_all(&self, namespace: &str) -> HashMap<String, OwnedValue> {
            let mut out = HashMap::new();
            for (k, v) in read_all_portal_settings(namespace) {
                if let Ok(owned) = OwnedValue::try_from(Value::from(v)) {
                    out.insert(k, owned);
                }
            }
            out
        }
    }

    struct PortalFileChooserIface;

    #[interface(name = "org.freedesktop.impl.portal.FileChooser")]
    impl PortalFileChooserIface {
        /// OpenFile — pure path selection under options["current_folder"].
        /// options["uris"] string array of basenames selects files (agent/test).
        fn open_file(
            &self,
            _handle: zbus::zvariant::ObjectPath<'_>,
            _app_id: &str,
            _parent_window: &str,
            title: &str,
            options: HashMap<String, OwnedValue>,
        ) -> (u32, HashMap<String, OwnedValue>) {
            let multiple = option_bool(&options, "multiple").unwrap_or(false);
            let directory = option_bool(&options, "directory").unwrap_or(false);
            let current_folder = option_string(&options, "current_folder");
            let req = PortalFileChooserRequest {
                title: title.into(),
                multiple,
                directory,
                current_folder,
                ..Default::default()
            };
            let names = option_string_array(&options, "selected");
            let names_ref: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            match handle_file_chooser_open(&req, &names_ref) {
                Ok(r) if r.uris.is_empty() => (1u32, HashMap::new()), // cancelled
                Ok(r) => {
                    let mut results = HashMap::new();
                    if let Ok(v) = OwnedValue::try_from(Value::from(r.uris)) {
                        results.insert("uris".into(), v);
                    }
                    (0u32, results)
                }
                Err(_) => (2u32, HashMap::new()),
            }
        }

        fn save_file(
            &self,
            _handle: zbus::zvariant::ObjectPath<'_>,
            _app_id: &str,
            _parent_window: &str,
            title: &str,
            options: HashMap<String, OwnedValue>,
        ) -> (u32, HashMap<String, OwnedValue>) {
            let current_folder = option_string(&options, "current_folder");
            let current_name = option_string(&options, "current_name");
            let confirm = option_bool(&options, "confirm").unwrap_or(true);
            let req = PortalFileChooserRequest {
                title: title.into(),
                current_folder,
                current_name,
                ..Default::default()
            };
            match handle_file_chooser_save(&req, confirm) {
                Ok(r) if r.uris.is_empty() => (1u32, HashMap::new()),
                Ok(r) => {
                    let mut results = HashMap::new();
                    if let Ok(v) = OwnedValue::try_from(Value::from(r.uris)) {
                        results.insert("uris".into(), v);
                    }
                    (0u32, results)
                }
                Err(_) => (2u32, HashMap::new()),
            }
        }
    }

    struct PortalOpenUriIface;

    #[interface(name = "org.freedesktop.impl.portal.OpenURI")]
    impl PortalOpenUriIface {
        /// OpenURI validation (pure); does not spawn a handler process.
        ///
        /// Returns response `0` on allowed schemes, `2` on rejection.
        fn open_uri(
            &self,
            _handle: zbus::zvariant::ObjectPath<'_>,
            _app_id: &str,
            _parent_window: &str,
            uri: &str,
            _options: HashMap<String, OwnedValue>,
        ) -> u32 {
            match handle_open_uri(uri) {
                Ok(()) => 0,
                Err(_) => 2,
            }
        }
    }

    fn option_bool(options: &HashMap<String, OwnedValue>, key: &str) -> Option<bool> {
        let value = options.get(key)?;
        if let Ok(b) = bool::try_from(value) {
            return Some(b);
        }
        if let Ok(v) = u32::try_from(value) {
            return Some(v != 0);
        }
        if let Ok(v) = i32::try_from(value) {
            return Some(v != 0);
        }
        None
    }

    fn option_string(options: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
        let value = options.get(key)?;
        String::try_from(value).ok()
    }

    fn option_string_array(options: &HashMap<String, OwnedValue>, key: &str) -> Vec<String> {
        let Some(value) = options.get(key) else {
            return Vec::new();
        };
        Vec::<String>::try_from(value).unwrap_or_default()
    }

    pub(super) fn register() -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(guard) = REGISTRATION.lock() {
            if guard.is_some() {
                return Ok(());
            }
        }

        let conn = ConnectionBuilder::session()?
            .name(PORTAL_BUS_NAME)?
            .serve_at(PORTAL_PATH, PortalScreenshotIface)?
            .serve_at(PORTAL_PATH, PortalSettingsIface)?
            .serve_at(PORTAL_PATH, PortalOpenUriIface)?
            .serve_at(PORTAL_PATH, PortalFileChooserIface)?
            .build()?;

        if let Ok(mut guard) = REGISTRATION.lock() {
            *guard = Some(PortalRegistration {
                _connection: conn,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_register_returns_bool_on_host() {
        // On macOS / non-Linux this must be false and never panic.
        let registered = try_register_portal_session_bus();
        #[cfg(not(target_os = "linux"))]
        {
            assert!(!registered);
        }
        let _ = registered;
    }

    #[test]
    fn constants_align_with_portal_module() {
        assert_eq!(PORTAL_BUS_NAME, crate::portal::PORTAL_BUS_NAME);
        assert_eq!(PORTAL_PATH, crate::portal::PORTAL_PATH);
    }
}
