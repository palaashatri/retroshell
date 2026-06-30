pub mod accessibility;
pub mod button;
pub mod clipboard;
pub mod dialog;
pub mod dnd;
pub mod event;
pub mod icon_view;
pub mod label;
pub mod layout;
pub mod list_view;
pub mod menu;
pub mod popup_button;
pub mod progress_bar;
pub mod scroll_view;
pub mod slider;
pub mod split_view;
pub mod status_bar;
pub mod tab_view;
pub mod text_field;
pub mod theme;
pub mod toolbar;
pub mod tree_view;
pub mod widget;
pub mod window;

pub use accessibility::{AccessibilityNode, AccessibilityRole, AccessibilityState};
pub use button::Button;
pub use clipboard::Clipboard;
pub use dialog::Dialog;
pub use dnd::{DragData, DragSession, DragSource, DropTarget};
pub use event::{Event, EventHandler, EventResult};
pub use icon_view::IconView;
pub use label::Label;
pub use layout::{Layout, LayoutConstraint, LayoutHints, LayoutView};
pub use list_view::ListView;
pub use menu::{Menu, MenuItem};
pub use popup_button::PopupButton;
pub use progress_bar::ProgressBar;
pub use retro_render::Color;
pub use scroll_view::ScrollView;
pub use slider::Slider;
pub use split_view::SplitView;
pub use status_bar::{StatusBar, StatusBarAlignment, StatusBarItem};
pub use tab_view::{Tab, TabView};
pub use text_field::TextField;
pub use theme::{ThemeContext, ThemeToken, ThemeValue};
pub use toolbar::Toolbar;
pub use tree_view::TreeView;
pub use widget::{Widget, WidgetId, WidgetState};
pub use window::Window;

pub type Result<T> = std::result::Result<T, KitError>;

#[derive(Debug)]
pub enum KitError {
    WidgetNotFound(WidgetId),
    Layout(String),
    Theme(String),
}

impl std::fmt::Display for KitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KitError::WidgetNotFound(id) => write!(f, "widget not found: {}", id),
            KitError::Layout(msg) => write!(f, "layout error: {}", msg),
            KitError::Theme(msg) => write!(f, "theme error: {}", msg),
        }
    }
}

impl std::error::Error for KitError {}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    NotAllowed,
    ResizeHorizontal,
    ResizeVertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapsed,
}
