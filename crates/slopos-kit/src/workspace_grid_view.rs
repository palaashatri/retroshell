use crate::{
    theme::ThemeContext, AccessibilityNode, AccessibilityRole, Event, EventResult,
    LayoutConstraint, Point, Rect, Size, Widget, WidgetState,
};

pub struct WorkspaceGridView {
    state: WidgetState,
    pub active_index: usize,
    pub items: Vec<String>,
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
            items: Vec::new(),
            activated: None,
        }
    }

    /// Screen rect of grid cell `index` (row-major over the fixed 2×2 grid),
    /// given the widget's current rect.
    pub fn cell_rect(&self, index: usize) -> Rect {
        let r = self.rect();
        let grid = Rect::new(
            r.x + GRID_MARGIN,
            r.y + GRID_MARGIN,
            (r.width - GRID_MARGIN * 2.0).max(0.0),
            (r.height - GRID_MARGIN * 2.0).max(0.0),
        );
        let cell_w = (grid.width - GRID_GUTTER) / GRID_COLS as f32;
        let cell_h = (grid.height - GRID_GUTTER) / GRID_ROWS as f32;
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
        (0..GRID_COLS * GRID_ROWS).find(|&i| self.cell_rect(i).contains(point))
    }

    /// Index of the most recently pressed cell; drains exactly once.
    pub fn take_activated(&mut self) -> Option<usize> {
        self.activated.take()
    }
}

impl Widget for WorkspaceGridView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(240.0, 160.0));
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
            Event::MouseDown {
                button: crate::event::MouseButton::Left,
                point,
                ..
            } => {
                if !self.rect().contains(*point) {
                    return EventResult::Ignored;
                }
                match self.cell_at(*point) {
                    Some(cell) => {
                        self.activated = Some(cell);
                        EventResult::Handled
                    }
                    None => EventResult::Ignored,
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
    use crate::event::{Modifiers, MouseButton};

    fn grid() -> WorkspaceGridView {
        let mut g = WorkspaceGridView::new();
        g.items = (0..4).map(|i| format!("Desktop {}", i + 1)).collect();
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

    #[test]
    fn each_cell_center_activates_that_cell() {
        for i in 0..4 {
            let mut g = grid();
            let c = g.cell_rect(i);
            let result = g.handle_event(&press(
                c.x + c.width * 0.5,
                c.y + c.height * 0.5,
            ));
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
}
