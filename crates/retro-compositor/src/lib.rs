//! Shared compositor policy that can be tested without a live Wayland server.

pub mod frame_timing;
pub mod hdr;

/// DRM/KMS + libseat session path (Linux only). Nested X11 lives in the binary.
#[cfg(target_os = "linux")]
pub mod session_drm;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use frame_timing::RefreshRate;
use hdr::ColorSpace;

/// How the session compositor process is expected to present.
///
/// Pure policy label for Phase A/B honesty: logs and entrypoints must say which
/// path was chosen (nested X11 under Xvfb, real DRM/KMS session, or external
/// labwc fallback) rather than implying DRM when only nested X11 is running.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CompositorBackendKind {
    /// Nested Smithay X11 backend (Xvfb / desktop X host). Needs DRI3 for GL.
    NestedX11,
    /// Session DRM/KMS (bare metal / seat) path when prefer_drm && dri3_ok.
    SessionDrm,
    /// External labwc process — not retro-compositor itself.
    LabwcFallback,
}

/// Select compositor session backend kind from capability flags.
///
/// Precedence:
/// 1. `force_labwc` → [`CompositorBackendKind::LabwcFallback`]
/// 2. `prefer_drm && dri3_available` → [`CompositorBackendKind::SessionDrm`]
/// 3. otherwise → [`CompositorBackendKind::NestedX11`]
///
/// Nested X11 remains the default when DRM is not preferred or DRI3 is missing;
/// actual GL init may still fail without DRI3 (entrypoint then falls back to labwc).
pub fn select_backend_kind(
    prefer_drm: bool,
    dri3_available: bool,
    force_labwc: bool,
) -> CompositorBackendKind {
    if force_labwc {
        return CompositorBackendKind::LabwcFallback;
    }
    if prefer_drm && dri3_available {
        return CompositorBackendKind::SessionDrm;
    }
    CompositorBackendKind::NestedX11
}

/// Detect DRI3 availability override from `RETROSHELL_DRI3`.
///
/// - `1` / truthy → `Some(true)`
/// - `0` / falsey → `Some(false)`
/// - unset / unrecognised → `None` (caller should probe the real display)
///
/// Intended for tests and CI; production can fall back to X11 extension probe.
pub fn detect_dri3_from_env() -> Option<bool> {
    detect_dri3_from_env_value(std::env::var("RETROSHELL_DRI3").ok().as_deref())
}

/// Pure form of [`detect_dri3_from_env`] for unit tests.
pub fn detect_dri3_from_env_value(value: Option<&str>) -> Option<bool> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    parse_bool_loose(value)
}

/// One-line honest label for logs (never claims DRM when nested / labwc).
pub fn session_mode_summary(kind: CompositorBackendKind) -> String {
    match kind {
        CompositorBackendKind::NestedX11 => {
            "session_mode=nested_x11 (not DRM/KMS; labwc may still be used if GL/DRI3 fails)"
                .to_string()
        }
        CompositorBackendKind::SessionDrm => {
            "session_mode=session_drm (DRM/KMS seat path)".to_string()
        }
        CompositorBackendKind::LabwcFallback => {
            "session_mode=labwc_fallback (external labwc; not retro-compositor)".to_string()
        }
    }
}

pub const DEFAULT_OUTPUT_W: i32 = 1024;
pub const DEFAULT_OUTPUT_H: i32 = 768;
pub const DEFAULT_WINDOW_W: i32 = 640;
pub const DEFAULT_WINDOW_H: i32 = 480;
pub const INITIAL_WINDOW_X: i32 = 64;
pub const INITIAL_WINDOW_Y: i32 = 64;
pub const CASCADE_STEP: i32 = 32;
pub const CASCADE_WRAP: i32 = 256;

/// A discovered DRM render/primary node path (session DRM path).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DrmNodePath {
    pub path: PathBuf,
    /// True if filename looks like `cardN` (modesetting primary).
    pub is_primary: bool,
}

/// Discover DRM device nodes under `/dev/dri`.
///
/// Pure filesystem scan — works without opening DRM (host-safe unit tests can
/// pass synthetic directory listings via [`discover_drm_nodes_from_names`]).
pub fn discover_drm_nodes() -> Vec<DrmNodePath> {
    let dir = Path::new("/dev/dri");
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for entry in rd.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    discover_drm_nodes_from_names(dir, &names)
}

/// Pure form of DRM node discovery from a directory path + file names.
pub fn discover_drm_nodes_from_names(dir: &Path, names: &[String]) -> Vec<DrmNodePath> {
    let mut out = Vec::new();
    for name in names {
        if name.starts_with("card") || name.starts_with("renderD") {
            out.push(DrmNodePath {
                path: dir.join(name),
                is_primary: name.starts_with("card"),
            });
        }
    }
    // Prefer primary cards first for session bootstrap.
    out.sort_by_key(|n| (!n.is_primary, n.path.clone()));
    out
}

/// Pick the preferred DRM primary node for session bootstrap.
pub fn preferred_primary_drm_node(nodes: &[DrmNodePath]) -> Option<&DrmNodePath> {
    nodes.iter().find(|n| n.is_primary).or_else(|| nodes.first())
}

