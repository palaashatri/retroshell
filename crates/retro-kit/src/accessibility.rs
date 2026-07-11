//! AT-SPI2 accessibility export and pure a11y helpers for RetroShell.
//!
//! # What works today
//! - Role name + numeric `AtspiRole` mapping for kit chrome and widgets
//! - Flat (+ one nested level) `AccessibilityTree` → D-Bus `Accessible` objects
//! - `org.a11y.atspi.Action` on actionable roles (Activate / Press / Focus)
//! - Pure keyboard chrome focus-order policy (menu bar → desktop icons → dock)
//! - Session / a11y-bus registration with best-effort registry `Socket.Embed`
//!
//! # Orca-incomplete (honest — do not claim “Orca complete”)
//! - **No D-Bus AT-SPI event emission**: an in-process [`AccessibilityEventBus`]
//!   queues Focus / StateChanged / etc. for toolkit + tests, but those events are
//!   **not** yet signalled on the atspi bus (`org.a11y.atspi.Event.Object`, …).
//! - **No live tree re-export**: the D-Bus tree is snapshotted at register time;
//!   focus/selection changes update pure helpers only until re-register/sync lands.
//! - **No `org.a11y.atspi.Text` / `EditableText`**: text fields export role + Focus
//!   only; Orca cannot read caret, selection, or typed content via AT-SPI.
//! - **No `org.a11y.atspi.Component`**: extents / window coords are not on the bus.
//! - **`DoAction` is advisory**: Action methods return success for valid indices
//!   but do not drive the real toolkit focus or activation path.
//! - **Shallow nesting**: only one child level under flat nodes is exported.
//! - **No Selection / Table / Value / Collection** interfaces for lists/trees/sliders.
//! - **No relation set** (label-for, controlled-by, etc.).
//! - **Shell chrome tree is structural**, not bound to live menu/dock widgets.
//!
//! Raising Orca-usable domain means keeping roles/actions present and testable,
//! not claiming full assistive-tech parity.

use crate::Rect;
use std::collections::VecDeque;
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

/// AT-SPI Action name: primary activation (menu item, list item, default click).
pub const ACTION_ACTIVATE: &str = "Activate";

/// AT-SPI Action name: press (buttons and similar push controls).
pub const ACTION_PRESS: &str = "Press";

/// AT-SPI Action name: move keyboard focus to the object.
pub const ACTION_FOCUS: &str = "Focus";

/// Interface name advertised for actionable accessibles.
pub const ATSPI_ACTION_IFACE: &str = "org.a11y.atspi.Action";

/// Interface name for every accessible object.
pub const ATSPI_ACCESSIBLE_IFACE: &str = "org.a11y.atspi.Accessible";

/// Interface name for the application root.
pub const ATSPI_APPLICATION_IFACE: &str = "org.a11y.atspi.Application";

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
///
/// Values from atspi-constants.h / org.a11y.atspi.Accessible.GetRole docs.
/// Exhaustive match — adding a role without a mapping is a compile error.
pub fn role_to_atspi_role(role: AccessibilityRole) -> u32 {
    match role {
        AccessibilityRole::Window => 23,        // FRAME
        AccessibilityRole::Button => 43,        // PUSH_BUTTON
        AccessibilityRole::Checkbox => 7,       // CHECK_BOX
        AccessibilityRole::RadioButton => 44,   // RADIO_BUTTON
        AccessibilityRole::TextField => 79,     // ENTRY
        AccessibilityRole::Label => 29,         // LABEL
        AccessibilityRole::List => 31,          // LIST
        AccessibilityRole::ListItem => 32,      // LIST_ITEM
        AccessibilityRole::Tree => 65,          // TREE
        AccessibilityRole::TreeItem => 91,      // TREE_ITEM
        AccessibilityRole::Menu => 33,          // MENU
        AccessibilityRole::MenuItem => 35,      // MENU_ITEM
        AccessibilityRole::MenuBar => 34,       // MENU_BAR
        AccessibilityRole::Toolbar => 63,       // TOOL_BAR
        AccessibilityRole::ScrollBar => 48,     // SCROLL_BAR
        AccessibilityRole::Slider => 51,        // SLIDER
        AccessibilityRole::ProgressBar => 42,   // PROGRESS_BAR
        AccessibilityRole::Dialog => 16,        // DIALOG
        AccessibilityRole::Tab => 37,           // PAGE_TAB
        AccessibilityRole::TabGroup => 38,      // PAGE_TAB_LIST
        AccessibilityRole::Image => 27,         // IMAGE
        AccessibilityRole::Link => 88,          // LINK
        AccessibilityRole::Group => 39,         // PANEL
        AccessibilityRole::Table => 55,         // TABLE
        AccessibilityRole::TableCell => 56,     // TABLE_CELL
        AccessibilityRole::TableRow => 90,      // TABLE_ROW
        AccessibilityRole::Column => 57,        // TABLE_COLUMN_HEADER
        AccessibilityRole::Row => 58,           // TABLE_ROW_HEADER
        AccessibilityRole::StaticText => 116,   // STATIC
        AccessibilityRole::ComboBox => 11,      // COMBO_BOX
        AccessibilityRole::SplitView => 53,     // SPLIT_PANE
        AccessibilityRole::Notification => 101, // NOTIFICATION
        // Dock has no dedicated AtspiRole; TOOL_BAR is the closest structural match.
        AccessibilityRole::Dock => 63, // TOOL_BAR
        AccessibilityRole::Desktop => 14, // DESKTOP_FRAME
        AccessibilityRole::Unknown => 67, // UNKNOWN
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
    if role.is_focusable() || role.is_chrome_focus_target() {
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

/// Structural shell chrome tree for desktop AT-SPI export.
///
/// Order of top-level nodes matches [`ChromeFocusRegion::ORDER`]:
/// menu bar → desktop (with icon list items) → dock (with launch buttons),
/// plus a frame named `app_name`.
///
/// This is **not** wired to live widgets; it deepens the exported tree for
/// ATs and unit tests. Orca still lacks events/text/component (see module docs).
pub fn shell_chrome_accessibility_tree(app_name: &str) -> AccessibilityTree {
    let mut tree = AccessibilityTree::new();

    let mut menu_bar = AccessibilityNode::new(AccessibilityRole::MenuBar, "Menu Bar");
    for title in ["Apple", "File", "Edit", "View", "Window", "Help"] {
        let mut menu = AccessibilityNode::new(AccessibilityRole::Menu, title);
        menu.children
            .push(AccessibilityNode::new(AccessibilityRole::MenuItem, title));
        menu_bar.children.push(menu);
    }
    tree.add(menu_bar);

    let mut desktop = AccessibilityNode::new(AccessibilityRole::Desktop, "Desktop");
    for icon in ["Home", "Trash", "Applications"] {
        desktop
            .children
            .push(AccessibilityNode::new(AccessibilityRole::ListItem, icon));
    }
    tree.add(desktop);

    let mut dock = AccessibilityNode::new(AccessibilityRole::Dock, "Dock");
    for app in ["Finder", "Terminal", "Settings"] {
        dock.children
            .push(AccessibilityNode::new(AccessibilityRole::Button, app));
    }
    tree.add(dock);

    tree.add(AccessibilityNode::new(AccessibilityRole::Window, app_name));
    tree
}

// ---------------------------------------------------------------------------
// AccessibleAction — pure Activate / Press / Focus set
// ---------------------------------------------------------------------------

/// Canonical AT-SPI actions RetroShell exposes on interactive nodes.
///
/// Names match common AT-SPI / ATK action strings so Orca and similar ATs can
/// discover them via `org.a11y.atspi.Action`. Invoking `DoAction` on the bus is
/// still advisory (see module-level Orca-incomplete notes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessibleAction {
    /// Primary activation (default click / open / invoke).
    Activate,
    /// Press for push-button style controls.
    Press,
    /// Move keyboard focus to the object.
    Focus,
}

impl AccessibleAction {
    /// All shipped action variants (order is stable for regression tests).
    pub const ALL: [AccessibleAction; 3] = [
        AccessibleAction::Activate,
        AccessibleAction::Press,
        AccessibleAction::Focus,
    ];

    /// AT-SPI action name string.
    pub fn name(self) -> &'static str {
        match self {
            Self::Activate => ACTION_ACTIVATE,
            Self::Press => ACTION_PRESS,
            Self::Focus => ACTION_FOCUS,
        }
    }

    /// Human-readable description for `GetDescription`.
    pub fn description(self) -> &'static str {
        match self {
            Self::Activate => "Activates the accessible object",
            Self::Press => "Presses the control",
            Self::Focus => "Gives keyboard focus to the object",
        }
    }

    /// Key binding string for AT-SPI (empty when toolkit-owned / unknown).
    pub fn key_binding(self) -> &'static str {
        match self {
            Self::Activate => "Return",
            Self::Press => "space",
            Self::Focus => "",
        }
    }

    /// Parse a canonical action name (case-sensitive AT-SPI form).
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            ACTION_ACTIVATE => Some(Self::Activate),
            ACTION_PRESS => Some(Self::Press),
            ACTION_FOCUS => Some(Self::Focus),
            _ => None,
        }
    }
}

