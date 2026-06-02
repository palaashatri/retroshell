use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::icon_view::IconView;
use retro_kit::layout::Layout;
use retro_kit::tree_view::{TreeNode, TreeView};
use retro_kit::window::Window;
use retro_kit::{Widget, WidgetState, LayoutConstraint, Size, Rect, Event, EventResult, ThemeContext, AccessibilityNode};
use retro_sdk::{build_menu, Application};
use std::path::PathBuf;

mod file_ops;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut app = Application::new("Finder", "com.retro.finder");

    let mut file_menu = build_menu("File");
    {
        let item = file_menu.add_action("New Finder Window");
        item.with_shortcut(
            KeyCode::N,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    {
        let item = file_menu.add_action("New Folder");
        item.with_shortcut(
            KeyCode::N,
            Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    file_menu.add_separator();
    {
        let item = file_menu.add_action("Open");
        item.with_shortcut(KeyCode::O, Modifiers::NONE);
    }
    {
        let item = file_menu.add_action("Close Window");
        item.with_shortcut(
            KeyCode::W,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    file_menu.add_separator();
    {
        let item = file_menu.add_action("Get Info");
        item.with_shortcut(
            KeyCode::I,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    file_menu.add_separator();
    {
        let item = file_menu.add_action("Move to Trash");
        item.with_shortcut(
            KeyCode::Backspace,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    file_menu.add_action("Empty Trash...");

    let mut edit_menu = build_menu("Edit");
    edit_menu.add_action("Undo");
    edit_menu.add_action("Redo");
    edit_menu.add_separator();
    edit_menu.add_action("Cut");
    edit_menu.add_action("Copy");
    edit_menu.add_action("Paste");
    edit_menu.add_separator();
    edit_menu.add_action("Select All");

    let mut view_menu = build_menu("View");
    view_menu.add_action("as Icons");
    view_menu.add_action("as List");
    view_menu.add_action("as Columns");
    view_menu.add_action("as Gallery");
    view_menu.add_separator();
    view_menu.add_action("Show Path Bar");
    view_menu.add_action("Show Status Bar");
    view_menu.add_action("Show Sidebar");
    view_menu.add_separator();
    {
        let item = view_menu.add_action("Enter Fullscreen");
        item.with_shortcut(
            KeyCode::F,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }

    let mut go_menu = build_menu("Go");
    go_menu.add_action("Back");
    go_menu.add_action("Forward");
    go_menu.add_separator();
    go_menu.add_action("Enclosing Folder");
    go_menu.add_separator();
    go_menu.add_action("Recent Folders");
    go_menu.add_action("Documents");
    go_menu.add_action("Desktop");
    go_menu.add_action("Downloads");
    go_menu.add_action("Home");
    go_menu.add_action("Applications");
    go_menu.add_action("Utilities");

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");
    window_menu.add_action("Zoom");
    window_menu.add_separator();
    window_menu.add_action("Show All");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("Finder Help");

    app.set_menus(vec![
        file_menu,
        edit_menu,
        view_menu,
        go_menu,
        window_menu,
        help_menu,
    ]);

    let finderview = FinderView::new();

    let mut window = Window::new("Finder");
    window.layout = Layout::vertical(0.0);
    window.set_content(Box::new(finderview));

    app.set_main_window(window);
    app.run();
}

pub struct FinderView {
    state: WidgetState,
    current_path: PathBuf,
    sidebar: TreeView,
    file_grid: IconView,
    last_selected_path: Option<Vec<usize>>,
}

impl Default for FinderView {
    fn default() -> Self {
        Self::new()
    }
}

impl FinderView {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let current_path = PathBuf::from(home);

        let mut sidebar = TreeView::new();
        let mut favorites = TreeNode::new("Favorites");
        favorites.children.push(TreeNode::new("AirDrop"));
        favorites.children.push(TreeNode::new("Recents"));
        favorites.children.push(TreeNode::new("Applications"));
        favorites.children.push(TreeNode::new("Desktop"));
        favorites.children.push(TreeNode::new("Documents"));
        favorites.children.push(TreeNode::new("Downloads"));
        favorites.expanded = true;
        let mut locations = TreeNode::new("Locations");
        locations.children.push(TreeNode::new("Macintosh HD"));
        locations.children.push(TreeNode::new("Network"));
        locations.expanded = true;
        sidebar.roots = vec![favorites, locations];

        let mut file_grid = IconView::new();
        file_grid.icon_size = 64.0;
        file_grid.spacing = 12.0;

        let mut view = FinderView {
            state: WidgetState::new(),
            current_path,
            sidebar,
            file_grid,
            last_selected_path: None,
        };
        view.reload_directory();
        view
    }

    pub fn reload_directory(&mut self) {
        self.file_grid.items.clear();
        if let Ok(entries) = file_ops::list_directory(&self.current_path) {
            for entry in entries {
                let icon = if entry.is_dir {
                    Some("folder".to_string())
                } else {
                    Some("document".to_string())
                };
                self.file_grid.items.push(retro_kit::icon_view::IconItem {
                    label: entry.name,
                    icon,
                    selected: false,
                    rect: Rect::ZERO,
                });
            }
        }
    }
}

impl Widget for FinderView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        let r = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(r);

        let sidebar_w = r.width * 0.25;
        let grid_w = r.width - sidebar_w;

        self.sidebar.set_rect(Rect::new(r.x, r.y, sidebar_w, r.height));
        let _ = self.sidebar.layout(LayoutConstraint::tight(Size::new(sidebar_w, r.height)));

        self.file_grid.set_rect(Rect::new(r.x + sidebar_w, r.y, grid_w, r.height));
        let _ = self.file_grid.layout(LayoutConstraint::tight(Size::new(grid_w, r.height)));

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.sidebar.draw(theme);
        self.file_grid.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::KeyDown { key, modifiers } = event {
            if modifiers.meta {
                match key {
                    KeyCode::N if modifiers.shift => {
                        let path = self.current_path.join("New Folder");
                        let _ = file_ops::create_directory(&path);
                        self.reload_directory();
                        return EventResult::Handled;
                    }
                    KeyCode::Backspace => {
                        let selected_item = self.file_grid.items.iter().find(|i| i.selected).cloned();
                        if let Some(item) = selected_item {
                            let path = self.current_path.join(&item.label);
                            let _ = file_ops::delete_file(&path);
                            self.reload_directory();
                        }
                        return EventResult::Handled;
                    }
                    KeyCode::D => {
                        let selected_item = self.file_grid.items.iter().find(|i| i.selected).cloned();
                        if let Some(item) = selected_item {
                            let path = self.current_path.join(&item.label);
                            let _ = file_ops::duplicate_file(&path);
                            self.reload_directory();
                        }
                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }
        }

        let mut res = self.sidebar.handle_event(event);
        if let EventResult::Ignored = res {
            res = self.file_grid.handle_event(event);
        }

        if let Event::DoubleClick { point, .. } = event {
            let mut folder_to_enter = None;
            for item in &self.file_grid.items {
                if item.rect.contains(*point) && item.icon == Some("folder".to_string()) {
                    folder_to_enter = Some(item.label.clone());
                    break;
                }
            }
            if let Some(folder) = folder_to_enter {
                self.current_path = self.current_path.join(folder);
                self.reload_directory();
                return EventResult::Handled;
            }
        }
        res
    }

    fn update(&mut self) {
        self.sidebar.update();
        self.file_grid.update();

        let sidebar_selected = self.sidebar.selected_path.clone();
        if sidebar_selected != self.last_selected_path {
            self.last_selected_path = sidebar_selected.clone();
            if let Some(ref selected) = sidebar_selected {
                if selected.len() > 1 && selected[0] == 0 {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    let path = match selected[1] {
                        3 => PathBuf::from(home).join("Desktop"),
                        4 => PathBuf::from(home).join("Documents"),
                        5 => PathBuf::from(home).join("Downloads"),
                        _ => PathBuf::from(home),
                    };
                    self.current_path = path;
                    self.reload_directory();
                }
            }
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&self.sidebar, &self.file_grid]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![&mut self.sidebar, &mut self.file_grid]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
