use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::scroll_view::ScrollView;
use retro_kit::text_field::TextField;
use retro_kit::toolbar::Toolbar;
use retro_kit::window::Window;
use retro_sdk::{build_menu, Application};

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let mut app = Application::new("TextEdit", "com.retro.textedit");

    let mut file_menu = build_menu("File");
    {
        let item = file_menu.add_action("New");
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
        let item = file_menu.add_action("Open...");
        item.with_shortcut(
            KeyCode::O,
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
        let item = file_menu.add_action("Save");
        item.with_shortcut(
            KeyCode::S,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    file_menu.add_action("Save As...");
    file_menu.add_separator();
    {
        let item = file_menu.add_action("Close");
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
        let item = edit_menu.add_action("Undo");
        item.with_shortcut(
            KeyCode::Z,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    {
        let item = edit_menu.add_action("Redo");
        item.with_shortcut(
            KeyCode::Z,
            Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
    edit_menu.add_separator();
    {
        let item = edit_menu.add_action("Cut");
        item.with_shortcut(
            KeyCode::X,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
    }
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
    edit_menu.add_separator();
    {
        let item = edit_menu.add_action("Find...");
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

    let mut format_menu = build_menu("Format");
    format_menu.add_action("Show Fonts");
    format_menu.add_action("Show Colors");
    format_menu.add_separator();
    format_menu.add_action("Bold");
    format_menu.add_action("Italic");
    format_menu.add_action("Underline");
    format_menu.add_separator();
    format_menu.add_action("Make Plain Text");

    let mut view_menu = build_menu("View");
    view_menu.add_action("Show Toolbar");
    view_menu.add_action("Show Ruler");
    view_menu.add_separator();
    view_menu.add_action("Enter Fullscreen");

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");
    window_menu.add_action("Zoom");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("TextEdit Help");

    app.set_menus(vec![
        file_menu,
        edit_menu,
        format_menu,
        view_menu,
        window_menu,
        help_menu,
    ]);

    let mut toolbar = Toolbar::new();
    toolbar.add(Box::new(Button::new("B")));
    toolbar.add(Box::new(Button::new("I")));
    toolbar.add(Box::new(Button::new("U")));

    let mut text_editor = TextField::new();
    text_editor.set_text("Untitled Document\n\nWelcome to TextEdit. Start typing...");

    let mut scroll = ScrollView::new();
    scroll.set_content(Box::new(text_editor));

    let window = Window::new("Untitled - TextEdit");
    app.set_main_window(window);
    app.run();
}
