use crate::pty::Pty;
use crate::terminal::Terminal;
use nix::unistd::Pid;
use retro_kit::{
    AccessibilityNode, Event, EventResult, LayoutConstraint, Rect, Size, ThemeContext, Widget,
    WidgetState,
};

#[allow(dead_code)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub term: Terminal,
    pub pty: Pty,
    pub child_pid: Pid,
}

#[allow(dead_code)]
impl Tab {
    pub fn pty(&self) -> &Pty {
        &self.pty
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn child_pid(&self) -> Pid {
        self.child_pid
    }
}

pub struct TabManager {
    state: WidgetState,
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    next_tab_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        TabManager {
            state: WidgetState::new(),
            tabs: vec![],
            active_tab_index: 0,
            next_tab_id: 1,
        }
    }

    pub fn open_tab(&mut self, cols: u16, rows: u16) -> Result<usize, String> {
        let (pty, pid) = Pty::new(cols, rows)?;
        let mut term = Terminal::new(cols as usize, rows as usize);

        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let mut reader_pty = pty.try_clone().map_err(|e| e.to_string())?;

        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                match reader_pty.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });

        term.pty = Some(pty.try_clone().map_err(|e| e.to_string())?);
        term.rx = Some(std::sync::Arc::new(std::sync::Mutex::new(rx)));

        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let title = format!("Shell {}", id);
        tracing::info!("Opening tab {} ({}) with PID {}", id, title, pid);

        let tab = Tab {
            id,
            title,
            term,
            pty,
            child_pid: pid,
        };
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        Ok(id)
    }

    #[allow(dead_code)]
    pub fn close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        let tab = &self.tabs[index];
        tracing::info!(
            "Closing tab {} ({}) with PID {}",
            tab.id,
            tab.title,
            tab.child_pid
        );
        self.tabs.remove(index);
        if self.active_tab_index >= self.tabs.len() && !self.tabs.is_empty() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        true
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    #[allow(dead_code)]
    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            true
        } else {
            false
        }
    }
}

impl Widget for TabManager {
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

        let rect = self.rect();
        if let Some(tab) = self.active_tab_mut() {
            let cols = (rect.width / 8.0).max(10.0) as usize;
            let rows = (rect.height / 16.0).max(5.0) as usize;
            tab.term.set_rect(rect);
            tab.term.resize_term(cols, rows);
            let _ = tab
                .term
                .layout(LayoutConstraint::tight(Size::new(rect.width, rect.height)));
        } else {
            return constraint.clamp(Size::ZERO);
        }
        size
    }

    fn draw(&self, theme: &ThemeContext) {
        if let Some(tab) = self.active_tab() {
            tab.term.draw(theme);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::KeyDown { key, modifiers } = event {
            if modifiers.meta {
                match key {
                    retro_kit::event::KeyCode::T => {
                        let _ = self.open_tab(80, 24);
                        return EventResult::Handled;
                    }
                    retro_kit::event::KeyCode::W if modifiers.shift => {
                        if !self.tabs.is_empty() {
                            let idx = self.active_tab_index;
                            self.close_tab(idx);
                        }
                        return EventResult::Handled;
                    }
                    retro_kit::event::KeyCode::Key1 => { self.switch_tab(0); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key2 => { self.switch_tab(1); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key3 => { self.switch_tab(2); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key4 => { self.switch_tab(3); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key5 => { self.switch_tab(4); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key6 => { self.switch_tab(5); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key7 => { self.switch_tab(6); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key8 => { self.switch_tab(7); return EventResult::Handled; }
                    retro_kit::event::KeyCode::Key9 => { self.switch_tab(8); return EventResult::Handled; }
                    _ => {}
                }
            }
        }

        if let Some(tab) = self.active_tab_mut() {
            tab.term.handle_event(event)
        } else {
            EventResult::Ignored
        }
    }

    fn update(&mut self) {
        for tab in &mut self.tabs {
            tab.term.update();
        }
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        if let Some(tab) = self.active_tab() {
            vec![&tab.term]
        } else {
            vec![]
        }
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        if let Some(tab) = self.active_tab_mut() {
            vec![&mut tab.term]
        } else {
            vec![]
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
