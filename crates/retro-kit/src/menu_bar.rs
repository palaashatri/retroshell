use crate::{
    event::MouseButton,
    menu::{Menu, MenuItemKind},
    theme::ThemeContext,
    AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, Rect, Size, Widget,
    WidgetState,
};
use std::any::Any;

pub struct MenuBar {
    state: WidgetState,
    pub menus: Vec<Menu>,
    pub open_menu: Option<usize>,
    pub hovered_menu: Option<usize>,
    pub hovered_item: Option<usize>,
    pub last_action: Option<String>,
    menu_rects: Vec<Rect>,
}

impl MenuBar {
    pub fn new(menus: Vec<Menu>) -> Self {
        Self {
            state: WidgetState::new(),
            menus,
            open_menu: None,
            hovered_menu: None,
            hovered_item: None,
            last_action: None,
            menu_rects: vec![],
        }
    }

    pub fn menu_rects(&self) -> &[Rect] {
        &self.menu_rects
    }

    pub fn open_menu(&mut self, index: usize) {
        if index < self.menus.len() {
            self.open_menu = Some(index);
            self.hovered_menu = Some(index);
            self.hovered_item = None;
        }
    }

    pub fn close(&mut self) {
        self.open_menu = None;
        self.hovered_item = None;
    }

    pub fn dropdown_rect(&self, index: usize) -> Option<Rect> {
        let menu = self.menus.get(index)?;
        let menu_rect = *self.menu_rects.get(index)?;
        let item_width = menu
            .items
            .iter()
            .filter(|item| !matches!(item.kind, MenuItemKind::Separator))
            .map(|item| {
                let shortcut_width = item
                    .shortcut
                    .map(|(key, modifiers)| shortcut_len(key, modifiers) as f32 * 7.0 + 18.0)
                    .unwrap_or(0.0);
                item.label.len() as f32 * 7.0 + shortcut_width + 44.0
            })
            .fold(180.0, f32::max);
        Some(Rect::new(
            menu_rect.x,
            self.rect().y + self.rect().height - 1.0,
            item_width,
            menu.items.len() as f32 * 20.0 + 8.0,
        ))
    }

    pub fn item_rect(&self, menu_index: usize, item_index: usize) -> Option<Rect> {
        let dropdown = self.dropdown_rect(menu_index)?;
        Some(Rect::new(
            dropdown.x + 4.0,
            dropdown.y + 4.0 + item_index as f32 * 20.0,
            dropdown.width - 8.0,
            20.0,
        ))
    }

    fn menu_at_point(&self, point: crate::Point) -> Option<usize> {
        self.menu_rects
            .iter()
            .position(|menu_rect| menu_rect.contains(point))
    }

    fn item_at_point(&self, point: crate::Point) -> Option<(usize, usize)> {
        let menu_index = self.open_menu?;
        let menu = self.menus.get(menu_index)?;
        menu.items.iter().enumerate().find_map(|(item_index, _)| {
            self.item_rect(menu_index, item_index)
                .filter(|rect| rect.contains(point))
                .map(|_| (menu_index, item_index))
        })
    }
}

fn shortcut_len(key: crate::event::KeyCode, modifiers: crate::event::Modifiers) -> usize {
    let mut len = key_label(key).len();
    if modifiers.control {
        len += 5;
    }
    if modifiers.alt {
        len += 4;
    }
    if modifiers.shift {
        len += 6;
    }
    if modifiers.meta {
        len += 4;
    }
    len
}

fn key_label(key: crate::event::KeyCode) -> &'static str {
    match key {
        crate::event::KeyCode::Backspace => "Del",
        crate::event::KeyCode::Escape => "Esc",
        crate::event::KeyCode::Enter => "Ret",
        crate::event::KeyCode::Space => "Space",
        crate::event::KeyCode::ArrowUp => "Up",
        crate::event::KeyCode::ArrowDown => "Down",
        crate::event::KeyCode::ArrowLeft => "Left",
        crate::event::KeyCode::ArrowRight => "Right",
        _ => "Key",
    }
}

