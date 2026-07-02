use crate::vt_parser::VtHandler;
use retro_kit::clipboard::Clipboard;
use retro_kit::event::KeyCode;
use retro_kit::theme::ThemeContext;
use retro_kit::Color;
use retro_kit::{
    AccessibilityNode, AccessibilityRole, Event, EventResult, LayoutConstraint, MonospaceCell,
    MonospaceView, Rect, Size, Widget, WidgetState,
};
use std::any::Any;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            c: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

pub struct Terminal {
    state: WidgetState,
    pub cols: usize,
    pub rows: usize,
    pub grid: Vec<Cell>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scrollback: Vec<Vec<Cell>>,
    pub max_scrollback: usize,
    pub current_fg: Color,
    pub current_bg: Color,
    pub current_bold: bool,
    pub current_italic: bool,
    pub current_underline: bool,
    parser: vte::Parser,
    pub pty: Option<crate::pty::Pty>,
    pub rx: Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Vec<u8>>>>>,
    display: MonospaceView,
    pub scroll_offset: usize,
}

impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        Terminal {
            state: WidgetState::new(),
            cols,
            rows,
            grid: vec![Cell::default(); cols * rows],
            cursor_x: 0,
            cursor_y: 0,
            scrollback: vec![],
            max_scrollback: 1000,
            current_fg: Color::WHITE,
            current_bg: Color::BLACK,
            current_bold: false,
            current_italic: false,
            current_underline: false,
            parser: vte::Parser::new(),
            pty: None,
            rx: None,
            display: MonospaceView::new(cols, rows),
            scroll_offset: 0,
        }
    }

    pub fn resize_term(&mut self, cols: usize, rows: usize) {
        let mut new_grid = vec![Cell::default(); cols * rows];
        for r in 0..rows.min(self.rows) {
            for c in 0..cols.min(self.cols) {
                new_grid[r * cols + c] = self.grid[r * self.cols + c].clone();
            }
        }
        self.grid = new_grid;
        self.cols = cols;
        self.rows = rows;
        self.display.resize(cols, rows);
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(rows.saturating_sub(1));
        self.scroll_offset = self.scroll_offset.min(self.scrollback.len());
        if let Some(ref pty) = self.pty {
            let _ = pty.resize(cols as u16, rows as u16);
        }
    }

    pub fn print_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }
        if self.cursor_y >= self.rows {
            self.scroll_up();
            self.cursor_y = self.rows - 1;
        }
        let idx = self.cursor_y * self.cols + self.cursor_x;
        if idx < self.grid.len() {
            self.grid[idx] = Cell {
                c,
                fg: self.current_fg,
                bg: self.current_bg,
                bold: self.current_bold,
                italic: self.current_italic,
                underline: self.current_underline,
            };
        }
        self.cursor_x += 1;
    }

    pub fn scroll_up(&mut self) {
        let first_row = self.grid[0..self.cols].to_vec();
        if self.scrollback.len() >= self.max_scrollback {
            self.scrollback.remove(0);
        }
        self.scrollback.push(first_row);
        self.grid.drain(0..self.cols);
        self.grid.extend(vec![Cell::default(); self.cols]);
    }

    pub fn scroll_lines(&mut self, lines: isize) {
        if lines > 0 {
            self.scroll_offset = (self.scroll_offset + lines as usize).min(self.scrollback.len());
        } else if lines < 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(lines.unsigned_abs());
        }
        self.sync_display();
    }

    pub fn visible_text(&self) -> String {
        self.visible_rows()
            .into_iter()
            .map(Self::row_text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn visible_rows(&self) -> Vec<Vec<Cell>> {
        if self.scroll_offset == 0 {
            return self
                .grid
                .chunks(self.cols)
                .map(|row| row.to_vec())
                .collect();
        }

        let history: Vec<Vec<Cell>> = self
            .scrollback
            .iter()
            .cloned()
            .chain(self.grid.chunks(self.cols).map(|row| row.to_vec()))
            .collect();
        let total = history.len();
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(self.rows);
        let mut rows = history[start..end].to_vec();
        while rows.len() < self.rows {
            rows.insert(0, vec![Cell::default(); self.cols]);
        }
        rows
    }

    fn row_text(row: Vec<Cell>) -> String {
        let mut text: String = row.into_iter().map(|cell| cell.c).collect();
        while text.ends_with(' ') {
            text.pop();
        }
        text
    }

    pub fn write_byte(&mut self, byte: u8) {
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());
        {
            let mut handler = VtHandler { term: self };
            parser.advance(&mut handler, byte);
        }
        self.parser = parser;
    }
}

