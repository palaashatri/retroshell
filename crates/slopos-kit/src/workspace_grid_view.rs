use crate::{
    event::{KeyCode, MouseButton},
    theme::ThemeContext,
    AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, Point, Rect, Size,
    Visibility, Widget, WidgetState,
};

pub struct WorkspaceGridView {
    state: WidgetState,
    pub active_index: usize,
    /// Cell that receives keyboard navigation and Enter/Space activation.
    /// This is intentionally independent from `active_index`: the latter is
    /// the compositor-authoritative Space, while this field is local pending
    /// keyboard focus until a selection is committed.
    pub focused_index: usize,
    pub items: Vec<String>,
    /// Number of ordinary windows currently assigned to each item.
    ///
    /// The shell fills this from the compositor's Spaces snapshot.  Keeping
    /// counts parallel to `items` lets the renderer show live membership
    /// without making the toolkit invent window records or geometry.
    pub window_counts: Vec<usize>,
    /// Cell pressed most recently, drained by [`WorkspaceGridView::take_activated`].
    activated: Option<usize>,
}

/// Cell geometry constants shared by the SDK painter and `handle_event`'s
/// hit-testing — one source of truth so a click always lands on the cell the
/// user sees.
pub const GRID_MARGIN: f32 = 8.0;
pub const GRID_GUTTER: f32 = 6.0;
pub const GRID_COLS: usize = 2;
pub const GRID_ROWS: usize = 2;
const GRID_MIN_CELL_HEIGHT: f32 = 34.0;

impl Default for WorkspaceGridView {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceGridView {
    pub fn new() -> Self {
        Self {
            state: WidgetState::new(),
            active_index: 0,
            focused_index: 0,
            items: Vec::new(),
            window_counts: Vec::new(),
            activated: None,
        }
    }

    /// Number of rows required to display all current items in the two-column
    /// overview.  Empty grids have no rows and therefore no hit targets.
    pub fn rows(&self) -> usize {
        self.items.len().div_ceil(GRID_COLS)
    }

    /// Screen rect of grid cell `index` (row-major over the dynamic two-column
    /// grid), given the widget's current rect.
    pub fn cell_rect(&self, index: usize) -> Rect {
        if index >= self.items.len() {
            return Rect::ZERO;
        }
        let r = self.rect();
        let grid = Rect::new(
            r.x + GRID_MARGIN,
            r.y + GRID_MARGIN,
            (r.width - GRID_MARGIN * 2.0).max(0.0),
            (r.height - GRID_MARGIN * 2.0).max(0.0),
        );
        let cell_w = (grid.width - GRID_GUTTER) / GRID_COLS as f32;
        let rows = self.rows();
        let cell_h = if rows == 0 {
            0.0
        } else {
            (grid.height - GRID_GUTTER * rows.saturating_sub(1) as f32) / rows as f32
        };
        let row = index / GRID_COLS;
        let col = index % GRID_COLS;
        Rect::new(
            grid.x + col as f32 * (cell_w + GRID_GUTTER),
            grid.y + row as f32 * (cell_h + GRID_GUTTER),
            cell_w,
            cell_h,
        )
    }

    /// Cell containing `point`, if any.
    pub fn cell_at(&self, point: Point) -> Option<usize> {
        (0..self.items.len()).find(|&i| self.cell_rect(i).contains(point))
    }

    /// Index of the most recently pressed cell; drains exactly once.
    pub fn take_activated(&mut self) -> Option<usize> {
        self.activated.take()
    }

    /// Keep keyboard focus valid after the shell replaces the live Space
    /// snapshot while this overview is open.
    pub fn normalize_focus(&mut self) {
        if self.items.is_empty() {
            self.focused_index = 0;
        } else if self.focused_index >= self.items.len() {
            self.focused_index = self.active_index.min(self.items.len().saturating_sub(1));
        }
    }

