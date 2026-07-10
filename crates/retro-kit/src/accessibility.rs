use crate::Rect;
use std::sync::Mutex;

use zbus::blocking::connection::Builder as ConnectionBuilder;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedObjectPath;
use zbus::{fdo, interface};

// ---------------------------------------------------------------------------
// AT-SPI2 constants & pure helpers (no D-Bus required)
// ---------------------------------------------------------------------------

/// Canonical object path for an application's accessible root.
pub const ATSPI_ROOT_PATH: &str = "/org/a11y/atspi/accessible/root";

/// Null parent reference path used by AT-SPI for the application root.
pub const ATSPI_NULL_PATH: &str = "/org/a11y/atspi/null";

/// Prefix for per-node accessible object paths.
pub const ATSPI_ACCESSIBLE_PREFIX: &str = "/org/a11y/atspi/accessible";

/// Build the D-Bus object path for the Nth flat accessibility-tree node.
///
/// Paths are `/org/a11y/atspi/accessible/{index}` (0-based).
pub fn atspi_object_path(index: usize) -> String {
    format!("{ATSPI_ACCESSIBLE_PREFIX}/{index}")
}

/// Sanitize a label for use in an optional path-segment form.
///
/// Keeps alphanumerics, converts other runs to `_`, trims edges, and falls
/// back to `"node"` when empty.
pub fn sanitize_path_segment(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut prev_us = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_us = false;
        } else if !prev_us {
            out.push('_');
            prev_us = true;
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "node".to_string()
    } else {
        trimmed
    }
}

/// Optional human-readable path including role + sanitized label.
///
/// Format: `/org/a11y/atspi/accessible/{index}/{role}_{label}`
pub fn atspi_object_path_with_label(index: usize, role_name: &str, label: &str) -> String {
    let role_seg = sanitize_path_segment(role_name);
    let label_seg = sanitize_path_segment(label);
    format!("{ATSPI_ACCESSIBLE_PREFIX}/{index}/{role_seg}_{label_seg}")
}

/// Map a RetroShell role to the numeric `AtspiRole` enum value.
pub fn role_to_atspi_role(role: AccessibilityRole) -> u32 {
    // Values from atspi-constants.h / org.a11y.atspi.Accessible.GetRole docs.
    match role {
        AccessibilityRole::Window => 23,       // FRAME
        AccessibilityRole::Button => 43,       // BUTTON
        AccessibilityRole::Checkbox => 7,      // CHECK_BOX
        AccessibilityRole::RadioButton => 44,  // RADIO_BUTTON
        AccessibilityRole::TextField => 79,    // ENTRY
        AccessibilityRole::Label => 29,        // LABEL
        AccessibilityRole::List => 31,         // LIST
        AccessibilityRole::ListItem => 32,     // LIST_ITEM
        AccessibilityRole::Tree => 65,         // TREE
        AccessibilityRole::TreeItem => 91,     // TREE_ITEM
        AccessibilityRole::Menu => 33,         // MENU
        AccessibilityRole::MenuItem => 35,     // MENU_ITEM
        AccessibilityRole::MenuBar => 34,      // MENU_BAR
        AccessibilityRole::Toolbar => 63,      // TOOL_BAR
        AccessibilityRole::ScrollBar => 48,    // SCROLL_BAR
        AccessibilityRole::Slider => 51,       // SLIDER
        AccessibilityRole::ProgressBar => 42,  // PROGRESS_BAR
        AccessibilityRole::Dialog => 16,       // DIALOG
        AccessibilityRole::Tab => 37,          // PAGE_TAB
        AccessibilityRole::TabGroup => 38,     // PAGE_TAB_LIST
        AccessibilityRole::Image => 27,        // IMAGE
        AccessibilityRole::Link => 88,         // LINK
        AccessibilityRole::Group => 39,        // PANEL
        AccessibilityRole::Table => 55,        // TABLE
        AccessibilityRole::TableCell => 56,    // TABLE_CELL
        AccessibilityRole::TableRow => 90,     // TABLE_ROW
        AccessibilityRole::Column => 57,       // TABLE_COLUMN_HEADER
        AccessibilityRole::Row => 58,          // TABLE_ROW_HEADER
        AccessibilityRole::StaticText => 116,  // STATIC
        AccessibilityRole::ComboBox => 11,     // COMBO_BOX
        AccessibilityRole::SplitView => 53,    // SPLIT_PANE
        AccessibilityRole::Notification => 101, // NOTIFICATION
        AccessibilityRole::Dock => 63,         // TOOL_BAR
        AccessibilityRole::Desktop => 14,      // DESKTOP_FRAME
        AccessibilityRole::Unknown => 67,      // UNKNOWN
    }
}