/// Layer-shell role labels used by shell chrome (bar/dock/notifications).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ChromeLayer {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl ChromeLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Bottom => "bottom",
            Self::Top => "top",
            Self::Overlay => "overlay",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "background" => Some(Self::Background),
            "bottom" => Some(Self::Bottom),
            "top" => Some(Self::Top),
            "overlay" => Some(Self::Overlay),
            _ => None,
        }
    }

    /// z-order key for sorting chrome layers (higher draws above).
    pub fn z_priority(self) -> u8 {
        match self {
            Self::Background => 0,
            Self::Bottom => 1,
            Self::Top => 2,
            Self::Overlay => 3,
        }
    }
}

/// Policy for a layer-shell chrome surface (menu bar, dock, etc.).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayerChromeSpec {
    pub name: String,
    pub layer: ChromeLayer,
    pub exclusive_zone: i32,
    pub anchor_top: bool,
    pub anchor_bottom: bool,
    pub anchor_left: bool,
    pub anchor_right: bool,
}

impl LayerChromeSpec {
    pub fn menu_bar(height: i32) -> Self {
        Self {
            name: "menu-bar".into(),
            layer: ChromeLayer::Top,
            exclusive_zone: height,
            anchor_top: true,
            anchor_bottom: false,
            anchor_left: true,
            anchor_right: true,
        }
    }

    pub fn dock(height: i32) -> Self {
        Self {
            name: "dock".into(),
            layer: ChromeLayer::Top,
            exclusive_zone: height,
            anchor_top: false,
            anchor_bottom: true,
            anchor_left: true,
            anchor_right: true,
        }
    }

    pub fn notification_overlay() -> Self {
        Self {
            name: "notifications".into(),
            layer: ChromeLayer::Overlay,
            exclusive_zone: 0,
            anchor_top: true,
            anchor_bottom: false,
            anchor_left: false,
            anchor_right: true,
        }
    }
}

/// Sort chrome specs by layer priority then name (stable layout order).
pub fn sort_chrome_layers(specs: &mut [LayerChromeSpec]) {
    specs.sort_by(|a, b| {
        a.layer
            .z_priority()
            .cmp(&b.layer.z_priority())
            .then_with(|| a.name.cmp(&b.name))
    });
}

/// Indices for composing surfaces: background/bottom first, then windows, then top/overlay.
///
/// Used by nested `render_frame` so layer-shell chrome is not skipped when buffers commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComposeOrder {
    /// Indices into the caller's layer surface list, low-to-high paint order.
    pub layer_indices_bottom_first: Vec<usize>,
    /// Whether windows should paint after bottom layers and before top/overlay.
    pub windows_after_bottom: bool,
}

/// Pure planner: given layer z priorities (higher = above), return paint order indices.
///
/// Layers with `z <= 1` (Background/Bottom) paint under windows; `z >= 2` (Top/Overlay)
/// paint above windows.
pub fn plan_compose_order(layer_z: &[u8]) -> ComposeOrder {
    let mut under: Vec<(u8, usize)> = Vec::new();
    let mut over: Vec<(u8, usize)> = Vec::new();
    for (i, &z) in layer_z.iter().enumerate() {
        if z <= 1 {
            under.push((z, i));
        } else {
            over.push((z, i));
        }
    }
    under.sort_by_key(|(z, i)| (*z, *i));
    over.sort_by_key(|(z, i)| (*z, *i));
    let mut layer_indices_bottom_first: Vec<usize> =
        under.into_iter().map(|(_, i)| i).collect();
    layer_indices_bottom_first.extend(over.into_iter().map(|(_, i)| i));
    ComposeOrder {
        layer_indices_bottom_first,
        windows_after_bottom: true,
    }
}

/// Map a layer name string to z priority (for tests / policy without smithay types).
pub fn layer_name_z_priority(name: &str) -> Option<u8> {
    ChromeLayer::from_str_loose(name).map(|l| l.z_priority())
}

// ---------------------------------------------------------------------------
// DRM presentation plan (pure) — scanout path stages for SessionDrm
// ---------------------------------------------------------------------------

/// Stages of a real DRM presentation pipeline (beyond open-device only).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DrmPresentationStage {
    OpenSeat,
    OpenPrimaryNode,
    CreateGbmEgl,
    EnumerateConnectors,
    PickConnectorMode,
    CreateDrmSurface,
    PageFlipOrPresent,
    ProtocolLoop,
}

impl DrmPresentationStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenSeat => "open_seat",
            Self::OpenPrimaryNode => "open_primary_node",
            Self::CreateGbmEgl => "create_gbm_egl",
            Self::EnumerateConnectors => "enumerate_connectors",
            Self::PickConnectorMode => "pick_connector_mode",
            Self::CreateDrmSurface => "create_drm_surface",
            Self::PageFlipOrPresent => "pageflip_or_present",
            Self::ProtocolLoop => "protocol_loop",
        }
    }
}

/// Ordered presentation pipeline for SessionDrm bootstrap.
pub fn drm_presentation_pipeline() -> &'static [DrmPresentationStage] {
    use DrmPresentationStage::*;
    &[
        OpenSeat,
        OpenPrimaryNode,
        CreateGbmEgl,
        EnumerateConnectors,
        PickConnectorMode,
        CreateDrmSurface,
        PageFlipOrPresent,
        ProtocolLoop,
    ]
}

