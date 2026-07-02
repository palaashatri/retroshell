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
        let mut system_menu = Menu::new("Retro");
        system_menu
            .add_action("About RetroShell")
            .with_action("shell.about");
        system_menu.add_separator();
        system_menu
            .add_action("System Settings...")
            .with_action("shell.settings");
        system_menu
            .add_action("Software Catalog...")
            .with_action("shell.software_catalog");
        system_menu.add_separator();
        system_menu.add_action("Recent Items").with_action("recent");
        system_menu.add_separator();
        system_menu.add_action("Force Quit...").with_shortcut(
            KeyCode::Escape,
            Modifiers {
                shift: false,
                control: false,
                alt: true,
                meta: true,
            },
        );
        system_menu.add_separator();
        system_menu.add_action("Sleep");
        system_menu.add_action("Restart...");
        system_menu.add_action("Shut Down...");
        system_menu.add_separator();
        system_menu.add_action("Lock Screen").with_shortcut(
            KeyCode::Q,
            Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        );
        system_menu.add_action("Log Out...").with_shortcut(
            KeyCode::Q,
            Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        );

        let mut file_menu = Menu::new("File");
        file_menu
            .add_action("New")
            .with_action("shell.new_finder_window")
            .with_shortcut(
                KeyCode::N,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        file_menu
            .add_action("Open...")
            .with_action("shell.open_finder")
            .with_shortcut(
                KeyCode::O,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        file_menu
            .add_action("Close Window")
            .with_action("shell.close_finder_window")
            .with_shortcut(
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
        view_menu
            .add_action("Enter Fullscreen")
            .with_action("shell.toggle_fullscreen")
            .with_shortcut(
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

        self.menus = vec![system_menu, file_menu, edit_menu, view_menu, help_menu];
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

    pub fn set_active_app_menus(&mut self, app_id: &str) {
        let title = match app_id {
            "com.retro.finder" => "Finder",
            "com.retro.textedit" => "TextEdit",
            "com.retro.terminal" => "Terminal",
            "com.retro.settings" => "Settings",
            "com.retro.appstore" => "App Store",
            _ => "Application",
        };

        let mut app_menu = Menu::new(title);
        let about_action = format!("{app_id}.about");
        let hide_action = format!("{app_id}.hide");
        let quit_action = format!("{app_id}.quit");
        app_menu
            .add_action(format!("About {title}"))
            .with_action(&about_action);
        app_menu.add_separator();
        app_menu
            .add_action(format!("Hide {title}"))
            .with_action(&hide_action);
        app_menu.add_separator();
        app_menu
            .add_action(format!("Quit {title}"))
            .with_action(&quit_action);

        let mut file_menu = Menu::new("File");
        file_menu
            .add_action("New Window")
            .with_action("shell.new_finder_window")
            .with_shortcut(
                KeyCode::N,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        file_menu
            .add_action("Close Window")
            .with_action("shell.close_finder_window")
            .with_shortcut(
                KeyCode::W,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );

        let mut edit_menu = Menu::new("Edit");
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
        view_menu
            .add_action("Enter Fullscreen")
            .with_action("shell.toggle_fullscreen")
            .with_shortcut(
                KeyCode::F,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );

        let mut window_menu = Menu::new("Window");
        window_menu.add_action("Minimize");
        window_menu
            .add_action("Zoom")
            .with_action("shell.zoom_window");

        let mut help_menu = Menu::new("Help");
        help_menu.add_action(format!("{title} Help"));

        let mut menus = vec![app_menu, file_menu, edit_menu, view_menu];
        if app_id == "com.retro.finder" {
            let mut go_menu = Menu::new("Go");
            go_menu.add_action("Home").with_action("shell.open_home");
            go_menu
                .add_action("Computer")
                .with_action("shell.open_computer");
            menus.push(go_menu);
        }
        menus.push(window_menu);
        menus.push(help_menu);

        self.set_app_menus(app_id, menus);
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
