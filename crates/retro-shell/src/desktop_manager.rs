use retro_kit::{Rect, Point, Color};

#[derive(Debug, Clone)]
pub struct DesktopIcon {
    pub label: String,
    pub path: String,
    pub rect: Rect,
    pub selected: bool,
    pub icon_name: String,
}

pub struct DesktopManager {
    pub icons: Vec<DesktopIcon>,
    pub background_color: Color,
    pub background_image: Option<String>,
    pub show_volumes: bool,
    pub show_hard_disks: bool,
    pub selected_icons: Vec<usize>,
}

impl DesktopManager {
    pub fn new() -> Self {
        Self {
            icons: vec![],
            background_color: Color::new(0.2, 0.2, 0.4, 1.0),
            background_image: None,
            show_volumes: true,
            show_hard_disks: true,
            selected_icons: vec![],
        }
    }

    pub fn add_icon(&mut self, label: &str, path: &str, position: Point) {
        self.icons.push(DesktopIcon {
            label: label.to_string(),
            path: path.to_string(),
            rect: Rect::new(position.x, position.y, 80.0, 90.0),
            selected: false,
            icon_name: "generic".to_string(),
        });
    }

    pub fn select_icon(&mut self, index: usize) {
        self.selected_icons.push(index);
        if let Some(icon) = self.icons.get_mut(index) {
            icon.selected = true;
        }
    }

    pub fn deselect_all(&mut self) {
        for icon in &mut self.icons {
            icon.selected = false;
        }
        self.selected_icons.clear();
    }

    pub fn icon_at_point(&self, point: Point) -> Option<usize> {
        self.icons.iter().position(|icon| icon.rect.contains(point))
    }

    pub fn set_background(&mut self, color: Color) {
        self.background_color = color;
    }
}
