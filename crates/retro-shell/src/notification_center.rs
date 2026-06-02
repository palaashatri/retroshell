
#[derive(Debug, Clone)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub app_id: String,
    pub title: String,
    pub message: String,
    pub icon: Option<String>,
    pub priority: NotificationPriority,
    pub timestamp: std::time::Instant,
    pub dismissed: bool,
}

pub struct NotificationCenter {
    pub notifications: Vec<Notification>,
    pub max_visible: usize,
}

impl NotificationCenter {
    pub fn new() -> Self {
        Self { notifications: vec![], max_visible: 5 }
    }

    pub fn post(&mut self, app_id: &str, title: &str, message: &str, priority: NotificationPriority) -> String {
        let id = format!("notif-{}", self.notifications.len());
        self.notifications.push(Notification {
            id: id.clone(),
            app_id: app_id.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            icon: None,
            priority,
            timestamp: std::time::Instant::now(),
            dismissed: false,
        });
        id
    }

    pub fn dismiss(&mut self, id: &str) {
        if let Some(notif) = self.notifications.iter_mut().find(|n| n.id == id) {
            notif.dismissed = true;
        }
    }

    pub fn dismiss_all(&mut self) {
        for notif in &mut self.notifications {
            notif.dismissed = true;
        }
    }

    pub fn visible(&self) -> Vec<&Notification> {
        self.notifications.iter()
            .filter(|n| !n.dismissed)
            .take(self.max_visible)
            .collect()
    }

    pub fn clear_expired(&mut self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        self.notifications.retain(|n| n.dismissed || now.duration_since(n.timestamp) < max_age);
    }
}
