use retro_kit::button::Button;
use retro_kit::event::{KeyCode, Modifiers, MouseButton};
use retro_kit::label::Label;
use retro_kit::text_field::TextField;
use retro_kit::toolbar::Toolbar;
use retro_kit::window::Window;
use retro_kit::{
    AccessibilityNode, Event, EventResult, LayoutConstraint, Point, Rect, Size, ThemeContext,
    Widget, WidgetState,
};
use retro_sdk::{build_menu, Application};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut app = Application::new("TextEdit", "com.retro.textedit");

    let mut file_menu = build_menu("File");
    file_menu.add_action("New").with_shortcut(
        KeyCode::N,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    file_menu.add_action("Open...").with_shortcut(
        KeyCode::O,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    file_menu.add_separator();
    file_menu.add_action("Save").with_shortcut(
        KeyCode::S,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    file_menu.add_action("Save As...");
    file_menu.add_separator();
    file_menu.add_action("Close").with_shortcut(
        KeyCode::W,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );

    let mut edit_menu = build_menu("Edit");
    edit_menu.add_action("Undo").with_shortcut(
        KeyCode::Z,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    edit_menu.add_action("Redo").with_shortcut(
        KeyCode::Z,
        Modifiers {
            shift: true,
            control: false,
            alt: false,
            meta: true,
        },
    );
    edit_menu.add_separator();
    edit_menu.add_action("Cut").with_shortcut(
        KeyCode::X,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    edit_menu.add_action("Copy").with_shortcut(
        KeyCode::C,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    edit_menu.add_action("Paste").with_shortcut(
        KeyCode::V,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );
    edit_menu.add_action("Select All").with_shortcut(
        KeyCode::A,
        Modifiers {
            shift: false,
            control: false,
            alt: false,
            meta: true,
        },
    );

    let mut format_menu = build_menu("Format");
    format_menu.add_action("Make Plain Text");
    format_menu.add_action("Wrap to Window");

    let mut window_menu = build_menu("Window");
    window_menu.add_action("Minimize");
    window_menu.add_action("Zoom");

    let mut help_menu = build_menu("Help");
    help_menu.add_action("TextEdit Help");

    app.set_menus(vec![
        file_menu,
        edit_menu,
        format_menu,
        window_menu,
        help_menu,
    ]);

    let document_path = std::env::args_os().nth(1).map(PathBuf::from);
    let view = TextEditView::open(document_path);
    let title = view.window_title();

    let mut window = Window::new(title);
    window.has_toolbar = true;
    window.set_content(Box::new(view));
    app.set_main_window(window);
    app.run();
}

struct TextEditView {
    state: WidgetState,
    toolbar: Toolbar,
    editor: TextField,
    status: Label,
    document_path: Option<PathBuf>,
    saved_text: String,
    dirty: bool,
    last_error: Option<String>,
}

impl TextEditView {
    fn open(document_path: Option<PathBuf>) -> Self {
        let (text, error) = match document_path.as_deref() {
            Some(path) => match fs::read_to_string(path) {
                Ok(text) => (text, None),
                Err(err) => (String::new(), Some(format!("Could not open: {err}"))),
            },
            None => (
                "Untitled Document\n\nWelcome to TextEdit. Start typing...".to_string(),
                None,
            ),
        };

        let mut toolbar = Toolbar::new();
        toolbar.add(Box::new(Button::new("NEW")));
        toolbar.add(Box::new(Button::new("SAVE")));

        let mut editor = TextField::new();
        editor.set_multiline(true);
        editor.set_text(text.clone());

        let mut view = Self {
            state: WidgetState::new(),
            toolbar,
            editor,
            status: Label::new(""),
            document_path,
            saved_text: text,
            dirty: false,
            last_error: error,
        };
        view.refresh_status();
        view
    }

    fn window_title(&self) -> String {
        let name = self
            .document_path
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        if self.dirty {
            format!("{name} - Edited - TextEdit")
        } else {
            format!("{name} - TextEdit")
        }
    }

    fn refresh_status(&mut self) {
        let path = self
            .document_path
            .as_deref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "No file path".to_string());
        let state = if self.dirty { "Edited" } else { "Saved" };
        let error = self
            .last_error
            .as_deref()
            .map(|error| format!(" - {error}"))
            .unwrap_or_default();
        self.status.text = format!("{state} - {path}{error}");
    }

    fn mark_dirty_from_editor(&mut self) {
        self.dirty = self.editor.text() != self.saved_text;
        self.last_error = None;
        self.refresh_status();
    }

    fn new_document(&mut self) -> bool {
        self.document_path = None;
        self.saved_text.clear();
        self.editor.set_text("");
        self.dirty = false;
        self.last_error = None;
        self.refresh_status();
        true
    }

    fn save_document(&mut self) -> bool {
        let Some(path) = self.document_path.as_deref() else {
            self.last_error = Some("Save needs a file path".to_string());
            self.refresh_status();
            return false;
        };

        match fs::write(path, self.editor.text()) {
            Ok(()) => {
                self.saved_text = self.editor.text().to_string();
                self.dirty = false;
                self.last_error = None;
                self.refresh_status();
                true
            }
            Err(err) => {
                self.last_error = Some(format!("Could not save: {err}"));
                self.refresh_status();
                false
            }
        }
    }

    fn handle_toolbar_click(&mut self, point: Point) -> bool {
        let Some(index) = self
            .toolbar
            .items
            .iter()
            .position(|item| item.rect().contains(point))
        else {
            return false;
        };

        match index {
            0 => self.new_document(),
            1 => self.save_document(),
            _ => false,
        }
    }
}

impl Widget for TextEditView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        let rect = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(rect);

        let toolbar_h = 32.0;
        let status_h = 24.0;
        let editor_h = (rect.height - toolbar_h - status_h).max(0.0);

        self.toolbar
            .set_rect(Rect::new(rect.x, rect.y, rect.width, toolbar_h));
        let _ = self
            .toolbar
            .layout(LayoutConstraint::tight(Size::new(rect.width, toolbar_h)));

        self.editor
            .set_rect(Rect::new(rect.x, rect.y + toolbar_h, rect.width, editor_h));
        let _ = self
            .editor
            .layout(LayoutConstraint::tight(Size::new(rect.width, editor_h)));

        self.status.set_rect(Rect::new(
            rect.x,
            rect.y + toolbar_h + editor_h,
            rect.width,
            status_h,
        ));
        let _ = self
            .status
            .layout(LayoutConstraint::tight(Size::new(rect.width, status_h)));

        size
    }

    fn draw(&self, theme: &ThemeContext) {
        self.toolbar.draw(theme);
        self.editor.draw(theme);
        self.status.draw(theme);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::KeyDown { key, modifiers } = event {
            if modifiers.meta {
                match key {
                    KeyCode::N => {
                        self.new_document();
                        return EventResult::Handled;
                    }
                    KeyCode::S => {
                        self.save_document();
                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }
        }

        if let Event::MouseDown {
            button: MouseButton::Left,
            point,
            ..
        } = event
        {
            if self.handle_toolbar_click(*point) {
                return EventResult::Handled;
            }
        }

        let result = self.editor.handle_event(event);
        if matches!(result, EventResult::Handled) {
            self.mark_dirty_from_editor();
        }
        result
    }

    fn update(&mut self) {
        self.toolbar.update();
        self.editor.update();
        self.status.update();
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&self.toolbar, &self.editor, &self.status]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![&mut self.toolbar, &mut self.editor, &mut self.status]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_textedit_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("retroshell_textedit_{unique}_{sequence}"))
            .join(name)
    }

    fn click_toolbar_button(view: &mut TextEditView, index: usize) -> EventResult {
        let rect = view.toolbar.items[index].rect();
        view.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
            modifiers: Modifiers::NONE,
        })
    }

    #[test]
    fn textedit_opens_existing_document_and_tracks_dirty_state() {
        let path = temp_textedit_path("note.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "hello").unwrap();

        let mut view = TextEditView::open(Some(path.clone()));
        assert_eq!(view.editor.text(), "hello");
        assert!(!view.dirty);
        assert!(view.status.text.contains("Saved"));

        let result = view.handle_event(&Event::Char { character: '!' });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(view.editor.text(), "hello!");
        assert!(view.dirty);
        assert!(view.status.text.contains("Edited"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_save_writes_document_and_clears_dirty_state() {
        let path = temp_textedit_path("note.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "hello").unwrap();

        let mut view = TextEditView::open(Some(path.clone()));
        view.layout(LayoutConstraint::tight(Size::new(640.0, 420.0)));
        let _ = view.handle_event(&Event::Char { character: '!' });

        let result = click_toolbar_button(&mut view, 1);

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello!");
        assert!(!view.dirty);
        assert!(view.status.text.contains("Saved"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_new_document_clears_text_and_path() {
        let path = temp_textedit_path("note.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "hello").unwrap();

        let mut view = TextEditView::open(Some(path.clone()));
        view.layout(LayoutConstraint::tight(Size::new(640.0, 420.0)));

        let result = click_toolbar_button(&mut view, 0);

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(view.editor.text(), "");
        assert!(view.document_path.is_none());
        assert!(!view.dirty);

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