/// Actions advertised for a role on the AT-SPI Action interface.
///
/// Pure policy — no D-Bus. Removing Activate/Press/Focus coverage for these
/// roles should fail unit tests in this module.
pub fn actions_for_role(role: AccessibilityRole) -> Vec<AccessibleAction> {
    use AccessibleAction::*;
    match role {
        AccessibilityRole::Button | AccessibilityRole::Link => {
            vec![Activate, Press, Focus]
        }
        AccessibilityRole::MenuItem
        | AccessibilityRole::ListItem
        | AccessibilityRole::TreeItem
        | AccessibilityRole::Checkbox
        | AccessibilityRole::RadioButton
        | AccessibilityRole::Tab => {
            vec![Activate, Focus]
        }
        AccessibilityRole::TextField
        | AccessibilityRole::Slider
        | AccessibilityRole::ComboBox
        | AccessibilityRole::MenuBar
        | AccessibilityRole::Dock
        | AccessibilityRole::Desktop
        | AccessibilityRole::Menu
        | AccessibilityRole::Toolbar => {
            vec![Focus]
        }
        AccessibilityRole::Window
        | AccessibilityRole::Label
        | AccessibilityRole::List
        | AccessibilityRole::Tree
        | AccessibilityRole::ScrollBar
        | AccessibilityRole::ProgressBar
        | AccessibilityRole::Dialog
        | AccessibilityRole::TabGroup
        | AccessibilityRole::Image
        | AccessibilityRole::Group
        | AccessibilityRole::Table
        | AccessibilityRole::TableCell
        | AccessibilityRole::TableRow
        | AccessibilityRole::Column
        | AccessibilityRole::Row
        | AccessibilityRole::StaticText
        | AccessibilityRole::SplitView
        | AccessibilityRole::Notification
        | AccessibilityRole::Unknown => Vec::new(),
    }
}

/// Whether this role should advertise `org.a11y.atspi.Action`.
pub fn role_has_actions(role: AccessibilityRole) -> bool {
    !actions_for_role(role).is_empty()
}

/// Interface names for a node with the given role (Accessible ± Action).
pub fn interfaces_for_role(role: AccessibilityRole) -> Vec<String> {
    let mut ifaces = vec![ATSPI_ACCESSIBLE_IFACE.to_string()];
    if role_has_actions(role) {
        ifaces.push(ATSPI_ACTION_IFACE.to_string());
    }
    ifaces
}

// ---------------------------------------------------------------------------
// Keyboard-only chrome navigation policy (pure)
// ---------------------------------------------------------------------------

/// Shell chrome regions in keyboard-only focus cycle order.
///
/// Pure policy used by shell keyboard paths (F6-style) and tests. Not a full
/// focus manager — live routing still belongs in the shell event loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromeFocusRegion {
    /// Global menu bar.
    MenuBar,
    /// Desktop icon field (list items under the desktop frame).
    DesktopIcons,
    /// Application dock.
    Dock,
}

