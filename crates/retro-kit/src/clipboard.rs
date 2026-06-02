use parking_lot::Mutex;

static CLIPBOARD_CONTENT: Mutex<String> = Mutex::new(String::new());

pub struct Clipboard;

impl Clipboard {
    pub fn copy(text: &str) {
        let mut guard = CLIPBOARD_CONTENT.lock();
        *guard = text.to_string();
    }

    pub fn paste() -> String {
        let guard = CLIPBOARD_CONTENT.lock();
        guard.clone()
    }

    pub fn clear() {
        let mut guard = CLIPBOARD_CONTENT.lock();
        guard.clear();
    }
}
