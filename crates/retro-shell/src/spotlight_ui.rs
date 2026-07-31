//! Spotlight search UI — rendering and input handling for the search overlay.
//!
//! This module integrates the search backend with retro-kit widgets and manages
//! the overlay surface presentation.

use crate::spotlight::{SearchResult, Spotlight};
use retro_kit::event::{KeyCode, Modifiers};
use retro_kit::list_view::ListView;
use retro_kit::text_field::TextField;
use retro_kit::{EventResult, Rect, ThemeContext, Widget};

/// Spotlight search UI state — owns the spotlight logic + UI widgets.
pub struct SpotlightUI {
    /// The search backend and state machine.
    pub spotlight: Spotlight,
    /// Text input field for the search query.
    search_field: TextField,
    /// List view for displaying search results.
    results_list: ListView,
    /// Current search results.
    current_results: Vec<SearchResult>,
    /// Index of the currently selected result (keyboard navigation).
    selected_index: usize,
}

impl SpotlightUI {
    /// Create a new Spotlight UI.
    pub fn new() -> Self {
        Self {
            spotlight: Spotlight::new(),
            search_field: TextField::new().with_placeholder("Search apps, files, settings..."),
            results_list: ListView::new(),
            current_results: Vec::new(),
            selected_index: 0,
        }
    }

    /// Check if the overlay is visible.
    pub fn is_visible(&self) -> bool {
        self.spotlight.is_visible()
    }

    /// Show the overlay (invoked on Super+Space).
    pub fn show(&mut self) {
        self.spotlight.show();
        self.search_field.set_text("");
        self.current_results.clear();
        self.selected_index = 0;
    }

    /// Hide the overlay.
    pub fn hide(&mut self) {
        self.spotlight.hide();
        self.search_field.set_text("");
        self.current_results.clear();
    }

    /// Update the search results based on current query and available apps.
    pub fn update_results(&mut self, apps: &[crate::launch_services::AppBundle]) {
        self.current_results = self.spotlight.search_results(apps);
        self.selected_index = 0.min(self.current_results.len().saturating_sub(1));
    }

    /// Get the currently selected result, if any.
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.current_results.get(self.selected_index)
    }

    /// Append a character to the search query.
    pub fn append_char(&mut self, c: char) {
        self.spotlight.append_char(c);
    }

    /// Handle a keyboard event (for the search UI overlay).
    /// Returns `EventResult::Handled` if the event was processed.
    pub fn handle_overlay_key(&mut self, key: KeyCode, _modifiers: &Modifiers) -> EventResult {
        match key {
            KeyCode::Escape => {
                self.hide();
                EventResult::Handled
            }
            KeyCode::Enter => {
                // Activation: user selected a result
                // (actual launch/open would be done by caller)
                EventResult::Handled
            }
            KeyCode::ArrowUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                EventResult::Handled
            }
            KeyCode::ArrowDown => {
                if self.selected_index < self.current_results.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                EventResult::Handled
            }
            KeyCode::Backspace => {
                self.spotlight.backspace();
                // Update search on backspace
                // (would call update_results in practice)
                EventResult::Handled
            }
            // For letter/number keys, delegate to text field for input
            // The actual character mapping happens at a higher level
            _ => EventResult::Ignored,
        }
    }

    /// Render the Spotlight overlay to a rect.
    /// This would be called from the shell's render loop when visible.
    pub fn layout(&mut self, rect: Rect) {
        // Overlay is centered and takes up ~80% of screen width/height
        let width = (rect.width * 0.8).max(400.0);
        let height = (rect.height * 0.6).max(300.0);
        let x = (rect.width - width) / 2.0;
        let y = (rect.height - height) / 2.0;

        let overlay_rect = Rect::new(x, y, width, height);

        // Layout: search field at top, results list below
        let field_height = 40.0;
        let padding = 16.0;

        // Search field
        let field_rect = Rect::new(
            overlay_rect.x + padding,
            overlay_rect.y + padding,
            overlay_rect.width - padding * 2.0,
            field_height,
        );
        self.search_field.set_rect(field_rect);

        // Results list
        let list_y = overlay_rect.y + padding + field_height + padding;
        let list_height = overlay_rect.height - (padding * 3.0 + field_height);
        let list_rect = Rect::new(
            overlay_rect.x + padding,
            list_y,
            overlay_rect.width - padding * 2.0,
            list_height,
        );
        self.results_list.set_rect(list_rect);
    }

    /// Get the overlay rect (for layer-shell positioning).
    /// Returns a rect that covers the centered search overlay.
    pub fn overlay_rect(&self) -> Rect {
        Rect::new(100.0, 100.0, 1080.0, 600.0) // stub: should be calculated from screen size
    }

    /// Get the current search query string.
    pub fn query(&self) -> &str {
        self.spotlight.query()
    }

    /// Get the current search results.
    pub fn results(&self) -> &[SearchResult] {
        &self.current_results
    }

    /// Get the index of the selected result.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Render the overlay visually (scrim + search field + results).
    /// This is called from ShellDesktop::draw() when Spotlight is visible.
    pub fn draw_overlay(&self, theme: &ThemeContext, screen_w: f32, screen_h: f32) {
        // This is a placeholder implementation.
        // In a real implementation, we would render:
        // 1. Semi-transparent scrim background
        // 2. Search field with typed text
        // 3. Results list with app icons and names
        // 4. Selection highlight on current item
        //
        // For now, the infrastructure is in place (search_field, results_list widgets)
        // and they can be drawn via the widget system.
        //
        // TODO: Wire up canvas rendering when canvas access is available in draw()
    }
}

impl Default for SpotlightUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spotlight_ui_visibility() {
        let mut ui = SpotlightUI::new();
        assert!(!ui.is_visible());

        ui.show();
        assert!(ui.is_visible());

        ui.hide();
        assert!(!ui.is_visible());
    }

    #[test]
    fn keyboard_navigation() {
        let mut ui = SpotlightUI::new();
        ui.show();

        // Simulate some results
        let results = vec![
            SearchResult::Setting {
                category: "Display".to_string(),
                title: "Brightness".to_string(),
            },
            SearchResult::Setting {
                category: "Sound".to_string(),
                title: "Volume".to_string(),
            },
        ];
        ui.current_results = results;
        ui.selected_index = 0;

        let modifiers = Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: false,
        };

        // Arrow down
        ui.handle_overlay_key(KeyCode::ArrowDown, &modifiers);
        assert_eq!(ui.selected_index, 1);

        // Arrow up
        ui.handle_overlay_key(KeyCode::ArrowUp, &modifiers);
        assert_eq!(ui.selected_index, 0);

        // Escape
        let result = ui.handle_overlay_key(KeyCode::Escape, &modifiers);
        match result {
            EventResult::Handled => {},
            _ => panic!("Expected Handled"),
        }
        assert!(!ui.is_visible());
    }
}