    fn move_focus(&mut self, key: KeyCode) -> bool {
        self.normalize_focus();
        let len = self.items.len();
        if len == 0 {
            return false;
        }

        let current = self.focused_index;
        let next = match key {
            KeyCode::ArrowLeft if !current.is_multiple_of(GRID_COLS) => Some(current - 1),
            KeyCode::ArrowRight if current % GRID_COLS + 1 < GRID_COLS && current + 1 < len => {
                Some(current + 1)
            }
            KeyCode::ArrowUp if current >= GRID_COLS => Some(current - GRID_COLS),
            KeyCode::ArrowDown if current + GRID_COLS < len => Some(current + GRID_COLS),
            _ => None,
        };
        if let Some(next) = next {
            self.focused_index = next;
        }
        true
    }
}

impl Widget for WorkspaceGridView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn focusable(&self) -> bool {
        self.state.enabled && self.state.visibility == Visibility::Visible
    }

    fn wants_click_focus(&self) -> bool {
        self.focusable()
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let rows = self.rows().max(1);
        let dynamic_height = GRID_MARGIN * 2.0
            + rows as f32 * GRID_MIN_CELL_HEIGHT
            + GRID_GUTTER * rows.saturating_sub(1) as f32;
        let size = constraint.clamp(Size::new(240.0, 160.0_f32.max(dynamic_height)));
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::FocusIn => {
                self.state.focused = true;
                self.normalize_focus();
                EventResult::Handled
            }
            Event::FocusOut => {
                self.state.focused = false;
                EventResult::Handled
            }
            Event::MouseDown {
                button: MouseButton::Left,
                point,
                ..
            } => {
                if !self.rect().contains(*point) {
                    return EventResult::Ignored;
                }
                match self.cell_at(*point) {
                    Some(cell) => {
                        self.state.focused = true;
                        self.focused_index = cell;
                        self.activated = Some(cell);
                        EventResult::Handled
                    }
                    None => EventResult::Ignored,
                }
            }
            Event::KeyDown { key, modifiers }
                if !modifiers.meta && !modifiers.control && !modifiers.alt =>
            {
                if !self.state.focused {
                    return EventResult::Ignored;
                }
                match key {
                    KeyCode::ArrowLeft
                    | KeyCode::ArrowRight
                    | KeyCode::ArrowUp
                    | KeyCode::ArrowDown => {
                        self.move_focus(*key);
                        EventResult::Handled
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        self.normalize_focus();
                        if self.focused_index < self.items.len() {
                            self.activated = Some(self.focused_index);
                        }
                        EventResult::Handled
                    }
                    _ => EventResult::Ignored,
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(
            AccessibilityRole::Group,
            "Workspace Grid",
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Modifiers;

    fn grid() -> WorkspaceGridView {
        let mut g = WorkspaceGridView::new();
        g.items = (0..4).map(|i| format!("Desktop {}", i + 1)).collect();
        g.window_counts = vec![0; 4];
        g.set_rect(Rect::new(100.0, 100.0, 240.0, 160.0));
        g
    }

    fn press(x: f32, y: f32) -> Event {
        Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(x, y),
            modifiers: Modifiers::NONE,
        }
    }

    fn key(key: KeyCode) -> Event {
        Event::KeyDown {
            key,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn each_cell_center_activates_that_cell() {
        for i in 0..4 {
            let mut g = grid();
            let c = g.cell_rect(i);
            let result = g.handle_event(&press(c.x + c.width * 0.5, c.y + c.height * 0.5));
            assert!(matches!(result, EventResult::Handled), "cell {i}");
            assert_eq!(g.take_activated(), Some(i));
            assert_eq!(g.take_activated(), None, "drains exactly once");
        }
    }

    #[test]
    fn press_in_the_margin_is_ignored() {
        let mut g = grid();
        // Inside the widget rect but within the 8px outer margin.
        let result = g.handle_event(&press(102.0, 102.0));
        assert!(matches!(result, EventResult::Ignored));
        assert_eq!(g.take_activated(), None);
    }

    #[test]
    fn press_outside_the_widget_is_ignored() {
        let mut g = grid();
        let result = g.handle_event(&press(10.0, 10.0));
        assert!(matches!(result, EventResult::Ignored));
        assert_eq!(g.take_activated(), None);
    }

    #[test]
    fn cells_tile_row_major() {
        let g = grid();
        let c0 = g.cell_rect(0);
        let c1 = g.cell_rect(1);
        let c2 = g.cell_rect(2);
        assert!(c1.x > c0.x && (c1.y - c0.y).abs() < f32::EPSILON);
        assert!(c2.y > c0.y && (c2.x - c0.x).abs() < f32::EPSILON);
    }

    #[test]
    fn dynamic_items_add_rows_and_only_their_cells_hit() {
        let mut g = WorkspaceGridView::new();
        g.items = (0..5).map(|i| format!("Space {}", i + 1)).collect();
        g.set_rect(Rect::new(0.0, 0.0, 240.0, 260.0));

        assert_eq!(g.rows(), 3);
        let fifth = g.cell_rect(4);
        assert!(fifth.y > g.cell_rect(0).y);
        assert!(g.cell_at(Point::new(fifth.x + 1.0, fifth.y + 1.0)) == Some(4));
        let unused = g.cell_rect(5);
        assert!(unused.width == 0.0 && unused.height == 0.0);
        assert_eq!(g.cell_at(Point::new(239.0, 259.0)), None);
    }

    #[test]
    fn empty_grid_has_no_hit_targets() {
        let mut g = WorkspaceGridView::new();
        g.set_rect(Rect::new(0.0, 0.0, 240.0, 160.0));
        assert_eq!(g.rows(), 0);
        let unused = g.cell_rect(0);
        assert!(unused.width == 0.0 && unused.height == 0.0);
        assert_eq!(g.cell_at(Point::new(100.0, 100.0)), None);
    }

    #[test]
    fn keyboard_navigation_is_focus_gated_and_bounds_safe() {
        let mut g = grid();
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowRight)),
            EventResult::Ignored
        ));
        assert_eq!(g.focused_index, 0);

        g.widget_state_mut().focused = true;
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowRight)),
            EventResult::Handled
        ));
        assert_eq!(g.focused_index, 1);
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowDown)),
            EventResult::Handled
        ));
        assert_eq!(g.focused_index, 3);
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowRight)),
            EventResult::Handled
        ));
        assert_eq!(g.focused_index, 3, "right at row edge must not wrap");
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowDown)),
            EventResult::Handled
        ));
        assert_eq!(
            g.focused_index, 3,
            "down beyond the last row must not overflow"
        );
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowLeft)),
            EventResult::Handled
        ));
        assert_eq!(g.focused_index, 2);
        assert!(matches!(
            g.handle_event(&key(KeyCode::ArrowUp)),
            EventResult::Handled
        ));
        assert_eq!(g.focused_index, 0);
    }

    #[test]
    fn enter_and_space_activate_the_focused_cell_once() {
        let mut g = grid();
        g.widget_state_mut().focused = true;
        g.focused_index = 2;

        assert!(matches!(
            g.handle_event(&key(KeyCode::Enter)),
            EventResult::Handled
        ));
        assert_eq!(g.take_activated(), Some(2));
        assert_eq!(g.take_activated(), None);

        assert!(matches!(
            g.handle_event(&key(KeyCode::Space)),
            EventResult::Handled
        ));
        assert_eq!(g.take_activated(), Some(2));
    }

    #[test]
    fn focus_is_clamped_when_items_shrink_or_empty() {
        let mut g = grid();
        g.focused_index = 3;
        g.items.truncate(2);
        g.normalize_focus();
        assert_eq!(g.focused_index, 0, "focus falls back to the active Space");

        g.items.clear();
        g.normalize_focus();
        assert_eq!(g.focused_index, 0);
        g.widget_state_mut().focused = true;
        assert!(matches!(
            g.handle_event(&key(KeyCode::Enter)),
            EventResult::Handled
        ));
        assert_eq!(
            g.take_activated(),
            None,
            "empty grids cannot activate a stale cell"
        );
    }
}