/// Build the AT-SPI state bitset (two `u32`s) from an accessibility state.
///
/// Bits correspond to `AtspiStateType` indices in atspi-constants.h.
pub fn state_to_atspi_bitset(state: &AccessibilityState, role: AccessibilityRole) -> [u32; 2] {
    let mut bits: u64 = 0;

    // Common always-useful bits for visible interactive UI.
    if state.enabled {
        bits |= 1u64 << 8; // ENABLED
        bits |= 1u64 << 24; // SENSITIVE
    }
    if state.visible {
        bits |= 1u64 << 30; // VISIBLE
        bits |= 1u64 << 25; // SHOWING
    }
    if role.is_focusable() {
        bits |= 1u64 << 11; // FOCUSABLE
    }
    if state.focused {
        bits |= 1u64 << 12; // FOCUSED
    }
    if state.selected {
        bits |= 1u64 << 23; // SELECTED
    }
    if state.busy {
        bits |= 1u64 << 3; // BUSY
    }
    if let Some(true) = state.checked {
        bits |= 1u64 << 4; // CHECKED
        bits |= 1u64 << 41; // CHECKABLE
    } else if state.checked == Some(false) {
        bits |= 1u64 << 41; // CHECKABLE
    }
    if let Some(true) = state.expanded {
        bits |= 1u64 << 10; // EXPANDED
        bits |= 1u64 << 9; // EXPANDABLE
    } else if state.expanded == Some(false) {
        bits |= 1u64 << 5; // COLLAPSED
        bits |= 1u64 << 9; // EXPANDABLE
    }

    [
        (bits & 0xffff_ffff) as u32,
        ((bits >> 32) & 0xffff_ffff) as u32,
    ]
}

/// Minimal tree used when the caller has no live widget tree yet.
pub fn default_accessibility_tree(app_name: &str) -> AccessibilityTree {
    let mut tree = AccessibilityTree::new();
    tree.add(AccessibilityNode::new(AccessibilityRole::Window, app_name));
    tree
}

// ---------------------------------------------------------------------------
// Role / state / node / tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityRole {
    Window,
    Button,
    Checkbox,
    RadioButton,
    TextField,
    Label,
    List,
    ListItem,
    Tree,
    TreeItem,
    Menu,
    MenuItem,
    MenuBar,
    Toolbar,
    ScrollBar,
    Slider,
    ProgressBar,
    Dialog,
    Tab,
    TabGroup,
    Image,
    Link,
    Group,
    Table,
    TableCell,
    TableRow,
    Column,
    Row,
    StaticText,
    ComboBox,
    SplitView,
    Notification,
    Dock,
    Desktop,
    Unknown,
}

impl AccessibilityRole {
    /// Returns the AT-SPI2 role name string for this role.
    pub fn role_name(&self) -> &'static str {
        match self {
            Self::Window => "frame",
            Self::Button => "push button",
            Self::Checkbox => "check box",
            Self::RadioButton => "radio button",
            Self::TextField => "text",
            Self::Label => "label",
            Self::List => "list",
            Self::ListItem => "list item",
            Self::Tree => "tree",
            Self::TreeItem => "tree item",
            Self::Menu => "menu",
            Self::MenuItem => "menu item",
            Self::MenuBar => "menu bar",
            Self::Toolbar => "tool bar",
            Self::ScrollBar => "scroll bar",
            Self::Slider => "slider",
            Self::ProgressBar => "progress bar",
            Self::Dialog => "dialog",
            Self::Tab => "page tab",
            Self::TabGroup => "page tab list",
            Self::Image => "image",
            Self::Link => "link",
            Self::Group => "panel",
            Self::Table => "table",
            Self::TableCell => "table cell",
            Self::TableRow => "table row",
            Self::Column => "table column header",
            Self::Row => "table row header",
            Self::StaticText => "label",
            Self::ComboBox => "combo box",
            Self::SplitView => "split pane",
            Self::Notification => "alert",
            Self::Dock => "tool bar",
            Self::Desktop => "desktop frame",
            Self::Unknown => "unknown",
        }
    }

    /// Returns true if an element with this role can receive keyboard focus.
    pub fn is_focusable(&self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::Checkbox
                | Self::RadioButton
                | Self::TextField
                | Self::ListItem
                | Self::TreeItem
                | Self::MenuItem
                | Self::Tab
                | Self::Slider
                | Self::ComboBox
                | Self::Link
        )
    }
}

