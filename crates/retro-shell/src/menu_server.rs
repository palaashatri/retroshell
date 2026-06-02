use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::menu::Menu;

pub struct MenuServer {
    pub menus: Vec<Menu>,
    pub active_app: Option<String>,
    pub status_items: Vec<StatusItem>,
    pub keyboard_shortcuts: Vec<ShortcutBinding>,
}

pub struct StatusItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub priority: i32,
}

pub struct ShortcutBinding {
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub action_id: String,
    pub app_id: Option<String>,
}

impl Default for MenuServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuServer {
    pub fn new() -> Self {
        let mut server = Self {
            menus: vec![],
            active_app: None,
            status_items: vec![],
            keyboard_shortcuts: vec![],
        };
        server.setup_default_menus();
        server
    }

    fn setup_default_menus(&mut self) {
        let mut apple_menu = Menu::new("");
        apple_menu.add_action("About This Mac");
        apple_menu.add_separator();
        apple_menu.add_action("System Settings...");
        apple_menu.add_action("App Store...");
        apple_menu.add_separator();
        apple_menu.add_action("Recent Items").with_action("recent");
        apple_menu.add_separator();
        apple_menu.add_action("Force Quit...").with_shortcut(
            KeyCode::Escape,
            Modifiers {
                shift: false,
                control: false,
                alt: true,
                meta: true,
            },
        );
        apple_menu.add_separator();
        apple_menu.add_action("Sleep");
        apple_menu.add_action("Restart...");
        apple_menu.add_action("Shut Down...");
        apple_menu.add_separator();
        apple_menu.add_action("Lock Screen").with_shortcut(
            KeyCode::Q,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        apple_menu.add_action("Log Out...").with_shortcut(
            KeyCode::Q,
            Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        );

        let mut file_menu = Menu::new("File");
        file_menu.add_action("New").with_shortcut(
            KeyCode::N,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        file_menu.add_action("Open...").with_shortcut(
            KeyCode::O,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        file_menu.add_action("Close Window").with_shortcut(
            KeyCode::W,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        file_menu.add_action("Save").with_shortcut(
            KeyCode::S,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        file_menu.add_separator();
        file_menu.add_action("Print...").with_shortcut(
            KeyCode::P,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );

        let mut edit_menu = Menu::new("Edit");
        edit_menu.add_action("Undo").with_shortcut(
            KeyCode::Z,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        edit_menu.add_action("Redo").with_shortcut(
            KeyCode::Z,
            Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        );
        edit_menu.add_separator();
        edit_menu.add_action("Cut").with_shortcut(
            KeyCode::X,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        edit_menu.add_action("Copy").with_shortcut(
            KeyCode::C,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        edit_menu.add_action("Paste").with_shortcut(
            KeyCode::V,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        edit_menu.add_action("Select All").with_shortcut(
            KeyCode::A,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );

        let mut view_menu = Menu::new("View");
        view_menu.add_action("Show Toolbar");
        view_menu.add_action("Show Sidebar");
        view_menu.add_separator();
        view_menu.add_action("Enter Fullscreen").with_shortcut(
            KeyCode::F,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );

        let mut help_menu = Menu::new("Help");
        help_menu.add_action("Search");

        self.menus = vec![apple_menu, file_menu, edit_menu, view_menu, help_menu];
    }

    pub fn set_app_menus(&mut self, app_id: &str, menus: Vec<Menu>) {
        self.active_app = Some(app_id.to_string());
        while self.menus.len() > 1 {
            self.menus.pop();
        }
        for menu in menus {
            self.menus.push(menu);
        }
    }

    pub fn reset_to_shell_menus(&mut self) {
        self.active_app = None;
        self.menus.clear();
        self.setup_default_menus();
    }

    pub fn add_status_item(&mut self, item: StatusItem) {
        self.status_items.push(item);
        self.status_items
            .sort_by_key(|i| std::cmp::Reverse(i.priority));
    }

    pub fn register_shortcut(&mut self, binding: ShortcutBinding) {
        self.keyboard_shortcuts.push(binding);
    }

    pub fn lookup_shortcut(&self, key: KeyCode, modifiers: Modifiers) -> Option<&ShortcutBinding> {
        self.keyboard_shortcuts
            .iter()
            .find(|s| s.key == key && s.modifiers == modifiers)
    }

    pub fn render_menu_bar(&self) -> retro_render::RenderNode {
        let mut children = vec![];
        children.push(retro_render::RenderNode::Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 24.0,
            color: retro_render::Color::new(0.9, 0.9, 0.9, 1.0),
            corner_radius: 0.0,
        });

        let mut x = 10.0;
        for menu in &self.menus {
            children.push(retro_render::RenderNode::Text {
                x,
                y: 16.0,
                text: menu.title.clone(),
                font_size: 13.0,
                color: retro_render::Color::BLACK,
            });
            x += (menu.title.len() as f32) * 8.0 + 15.0;
        }

        retro_render::RenderNode::Group { children }
    }
}
