use parking_lot::Mutex;
use std::fs;
use std::path::PathBuf;

static CLIPBOARD_CONTENT: Mutex<String> = Mutex::new(String::new());

pub struct Clipboard;

impl Clipboard {
    pub fn copy(text: &str) {
        let content = text.to_string();
        {
            let mut guard = CLIPBOARD_CONTENT.lock();
            *guard = content.clone();
        }

        if let Some(path) = clipboard_path() {
            if let Some(parent) = path.parent() {
                if let Err(err) = fs::create_dir_all(parent) {
                    log::warn!("failed to create clipboard directory: {err}");
                    return;
                }
            }
            if let Err(err) = fs::write(path, content) {
                log::warn!("failed to write clipboard content: {err}");
            }
        }
    }

    pub fn paste() -> String {
        if let Some(content) = clipboard_path().and_then(|path| fs::read_to_string(path).ok()) {
            let mut guard = CLIPBOARD_CONTENT.lock();
            *guard = content.clone();
            return content;
        }

        CLIPBOARD_CONTENT.lock().clone()
    }

    pub fn clear() {
        {
            let mut guard = CLIPBOARD_CONTENT.lock();
            guard.clear();
        }

        if let Some(path) = clipboard_path() {
            if let Err(err) = fs::remove_file(path) {
                if err.kind() != std::io::ErrorKind::NotFound {
                    log::warn!("failed to clear clipboard content: {err}");
                }
            }
        }
    }
}

fn clipboard_path() -> Option<PathBuf> {
    std::env::var_os("RETROSHELL_CLIPBOARD_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_RUNTIME_DIR").map(|runtime| {
                PathBuf::from(runtime)
                    .join("retroshell")
                    .join("clipboard.txt")
            })
        })
        .or_else(|| {
            std::env::var_os("TMPDIR")
                .map(PathBuf::from)
                .map(|tmp| tmp.join("retroshell-clipboard.txt"))
        })
        .or_else(|| Some(std::env::temp_dir().join("retroshell-clipboard.txt")))
}

#[cfg(test)]
mod tests {
    use super::Clipboard;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn clipboard_persists_to_runtime_path() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("retroshell_clipboard_{unique}.txt"));
        std::env::set_var("RETROSHELL_CLIPBOARD_PATH", &path);

        Clipboard::clear();
        Clipboard::copy("hello from another app");

        assert_eq!(fs::read_to_string(&path).unwrap(), "hello from another app");
        Clipboard::clear();
        assert!(!path.exists());

        std::env::remove_var("RETROSHELL_CLIPBOARD_PATH");
    }
}