impl Widget for MenuBar {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, 24.0));
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));

        self.menu_rects.clear();
        let mut x = self.rect().x + 8.0;
        for menu in &self.menus {
            let width = menu.title.len() as f32 * 7.0 + 18.0;
            self.menu_rects
                .push(Rect::new(x, self.rect().y, width, size.height));
            x += width;
        }

        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseMove { point, .. } => {
                self.hovered_menu = self.menu_at_point(*point);
                if self.open_menu.is_some() {
                    if let Some(menu_index) = self.hovered_menu {
                        self.open_menu = Some(menu_index);
                        self.hovered_item = None;
                    } else {
                        self.hovered_item =
                            self.item_at_point(*point).map(|(_, item_index)| item_index);
                    }
                }
                EventResult::RequestRedraw
            }
            Event::MouseDown {
                button: MouseButton::Left,
                point,
                ..
            } => {
                if let Some(menu_index) = self.menu_at_point(*point) {
                    if self.open_menu == Some(menu_index) {
                        self.close();
                    } else {
                        self.open_menu(menu_index);
                    }
                    return EventResult::Handled;
                }

                if let Some((menu_index, item_index)) = self.item_at_point(*point) {
                    if let Some(item) = self.menus[menu_index].items.get(item_index) {
                        if !matches!(item.kind, MenuItemKind::Separator) && item.enabled {
                            self.last_action = Some(if item.action_id.is_empty() {
                                item.label.clone()
                            } else {
                                item.action_id.clone()
                            });
                        }
                    }
                    self.close();
                    return EventResult::Handled;
                }

                if self.open_menu.is_some() {
                    self.close();
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            Event::MouseLeave => {
                self.hovered_menu = None;
                self.hovered_item = None;
                EventResult::RequestRedraw
            }
            _ => EventResult::Ignored,
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(
            AccessibilityRole::MenuBar,
            "menu bar",
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{event::Modifiers, Point};

    fn test_menu_bar() -> MenuBar {
        let mut file = Menu::new("File");
        file.add_action("Open").with_action("open");
        file.add_separator();
        file.add_action("Close").with_action("close");

        let mut edit = Menu::new("Edit");
        edit.add_action("Copy").with_action("copy");

        MenuBar::new(vec![file, edit])
    }

    #[test]
    fn menu_bar_opens_switches_and_closes() {
        let mut menu_bar = test_menu_bar();
        menu_bar.layout(LayoutConstraint::tight(Size::new(640.0, 24.0)));

        let file_point = Point::new(16.0, 10.0);
        let edit_point = Point::new(menu_bar.menu_rects()[1].x + 6.0, 10.0);

        assert!(matches!(
            menu_bar.handle_event(&Event::MouseDown {
                button: MouseButton::Left,
                point: file_point,
                modifiers: Modifiers::NONE,
            }),
            EventResult::Handled
        ));
        assert_eq!(menu_bar.open_menu, Some(0));

        let _ = menu_bar.handle_event(&Event::MouseMove {
            point: edit_point,
            modifiers: Modifiers::NONE,
        });
        assert_eq!(menu_bar.open_menu, Some(1));

        let _ = menu_bar.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: edit_point,
            modifiers: Modifiers::NONE,
        });
        assert_eq!(menu_bar.open_menu, None);
    }

    #[test]
    fn menu_bar_records_action_from_dropdown_click() {
        let mut menu_bar = test_menu_bar();
        menu_bar.layout(LayoutConstraint::tight(Size::new(640.0, 24.0)));
        menu_bar.open_menu(0);

        let open_rect = menu_bar.item_rect(0, 0).unwrap();
        let _ = menu_bar.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(open_rect.x + 8.0, open_rect.y + 8.0),
            modifiers: Modifiers::NONE,
        });

        assert_eq!(menu_bar.last_action.as_deref(), Some("open"));
        assert_eq!(menu_bar.open_menu, None);
    }
}
