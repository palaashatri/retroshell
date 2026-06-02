#![allow(dead_code)]

use crate::pty::Pty;
use crate::terminal::Terminal;
use nix::unistd::Pid;

pub struct Tab {
    pub id: usize,
    pub title: String,
    pub term: Terminal,
    pub pty: Pty,
    pub child_pid: Pid,
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    next_tab_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        TabManager {
            tabs: vec![],
            active_tab_index: 0,
            next_tab_id: 1,
        }
    }

    pub fn open_tab(&mut self, cols: u16, rows: u16) -> Result<usize, String> {
        let (pty, pid) = Pty::new(cols, rows)?;
        let term = Terminal::new(cols as usize, rows as usize);
        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = Tab {
            id,
            title: format!("Shell {}", id),
            term,
            pty,
            child_pid: pid,
        };
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        Ok(id)
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
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

    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            true
        } else {
            false
        }
    }
}
