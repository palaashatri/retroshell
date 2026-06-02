use crate::pty::Pty;
use crate::terminal::Terminal;
use nix::unistd::Pid;
use retro_kit::{Widget, WidgetState, LayoutConstraint, Size, Rect, Event, EventResult, ThemeContext, AccessibilityNode};

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
        tracing::info!("Closing tab {} ({}) with PID {}", tab.id, tab.title, tab.child_pid);
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
        if let Some(tab) = self.active_tab_mut() {
            let cols = (constraint.max_width / 8.0).max(10.0) as usize;
            let rows = (constraint.max_height / 16.0).max(5.0) as usize;
            tab.term.resize_term(cols, rows);
            let size = tab.term.layout(constraint);
            self.set_rect(Rect::new(
                self.rect().x,
                self.rect().y,
                size.width,
                size.height,
            ));
            size
        } else {
            constraint.clamp(Size::ZERO)
        }
    }

    fn draw(&self, theme: &ThemeContext) {
        if let Some(tab) = self.active_tab() {
            tab.term.draw(theme);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
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
