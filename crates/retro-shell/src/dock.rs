use retro_kit::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    NotRunning,
    Running,
    Focused,
    AttentionRequired,
}

#[derive(Debug, Clone)]
pub struct DockItem {
    pub app_id: String,
    pub label: String,
    pub icon: Option<String>,
    pub state: AppState,
    pub is_trash: bool,
    pub is_folder: bool,
    pub position: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPosition {
    Bottom,
    Left,
    Right,
}

pub struct Dock {
    pub items: Vec<DockItem>,
    pub position: DockPosition,
    pub auto_hide: bool,
    pub magnification: bool,
    pub icon_size: f32,
    pub running_apps: Vec<String>,
    pub trash_items: usize,
}

impl Dock {
    pub fn new() -> Self {
        let mut dock = Self {
            items: vec![],
            position: DockPosition::Bottom,
            auto_hide: false,
            magnification: false,
            icon_size: 48.0,
            running_apps: vec![],
            trash_items: 0,
        };
        dock.setup_default_items();
        dock
    }

    fn setup_default_items(&mut self) {
        self.items.push(DockItem {
            app_id: "com.retro.finder".into(),
            label: "Finder".into(),
            icon: Some("finder".into()),
            state: AppState::Focused,
            is_trash: false,
            is_folder: false,
            position: Rect::ZERO,
        });
        self.items.push(DockItem {
            app_id: "com.retro.settings".into(),
            label: "Settings".into(),
            icon: Some("settings".into()),
            state: AppState::NotRunning,
            is_trash: false,
            is_folder: false,
            position: Rect::ZERO,
        });
        self.items.push(DockItem {
            app_id: "com.retro.textedit".into(),
            label: "TextEdit".into(),
            icon: Some("textedit".into()),
            state: AppState::NotRunning,
            is_trash: false,
            is_folder: false,
            position: Rect::ZERO,
        });
        self.items.push(DockItem {
            app_id: "com.retro.terminal".into(),
            label: "Terminal".into(),
            icon: Some("terminal".into()),
            state: AppState::NotRunning,
            is_trash: false,
            is_folder: false,
            position: Rect::ZERO,
        });
    }

    pub fn add_item(&mut self, app_id: &str, label: &str) {
        self.items.push(DockItem {
            app_id: app_id.to_string(),
            label: label.to_string(),
            icon: None,
            state: AppState::NotRunning,
            is_trash: false,
            is_folder: false,
            position: Rect::ZERO,
        });
    }

    pub fn set_app_state(&mut self, app_id: &str, state: AppState) {
        if let Some(item) = self.items.iter_mut().find(|i| i.app_id == app_id) {
            item.state = state;
        }
        if state == AppState::Running || state == AppState::Focused {
            if !self.running_apps.contains(&app_id.to_string()) {
                self.running_apps.push(app_id.to_string());
            }
        }
    }

    pub fn launch_app(&mut self, index: usize) -> Option<String> {
        self.items.get(index).map(|item| item.app_id.clone())
    }

    pub fn layout_dock(&mut self, screen_width: f32, screen_height: f32) {
        let padding = 8.0;
        let item_spacing = 4.0;
        let dock_height = self.icon_size + padding * 2.0;
        let total_width = self.items.len() as f32 * (self.icon_size + item_spacing) + padding * 2.0;
        let start_x = (screen_width - total_width) / 2.0;
        let y = screen_height - dock_height;

        for (i, item) in self.items.iter_mut().enumerate() {
            item.position = Rect::new(
                start_x + padding + i as f32 * (self.icon_size + item_spacing),
                y + padding,
                self.icon_size,
                self.icon_size,
            );
        }
    }
}