#[derive(Debug, Clone)]
pub struct AccessibilityState {
    pub focused: bool,
    pub enabled: bool,
    pub visible: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub expanded: Option<bool>,
    pub busy: bool,
}

impl Default for AccessibilityState {
    fn default() -> Self {
        Self {
            focused: false,
            enabled: true,
            visible: true,
            selected: false,
            checked: None,
            expanded: None,
            busy: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    pub role: AccessibilityRole,
    pub label: String,
    pub description: String,
    pub rect: Rect,
    pub state: AccessibilityState,
    pub children: Vec<AccessibilityNode>,
    pub index: usize,
    pub parent: Option<usize>,
}

impl AccessibilityNode {
    pub fn new(role: AccessibilityRole, label: &str) -> Self {
        Self {
            role,
            label: label.to_string(),
            description: String::new(),
            rect: Rect::ZERO,
            state: AccessibilityState::default(),
            children: vec![],
            index: 0,
            parent: None,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Convenience wrapper: returns the AT-SPI role name for this node's role.
    pub fn role_name(&self) -> &'static str {
        self.role.role_name()
    }

    /// Returns true if this node's role can receive keyboard focus.
    pub fn is_focusable(&self) -> bool {
        self.role.is_focusable()
    }
}

// ---------------------------------------------------------------------------
// AccessibilityTree — flat list of nodes for the current render frame
// ---------------------------------------------------------------------------

/// A flat, ordered collection of `AccessibilityNode` items representing
/// the accessibility tree for the current frame.
#[derive(Debug, Default, Clone)]
pub struct AccessibilityTree {
    nodes: Vec<AccessibilityNode>,
}

impl AccessibilityTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a node to the tree.
    pub fn add(&mut self, node: AccessibilityNode) {
        self.nodes.push(node);
    }

    /// Remove all nodes from the tree.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Number of top-level (flat) nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True when the tree has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Borrow the flat node list.
    pub fn nodes(&self) -> &[AccessibilityNode] {
        &self.nodes
    }

    /// Node at a flat index, if any.
    pub fn get(&self, index: usize) -> Option<&AccessibilityNode> {
        self.nodes.get(index)
    }

    /// Return a text representation of each node suitable for AT-SPI object paths.
    /// Format: `"role:<role_name> label:<label>"`.
    pub fn to_atspi_objects(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|n| format!("role:{} label:{}", n.role_name(), n.label))
            .collect()
    }

    /// D-Bus object paths for each flat node (canonical numeric form).
    pub fn to_atspi_paths(&self) -> Vec<String> {
        (0..self.nodes.len()).map(atspi_object_path).collect()
    }
}

// ---------------------------------------------------------------------------
// D-Bus AT-SPI interfaces
// ---------------------------------------------------------------------------

/// Object reference `(bus_name, object_path)` used throughout AT-SPI.
type ObjectRef = (String, OwnedObjectPath);

fn owned_path(path: &str) -> fdo::Result<OwnedObjectPath> {
    OwnedObjectPath::try_from(path).map_err(|e| fdo::Error::Failed(e.to_string()))
}

fn object_ref(bus: &str, path: &str) -> fdo::Result<ObjectRef> {
    Ok((bus.to_string(), owned_path(path)?))
}

/// `org.a11y.atspi.Accessible` — base interface for every accessible object.
struct AtspiAccessible {
    name: String,
    description: String,
    role: u32,
    role_name: String,
    bus_name: String,
    /// Parent `(bus_name, path)`. Empty bus + null path for the app root.
    parent_bus: String,
    parent_path: String,
    child_paths: Vec<String>,
    index_in_parent: i32,
    state: [u32; 2],
    accessible_id: String,
    /// Interfaces advertised by GetInterfaces (excluding Accessible itself is ok; include it).
    interfaces: Vec<String>,
}

#[interface(name = "org.a11y.atspi.Accessible")]
impl AtspiAccessible {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }

    #[zbus(property, name = "Name")]
    fn name(&self) -> &str {
        &self.name
    }

    #[zbus(property, name = "Description")]
    fn description(&self) -> &str {
        &self.description
    }

    #[zbus(property, name = "Parent")]
    fn parent(&self) -> fdo::Result<ObjectRef> {
        object_ref(&self.parent_bus, &self.parent_path)
    }

    #[zbus(property, name = "ChildCount")]
    fn child_count(&self) -> i32 {
        self.child_paths.len() as i32
    }

    #[zbus(property, name = "Locale")]
    fn locale(&self) -> &str {
        ""
    }

    #[zbus(property, name = "AccessibleId")]
    fn accessible_id(&self) -> &str {
        &self.accessible_id
    }

    #[zbus(property, name = "HelpText")]
    fn help_text(&self) -> &str {
        ""
    }

    fn get_child_at_index(&self, index: i32) -> fdo::Result<ObjectRef> {
        if index < 0 {
            return Err(fdo::Error::InvalidArgs(format!(
                "child index {index} out of range"
            )));
        }
        let idx = index as usize;
        match self.child_paths.get(idx) {
            Some(path) => object_ref(&self.bus_name, path),
            None => Err(fdo::Error::InvalidArgs(format!(
                "child index {index} out of range (count={})",
                self.child_paths.len()
            ))),
        }
    }

    fn get_children(&self) -> fdo::Result<Vec<ObjectRef>> {
        self.child_paths
            .iter()
            .map(|p| object_ref(&self.bus_name, p))
            .collect()
    }

    fn get_index_in_parent(&self) -> i32 {
        self.index_in_parent
    }

    fn get_relation_set(&self) -> Vec<(u32, Vec<ObjectRef>)> {
        Vec::new()
    }

    fn get_role(&self) -> u32 {
        self.role
    }

    fn get_role_name(&self) -> String {
        self.role_name.clone()
    }

    fn get_localized_role_name(&self) -> String {
        self.role_name.clone()
    }

    fn get_state(&self) -> Vec<u32> {
        self.state.to_vec()
    }

    fn get_attributes(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn get_application(&self) -> fdo::Result<ObjectRef> {
        object_ref(&self.bus_name, ATSPI_ROOT_PATH)
    }

    fn get_interfaces(&self) -> Vec<String> {
        self.interfaces.clone()
    }
}

/// `org.a11y.atspi.Application` — required on the application root object.
struct AtspiApplication {
    id: i32,
    bus_address: String,
}

#[interface(name = "org.a11y.atspi.Application")]
impl AtspiApplication {
    #[zbus(property, name = "ToolkitName")]
    fn toolkit_name(&self) -> &str {
        "RetroShell"
    }

    #[zbus(property, name = "Version")]
    fn version(&self) -> &str {
        "0.1.0"
    }

    #[zbus(property, name = "ToolkitVersion")]
    fn toolkit_version(&self) -> &str {
        "0.1.0"
    }

    #[zbus(property, name = "AtspiVersion")]
    fn atspi_version(&self) -> &str {
        "2.1"
    }

    #[zbus(property, name = "InterfaceVersion")]
    fn interface_version(&self) -> u32 {
        2
    }

    #[zbus(property, name = "Id")]
    fn id(&self) -> i32 {
        self.id
    }

    #[zbus(property, name = "Id")]
    fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    fn get_locale(&self, _lctype: u32) -> String {
        String::new()
    }