/// Result of attempting connector-based modeset (pure bookkeeping for tests/logs).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrmModesetPlan {
    pub connector_name: String,
    pub mode_w: i32,
    pub mode_h: i32,
    pub refresh_mhz: i32,
    pub crtc_index: usize,
}

/// Pick a modeset plan from discovered connector summaries.
///
/// Prefers the first connected connector with a preferred mode; falls back to env-sized
/// virtual mode when none are connected (nested/test).
pub fn plan_drm_modeset(
    connectors: &[(String, bool, Option<(i32, i32, i32)>)],
    fallback_w: i32,
    fallback_h: i32,
    fallback_refresh_mhz: i32,
) -> DrmModesetPlan {
    for (i, (name, connected, mode)) in connectors.iter().enumerate() {
        if *connected {
            if let Some((w, h, refresh)) = mode {
                return DrmModesetPlan {
                    connector_name: name.clone(),
                    mode_w: *w,
                    mode_h: *h,
                    refresh_mhz: *refresh,
                    crtc_index: i,
                };
            }
        }
    }
    DrmModesetPlan {
        connector_name: "virtual-fallback".into(),
        mode_w: fallback_w,
        mode_h: fallback_h,
        refresh_mhz: fallback_refresh_mhz,
        crtc_index: 0,
    }
}

// ---------------------------------------------------------------------------
// Server-side decoration policy (xdg-decoration)
// ---------------------------------------------------------------------------

/// Preferred window decoration mode for first-party vs external clients.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecorationPreference {
    /// Compositor draws decorations (CSD alternative).
    ServerSide,
    /// Client draws its own decorations.
    ClientSide,
}

/// Decide decoration preference from app_id hints (pure).
pub fn decoration_preference_for_app_id(app_id: &str) -> DecorationPreference {
    let id = app_id.to_ascii_lowercase();
    // First-party suite draws own chrome via kit; external apps get SSD when possible.
    if id.starts_with("retroshell.")
        || id == "finder"
        || id == "textedit"
        || id == "terminal"
        || id == "settings"
        || id == "appstore"
        || id == "retro-shell"
    {
        DecorationPreference::ClientSide
    } else {
        DecorationPreference::ServerSide
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputConfig {
    pub width: i32,
    pub height: i32,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_OUTPUT_W,
            height: DEFAULT_OUTPUT_H,
        }
    }
}

impl OutputConfig {
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var("RETROSHELL_COMPOSITOR_WIDTH").ok(),
            std::env::var("RETROSHELL_COMPOSITOR_HEIGHT").ok(),
        )
    }

    pub fn from_env_values(width: Option<String>, height: Option<String>) -> Self {
        Self {
            width: parse_positive_i32(width).unwrap_or(DEFAULT_OUTPUT_W),
            height: parse_positive_i32(height).unwrap_or(DEFAULT_OUTPUT_H),
        }
    }
}

/// One logical output with a compositor-space origin (side-by-side layout).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaidOutOutput {
    pub config: OutputConfig,
    pub x: i32,
    pub y: i32,
}

/// Parse `RETROSHELL_OUTPUTS=WxH,WxH` (comma-separated). Invalid tokens are skipped.
///
/// Returns an empty vec when the string has no valid entries.
pub fn parse_outputs_spec(spec: &str) -> Vec<OutputConfig> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((w_str, h_str)) = part
            .split_once('x')
            .or_else(|| part.split_once('X'))
        else {
            continue;
        };
        let Ok(w) = w_str.trim().parse::<i32>() else {
            continue;
        };
        let Ok(h) = h_str.trim().parse::<i32>() else {
            continue;
        };
        if w > 0 && h > 0 {
            out.push(OutputConfig {
                width: w,
                height: h,
            });
        }
    }
    out
}

/// Lay out outputs left-to-right starting at (0,0). Y is always 0 for the simple
/// side-by-side policy used under the nested X11 backend.
pub fn layout_outputs_side_by_side(outputs: &[OutputConfig]) -> Vec<LaidOutOutput> {
    let mut x = 0;
    let mut result = Vec::with_capacity(outputs.len());
    for config in outputs {
        result.push(LaidOutOutput {
            config: *config,
            x,
            y: 0,
        });
        x = x.saturating_add(config.width);
    }
    result
}

/// Total canvas size covering all laid-out outputs (union bounding box).
pub fn total_output_size(laid_out: &[LaidOutOutput]) -> OutputConfig {
    if laid_out.is_empty() {
        return OutputConfig::default();
    }
    let mut max_right = 0;
    let mut max_bottom = 0;
    for o in laid_out {
        max_right = max_right.max(o.x + o.config.width);
        max_bottom = max_bottom.max(o.y + o.config.height);
    }
    OutputConfig {
        width: max_right.max(1),
        height: max_bottom.max(1),
    }
}

/// Resolve output list from the environment.
///
/// - If `RETROSHELL_OUTPUTS` parses to one or more sizes, use those.
/// - Otherwise fall back to a single `OutputConfig::from_env()` (WIDTH/HEIGHT).
pub fn outputs_from_env() -> Vec<OutputConfig> {
    outputs_from_env_values(
        std::env::var("RETROSHELL_OUTPUTS").ok(),
        std::env::var("RETROSHELL_COMPOSITOR_WIDTH").ok(),
        std::env::var("RETROSHELL_COMPOSITOR_HEIGHT").ok(),
    )
}

