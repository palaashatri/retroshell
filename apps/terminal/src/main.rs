use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::window::Window;
use retro_sdk::{build_menu, Application};

mod pty;
mod tabs;
mod terminal;
mod vt_parser;

use tabs::TabManager;

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut app = Application::new("Terminal", "com.retro.terminal");

    let mut shell_menu = build_menu("Shell");
    {
        let item = shell_menu.add_action("New Window");
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
        let item = shell_menu.add_action("New Tab");
        item.with_shortcut(
            KeyCode::T,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    shell_menu.add_separator();
    {
        let item = shell_menu.add_action("Close Tab");
        item.with_shortcut(
            KeyCode::W,
            Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    {
        let item = shell_menu.add_action("Close Window");
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

    let mut edit_menu = build_menu("Edit");
    {
        let item = edit_menu.add_action("Copy");
        item.with_shortcut(
            KeyCode::C,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    {
        let item = edit_menu.add_action("Paste");
        item.with_shortcut(
            KeyCode::V,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    edit_menu.add_separator();
    {
        let item = edit_menu.add_action("Select All");
        item.with_shortcut(
            KeyCode::A,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }

    let mut view_menu = build_menu("View");
    view_menu.add_action("Zoom In");
    view_menu.add_action("Zoom Out");

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");
    window_menu.add_action("Zoom");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("Terminal Help");

    app.set_menus(vec![
        shell_menu,
        edit_menu,
        view_menu,
        window_menu,
        help_menu,
    ]);

    let mut tab_manager = TabManager::new();
    if let Err(e) = tab_manager.open_tab(80, 24) {
        tracing::error!("Failed to open initial tab: {}", e);
    }

    let mut window = Window::new("Terminal");
    window.set_content(Box::new(tab_manager));
    app.set_main_window(window);
    app.run();
}
