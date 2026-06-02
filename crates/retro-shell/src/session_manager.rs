use std::collections::HashMap;

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
    pub session_state: HashMap<String, String>,
    #[allow(dead_code)]
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
            session_state: HashMap::new(),
            lock_on_sleep: true,
        }
    }

    pub fn login(&mut self, username: &str) {
        self.logged_in = true;
        self.username = username.to_string();
    }

    pub fn logout(&mut self) {
        self.logged_in = false;
        self.save_state();
    }

    pub fn lock(&mut self) {
        // Lock the screen placeholder
    }

    pub fn unlock(&mut self) {
        // Unlock the screen placeholder
    }

    pub fn shutdown(&self) {
        // Initiate system shutdown placeholder
    }

    pub fn restart(&self) {
        // Initiate system restart placeholder
    }

    pub fn save_state(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let config_dir = std::path::PathBuf::from(home).join(".config/retroshell");
        let _ = std::fs::create_dir_all(&config_dir);
        let path = config_dir.join("session.toml");

        let mut content = String::new();
        content.push_str("[session]\n");
        content.push_str(&format!("username = \"{}\"\n", self.username));
        content.push_str(&format!("logged_in = {}\n", self.logged_in));
        for (k, v) in &self.session_state {
            content.push_str(&format!("{} = \"{}\"\n", k, v));
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
                    } else if key != "[session]" {
                        self.session_state.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
    }
}