pub fn outputs_from_env_values(
    outputs_spec: Option<String>,
    width: Option<String>,
    height: Option<String>,
) -> Vec<OutputConfig> {
    if let Some(spec) = outputs_spec {
        let parsed = parse_outputs_spec(&spec);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    vec![OutputConfig::from_env_values(width, height)]
}

/// Compositor display policy (HDR / VRR / refresh / color space).
///
/// Resolved from optional `settings.conf` keys then overridden by environment
/// variables. Nested X11/Xvfb has no real HDR path; `hdr_supported` stays false
/// unless hardware detection (elsewhere) proves otherwise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayPolicy {
    pub hdr_requested: bool,
    pub vrr_adaptive: bool,
    pub refresh_rate: RefreshRate,
    pub color_space: ColorSpace,
}

impl Default for DisplayPolicy {
    fn default() -> Self {
        Self {
            hdr_requested: false,
            vrr_adaptive: false,
            refresh_rate: RefreshRate::Hz60,
            color_space: ColorSpace::SRgb,
        }
    }
}

impl DisplayPolicy {
    /// Full resolution order: defaults → settings file → environment (env wins).
    pub fn resolve() -> Self {
        let mut policy = Self::default();
        if let Some(path) = settings_conf_path() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                policy.apply_settings_text(&text);
            }
        }
        policy.apply_env_map(std::env::vars().collect());
        policy
    }

    /// Apply flat `key=value` lines from settings.conf (or tests).
    ///
    /// Recognised keys: `hdr_requested` / `hdr_request`, `vrr_adaptive`,
    /// `refresh_rate`, `color_space`.
    pub fn apply_settings_text(&mut self, text: &str) {
        for (key, value) in parse_key_value_conf(text) {
            match key.as_str() {
                "hdr_requested" | "hdr_request" => {
                    if let Some(b) = parse_bool_loose(&value) {
                        self.hdr_requested = b;
                    }
                }
                "vrr_adaptive" | "vrr_enabled" => {
                    if let Some(b) = parse_bool_loose(&value) {
                        self.vrr_adaptive = b;
                    }
                }
                "refresh_rate" => {
                    if let Some(r) = RefreshRate::parse_flexible(&value) {
                        self.refresh_rate = r;
                    }
                }
                "color_space" => {
                    if let Some(cs) = ColorSpace::from_str_flexible(&value) {
                        self.color_space = cs;
                    }
                }
                _ => {}
            }
        }
    }

    /// Apply environment overrides.
    ///
    /// - `RETROSHELL_HDR` — truthy enables HDR request
    /// - `RETROSHELL_VRR` — truthy enables adaptive VRR
    /// - `RETROSHELL_REFRESH` — e.g. `60`, `60hz`, `adaptive`
    /// - `RETROSHELL_COLOR_SPACE` — `srgb` / `rec2020` / `scrgb`
    pub fn apply_env_map(&mut self, env: HashMap<String, String>) {
        if let Some(v) = env.get("RETROSHELL_HDR") {
            if let Some(b) = parse_bool_loose(v) {
                self.hdr_requested = b;
            }
        }
        if let Some(v) = env.get("RETROSHELL_VRR") {
            if let Some(b) = parse_bool_loose(v) {
                self.vrr_adaptive = b;
            }
        }
        if let Some(v) = env.get("RETROSHELL_REFRESH") {
            if let Some(r) = RefreshRate::parse_flexible(v) {
                self.refresh_rate = r;
            }
        }
        if let Some(v) = env.get("RETROSHELL_COLOR_SPACE") {
            if let Some(cs) = ColorSpace::from_str_flexible(v) {
                self.color_space = cs;
            }
        }
    }

    /// Effective refresh rate after VRR policy (Adaptive when vrr_adaptive).
    pub fn effective_refresh_rate(&self) -> RefreshRate {
        if self.vrr_adaptive {
            RefreshRate::Adaptive
        } else {
            self.refresh_rate
        }
    }

    /// Human-readable one-line summary for logging.
    pub fn summary_line(&self, hdr_supported: bool) -> String {
        format!(
            "hdr_requested={} hdr_supported={} vrr_adaptive={} refresh={} color_space={}",
            self.hdr_requested,
            hdr_supported,
            self.vrr_adaptive,
            self.effective_refresh_rate().as_str(),
            self.color_space.as_str(),
        )
    }
}

/// Look up mime payload bytes in a selection store. Returns `None` when missing
/// (callers should close the fd for EOF without hanging the client).
pub fn selection_bytes_for_mime<'a>(
    store: &'a HashMap<String, Vec<u8>>,
    mime_type: &str,
) -> Option<&'a [u8]> {
    store.get(mime_type).map(|v| v.as_slice())
}