impl Widget for Terminal {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }
    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        self.set_rect(Rect::new(
            self.rect().x,
            self.rect().y,
            size.width,
            size.height,
        ));
        self.display.set_rect(self.rect());
        self.display.layout(constraint);
        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.display.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::KeyDown { key, modifiers } => {
                if modifiers.meta {
                    match key {
                        KeyCode::C => {
                            Clipboard::copy(&self.visible_text());
                            return EventResult::Handled;
                        }
                        KeyCode::V => {
                            let pasted = Clipboard::paste();
                            if !pasted.is_empty() {
                                if let Some(ref mut pty) = self.pty {
                                    let _ = pty.write(pasted.as_bytes());
                                    return EventResult::Handled;
                                }
                            }
                            return EventResult::Ignored;
                        }
                        KeyCode::A => {
                            Clipboard::copy(&self.visible_text());
                            return EventResult::Handled;
                        }
                        _ => {}
                    }
                }

                if modifiers.shift {
                    match key {
                        KeyCode::PageUp => {
                            self.scroll_lines(self.rows as isize);
                            return EventResult::Handled;
                        }
                        KeyCode::PageDown => {
                            self.scroll_lines(-(self.rows as isize));
                            return EventResult::Handled;
                        }
                        KeyCode::Home => {
                            self.scroll_lines(self.scrollback.len() as isize);
                            return EventResult::Handled;
                        }
                        KeyCode::End => {
                            self.scroll_offset = 0;
                            self.sync_display();
                            return EventResult::Handled;
                        }
                        _ => {}
                    }
                }

                if let Some(ref mut pty) = self.pty {
                    let mut bytes = vec![];
                    if modifiers.control {
                        if let Some(byte) = match key {
                            KeyCode::A => Some(b'\x01'),
                            KeyCode::B => Some(b'\x02'),
                            KeyCode::C => Some(b'\x03'),
                            KeyCode::D => Some(b'\x04'),
                            KeyCode::E => Some(b'\x05'),
                            KeyCode::F => Some(b'\x06'),
                            KeyCode::G => Some(b'\x07'),
                            KeyCode::H => Some(b'\x08'),
                            KeyCode::I => Some(b'\x09'),
                            KeyCode::J => Some(b'\x0a'),
                            KeyCode::K => Some(b'\x0b'),
                            KeyCode::L => Some(b'\x0c'),
                            KeyCode::M => Some(b'\x0d'),
                            KeyCode::N => Some(b'\x0e'),
                            KeyCode::O => Some(b'\x0f'),
                            KeyCode::P => Some(b'\x10'),
                            KeyCode::Q => Some(b'\x11'),
                            KeyCode::R => Some(b'\x12'),
                            KeyCode::S => Some(b'\x13'),
                            KeyCode::T => Some(b'\x14'),
                            KeyCode::U => Some(b'\x15'),
                            KeyCode::V => Some(b'\x16'),
                            KeyCode::W => Some(b'\x17'),
                            KeyCode::X => Some(b'\x18'),
                            KeyCode::Y => Some(b'\x19'),
                            KeyCode::Z => Some(b'\x1a'),
                            _ => None,
                        } {
                            bytes.push(byte);
                        }
                    } else {
                        match key {
                            KeyCode::Backspace => {
                                bytes.push(0x7f);
                            }
                            KeyCode::Enter => {
                                bytes.push(b'\r');
                            }
                            KeyCode::Tab => {
                                bytes.push(b'\t');
                            }
                            KeyCode::Escape => {
                                bytes.push(0x1b);
                            }
                            KeyCode::ArrowUp => {
                                bytes.extend_from_slice(b"\x1b[A");
                            }
                            KeyCode::ArrowDown => {
                                bytes.extend_from_slice(b"\x1b[B");
                            }
                            KeyCode::ArrowRight => {
                                bytes.extend_from_slice(b"\x1b[C");
                            }
                            KeyCode::ArrowLeft => {
                                bytes.extend_from_slice(b"\x1b[D");
                            }
                            KeyCode::Home => {
                                bytes.extend_from_slice(b"\x1b[H");
                            }
                            KeyCode::End => {
                                bytes.extend_from_slice(b"\x1b[F");
                            }
                            KeyCode::Insert => {
                                bytes.extend_from_slice(b"\x1b[2~");
                            }
                            KeyCode::Delete => {
                                bytes.extend_from_slice(b"\x1b[3~");
                            }
                            KeyCode::PageUp => {
                                bytes.extend_from_slice(b"\x1b[5~");
                            }
                            KeyCode::PageDown => {
                                bytes.extend_from_slice(b"\x1b[6~");
                            }
                            _ => {}
                        }
                    }

                    if !bytes.is_empty() {
                        let _ = pty.write(&bytes);
                        return EventResult::Handled;
                    }
                }
                EventResult::Ignored
            }
            Event::Char { character } => {
                if let Some(ref mut pty) = self.pty {
                    self.scroll_offset = 0;
                    let mut buf = [0u8; 4];
                    let s = character.encode_utf8(&mut buf);
                    let _ = pty.write(s.as_bytes());
                    return EventResult::Handled;
                }
                EventResult::Ignored
            }
            Event::Scroll { delta, .. } => {
                if delta.y > 0.0 {
                    self.scroll_lines(3);
                } else if delta.y < 0.0 {
                    self.scroll_lines(-3);
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn update(&mut self) {
        let rx = self.rx.clone();
        if let Some(rx) = rx {
            if let Ok(rx_lock) = rx.try_lock() {
                while let Ok(bytes) = rx_lock.try_recv() {
                    for b in bytes {
                        self.write_byte(b);
                    }
                }
            }
        }
        self.sync_display();
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&self.display]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![&mut self.display]
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(
            AccessibilityRole::Unknown,
            "Terminal window",
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Terminal {
    pub fn sync_display(&mut self) {
        let rows = self.visible_rows();
        for row in 0..self.rows {
            for col in 0..self.cols {
                let index = row * self.cols + col;
                if let Some(slot) = self.display.cells.get_mut(index) {
                    let cell = rows
                        .get(row)
                        .and_then(|row| row.get(col))
                        .cloned()
                        .unwrap_or_default();
                    *slot = MonospaceCell {
                        ch: cell.c,
                        fg: [cell.fg.r, cell.fg.g, cell.fg.b, cell.fg.a],
                        bg: [cell.bg.r, cell.bg.g, cell.bg.b, cell.bg.a],
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::Pty;
    use crate::tabs::TabManager;

    #[test]
    fn test_terminal_emulator() {
        let mut term = Terminal::new(80, 24);
        assert_eq!(term.cols, 80);
        assert_eq!(term.rows, 24);
        assert_eq!(term.cursor_x, 0);
        assert_eq!(term.cursor_y, 0);

        // Print character
        term.print_char('A');
        assert_eq!(term.cursor_x, 1);
        assert_eq!(term.cursor_y, 0);
        assert_eq!(term.grid[0].c, 'A');

        // Verify Cell fields
        let cell = &term.grid[0];
        let _ = cell.fg;
        let _ = cell.bg;
        let _ = cell.bold;
        let _ = cell.italic;
        let _ = cell.underline;

        // Write VT bytes
        term.write_byte(b'B');
        assert_eq!(term.cursor_x, 2);
        assert_eq!(term.grid[1].c, 'B');

        // Backspace CSI
        term.write_byte(0x08);
        assert_eq!(term.cursor_x, 1);

        // Carriage Return CSI
        term.write_byte(0x0d);
        assert_eq!(term.cursor_x, 0);

        // Line Feed CSI
        term.write_byte(0x0a);
        assert_eq!(term.cursor_y, 1);

        // Write ANSI styling CSI (Set red foreground)
        term.write_byte(0x1b);
        term.write_byte(b'[');
        term.write_byte(b'3');
        term.write_byte(b'1');
        term.write_byte(b'm');
        assert_eq!(term.current_fg.r, 0.8);

        // Clear screen (ESC [ 2 J)
        term.write_byte(0x1b);
        term.write_byte(b'[');
        term.write_byte(b'2');
        term.write_byte(b'J');
        assert_eq!(term.cursor_x, 0);
        assert_eq!(term.cursor_y, 0);
        assert_eq!(term.grid[0].c, ' ');

        // Test resize
        term.resize_term(40, 12);
        assert_eq!(term.cols, 40);
        assert_eq!(term.rows, 12);
        let _ = term.max_scrollback;
        let _ = term.scrollback;
    }

    #[test]
    fn test_tab_manager() {
        let mut tab_manager = TabManager::new();
        assert!(tab_manager.tabs.is_empty());

        let _tab_id = tab_manager.open_tab(80, 24).unwrap();
        assert_eq!(tab_manager.tabs.len(), 1);
        assert_eq!(tab_manager.active_tab_index, 0);

        // Open another tab
        let _tab_id_2 = tab_manager.open_tab(80, 24).unwrap();
        assert_eq!(tab_manager.tabs.len(), 2);
        assert_eq!(tab_manager.active_tab_index, 1);

        // Switch tab
        assert!(tab_manager.switch_tab(0));
        assert_eq!(tab_manager.active_tab_index, 0);

        // Get tab references and read fields to avoid dead_code warnings
        {
            let active = tab_manager.active_tab().unwrap();
            assert_eq!(active.id, 1);
            assert!(!active.title.is_empty());
            let _ = active.term.cols;
            let _ = active.pty.master_file.metadata();
        }
        assert!(tab_manager.active_tab_mut().is_some());

        // Close tabs and clean up processes
        let child_pid = tab_manager.tabs[1].child_pid;
        let _ = nix::sys::signal::kill(child_pid, nix::sys::signal::Signal::SIGKILL);

        let child_pid_active = tab_manager.tabs[0].child_pid;
        let _ = nix::sys::signal::kill(child_pid_active, nix::sys::signal::Signal::SIGKILL);

        assert!(tab_manager.close_tab(1));
        assert_eq!(tab_manager.tabs.len(), 1);
    }

    #[test]
    fn test_pty_creation_and_resize() {
        let res = Pty::new(80, 24);
        assert!(res.is_ok());
        let (pty, pid) = res.unwrap();

        // Test PTY resizing
        assert!(pty.resize(100, 30).is_ok());

        // Clean up process
        let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGKILL);
    }

    #[test]
    fn terminal_scrollback_changes_visible_text() {
        let mut term = Terminal::new(20, 2);
        for byte in b"first\r\nsecond\r\nthird" {
            term.write_byte(*byte);
        }
        term.sync_display();

        assert!(term.visible_text().contains("third"));

        term.scroll_lines(1);

        assert_eq!(term.scroll_offset, 1);
        assert!(term.visible_text().contains("second"));
        assert!(!term.visible_text().contains("third"));
    }

    #[test]
    fn terminal_meta_copy_copies_visible_text() {
        Clipboard::clear();
        let mut term = Terminal::new(8, 2);
        for byte in b"copy me" {
            term.write_byte(*byte);
        }

        let result = term.handle_event(&Event::KeyDown {
            key: KeyCode::C,
            modifiers: retro_kit::event::Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });

        assert!(matches!(result, EventResult::Handled));
        assert!(Clipboard::paste().contains("copy me"));
    }
}