impl ChromeFocusRegion {
    /// Fixed keyboard cycle: menu bar → desktop icons → dock → (wrap).
    pub const ORDER: [ChromeFocusRegion; 3] = [
        ChromeFocusRegion::MenuBar,
        ChromeFocusRegion::DesktopIcons,
        ChromeFocusRegion::Dock,
    ];

    /// Next region in the chrome cycle (wraps).
    pub fn next(self) -> Self {
        match self {
            Self::MenuBar => Self::DesktopIcons,
            Self::DesktopIcons => Self::Dock,
            Self::Dock => Self::MenuBar,
        }
    }

    /// Previous region in the chrome cycle (wraps).
    pub fn prev(self) -> Self {
        match self {
            Self::MenuBar => Self::Dock,
            Self::DesktopIcons => Self::MenuBar,
            Self::Dock => Self::DesktopIcons,
        }
    }

    /// Primary accessibility role for this chrome region.
    pub fn primary_role(self) -> AccessibilityRole {
        match self {
            Self::MenuBar => AccessibilityRole::MenuBar,
            Self::DesktopIcons => AccessibilityRole::Desktop,
            Self::Dock => AccessibilityRole::Dock,
        }
    }

    /// True if `role` belongs to this chrome region (container or children).
    pub fn matches_role(self, role: AccessibilityRole) -> bool {
        match self {
            Self::MenuBar => matches!(
                role,
                AccessibilityRole::MenuBar
                    | AccessibilityRole::Menu
                    | AccessibilityRole::MenuItem
            ),
            Self::DesktopIcons => matches!(
                role,
                AccessibilityRole::Desktop | AccessibilityRole::ListItem | AccessibilityRole::Image
            ),
            Self::Dock => matches!(
                role,
                AccessibilityRole::Dock | AccessibilityRole::Button | AccessibilityRole::Toolbar
            ),
        }
    }

    /// Map a role to its chrome region, if any.
    pub fn from_role(role: AccessibilityRole) -> Option<Self> {
        for region in Self::ORDER {
            // Prefer primary container roles for ambiguous children (Button also
            // appears outside the dock). Container match first.
            if role == region.primary_role() {
                return Some(region);
            }
        }
        match role {
            AccessibilityRole::Menu | AccessibilityRole::MenuItem => Some(Self::MenuBar),
            AccessibilityRole::ListItem | AccessibilityRole::Image => Some(Self::DesktopIcons),
            // Buttons are not uniquely dock; callers should use tree position.
            _ => None,
        }
    }
}

/// Advance chrome focus region for keyboard-only navigation.
///
/// `current = None` starts at the first region (menu bar).
pub fn next_chrome_focus_region(current: Option<ChromeFocusRegion>) -> ChromeFocusRegion {
    match current {
        None => ChromeFocusRegion::ORDER[0],
        Some(r) => r.next(),
    }
}

/// Step backward through chrome regions.
pub fn prev_chrome_focus_region(current: Option<ChromeFocusRegion>) -> ChromeFocusRegion {
    match current {
        None => *ChromeFocusRegion::ORDER.last().unwrap(),
        Some(r) => r.prev(),
    }
}

/// Flat indices of top-level tree nodes matching chrome cycle order.
///
/// Looks for the first top-level node whose role is the region's primary role,
/// in [`ChromeFocusRegion::ORDER`]. Used to test keyboard path wiring against
/// a structural tree without a live compositor.
pub fn chrome_focus_indices(tree: &AccessibilityTree) -> Vec<(ChromeFocusRegion, usize)> {
    let mut out = Vec::new();
    for region in ChromeFocusRegion::ORDER {
        if let Some((idx, _)) = tree
            .nodes()
            .iter()
            .enumerate()
            .find(|(_, n)| n.role == region.primary_role())
        {
            out.push((region, idx));
        }
    }
    out
}

/// Next chrome flat index after `current` (wraps within available chrome nodes).
pub fn next_chrome_focus_index(tree: &AccessibilityTree, current: Option<usize>) -> Option<usize> {
    let order = chrome_focus_indices(tree);
    if order.is_empty() {
        return None;
    }
    let positions: Vec<usize> = order.iter().map(|(_, i)| *i).collect();
    match current {
        None => Some(positions[0]),
        Some(cur) => {
            if let Some(pos) = positions.iter().position(|&i| i == cur) {
                Some(positions[(pos + 1) % positions.len()])
            } else {
                Some(positions[0])
            }
        }
    }
}

/// Focusable node indices in tree order (flat nodes only, not nested children).
pub fn focusable_indices(tree: &AccessibilityTree) -> Vec<usize> {
    tree.nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.role.is_focusable() || n.role.is_chrome_focus_target())
        .map(|(i, _)| i)
        .collect()
}

// ---------------------------------------------------------------------------
// In-process AT-SPI-shaped events (pure — not yet on the D-Bus atspi bus)
// ---------------------------------------------------------------------------

/// Kind of accessibility event mirrored after common AT-SPI Object / Focus events.
///
/// Names are toolkit-facing; mapping onto exact `org.a11y.atspi.Event.*` signal
/// names happens only when D-Bus emission is wired (not yet).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessibleEventKind {
    /// Keyboard / AT focus moved to (or left) an accessible.
    Focus,
    /// A state bit changed (e.g. focused, selected, checked).
    StateChanged,
    /// Component extents changed.
    BoundsChanged,
    /// Active descendant within a container changed.
    ActiveDescendantChanged,
    /// Accessible object created / added to the tree.
    ObjectCreated,
    /// Accessible object destroyed / removed from the tree.
    ObjectDestroyed,
}

impl AccessibleEventKind {
    /// Stable string tag for tests and future D-Bus signal naming.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Focus => "Focus",
            Self::StateChanged => "StateChanged",
            Self::BoundsChanged => "BoundsChanged",
            Self::ActiveDescendantChanged => "ActiveDescendantChanged",
            Self::ObjectCreated => "ObjectCreated",
            Self::ObjectDestroyed => "ObjectDestroyed",
        }
    }
}