/// Prefer exact mime match; fall back to `text/plain` / `TEXT` / `STRING` for text clients.
pub fn selection_bytes_for_mime_with_text_fallback<'a>(
    store: &'a HashMap<String, Vec<u8>>,
    mime_type: &str,
) -> Option<&'a [u8]> {
    if let Some(b) = selection_bytes_for_mime(store, mime_type) {
        return Some(b);
    }
    const TEXT_FALLBACKS: &[&str] = &[
        "text/plain;charset=utf-8",
        "text/plain",
        "UTF8_STRING",
        "STRING",
        "TEXT",
    ];
    if mime_type.starts_with("text/")
        || mime_type.eq_ignore_ascii_case("STRING")
        || mime_type.eq_ignore_ascii_case("TEXT")
        || mime_type.eq_ignore_ascii_case("UTF8_STRING")
    {
        for candidate in TEXT_FALLBACKS {
            if let Some(b) = selection_bytes_for_mime(store, candidate) {
                return Some(b);
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WindowGeometry {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_f64(self, x: f64, y: f64) -> bool {
        let x = x as i32;
        let y = y as i32;
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub fn cascade_position(offset: i32) -> (i32, i32) {
    (INITIAL_WINDOW_X + offset, INITIAL_WINDOW_Y + offset)
}

pub fn next_cascade_offset(offset: i32) -> i32 {
    (offset + CASCADE_STEP) % CASCADE_WRAP
}

pub fn topmost_window_at(windows: &[WindowGeometry], x: f64, y: f64) -> Option<usize> {
    windows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, window)| window.contains_f64(x, y))
        .map(|(idx, _)| idx)
}

pub fn move_to_top<T>(windows: &mut Vec<T>, idx: usize) {
    let window = windows.remove(idx);
    windows.push(window);
}

/// Identifier for a compositor-managed client surface (independent process).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClientSurfaceId(pub u64);

/// One mapped client window in compositor space (multi-client session model).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappedClientWindow {
    pub id: ClientSurfaceId,
    pub title: String,
    pub geometry: WindowGeometry,
    /// Process id of the Wayland/X11 client when known (0 = unknown).
    pub pid: u32,
}

/// Focus and z-order stack for independent client windows.
///
/// Back is bottom; front is topmost / focused. Pure policy — used by the
/// Linux compositor runtime and host unit tests.
#[derive(Clone, Debug, Default)]
pub struct ClientWindowStack {
    windows: Vec<MappedClientWindow>,
    next_id: u64,
    cascade_offset: i32,
}

impl ClientWindowStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn windows(&self) -> &[MappedClientWindow] {
        &self.windows
    }

    /// Map a new client surface; returns its id. Cascades position like classic DE.
    pub fn map_window(&mut self, title: impl Into<String>, pid: u32) -> ClientSurfaceId {
        let (x, y) = cascade_position(self.cascade_offset);
        self.cascade_offset = next_cascade_offset(self.cascade_offset);
        self.map_window_at(
            title,
            pid,
            WindowGeometry::new(x, y, DEFAULT_WINDOW_W, DEFAULT_WINDOW_H),
        )
    }

    /// Map a client at an explicit geometry (tests / multi-output placement).
    pub fn map_window_at(
        &mut self,
        title: impl Into<String>,
        pid: u32,
        geometry: WindowGeometry,
    ) -> ClientSurfaceId {
        let id = ClientSurfaceId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.windows.push(MappedClientWindow {
            id: id.clone(),
            title: title.into(),
            geometry,
            pid,
        });
        id
    }

    /// Remove a mapped window; returns true if found.
    pub fn unmap(&mut self, id: &ClientSurfaceId) -> bool {
        if let Some(idx) = self.windows.iter().position(|w| &w.id == id) {
            self.windows.remove(idx);
            true
        } else {
            false
        }
    }

    /// Focus / raise by id (moves to top of z-order).
    pub fn focus(&mut self, id: &ClientSurfaceId) -> bool {
        if let Some(idx) = self.windows.iter().position(|w| &w.id == id) {
            move_to_top(&mut self.windows, idx);
            true
        } else {
            false
        }
    }

    /// Focus topmost window containing the point (click-to-raise).
    pub fn focus_at(&mut self, x: f64, y: f64) -> Option<ClientSurfaceId> {
        let geos: Vec<WindowGeometry> = self.windows.iter().map(|w| w.geometry).collect();
        let idx = topmost_window_at(&geos, x, y)?;
        let id = self.windows[idx].id.clone();
        move_to_top(&mut self.windows, idx);
        Some(id)
    }

    /// Currently focused window (top of stack), if any.
    pub fn focused(&self) -> Option<&MappedClientWindow> {
        self.windows.last()
    }

    /// Z-order from bottom to top (ids only).
    pub fn z_order_ids(&self) -> Vec<ClientSurfaceId> {
        self.windows.iter().map(|w| w.id.clone()).collect()
    }
}

fn parse_positive_i32(value: Option<String>) -> Option<i32> {
    value?.parse::<i32>().ok().filter(|value| *value > 0)
}

fn parse_bool_loose(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Parse flat `key=value` lines; `#` comments and blank lines ignored.
pub fn parse_key_value_conf(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim();
        let v = v.trim();
        if !k.is_empty() {
            out.push((k.to_string(), v.to_string()));
        }
    }
    out
}

fn settings_conf_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("RETROSHELL_CONFIG_DIR") {
        return Some(Path::new(&dir).join("settings.conf"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(
            Path::new(&home)
                .join(".config")
                .join("retroshell")
                .join("settings.conf"),
        );
    }
    None
}


// ---------------------------------------------------------------------------
// Text-input / IME policy (pure) — Phase D8 scaffold
// ---------------------------------------------------------------------------

/// Compositor preference for text-input-v3 / input-method availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextInputCapability {
    /// No IME; clients use raw key events only.
    None,
    /// text-input-v3 global advertised; no input-method seat yet.
    TextInputV3,
    /// Full input-method-v2 + text-input-v3 (not yet implemented end-to-end).
    InputMethodAndTextInput,
}

/// Pure policy: which text-input features the session claims.
pub fn text_input_capability_from_env(value: Option<&str>) -> TextInputCapability {
    match value.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("im" | "input-method" | "full") => TextInputCapability::InputMethodAndTextInput,
        Some("text-input" | "v3" | "1" | "true" | "on") => TextInputCapability::TextInputV3,
        _ => TextInputCapability::None,
    }
}

