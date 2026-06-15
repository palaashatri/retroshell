use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    Login,
    Logout,
    Lock,
    Unlock,
    Shutdown,
    Restart,
    Sleep,
}

pub struct SessionManager {
    pub logged_in: bool,
    pub username: String,
    pub autologin: bool,
    pub restore_windows: bool,
    pub locked: bool,
    pub pending_action: Option<SessionAction>,
    pub session_state: HashMap<String, String>,
    lock_on_sleep: bool,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            logged_in: false,
            username: String::new(),
            autologin: false,
            restore_windows: true,
            locked: false,
            pending_action: None,
            session_state: HashMap::new(),
            lock_on_sleep: true,
        }
    }

    pub fn login(&mut self, username: &str) {
        self.logged_in = true;
        self.locked = false;
        self.pending_action = Some(SessionAction::Login);
        self.username = username.to_string();
    }

    pub fn logout(&mut self) {
        self.logged_in = false;
        self.locked = false;
        self.pending_action = Some(SessionAction::Logout);
        self.save_state();
    }

    pub fn lock(&mut self) {
        if self.logged_in {
            self.locked = true;
            self.pending_action = Some(SessionAction::Lock);
        }
    }

    pub fn unlock(&mut self) {
        self.locked = false;
        self.pending_action = Some(SessionAction::Unlock);
    }

    pub fn sleep(&mut self) {
        if self.lock_on_sleep {
            self.lock();
        }
        self.pending_action = Some(SessionAction::Sleep);
    }

    pub fn shutdown(&mut self) {
        self.pending_action = Some(SessionAction::Shutdown);
        self.save_state();
    }

    pub fn restart(&mut self) {
        self.pending_action = Some(SessionAction::Restart);
        self.save_state();
    }

    pub fn save_state(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_dir = std::path::PathBuf::from(home).join(".config/retroshell");
        let _ = std::fs::create_dir_all(&config_dir);
        let path = config_dir.join("session.toml");

        let mut content = String::new();
        content.push_str("[session]\n");
        content.push_str(&format!(
            "username = \"{}\"\n",
            escape_toml_string(&self.username)
        ));
        content.push_str(&format!("logged_in = {}\n", self.logged_in));
        content.push_str(&format!("locked = {}\n", self.locked));
        content.push_str(&format!("restore_windows = {}\n", self.restore_windows));
        for (k, v) in &self.session_state {
            content.push_str(&format!("{} = \"{}\"\n", k, escape_toml_string(v)));
        }
        let _ = std::fs::write(path, content);
    }

    pub fn restore_state(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = std::path::PathBuf::from(home).join(".config/retroshell/session.toml");
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some(pos) = line.find('=') {
                    let key = line[..pos].trim();
                    let val = line[pos + 1..].trim().trim_matches('"');
                    if key == "username" {
                        self.username = val.to_string();
                    } else if key == "logged_in" {
                        self.logged_in = val.parse().unwrap_or(false);
                    } else if key == "locked" {
                        self.locked = val.parse().unwrap_or(false);
                    } else if key == "restore_windows" {
                        self.restore_windows = val.parse().unwrap_or(true);
                    } else if key != "[session]" {
                        self.session_state.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
    }
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
