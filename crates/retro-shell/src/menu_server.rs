use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::menu::{Menu, MenuItem, MenuItemKind};
use retro_sdk::MenuManifest;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct MenuServer {
    pub menus: Vec<Menu>,
    pub active_app: Option<String>,
    pub status_items: Vec<StatusItem>,
    pub keyboard_shortcuts: Vec<ShortcutBinding>,
    pub app_menus: HashMap<String, Vec<Menu>>,
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
            app_menus: HashMap::new(),
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
        system_menu
            .add_action("Recent Items")
            .with_action("shell.recent_items");
        system_menu
            .add_action("Notification Center...")
            .with_action("shell.notification_center");
        system_menu
            .add_action("Clear Notifications")
            .with_action("shell.clear_notifications");
        system_menu.add_separator();
        system_menu
            .add_action("Lock Screen")
            .with_action("shell.lock")
            .with_shortcut(
                KeyCode::L,
                Modifiers {
                    shift: false,
                    control: true,
                    alt: false,
                    meta: true,
                },
            );
        system_menu.add_separator();
        system_menu
            .add_action("Force Quit...")
            .with_action("shell.force_quit")
            .with_shortcut(
                KeyCode::Escape,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: true,
                    meta: true,
                },
            );
        system_menu.add_separator();
        system_menu
            .add_action("Quit RetroShell")
            .with_action("shell.quit")
            .with_shortcut(
                KeyCode::Q,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        system_menu
            .add_action("Log Out...")
            .with_action("shell.log_out")
            .with_shortcut(
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
        file_menu
            .add_action("Save")
            .with_action("shell.save")
            .with_shortcut(
                KeyCode::S,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        file_menu.add_separator();
        file_menu
            .add_action("Print...")
            .with_action("shell.print")
            .with_shortcut(
                KeyCode::P,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );

        let mut edit_menu = Menu::new("Edit");
        edit_menu
            .add_action("Undo")
            .with_action("shell.undo")
            .with_shortcut(
                KeyCode::Z,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        edit_menu
            .add_action("Redo")
            .with_action("shell.redo")
            .with_shortcut(
                KeyCode::Z,
                Modifiers {
                    shift: true,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        edit_menu.add_separator();
        edit_menu
            .add_action("Cut")
            .with_action("shell.cut")
            .with_shortcut(
                KeyCode::X,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        edit_menu
            .add_action("Copy")
            .with_action("shell.copy")
            .with_shortcut(
                KeyCode::C,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        edit_menu
            .add_action("Paste")
            .with_action("shell.paste")
            .with_shortcut(
                KeyCode::V,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        edit_menu
            .add_action("Select All")
            .with_action("shell.select_all")
            .with_shortcut(
                KeyCode::A,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );

        let mut view_menu = Menu::new("View");
        view_menu
            .add_action("Show Toolbar")
            .with_action("shell.show_toolbar");
        view_menu
            .add_action("Show Sidebar")
            .with_action("shell.show_sidebar");
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

        let window_menu = workspace_window_menu();

        let mut help_menu = Menu::new("Help");
        help_menu
            .add_action("Search")
            .with_action("shell.help_search");

        self.menus = vec![
            system_menu,
            file_menu,
            edit_menu,
            view_menu,
            window_menu,
            help_menu,
        ];
    }

    pub fn set_app_menus(&mut self, app_id: &str, menus: Vec<Menu>) {
        self.active_app = Some(app_id.to_string());
        while self.menus.len() > 1 {
            self.menus.pop();
        }
        let menus = ensure_workspace_window_menu(menus);
        for menu in menus {
            self.menus.push(menu);
        }
    }

    pub fn apply_menu_manifest(&mut self, manifest: MenuManifest) {
        let bundle_id = manifest.bundle_id;
        let menus = manifest.menus;
        self.app_menus.insert(bundle_id.clone(), menus.clone());
        if self.active_app.as_deref() == Some(bundle_id.as_str()) {
            self.set_app_menus(&bundle_id, menus);
        }
    }

    pub fn load_menu_manifest<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        let manifest: MenuManifest =
            serde_json::from_str(&content).map_err(std::io::Error::other)?;
        self.apply_menu_manifest(manifest);
        Ok(())
    }

    pub fn load_menu_manifests_from_dir<P: AsRef<Path>>(
        &mut self,
        dir: P,
    ) -> std::io::Result<usize> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(0);
        }

        let mut loaded = 0;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            self.load_menu_manifest(&path)?;
            loaded += 1;
        }
        Ok(loaded)
    }

    pub fn reset_to_shell_menus(&mut self) {
        self.active_app = None;
        self.menus.clear();
        self.setup_default_menus();
    }

    pub fn set_active_app_menus(&mut self, app_id: &str) {
        if let Some(menus) = self.app_menus.get(app_id).cloned() {
            self.set_app_menus(app_id, menus);
            return;
        }

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
            .add_action("New Folder")
            .with_action("finder.new_folder")
            .with_shortcut(
                KeyCode::N,
                Modifiers {
                    shift: true,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
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
        file_menu.add_separator();
        file_menu
            .add_action("Get Info")
            .with_action("finder.get_info")
            .with_shortcut(
                KeyCode::I,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        file_menu
            .add_action("Rename...")
            .with_action("finder.rename");
        file_menu
            .add_action("Move to Trash")
            .with_action("finder.move_to_trash")
            .with_shortcut(
                KeyCode::Delete,
                Modifiers {
                    shift: false,
                    control: false,
                    alt: false,
                    meta: true,
                },
            );
        file_menu.add_separator();
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

    pub fn action_for_shortcut(&self, key: KeyCode, modifiers: Modifiers) -> Option<String> {
        self.menus
            .iter()
            .find_map(|menu| find_shortcut_action(&menu.items, key, modifiers))
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

fn ensure_workspace_window_menu(mut menus: Vec<Menu>) -> Vec<Menu> {
    if let Some(window_menu) = menus.iter_mut().find(|menu| menu.title == "Window") {
        if !window_menu
            .items
            .iter()
            .any(|item| item.action_id == "workspace.next")
        {
            window_menu.add_separator();
            append_workspace_items(window_menu);
        }
    } else {
        menus.push(workspace_window_menu());
    }
    menus
}

fn workspace_window_menu() -> Menu {
    let mut window_menu = Menu::new("Window");
    append_workspace_items(&mut window_menu);
    window_menu
}

fn append_workspace_items(window_menu: &mut Menu) {
    window_menu
        .add_action("Previous Workspace")
        .with_action("workspace.previous")
        .with_shortcut(
            KeyCode::ArrowLeft,
            Modifiers {
                shift: false,
                control: true,
                alt: true,
                meta: false,
            },
        );
    window_menu
        .add_action("Next Workspace")
        .with_action("workspace.next")
        .with_shortcut(
            KeyCode::ArrowRight,
            Modifiers {
                shift: false,
                control: true,
                alt: true,
                meta: false,
            },
        );
    window_menu.add_separator();
    for index in 0..4 {
        let key = match index {
            0 => KeyCode::Key1,
            1 => KeyCode::Key2,
            2 => KeyCode::Key3,
            _ => KeyCode::Key4,
        };
        let action = format!("workspace.switch.{}", index);
        window_menu
            .add_action(format!("Desktop {}", index + 1))
            .with_action(&action)
            .with_shortcut(
                key,
                Modifiers {
                    shift: false,
                    control: true,
                    alt: true,
                    meta: false,
                },
            );
    }
}

fn find_shortcut_action(items: &[MenuItem], key: KeyCode, modifiers: Modifiers) -> Option<String> {
    for item in items {
        if !item.enabled {
            continue;
        }
        if item.shortcut == Some((key, modifiers)) && !item.action_id.is_empty() {
            return Some(item.action_id.clone());
        }
        if matches!(item.kind, MenuItemKind::Submenu) {
            if let Some(submenu) = &item.submenu {
                if let Some(action) = find_shortcut_action(&submenu.items, key, modifiers) {
                    return Some(action);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use retro_kit::menu::Menu;
    use retro_sdk::MenuManifest;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn menu_server_loads_sdk_menu_manifest() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("retroshell_menu_manifest_{unique}.json"));

        let mut file_menu = Menu::new("File");
        file_menu.add_action("New").with_action("com.test.app.new");
        let manifest = MenuManifest {
            app_name: "TestApp".to_string(),
            bundle_id: "com.test.app".to_string(),
            menus: vec![file_menu],
            updated_at_millis: 1,
        };
        fs::write(&path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let mut server = MenuServer::new();
        server.load_menu_manifest(&path).unwrap();

        assert_eq!(server.active_app, None);
        assert!(server.app_menus.contains_key("com.test.app"));

        server.set_active_app_menus("com.test.app");

        assert_eq!(server.active_app.as_deref(), Some("com.test.app"));
        assert!(server.menus.iter().any(|menu| menu.title == "File"));
        assert!(server.menus.iter().any(|menu| {
            menu.items
                .iter()
                .any(|item| item.action_id == "com.test.app.new")
        }));

        let window_menu = server
            .menus
            .iter()
            .find(|menu| menu.title == "Window")
            .expect("workspace window menu");
        assert!(window_menu
            .items
            .iter()
            .any(|item| item.action_id == "workspace.next"));
        assert!(window_menu
            .items
            .iter()
            .any(|item| item.action_id == "workspace.switch.0"));
        let _ = fs::remove_file(path);
    }
}
