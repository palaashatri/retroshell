use retro_kit::button::Button;
use retro_kit::label::Label;
use retro_kit::layout::Layout;
use retro_kit::split_view::{SplitDirection, SplitView};
use retro_kit::tree_view::{TreeNode, TreeView};
use retro_kit::window::Window;
use retro_sdk::{build_menu, Application};

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut app = Application::new("Settings", "com.retro.settings");

    let mut file_menu = build_menu("File");
    file_menu.add_action("Close");
    file_menu.add_separator();
    file_menu.add_action("Show All Settings");

    let mut edit_menu = build_menu("Edit");
    edit_menu.add_action("Undo");
    edit_menu.add_action("Redo");

    let mut view_menu = build_menu("View");
    view_menu.add_action("Show Search");

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("Settings Help");

    app.set_menus(vec![
        file_menu,
        edit_menu,
        view_menu,
        window_menu,
        help_menu,
    ]);

    let mut categories = TreeView::new();
    categories.roots = vec![
        TreeNode::new("General"),
        TreeNode::new("Appearance"),
        TreeNode::new("Desktop & Dock"),
        TreeNode::new("Display"),
        TreeNode::new("Sound"),
        TreeNode::new("Network"),
        TreeNode::new("Keyboard"),
        TreeNode::new("Mouse"),
        TreeNode::new("Accessibility"),
        TreeNode::new("Privacy & Security"),
        TreeNode::new("Notifications"),
    ];

    let mut appearance_content = Layout::vertical(12.0);
    appearance_content.add(Box::new(Label::new("Appearance Settings")));
    appearance_content.add(Box::new(Label::new("Theme:")));
    appearance_content.add(Box::new(Button::new("Platinum")));
    appearance_content.add(Box::new(Button::new("Graphite")));
    appearance_content.add(Box::new(Button::new("OLED Graphite")));
    appearance_content.add(Box::new(Button::new("High Contrast")));

    let mut split = SplitView::new(SplitDirection::Horizontal);
    split.divider_position = 0.25;
    split.set_first(Box::new(categories));

    let window = Window::new("Settings");
    app.set_main_window(window);
    app.run();
}