    fn get_application_bus_address(&self) -> String {
        self.bus_address.clone()
    }
}

// ---------------------------------------------------------------------------
// Registration (session / a11y bus)
// ---------------------------------------------------------------------------

/// Keeps the a11y (or session) bus connection alive for the process lifetime.
struct AtSpiRegistration {
    /// Retained so exported objects remain available to ATs.
    _connection: Connection,
    app_name: String,
    child_count: usize,
    bus_name: String,
    embedded: bool,
}

static REGISTRATION: Mutex<Option<AtSpiRegistration>> = Mutex::new(None);

/// Result of a successful AT-SPI registration attempt.
#[derive(Debug, Clone)]
pub struct AtSpiRegistrationInfo {
    pub app_name: String,
    pub bus_name: String,
    pub child_count: usize,
    pub embedded_with_registry: bool,
}

/// Returns info about the active registration, if any.
pub fn at_spi_registration_info() -> Option<AtSpiRegistrationInfo> {
    REGISTRATION.lock().ok().and_then(|g| {
        g.as_ref().map(|r| AtSpiRegistrationInfo {
            app_name: r.app_name.clone(),
            bus_name: r.bus_name.clone(),
            child_count: r.child_count,
            embedded_with_registry: r.embedded,
        })
    })
}

/// Register this process as an AT-SPI2 application with a default tree.
///
/// Builds a minimal one-window tree named `app_name`. See
/// [`register_at_spi_app_with_tree`] for details.
pub fn register_at_spi_app(app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tree = default_accessibility_tree(app_name);
    register_at_spi_app_with_tree(app_name, &tree)
}

/// Register this process as an AT-SPI2 application exposing `tree`.
///
/// Steps:
/// 1. Connect to the session bus (skip with log if unavailable — e.g. macOS CI).
/// 2. Prefer the dedicated accessibility bus via `org.a11y.Bus.GetAddress`; fall
///    back to the session bus when the a11y bus is not running.
/// 3. Export `/org/a11y/atspi/accessible/root` with Application + Accessible.
/// 4. Export each flat tree node as `/org/a11y/atspi/accessible/{i}`.
/// 5. Best-effort `Socket.Embed` with the AT-SPI registry.
///
/// Returns `Ok(())` after a successful object export. When no D-Bus session is
/// available, returns `Ok(())` after logging a skip — never claims registration
/// in that case. Hard failures after a bus is available are returned as `Err`.
pub fn register_at_spi_app_with_tree(
    app_name: &str,
    tree: &AccessibilityTree,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Session bus
    let session = match Connection::session() {
        Ok(c) => c,
        Err(err) => {
            tracing::warn!(
                app = app_name,
                error = %err,
                "AT-SPI2 registration skipped: D-Bus session bus not available"
            );
            return Ok(());
        }
    };

    // 2. Prefer a11y bus; fall back to session.
    let (conn, bus_kind) = match a11y_bus_address(&session) {
        Ok(addr) => match ConnectionBuilder::address(addr.as_str())?.build() {
            Ok(a11y) => {
                tracing::debug!(app = app_name, address = %addr, "Connected to accessibility bus");
                (a11y, "a11y")
            }
            Err(err) => {
                tracing::warn!(
                    app = app_name,
                    error = %err,
                    "AT-SPI2 a11y bus connect failed; falling back to session bus"
                );
                (session, "session")
            }
        },
        Err(err) => {
            tracing::info!(
                app = app_name,
                error = %err,
                "AT-SPI2 a11y bus unavailable; exporting on session bus"
            );
            (session, "session")
        }
    };

    let bus_name = conn
        .unique_name()
        .map(|n| n.as_str().to_string())
        .unwrap_or_default();

    if bus_name.is_empty() {
        tracing::warn!(
            app = app_name,
            "AT-SPI2 registration skipped: connection has no unique bus name"
        );
        return Ok(());
    }

    // Ensure non-empty children for demo/AT visibility when caller passes empty tree.
    let owned_tree;
    let tree = if tree.is_empty() {
        owned_tree = default_accessibility_tree(app_name);
        &owned_tree
    } else {
        tree
    };

    let child_paths: Vec<String> = tree.to_atspi_paths();

    // 3–4. Export root + children. Scope the object_server borrow so `conn`
    // can be moved into the process-lifetime registration handle afterward.
    {
        let server = conn.object_server();

        // Root accessible (Application role) + Application interface
        let root = AtspiAccessible {
            name: app_name.to_string(),
            description: format!("RetroShell application {app_name}"),
            role: 75, // ATSPI_ROLE_APPLICATION
            role_name: "application".to_string(),
            bus_name: bus_name.clone(),
            parent_bus: String::new(),
            parent_path: ATSPI_NULL_PATH.to_string(),
            child_paths: child_paths.clone(),
            index_in_parent: -1,
            state: state_to_atspi_bitset(
                &AccessibilityState::default(),
                AccessibilityRole::Window,
            ),
            accessible_id: "root".to_string(),
            interfaces: vec![
                "org.a11y.atspi.Accessible".to_string(),
                "org.a11y.atspi.Application".to_string(),
            ],
        };
        server.at(ATSPI_ROOT_PATH, root)?;

        let app_iface = AtspiApplication {
            id: 0,
            bus_address: String::new(),
        };
        server.at(ATSPI_ROOT_PATH, app_iface)?;

        // Child nodes
        for (i, node) in tree.nodes().iter().enumerate() {
            let path = atspi_object_path(i);
            // Nested children of a flat node (if any) are exported as shallow
            // leaf objects under `{path}/c{j}` for structural honesty.
            let nested_paths: Vec<String> = (0..node.children.len())
                .map(|j| format!("{path}/c{j}"))
                .collect();

            for (j, child) in node.children.iter().enumerate() {
                let cpath = format!("{path}/c{j}");
                let child_obj = AtspiAccessible {
                    name: child.label.clone(),
                    description: child.description.clone(),
                    role: role_to_atspi_role(child.role),
                    role_name: child.role_name().to_string(),
                    bus_name: bus_name.clone(),
                    parent_bus: bus_name.clone(),
                    parent_path: path.clone(),
                    child_paths: vec![],
                    index_in_parent: j as i32,
                    state: state_to_atspi_bitset(&child.state, child.role),
                    accessible_id: format!("n{i}_c{j}"),
                    interfaces: vec!["org.a11y.atspi.Accessible".to_string()],
                };
                server.at(cpath.as_str(), child_obj)?;
            }

            let obj = AtspiAccessible {
                name: node.label.clone(),
                description: node.description.clone(),
                role: role_to_atspi_role(node.role),
                role_name: node.role_name().to_string(),
                bus_name: bus_name.clone(),
                parent_bus: bus_name.clone(),
                parent_path: ATSPI_ROOT_PATH.to_string(),
                child_paths: nested_paths,
                index_in_parent: i as i32,
                state: state_to_atspi_bitset(&node.state, node.role),
                accessible_id: format!("n{i}"),
                interfaces: vec!["org.a11y.atspi.Accessible".to_string()],
            };
            server.at(path.as_str(), obj)?;
        }
    }

    // 5. Best-effort registry Embed
    let embedded = match embed_with_registry(&conn, &bus_name) {
        Ok(()) => {
            tracing::info!(
                app = app_name,
                bus = %bus_name,
                bus_kind,
                children = child_paths.len(),
                "AT-SPI2 registered with accessibility registry (Socket.Embed)"
            );
            true
        }
        Err(err) => {
            tracing::warn!(
                app = app_name,
                bus = %bus_name,
                bus_kind,
                children = child_paths.len(),
                error = %err,
                "AT-SPI2 objects exported but registry Embed failed (registry may be absent)"
            );
            false
        }
    };

    tracing::info!(
        app = app_name,
        bus = %bus_name,
        bus_kind,
        root = ATSPI_ROOT_PATH,
        children = child_paths.len(),
        embedded,
        "AT-SPI2 Accessible tree exported"
    );

    if let Ok(mut guard) = REGISTRATION.lock() {
        *guard = Some(AtSpiRegistration {
            _connection: conn,
            app_name: app_name.to_string(),
            child_count: child_paths.len(),
            bus_name,
            embedded,
        });
    }

    Ok(())
}

fn a11y_bus_address(session: &Connection) -> Result<String, Box<dyn std::error::Error>> {
    let reply = session.call_method(
        Some("org.a11y.Bus"),
        "/org/a11y/bus",
        Some("org.a11y.Bus"),
        "GetAddress",
        &(),
    )?;
    let address: String = reply.body().deserialize()?;
    if address.is_empty() {
        return Err("org.a11y.Bus.GetAddress returned empty address".into());
    }
    Ok(address)
}

fn embed_with_registry(
    conn: &Connection,
    bus_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Socket.Embed(IN plug (so), OUT socket (so))
    let plug: ObjectRef = object_ref(bus_name, ATSPI_ROOT_PATH)?;
    let reply = conn.call_method(
        Some("org.a11y.atspi.Registry"),
        "/org/a11y/atspi/accessible/root",
        Some("org.a11y.atspi.Socket"),
        "Embed",
        &plug,
    )?;
    // Registry returns its own (so); we don't need it beyond acknowledging success.
    let _socket: ObjectRef = reply.body().deserialize()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests (pure helpers — no D-Bus)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atspi_object_path_generation() {
        assert_eq!(atspi_object_path(0), "/org/a11y/atspi/accessible/0");
        assert_eq!(atspi_object_path(12), "/org/a11y/atspi/accessible/12");
        assert_eq!(ATSPI_ROOT_PATH, "/org/a11y/atspi/accessible/root");
    }

    #[test]
    fn atspi_object_path_with_label_sanitizes() {
        let p = atspi_object_path_with_label(0, "push button", "Save As...");
        assert_eq!(p, "/org/a11y/atspi/accessible/0/push_button_save_as");
        let empty = atspi_object_path_with_label(1, "!!!", "???");
        assert_eq!(empty, "/org/a11y/atspi/accessible/1/node_node");
    }

    #[test]
    fn sanitize_path_segment_edges() {
        assert_eq!(sanitize_path_segment(""), "node");
        assert_eq!(sanitize_path_segment("OK"), "ok");
        assert_eq!(sanitize_path_segment("Hello World"), "hello_world");
    }

    #[test]
    fn role_to_atspi_role_known_values() {
        assert_eq!(role_to_atspi_role(AccessibilityRole::Button), 43);
        assert_eq!(role_to_atspi_role(AccessibilityRole::Window), 23);
        assert_eq!(role_to_atspi_role(AccessibilityRole::TextField), 79);
        assert_eq!(role_to_atspi_role(AccessibilityRole::Unknown), 67);
    }

    #[test]
    fn state_bitset_sets_enabled_visible() {
        let state = AccessibilityState::default();
        let bits = state_to_atspi_bitset(&state, AccessibilityRole::Button);
        // ENABLED bit 8, SENSITIVE 24, VISIBLE 30, SHOWING 25, FOCUSABLE 11
        assert_ne!(bits[0] & (1 << 8), 0);
        assert_ne!(bits[0] & (1 << 11), 0);
        assert_ne!(bits[0] & (1 << 24), 0);
        assert_ne!(bits[0] & (1 << 25), 0);
        assert_ne!(bits[0] & (1 << 30), 0);
    }

    #[test]
    fn tree_to_atspi_paths_matches_indices() {
        let mut tree = AccessibilityTree::new();
        tree.add(AccessibilityNode::new(AccessibilityRole::Button, "A"));
        tree.add(AccessibilityNode::new(AccessibilityRole::Label, "B"));
        assert_eq!(
            tree.to_atspi_paths(),
            vec![
                "/org/a11y/atspi/accessible/0".to_string(),
                "/org/a11y/atspi/accessible/1".to_string(),
            ]
        );
        assert_eq!(tree.len(), 2);
        assert!(!tree.is_empty());
        assert_eq!(tree.get(0).map(|n| n.label.as_str()), Some("A"));
    }

    #[test]
    fn default_tree_is_non_empty() {
        let tree = default_accessibility_tree("Demo");
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.nodes()[0].label, "Demo");
        assert_eq!(tree.nodes()[0].role, AccessibilityRole::Window);
    }
}