pub fn text_input_capability_summary(cap: TextInputCapability) -> &'static str {
    match cap {
        TextInputCapability::None => "text_input=none",
        TextInputCapability::TextInputV3 => "text_input=text-input-v3",
        TextInputCapability::InputMethodAndTextInput => "text_input=im+text-input-v3",
    }
}

/// One client surface placement for DRM scanout composition planning (pure).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanoutElement {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// Higher paints above.
    pub z: i32,
}

/// Pure: sort scanout elements back-to-front for a DRM present pass.
pub fn plan_scanout_paint_order(elements: &mut [ScanoutElement]) {
    elements.sort_by(|a, b| a.z.cmp(&b.z).then_with(|| a.id.cmp(&b.id)));
}

/// Pure: clip element rect to output bounds; returns None if fully outside.
pub fn clip_scanout_element_to_output(
    el: &ScanoutElement,
    out_w: i32,
    out_h: i32,
) -> Option<(i32, i32, i32, i32)> {
    let x0 = el.x.max(0);
    let y0 = el.y.max(0);
    let x1 = (el.x + el.w).min(out_w);
    let y1 = (el.y + el.h).min(out_h);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some((x0, y0, x1 - x0, y1 - y0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_outputs_spec_single_and_multi() {
        assert_eq!(
            parse_outputs_spec("1280x800"),
            vec![OutputConfig {
                width: 1280,
                height: 800
            }]
        );
        assert_eq!(
            parse_outputs_spec("1024x768,800x600"),
            vec![
                OutputConfig {
                    width: 1024,
                    height: 768
                },
                OutputConfig {
                    width: 800,
                    height: 600
                },
            ]
        );
        assert_eq!(
            parse_outputs_spec(" 640x480 , 320x240 "),
            vec![
                OutputConfig {
                    width: 640,
                    height: 480
                },
                OutputConfig {
                    width: 320,
                    height: 240
                },
            ]
        );
    }

    #[test]
    fn parse_outputs_spec_rejects_garbage() {
        assert!(parse_outputs_spec("").is_empty());
        assert!(parse_outputs_spec("nope").is_empty());
        assert!(parse_outputs_spec("0x0,-1x10,10x-1").is_empty());
        // partial: keep valid entries only
        assert_eq!(
            parse_outputs_spec("bad,800x600,also-bad"),
            vec![OutputConfig {
                width: 800,
                height: 600
            }]
        );
    }

    #[test]
    fn layout_side_by_side_and_total_size() {
        let outs = parse_outputs_spec("100x50,200x80");
        let laid = layout_outputs_side_by_side(&outs);
        assert_eq!(laid.len(), 2);
        assert_eq!(laid[0].x, 0);
        assert_eq!(laid[1].x, 100);
        assert_eq!(
            total_output_size(&laid),
            OutputConfig {
                width: 300,
                height: 80
            }
        );
    }

    #[test]
    fn outputs_from_env_values_prefers_outputs_spec() {
        let multi = outputs_from_env_values(
            Some("800x600,640x480".into()),
            Some("9999".into()),
            Some("9999".into()),
        );
        assert_eq!(multi.len(), 2);
        assert_eq!(multi[0].width, 800);

        let single = outputs_from_env_values(None, Some("1280".into()), Some("720".into()));
        assert_eq!(
            single,
            vec![OutputConfig {
                width: 1280,
                height: 720
            }]
        );

        let fallback = outputs_from_env_values(Some("garbage".into()), None, None);
        assert_eq!(fallback, vec![OutputConfig::default()]);
    }

    #[test]
    fn client_window_stack_map_focus_z_order() {
        let mut stack = ClientWindowStack::new();
        // Non-overlapping geometries so click-to-raise is unambiguous.
        let a = stack.map_window_at(
            "Finder",
            101,
            WindowGeometry::new(0, 0, 100, 100),
        );
        let b = stack.map_window_at(
            "Terminal",
            102,
            WindowGeometry::new(200, 0, 100, 100),
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.focused().map(|w| w.id.clone()), Some(b.clone()));
        assert_eq!(stack.z_order_ids(), vec![a.clone(), b.clone()]);

        assert!(stack.focus(&a));
        assert_eq!(stack.focused().map(|w| w.id.clone()), Some(a.clone()));
        assert_eq!(stack.z_order_ids(), vec![b.clone(), a.clone()]);

        let hit = stack.focus_at(210.0, 10.0).expect("hit terminal");
        assert_eq!(hit, b);
        assert_eq!(
            stack.focused().map(|w| w.title.as_str()),
            Some("Terminal")
        );

        assert!(stack.unmap(&a));
        assert_eq!(stack.len(), 1);
        assert!(!stack.unmap(&a));
    }

    #[test]
    fn client_window_stack_independent_of_shell_paint_rects() {
        // Two clients → two mapped surfaces in compositor stack (multi-client model).
        let mut stack = ClientWindowStack::new();
        stack.map_window("settings", 1);
        stack.map_window("textedit", 2);
        assert_eq!(stack.windows().len(), 2);
        assert_ne!(stack.windows()[0].pid, 0);
        assert_ne!(stack.windows()[0].id, stack.windows()[1].id);
    }

    #[test]
    fn display_policy_settings_and_env() {
        let mut p = DisplayPolicy::default();
        p.apply_settings_text(
            "hdr_requested=true\nvrr_adaptive=true\nrefresh_rate=120hz\ncolor_space=rec2020\n",
        );
        assert!(p.hdr_requested);
        assert!(p.vrr_adaptive);
        assert_eq!(p.refresh_rate, RefreshRate::Hz120);
        assert_eq!(p.color_space, ColorSpace::Rec2020);
        assert_eq!(p.effective_refresh_rate(), RefreshRate::Adaptive);

        let mut env = HashMap::new();
        env.insert("RETROSHELL_HDR".into(), "0".into());
        env.insert("RETROSHELL_VRR".into(), "false".into());
        env.insert("RETROSHELL_REFRESH".into(), "60".into());
        env.insert("RETROSHELL_COLOR_SPACE".into(), "srgb".into());
        p.apply_env_map(env);
        assert!(!p.hdr_requested);
        assert!(!p.vrr_adaptive);
        assert_eq!(p.refresh_rate, RefreshRate::Hz60);
        assert_eq!(p.color_space, ColorSpace::SRgb);
        assert_eq!(p.effective_refresh_rate(), RefreshRate::Hz60);
    }

    #[test]
    fn display_policy_accepts_hdr_request_alias() {
        let mut p = DisplayPolicy::default();
        p.apply_settings_text("hdr_request=true\n");
        assert!(p.hdr_requested);
    }

    #[test]
    fn selection_mime_lookup_and_fallback() {
        let mut store = HashMap::new();
        store.insert("text/plain".into(), b"hello".to_vec());
        assert_eq!(
            selection_bytes_for_mime(&store, "text/plain"),
            Some(b"hello".as_slice())
        );
        assert_eq!(selection_bytes_for_mime(&store, "image/png"), None);
        assert_eq!(
            selection_bytes_for_mime_with_text_fallback(&store, "text/plain;charset=utf-8"),
            Some(b"hello".as_slice())
        );
        assert_eq!(
            selection_bytes_for_mime_with_text_fallback(&store, "image/png"),
            None
        );
    }

    #[test]
    fn select_backend_kind_force_labwc_wins() {
        assert_eq!(
            select_backend_kind(true, true, true),
            CompositorBackendKind::LabwcFallback
        );
        assert_eq!(
            select_backend_kind(false, false, true),
            CompositorBackendKind::LabwcFallback
        );
        assert_eq!(
            select_backend_kind(true, false, true),
            CompositorBackendKind::LabwcFallback
        );
    }

    #[test]
    fn select_backend_kind_session_drm_when_prefer_and_dri3() {
        assert_eq!(
            select_backend_kind(true, true, false),
            CompositorBackendKind::SessionDrm
        );
    }

    #[test]
    fn select_backend_kind_nested_x11_otherwise() {
        // prefer_drm but no DRI3 → nested (honest default; may fail GL later)
        assert_eq!(
            select_backend_kind(true, false, false),
            CompositorBackendKind::NestedX11
        );
        // no prefer_drm even with DRI3 → nested
        assert_eq!(
            select_backend_kind(false, true, false),
            CompositorBackendKind::NestedX11
        );
        assert_eq!(
            select_backend_kind(false, false, false),
            CompositorBackendKind::NestedX11
        );
    }

    #[test]
    fn detect_dri3_from_env_value_parses_0_1() {
        assert_eq!(detect_dri3_from_env_value(Some("1")), Some(true));
        assert_eq!(detect_dri3_from_env_value(Some("0")), Some(false));
        assert_eq!(detect_dri3_from_env_value(Some("true")), Some(true));
        assert_eq!(detect_dri3_from_env_value(Some("false")), Some(false));
        assert_eq!(detect_dri3_from_env_value(None), None);
        assert_eq!(detect_dri3_from_env_value(Some("")), None);
        assert_eq!(detect_dri3_from_env_value(Some("maybe")), None);
    }

    #[test]
    fn session_mode_summary_is_honest() {
        let nested = session_mode_summary(CompositorBackendKind::NestedX11);
        assert!(nested.contains("nested_x11"));
        assert!(!nested.contains("session_drm"));

        let drm = session_mode_summary(CompositorBackendKind::SessionDrm);
        assert!(drm.contains("session_drm"));
        assert!(drm.contains("DRM"));

        let labwc = session_mode_summary(CompositorBackendKind::LabwcFallback);
        assert!(labwc.contains("labwc"));
        assert!(labwc.contains("fallback") || labwc.contains("not retro-compositor"));
    }

    #[test]
    fn discover_drm_nodes_from_names_orders_primary_first() {
        let names = vec![
            "renderD128".into(),
            "card1".into(),
            "card0".into(),
            "controlD64".into(),
        ];
        let nodes = discover_drm_nodes_from_names(Path::new("/dev/dri"), &names);
        assert_eq!(nodes.len(), 3); // controlD ignored
        assert!(nodes[0].is_primary);
        assert!(nodes[0].path.ends_with("card0") || nodes[0].path.ends_with("card1"));
        assert!(nodes.iter().any(|n| n.path.ends_with("renderD128")));
        assert_eq!(
            preferred_primary_drm_node(&nodes).map(|n| n.is_primary),
            Some(true)
        );
    }

    #[test]
    fn plan_compose_order_puts_overlay_after_under() {
        let z = vec![
            ChromeLayer::Overlay.z_priority(),
            ChromeLayer::Background.z_priority(),
            ChromeLayer::Top.z_priority(),
            ChromeLayer::Bottom.z_priority(),
        ];
        let order = plan_compose_order(&z);
        // Under: Background(1), Bottom(3) then Over indices included in full list
        assert!(order.windows_after_bottom);
        // First under layers should be background then bottom (indices 1 then 3)
        assert_eq!(order.layer_indices_bottom_first[0], 1);
        assert_eq!(order.layer_indices_bottom_first[1], 3);
        // Then top then overlay
        assert_eq!(order.layer_indices_bottom_first[2], 2);
        assert_eq!(order.layer_indices_bottom_first[3], 0);
    }

    #[test]
    fn drm_presentation_pipeline_includes_scanout_stages() {
        let p = drm_presentation_pipeline();
        assert!(p.contains(&DrmPresentationStage::EnumerateConnectors));
        assert!(p.contains(&DrmPresentationStage::CreateDrmSurface));
        assert!(p.contains(&DrmPresentationStage::PageFlipOrPresent));
        assert_eq!(p.first(), Some(&DrmPresentationStage::OpenSeat));
        assert_eq!(p.last(), Some(&DrmPresentationStage::ProtocolLoop));
    }

    #[test]
    fn plan_drm_modeset_prefers_connected() {
        let connectors = vec![
            ("HDMI-A-1".into(), false, Some((1920, 1080, 60_000))),
            ("eDP-1".into(), true, Some((2560, 1600, 60_000))),
        ];
        let plan = plan_drm_modeset(&connectors, 1024, 768, 60_000);
        assert_eq!(plan.connector_name, "eDP-1");
        assert_eq!(plan.mode_w, 2560);
        assert_eq!(plan.mode_h, 1600);
    }

    #[test]
    fn plan_drm_modeset_fallback_when_none() {
        let plan = plan_drm_modeset(&[], 800, 600, 60_000);
        assert_eq!(plan.connector_name, "virtual-fallback");
        assert_eq!(plan.mode_w, 800);
    }

    #[test]
    fn decoration_preference_first_party_csd_external_ssd() {
        assert_eq!(
            decoration_preference_for_app_id("retroshell.finder"),
            DecorationPreference::ClientSide
        );
        assert_eq!(
            decoration_preference_for_app_id("firefox"),
            DecorationPreference::ServerSide
        );
        assert_eq!(
            decoration_preference_for_app_id("org.gnome.Nautilus"),
            DecorationPreference::ServerSide
        );
    }

    #[test]
    fn chrome_layer_specs_sort_and_parse() {
        assert_eq!(ChromeLayer::from_str_loose("TOP"), Some(ChromeLayer::Top));
        let mut specs = vec![
            LayerChromeSpec::notification_overlay(),
            LayerChromeSpec::dock(48),
            LayerChromeSpec::menu_bar(28),
        ];
        sort_chrome_layers(&mut specs);
        assert_eq!(specs[0].name, "dock");
        assert_eq!(specs[1].name, "menu-bar");
        assert_eq!(specs[2].name, "notifications");
        assert!(specs[2].layer.z_priority() > specs[0].layer.z_priority());
    }

    #[test]
    fn text_input_capability_env_parses() {
        assert_eq!(text_input_capability_from_env(None), TextInputCapability::None);
        assert_eq!(
            text_input_capability_from_env(Some("v3")),
            TextInputCapability::TextInputV3
        );
        assert_eq!(
            text_input_capability_from_env(Some("full")),
            TextInputCapability::InputMethodAndTextInput
        );
        assert!(text_input_capability_summary(TextInputCapability::TextInputV3).contains("v3"));
    }

    #[test]
    fn plan_scanout_paint_order_and_clip() {
        let mut els = vec![
            ScanoutElement {
                id: "top".into(),
                x: 10,
                y: 10,
                w: 100,
                h: 100,
                z: 2,
            },
            ScanoutElement {
                id: "bot".into(),
                x: 0,
                y: 0,
                w: 50,
                h: 50,
                z: 0,
            },
        ];
        plan_scanout_paint_order(&mut els);
        assert_eq!(els[0].id, "bot");
        assert_eq!(els[1].id, "top");
        assert_eq!(
            clip_scanout_element_to_output(&els[1], 80, 80),
            Some((10, 10, 70, 70))
        );
        assert_eq!(
            clip_scanout_element_to_output(
                &ScanoutElement {
                    id: "out".into(),
                    x: 100,
                    y: 100,
                    w: 10,
                    h: 10,
                    z: 0
                },
                50,
                50
            ),
            None
        );
    }
}
