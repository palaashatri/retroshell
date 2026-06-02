pub struct WorkspaceManager {
    pub workspaces: Vec<Workspace>,
    pub active: usize,
    pub total: usize,
}

pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub background: Option<String>,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let workspaces = (0..4)
            .map(|i| Workspace {
                id: i,
                name: format!("Desktop {}", i + 1),
                background: None,
            })
            .collect();
        Self {
            workspaces,
            active: 0,
            total: 4,
        }
    }

    pub fn switch_to(&mut self, index: usize) -> bool {
        if index < self.total {
            self.active = index;
            true
        } else {
            false
        }
    }

    pub fn next(&mut self) {
        self.active = (self.active + 1) % self.total;
    }

    pub fn previous(&mut self) {
        self.active = if self.active == 0 {
            self.total - 1
        } else {
            self.active - 1
        };
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.active)
    }

    pub fn add_workspace(&mut self, name: &str) {
        let id = self.total;
        self.workspaces.push(Workspace {
            id,
            name: name.to_string(),
            background: None,
        });
        self.total += 1;
    }
}