/// One queued accessibility event (AT-SPI-shaped payload).
///
/// Field layout follows the common AT-SPI event tuple:
/// `(path, detail1, detail2, any_data)` plus a typed `kind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessibleEvent {
    /// D-Bus-style object path of the accessible that emitted the event.
    pub path: String,
    pub kind: AccessibleEventKind,
    /// Primary integer detail (e.g. 1/0 for state on/off).
    pub detail1: i32,
    /// Secondary integer detail.
    pub detail2: i32,
    /// Free-form string payload (state name, role, etc.).
    pub any_data: String,
}

impl AccessibleEvent {
    /// Construct a Focus event for `path` (detail1 = 1, focused).
    pub fn focus(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: AccessibleEventKind::Focus,
            detail1: 1,
            detail2: 0,
            any_data: String::new(),
        }
    }

    /// Construct a StateChanged event (e.g. `any_data = "focused"`, detail1 = 1/0).
    pub fn state_changed(path: impl Into<String>, state_name: &str, enabled: bool) -> Self {
        Self {
            path: path.into(),
            kind: AccessibleEventKind::StateChanged,
            detail1: if enabled { 1 } else { 0 },
            detail2: 0,
            any_data: state_name.to_string(),
        }
    }

    /// Construct a BoundsChanged event.
    pub fn bounds_changed(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: AccessibleEventKind::BoundsChanged,
            detail1: 0,
            detail2: 0,
            any_data: String::new(),
        }
    }

    /// Construct an ActiveDescendantChanged event; `any_data` is the descendant path.
    pub fn active_descendant_changed(
        path: impl Into<String>,
        descendant_path: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            kind: AccessibleEventKind::ActiveDescendantChanged,
            detail1: 0,
            detail2: 0,
            any_data: descendant_path.into(),
        }
    }

    /// Construct an ObjectCreated event.
    pub fn object_created(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: AccessibleEventKind::ObjectCreated,
            detail1: 0,
            detail2: 0,
            any_data: String::new(),
        }
    }

    /// Construct an ObjectDestroyed event.
    pub fn object_destroyed(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind: AccessibleEventKind::ObjectDestroyed,
            detail1: 0,
            detail2: 0,
            any_data: String::new(),
        }
    }
}

/// Pure in-process event queue for accessibility notifications.
///
/// # Honest limitation — D-Bus not wired
///
/// This bus is **in-process only**. Pushing events does **not** emit AT-SPI D-Bus
/// signals on the accessibility bus (`org.a11y.atspi.Event.Object`,
/// `org.a11y.atspi.Event.Focus`, registry listeners, etc.). Orca and other remote
/// ATs will not see these until emission is connected to the atspi bus. Use this
/// type for toolkit-side consumers and unit tests of event policy.
///
/// Alias-style name: callers may also think of this as an `EventQueue`.
#[derive(Debug, Default, Clone)]
pub struct AccessibilityEventBus {
    queue: VecDeque<AccessibleEvent>,
}

/// Type alias for callers that prefer queue naming.
pub type EventQueue = AccessibilityEventBus;

impl AccessibilityEventBus {
    /// Empty bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of pending events.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// True when no events are queued.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Append an event to the tail of the queue.
    pub fn push(&mut self, event: AccessibleEvent) {
        self.queue.push_back(event);
    }

    /// Remove and return the oldest event, if any.
    pub fn pop(&mut self) -> Option<AccessibleEvent> {
        self.queue.pop_front()
    }

    /// Drain all pending events in FIFO order, leaving the bus empty.
    pub fn drain(&mut self) -> Vec<AccessibleEvent> {
        self.queue.drain(..).collect()
    }

    /// Peek at pending events without removing them.
    pub fn events(&self) -> impl Iterator<Item = &AccessibleEvent> {
        self.queue.iter()
    }

    /// Push a Focus event for `path`.
    ///
    /// This is the pure helper used by tree focus-path hooks. Still not D-Bus.
    pub fn focus_changed(&mut self, path: impl Into<String>) {
        self.push(AccessibleEvent::focus(path));
    }

    /// Push StateChanged for focus on/off (any_data = `"focused"`).
    pub fn focused_state_changed(&mut self, path: impl Into<String>, focused: bool) {
        self.push(AccessibleEvent::state_changed(path, "focused", focused));
    }

    /// Push BoundsChanged for `path`.
    pub fn bounds_changed(&mut self, path: impl Into<String>) {
        self.push(AccessibleEvent::bounds_changed(path));
    }

    /// Push ActiveDescendantChanged.
    pub fn active_descendant_changed(
        &mut self,
        path: impl Into<String>,
        descendant_path: impl Into<String>,
    ) {
        self.push(AccessibleEvent::active_descendant_changed(
            path,
            descendant_path,
        ));
    }

    /// Push ObjectCreated for `path`.
    pub fn object_created(&mut self, path: impl Into<String>) {
        self.push(AccessibleEvent::object_created(path));
    }

    /// Push ObjectDestroyed for `path`.
    pub fn object_destroyed(&mut self, path: impl Into<String>) {
        self.push(AccessibleEvent::object_destroyed(path));
    }
}

/// Parse a flat node index from a canonical AT-SPI path, if present.
///
/// Accepts `/org/a11y/atspi/accessible/{n}` and optional label suffixes
/// (`…/{n}/…`). Nested `…/{n}/c{j}` paths resolve to the flat parent index `n`.
pub fn flat_index_from_atspi_path(path: &str) -> Option<usize> {
    let prefix = format!("{ATSPI_ACCESSIBLE_PREFIX}/");
    let rest = path.strip_prefix(&prefix)?;
    let first = rest.split('/').next()?;
    if first == "root" {
        return None;
    }
    first.parse().ok()
}

/// Free helper: push a Focus event onto `bus` for `path`.
///
/// Same as [`AccessibilityEventBus::focus_changed`]. D-Bus signal emission is
/// **not** performed.
pub fn focus_changed(bus: &mut AccessibilityEventBus, path: impl Into<String>) {
    bus.focus_changed(path);
}

