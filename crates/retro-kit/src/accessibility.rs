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
}
