use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::icon_view::IconView;
use retro_kit::layout::Layout;
use retro_kit::split_view::{SplitDirection, SplitView};
use retro_kit::tree_view::{TreeNode, TreeView};
use retro_kit::window::Window;
use retro_sdk::{build_menu, Application};

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

    let mut content_area = SplitView::new(SplitDirection::Horizontal);
    content_area.divider_position = 0.25;
    content_area.set_first(Box::new(sidebar));

    let mut window = Window::new("Finder");
    window.layout = Layout::vertical(0.0);

    app.set_main_window(window);
    app.run();
}