// ---------------------------------------------------------------------------
// Role / state / node / tree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Every known role (stable order for exhaustive regression tests).
    pub const ALL: [AccessibilityRole; 35] = [
        Self::Window,
        Self::Button,
        Self::Checkbox,
        Self::RadioButton,
        Self::TextField,
        Self::Label,
        Self::List,
        Self::ListItem,
        Self::Tree,
        Self::TreeItem,
        Self::Menu,
        Self::MenuItem,
        Self::MenuBar,
        Self::Toolbar,
        Self::ScrollBar,
        Self::Slider,
        Self::ProgressBar,
        Self::Dialog,
        Self::Tab,
        Self::TabGroup,
        Self::Image,
        Self::Link,
        Self::Group,
        Self::Table,
        Self::TableCell,
        Self::TableRow,
        Self::Column,
        Self::Row,
        Self::StaticText,
        Self::ComboBox,
        Self::SplitView,
        Self::Notification,
        Self::Dock,
        Self::Desktop,
        Self::Unknown,
    ];

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
            // Dock reuses tool-bar naming in AT-SPI (no dock-specific role name).
            Self::Dock => "tool bar",
            Self::Desktop => "desktop frame",
            Self::Unknown => "unknown",
        }
    }

    /// Returns true if an element with this role can receive keyboard focus
    /// as a typical interactive control (not chrome region containers).
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

    /// True for shell chrome containers that participate in the keyboard-only
    /// F6-style cycle even when not classic interactive widgets.
    pub fn is_chrome_focus_target(&self) -> bool {
        matches!(
            self,
            Self::MenuBar | Self::Desktop | Self::Dock | Self::Menu | Self::Toolbar
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

    /// Actions advertised for this node's role.
    pub fn actions(&self) -> Vec<AccessibleAction> {
        actions_for_role(self.role)
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

    /// Apply a focus change for the accessible at `path` and emit events.
    ///
    /// Updates flat-node `state.focused` when `path` maps to a flat index via
    /// [`flat_index_from_atspi_path`], then pushes a Focus event (and
    /// StateChanged focused on/off transitions) onto `bus`.
    ///
    /// # Honest limitation
    /// Events stay on the in-process [`AccessibilityEventBus`]. They are **not**
    /// emitted as D-Bus AT-SPI signals yet.
    pub fn focus_changed(&mut self, path: &str, bus: &mut AccessibilityEventBus) {
        let target = flat_index_from_atspi_path(path);

        // Emit StateChanged for nodes whose focused bit flips.
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let now = target == Some(i);
            if node.state.focused != now {
                let node_path = atspi_object_path(i);
                bus.focused_state_changed(&node_path, now);
                node.state.focused = now;
            }
        }

        // Always emit the Focus event for the requested path (AT focus path).
        bus.focus_changed(path);
    }

    /// Focus the flat node at `index` (canonical path) and emit events.
    pub fn focus_changed_index(&mut self, index: usize, bus: &mut AccessibilityEventBus) {
        let path = atspi_object_path(index);
        self.focus_changed(&path, bus);
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

/// `org.a11y.atspi.Action` — Activate / Press / Focus for actionable nodes.
///
/// `DoAction` validates the index and returns `true` when in range. It does
/// **not** drive live UI (Orca-incomplete; see module docs).
struct AtspiAction {
    actions: Vec<AccessibleAction>,
}

impl AtspiAction {
    fn from_role(role: AccessibilityRole) -> Self {
        Self {
            actions: actions_for_role(role),
        }
    }

    fn get(&self, index: i32) -> fdo::Result<&AccessibleAction> {
        if index < 0 {
            return Err(fdo::Error::InvalidArgs(format!(
                "action index {index} out of range"
            )));
        }
        self.actions.get(index as usize).ok_or_else(|| {
            fdo::Error::InvalidArgs(format!(
                "action index {index} out of range (count={})",
                self.actions.len()
            ))
        })
    }
}

#[interface(name = "org.a11y.atspi.Action")]
impl AtspiAction {
    #[zbus(property, name = "NActions")]
    fn n_actions(&self) -> i32 {
        self.actions.len() as i32
    }

    fn get_description(&self, index: i32) -> fdo::Result<String> {
        Ok(self.get(index)?.description().to_string())
    }

    fn get_name(&self, index: i32) -> fdo::Result<String> {
        Ok(self.get(index)?.name().to_string())
    }

    fn get_key_binding(&self, index: i32) -> fdo::Result<String> {
        Ok(self.get(index)?.key_binding().to_string())
    }

    /// Advisory only — does not activate real toolkit widgets.
    fn do_action(&self, index: i32) -> fdo::Result<bool> {
        let _ = self.get(index)?;
        Ok(true)
    }

    /// `(name, description, keybinding)` triples for all actions.
    fn get_actions(&self) -> Vec<(String, String, String)> {
        self.actions
            .iter()
            .map(|a| {
                (
                    a.name().to_string(),
                    a.description().to_string(),
                    a.key_binding().to_string(),
                )
            })
            .collect()
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
/// [`register_at_spi_app_with_tree`] for details. Prefer
/// [`register_at_spi_shell_chrome`] for the desktop shell process.
pub fn register_at_spi_app(app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tree = default_accessibility_tree(app_name);
    register_at_spi_app_with_tree(app_name, &tree)
}

/// Register with the structural shell chrome tree (menu bar / desktop / dock).
///
/// Still Orca-incomplete (no live events/text/component); richer than a single
/// window node so ATs see chrome roles and Action interfaces.
pub fn register_at_spi_shell_chrome(app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tree = shell_chrome_accessibility_tree(app_name);
    register_at_spi_app_with_tree(app_name, &tree)
}

/// Register this process as an AT-SPI2 application exposing `tree`.
///
/// Steps:
/// 1. Connect to the session bus (skip with log if unavailable — e.g. macOS CI).
/// 2. Prefer the dedicated accessibility bus via `org.a11y.Bus.GetAddress`; fall
///    back to the session bus when the a11y bus is not running.
/// 3. Export `/org/a11y/atspi/accessible/root` with Application + Accessible.
/// 4. Export each flat tree node as `/org/a11y/atspi/accessible/{i}` with
///    Action when [`role_has_actions`].
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
                ATSPI_ACCESSIBLE_IFACE.to_string(),
                ATSPI_APPLICATION_IFACE.to_string(),
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
                    interfaces: interfaces_for_role(child.role),
                };
                server.at(cpath.as_str(), child_obj)?;
                if role_has_actions(child.role) {
                    server.at(cpath.as_str(), AtspiAction::from_role(child.role))?;
                }
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
                interfaces: interfaces_for_role(node.role),
            };
            server.at(path.as_str(), obj)?;
            if role_has_actions(node.role) {
                server.at(path.as_str(), AtspiAction::from_role(node.role))?;
            }
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
        "AT-SPI2 Accessible tree exported (Action on actionable roles; still Orca-incomplete)"
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
        // Core chrome + widget roles required for shell a11y.
        assert_eq!(role_to_atspi_role(AccessibilityRole::Button), 43);
        assert_eq!(role_to_atspi_role(AccessibilityRole::Window), 23);
        assert_eq!(role_to_atspi_role(AccessibilityRole::TextField), 79);
        assert_eq!(role_to_atspi_role(AccessibilityRole::MenuBar), 34);
        assert_eq!(role_to_atspi_role(AccessibilityRole::ListItem), 32);
        assert_eq!(role_to_atspi_role(AccessibilityRole::Dock), 63);
        assert_eq!(role_to_atspi_role(AccessibilityRole::Desktop), 14);
        assert_eq!(role_to_atspi_role(AccessibilityRole::Unknown), 67);
    }

    #[test]
    fn role_to_atspi_role_covers_all_roles() {
        // Fails at compile time if ALL grows without match arms; at runtime
        // ensures every role maps to a non-zero / known-ish value.
        for role in AccessibilityRole::ALL {
            let n = role_to_atspi_role(role);
            assert!(n > 0, "role {role:?} mapped to 0");
            assert!(!role.role_name().is_empty(), "empty role_name for {role:?}");
        }
        assert_eq!(AccessibilityRole::ALL.len(), 35);
    }

    #[test]
    fn role_names_for_chrome_and_core_widgets() {
        assert_eq!(AccessibilityRole::MenuBar.role_name(), "menu bar");
        assert_eq!(AccessibilityRole::Dock.role_name(), "tool bar");
        assert_eq!(AccessibilityRole::Button.role_name(), "push button");
        assert_eq!(AccessibilityRole::TextField.role_name(), "text");
        assert_eq!(AccessibilityRole::Window.role_name(), "frame");
        assert_eq!(AccessibilityRole::ListItem.role_name(), "list item");
        assert_eq!(AccessibilityRole::Desktop.role_name(), "desktop frame");
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
    fn state_bitset_chrome_targets_are_focusable() {
        let state = AccessibilityState::default();
        for role in [
            AccessibilityRole::MenuBar,
            AccessibilityRole::Desktop,
            AccessibilityRole::Dock,
        ] {
            let bits = state_to_atspi_bitset(&state, role);
            assert_ne!(
                bits[0] & (1 << 11),
                0,
                "{role:?} should set FOCUSABLE for chrome keyboard path"
            );
        }
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

    // ----- AccessibleAction -------------------------------------------------

    #[test]
    fn accessible_action_names_are_stable() {
        // Removing or renaming these breaks AT clients — fail hard.
        assert_eq!(AccessibleAction::Activate.name(), "Activate");
        assert_eq!(AccessibleAction::Press.name(), "Press");
        assert_eq!(AccessibleAction::Focus.name(), "Focus");
        assert_eq!(AccessibleAction::ALL.len(), 3);
        for a in AccessibleAction::ALL {
            assert_eq!(AccessibleAction::from_name(a.name()), Some(a));
            assert!(!a.description().is_empty());
        }
        assert_eq!(ACTION_ACTIVATE, "Activate");
        assert_eq!(ACTION_PRESS, "Press");
        assert_eq!(ACTION_FOCUS, "Focus");
    }

    #[test]
    fn actions_for_button_include_activate_press_focus() {
        let actions = actions_for_role(AccessibilityRole::Button);
        assert!(actions.contains(&AccessibleAction::Activate));
        assert!(actions.contains(&AccessibleAction::Press));
        assert!(actions.contains(&AccessibleAction::Focus));
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn actions_for_text_field_include_focus() {
        let actions = actions_for_role(AccessibilityRole::TextField);
        assert_eq!(actions, vec![AccessibleAction::Focus]);
    }

    #[test]
    fn actions_for_list_item_include_activate_and_focus() {
        let actions = actions_for_role(AccessibilityRole::ListItem);
        assert!(actions.contains(&AccessibleAction::Activate));
        assert!(actions.contains(&AccessibleAction::Focus));
        assert!(!actions.contains(&AccessibleAction::Press));
    }

    #[test]
    fn actions_for_chrome_regions_include_focus() {
        for role in [
            AccessibilityRole::MenuBar,
            AccessibilityRole::Desktop,
            AccessibilityRole::Dock,
        ] {
            let actions = actions_for_role(role);
            assert!(
                actions.contains(&AccessibleAction::Focus),
                "{role:?} must advertise Focus for keyboard-only chrome path"
            );
        }
    }

    #[test]
    fn actions_for_static_roles_are_empty() {
        assert!(actions_for_role(AccessibilityRole::Label).is_empty());
        assert!(actions_for_role(AccessibilityRole::StaticText).is_empty());
        assert!(actions_for_role(AccessibilityRole::Window).is_empty());
        assert!(actions_for_role(AccessibilityRole::Unknown).is_empty());
    }

    #[test]
    fn interfaces_for_role_advertise_action_when_present() {
        let button = interfaces_for_role(AccessibilityRole::Button);
        assert!(button.iter().any(|s| s == ATSPI_ACCESSIBLE_IFACE));
        assert!(button.iter().any(|s| s == ATSPI_ACTION_IFACE));

        let label = interfaces_for_role(AccessibilityRole::Label);
        assert!(label.iter().any(|s| s == ATSPI_ACCESSIBLE_IFACE));
        assert!(!label.iter().any(|s| s == ATSPI_ACTION_IFACE));
    }

    #[test]
    fn node_actions_delegate_to_role() {
        let node = AccessibilityNode::new(AccessibilityRole::Button, "OK");
        assert_eq!(node.actions(), actions_for_role(AccessibilityRole::Button));
    }

    // ----- Chrome keyboard policy ------------------------------------------

    #[test]
    fn chrome_focus_region_order_is_menu_desktop_dock() {
        assert_eq!(
            ChromeFocusRegion::ORDER,
            [
                ChromeFocusRegion::MenuBar,
                ChromeFocusRegion::DesktopIcons,
                ChromeFocusRegion::Dock,
            ]
        );
        assert_eq!(
            ChromeFocusRegion::MenuBar.next(),
            ChromeFocusRegion::DesktopIcons
        );
        assert_eq!(
            ChromeFocusRegion::DesktopIcons.next(),
            ChromeFocusRegion::Dock
        );
        assert_eq!(ChromeFocusRegion::Dock.next(), ChromeFocusRegion::MenuBar);
        assert_eq!(ChromeFocusRegion::MenuBar.prev(), ChromeFocusRegion::Dock);
    }

    #[test]
    fn next_chrome_focus_region_starts_at_menu_bar() {
        assert_eq!(
            next_chrome_focus_region(None),
            ChromeFocusRegion::MenuBar
        );
        assert_eq!(
            next_chrome_focus_region(Some(ChromeFocusRegion::MenuBar)),
            ChromeFocusRegion::DesktopIcons
        );
        assert_eq!(
            prev_chrome_focus_region(None),
            ChromeFocusRegion::Dock
        );
    }

    #[test]
    fn chrome_region_primary_roles() {
        assert_eq!(
            ChromeFocusRegion::MenuBar.primary_role(),
            AccessibilityRole::MenuBar
        );
        assert_eq!(
            ChromeFocusRegion::DesktopIcons.primary_role(),
            AccessibilityRole::Desktop
        );
        assert_eq!(
            ChromeFocusRegion::Dock.primary_role(),
            AccessibilityRole::Dock
        );
    }

    #[test]
    fn shell_chrome_tree_has_menu_desktop_dock_window() {
        let tree = shell_chrome_accessibility_tree("RetroShell");
        assert!(tree.len() >= 4);
        assert_eq!(tree.nodes()[0].role, AccessibilityRole::MenuBar);
        assert_eq!(tree.nodes()[1].role, AccessibilityRole::Desktop);
        assert_eq!(tree.nodes()[2].role, AccessibilityRole::Dock);
        assert_eq!(tree.nodes()[3].role, AccessibilityRole::Window);
        assert_eq!(tree.nodes()[3].label, "RetroShell");

        // Nested structure for AT depth
        assert!(!tree.nodes()[0].children.is_empty());
        assert!(tree.nodes()[1]
            .children
            .iter()
            .all(|c| c.role == AccessibilityRole::ListItem));
        assert!(tree.nodes()[2]
            .children
            .iter()
            .all(|c| c.role == AccessibilityRole::Button));
    }

    #[test]
    fn chrome_focus_indices_follow_cycle_order() {
        let tree = shell_chrome_accessibility_tree("RetroShell");
        let indices = chrome_focus_indices(&tree);
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0].0, ChromeFocusRegion::MenuBar);
        assert_eq!(indices[1].0, ChromeFocusRegion::DesktopIcons);
        assert_eq!(indices[2].0, ChromeFocusRegion::Dock);
        assert_eq!(indices[0].1, 0);
        assert_eq!(indices[1].1, 1);
        assert_eq!(indices[2].1, 2);
    }

    #[test]
    fn next_chrome_focus_index_cycles() {
        let tree = shell_chrome_accessibility_tree("RetroShell");
        assert_eq!(next_chrome_focus_index(&tree, None), Some(0));
        assert_eq!(next_chrome_focus_index(&tree, Some(0)), Some(1));
        assert_eq!(next_chrome_focus_index(&tree, Some(1)), Some(2));
        assert_eq!(next_chrome_focus_index(&tree, Some(2)), Some(0));
        // Unknown current → start of cycle
        assert_eq!(next_chrome_focus_index(&tree, Some(99)), Some(0));
    }

    #[test]
    fn next_chrome_focus_index_empty_tree() {
        let tree = AccessibilityTree::new();
        assert_eq!(next_chrome_focus_index(&tree, None), None);
    }

    #[test]
    fn focusable_indices_includes_chrome_and_interactive() {
        let mut tree = AccessibilityTree::new();
        tree.add(AccessibilityNode::new(AccessibilityRole::MenuBar, "MB"));
        tree.add(AccessibilityNode::new(AccessibilityRole::Label, "L"));
        tree.add(AccessibilityNode::new(AccessibilityRole::Button, "B"));
        let idx = focusable_indices(&tree);
        assert!(idx.contains(&0)); // chrome MenuBar
        assert!(!idx.contains(&1)); // Label
        assert!(idx.contains(&2)); // Button
    }

    // ----- In-process event bus / focus path hooks -------------------------

    #[test]
    fn accessible_event_kind_tags_are_stable() {
        assert_eq!(AccessibleEventKind::Focus.as_str(), "Focus");
        assert_eq!(AccessibleEventKind::StateChanged.as_str(), "StateChanged");
        assert_eq!(AccessibleEventKind::BoundsChanged.as_str(), "BoundsChanged");
        assert_eq!(
            AccessibleEventKind::ActiveDescendantChanged.as_str(),
            "ActiveDescendantChanged"
        );
        assert_eq!(AccessibleEventKind::ObjectCreated.as_str(), "ObjectCreated");
        assert_eq!(
            AccessibleEventKind::ObjectDestroyed.as_str(),
            "ObjectDestroyed"
        );
    }

    #[test]
    fn event_bus_push_pop_drain_fifo() {
        let mut bus = AccessibilityEventBus::new();
        assert!(bus.is_empty());
        bus.push(AccessibleEvent::object_created("/org/a11y/atspi/accessible/0"));
        bus.push(AccessibleEvent::object_destroyed("/org/a11y/atspi/accessible/1"));
        assert_eq!(bus.len(), 2);

        let first = bus.pop().expect("first event");
        assert_eq!(first.kind, AccessibleEventKind::ObjectCreated);
        assert_eq!(first.path, "/org/a11y/atspi/accessible/0");

        let rest = bus.drain();
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].kind, AccessibleEventKind::ObjectDestroyed);
        assert!(bus.is_empty());
        assert!(bus.pop().is_none());
    }

    #[test]
    fn free_focus_changed_helper_pushes_focus_event() {
        let mut bus = EventQueue::new();
        let path = atspi_object_path(0);
        focus_changed(&mut bus, &path);
        let events = bus.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, AccessibleEventKind::Focus);
        assert_eq!(events[0].path, path);
        assert_eq!(events[0].detail1, 1);
        assert_eq!(events[0].detail2, 0);
        assert!(events[0].any_data.is_empty());
    }

    #[test]
    fn bus_focus_changed_emits_on_focus_path_helper() {
        let mut bus = AccessibilityEventBus::new();
        let path = atspi_object_path(2);
        bus.focus_changed(&path);
        let e = bus.pop().expect("Focus event");
        assert_eq!(e, AccessibleEvent::focus(path));
    }

    #[test]
    fn tree_focus_changed_pushes_focus_and_updates_state() {
        let mut tree = AccessibilityTree::new();
        tree.add(AccessibilityNode::new(AccessibilityRole::Button, "A"));
        tree.add(AccessibilityNode::new(AccessibilityRole::Button, "B"));
        let mut bus = AccessibilityEventBus::new();

        let path0 = atspi_object_path(0);
        tree.focus_changed(&path0, &mut bus);

        assert!(tree.get(0).unwrap().state.focused);
        assert!(!tree.get(1).unwrap().state.focused);

        let events = bus.drain();
        // StateChanged(focused=true) for index 0 + Focus event
        assert!(
            events
                .iter()
                .any(|e| e.kind == AccessibleEventKind::Focus && e.path == path0),
            "expected Focus event for {path0}, got {events:?}"
        );
        assert!(
            events.iter().any(|e| {
                e.kind == AccessibleEventKind::StateChanged
                    && e.path == path0
                    && e.any_data == "focused"
                    && e.detail1 == 1
            }),
            "expected StateChanged focused=1 for path0, got {events:?}"
        );

        // Move focus to index 1 — previous loses focused, new gains it + Focus.
        let path1 = atspi_object_path(1);
        tree.focus_changed(&path1, &mut bus);
        assert!(!tree.get(0).unwrap().state.focused);
        assert!(tree.get(1).unwrap().state.focused);

        let events = bus.drain();
        assert!(events
            .iter()
            .any(|e| e.kind == AccessibleEventKind::Focus && e.path == path1));
        assert!(events.iter().any(|e| {
            e.kind == AccessibleEventKind::StateChanged
                && e.path == path0
                && e.any_data == "focused"
                && e.detail1 == 0
        }));
        assert!(events.iter().any(|e| {
            e.kind == AccessibleEventKind::StateChanged
                && e.path == path1
                && e.any_data == "focused"
                && e.detail1 == 1
        }));
    }

    #[test]
    fn tree_focus_changed_index_uses_canonical_path() {
        let mut tree = shell_chrome_accessibility_tree("RetroShell");
        let mut bus = AccessibilityEventBus::new();
        // Menu bar is index 0 in shell chrome tree.
        tree.focus_changed_index(0, &mut bus);
        assert!(tree.get(0).unwrap().state.focused);
        let events = bus.drain();
        assert!(events.iter().any(|e| {
            e.kind == AccessibleEventKind::Focus && e.path == atspi_object_path(0)
        }));
    }

    #[test]
    fn chrome_focus_path_emits_focus_events_for_cycle() {
        // Keyboard chrome path: next_chrome_focus_index + tree focus hooks.
        let mut tree = shell_chrome_accessibility_tree("RetroShell");
        let mut bus = AccessibilityEventBus::new();
        let mut current: Option<usize> = None;
        let mut focused_paths = Vec::new();

        for _ in 0..3 {
            let idx = next_chrome_focus_index(&tree, current).expect("chrome index");
            tree.focus_changed_index(idx, &mut bus);
            focused_paths.push(atspi_object_path(idx));
            current = Some(idx);
        }

        let events = bus.drain();
        let focus_paths: Vec<&str> = events
            .iter()
            .filter(|e| e.kind == AccessibleEventKind::Focus)
            .map(|e| e.path.as_str())
            .collect();
        assert_eq!(
            focus_paths,
            vec![
                focused_paths[0].as_str(),
                focused_paths[1].as_str(),
                focused_paths[2].as_str(),
            ]
        );
        // Cycle order: menu bar (0) → desktop (1) → dock (2)
        assert_eq!(focused_paths[0], atspi_object_path(0));
        assert_eq!(focused_paths[1], atspi_object_path(1));
        assert_eq!(focused_paths[2], atspi_object_path(2));
        assert!(tree.get(2).unwrap().state.focused);
        assert!(!tree.get(0).unwrap().state.focused);
    }

    #[test]
    fn flat_index_from_atspi_path_parses_canonical_and_nested() {
        assert_eq!(
            flat_index_from_atspi_path("/org/a11y/atspi/accessible/3"),
            Some(3)
        );
        assert_eq!(
            flat_index_from_atspi_path("/org/a11y/atspi/accessible/3/c0"),
            Some(3)
        );
        assert_eq!(
            flat_index_from_atspi_path("/org/a11y/atspi/accessible/root"),
            None
        );
        assert_eq!(flat_index_from_atspi_path("/other"), None);
    }

    #[test]
    fn other_event_helpers_push_expected_kinds() {
        let mut bus = AccessibilityEventBus::new();
        bus.bounds_changed("/p");
        bus.active_descendant_changed("/parent", "/child");
        bus.object_created("/new");
        bus.object_destroyed("/old");
        let kinds: Vec<_> = bus.drain().into_iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            vec![
                AccessibleEventKind::BoundsChanged,
                AccessibleEventKind::ActiveDescendantChanged,
                AccessibleEventKind::ObjectCreated,
                AccessibleEventKind::ObjectDestroyed,
            ]
        );
    }
}
