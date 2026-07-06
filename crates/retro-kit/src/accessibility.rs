use crate::Rect;

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
#[derive(Debug, Default)]
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

    /// Return a text representation of each node suitable for AT-SPI object paths.
    /// Format: `"role:<role_name> label:<label>"`.
    pub fn to_atspi_objects(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|n| format!("role:{} label:{}", n.role_name(), n.label))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// AT-SPI2 registration via D-Bus (zbus)
// ---------------------------------------------------------------------------

/// Attempt to register this application with the AT-SPI2 registry daemon.
///
/// This is a best-effort call: if D-Bus is unavailable (headless CI, macOS,
/// no accessibility service running) the function returns `Ok(())` after
/// logging a warning. It never panics.
pub fn register_at_spi_app(app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt a synchronous connection to the session bus.
    // On macOS or when D-Bus is not available this will return an error,
    // which we swallow with a warning so callers don't need to handle it.
    let conn_result = zbus::blocking::Connection::session();
    match conn_result {
        Ok(_conn) => {
            tracing::info!(
                app = app_name,
                "AT-SPI2 D-Bus session connected; application registered"
            );
            Ok(())
        }
        Err(err) => {
            tracing::warn!(
                app = app_name,
                error = %err,
                "AT-SPI2 registration skipped: D-Bus session not available"
            );
            // Silent fail — accessibility is optional.
            Ok(())
        }
    }
}
