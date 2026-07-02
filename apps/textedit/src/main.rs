use retro_kit::button::Button;
use retro_kit::clipboard::Clipboard;
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
    file_menu.add_action("Save As...").with_shortcut(
        KeyCode::S,
        Modifiers {
            shift: true,
            control: false,
            alt: false,
            meta: true,
        },
    );
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
    path_label: Label,
    path_field: TextField,
    editor: TextField,
    status: Label,
    document_path: Option<PathBuf>,
    saved_text: String,
    dirty: bool,
    last_error: Option<String>,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    path_focused: bool,
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
        toolbar.add(Box::new(Button::new("OPEN")));
        toolbar.add(Box::new(Button::new("SAVE")));
        toolbar.add(Box::new(Button::new("SAVE AS")));
        toolbar.add(Box::new(Button::new("UNDO")));
        toolbar.add(Box::new(Button::new("REDO")));
        toolbar.add(Box::new(Button::new("COPY")));
        toolbar.add(Box::new(Button::new("PASTE")));

        let mut path_field = TextField::new().with_placeholder("Document path");
        path_field.set_expands_horizontally(true);
        if let Some(path) = document_path.as_deref() {
            path_field.set_text(path.display().to_string());
        }

        let mut editor = TextField::new();
        editor.set_multiline(true);
        editor.set_text(text.clone());

        let mut view = Self {
            state: WidgetState::new(),
            toolbar,
            path_label: Label::new("PATH"),
            path_field,
            editor,
            status: Label::new(""),
            document_path,
            saved_text: text,
            dirty: false,
            last_error: error,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            path_focused: false,
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

    fn sync_path_field(&mut self) {
        if let Some(path) = self.document_path.as_deref() {
            self.path_field.set_text(path.display().to_string());
        } else {
            self.path_field.set_text("");
        }
    }

    fn mark_dirty_from_editor(&mut self) {
        self.dirty = self.editor.text() != self.saved_text;
        self.last_error = None;
        self.refresh_status();
    }

    fn push_undo_snapshot(&mut self) {
        let current = self.editor.text().to_string();
        if self.undo_stack.last() != Some(&current) {
            self.undo_stack.push(current);
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
        }
        self.redo_stack.clear();
    }

    fn replace_editor_text(&mut self, text: String) {
        self.editor.set_text(text);
        self.mark_dirty_from_editor();
    }

    fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_stack.pop() else {
            self.last_error = Some("Nothing to undo".to_string());
            self.refresh_status();
            return false;
        };
        self.redo_stack.push(self.editor.text().to_string());
        self.replace_editor_text(previous);
        true
    }

    fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            self.last_error = Some("Nothing to redo".to_string());
            self.refresh_status();
            return false;
        };
        self.undo_stack.push(self.editor.text().to_string());
        self.replace_editor_text(next);
        true
    }

    fn copy_document(&mut self) -> bool {
        Clipboard::copy(self.editor.text());
        self.last_error = None;
        self.refresh_status();
        true
    }

    fn cut_document(&mut self) -> bool {
        if self.editor.text().is_empty() {
            return self.copy_document();
        }
        self.push_undo_snapshot();
        Clipboard::copy(self.editor.text());
        self.replace_editor_text(String::new());
        true
    }

    fn paste_document(&mut self) -> bool {
        let pasted = Clipboard::paste();
        if pasted.is_empty() {
            self.last_error = Some("Clipboard empty".to_string());
            self.refresh_status();
            return false;
        }
        self.push_undo_snapshot();
        let mut text = self.editor.text().to_string();
        text.push_str(&pasted);
        self.replace_editor_text(text);
        true
    }

    fn select_all_document(&mut self) -> bool {
        self.copy_document()
    }

    fn new_document(&mut self) -> bool {
        self.push_undo_snapshot();
        self.document_path = None;
        self.sync_path_field();
        self.saved_text.clear();
        self.editor.set_text("");
        self.dirty = false;
        self.last_error = None;
        self.redo_stack.clear();
        self.refresh_status();
        true
    }

    fn path_from_field(&mut self) -> Option<PathBuf> {
        let path = self.path_field.text().trim();
        if path.is_empty() {
            self.last_error = Some("Enter a document path".to_string());
            self.refresh_status();
            None
        } else {
            Some(PathBuf::from(path))
        }
    }

    fn open_path(&mut self, path: PathBuf) -> bool {
        match fs::read_to_string(&path) {
            Ok(text) => {
                self.push_undo_snapshot();
                self.document_path = Some(path);
                self.sync_path_field();
                self.editor.set_text(text.clone());
                self.saved_text = text;
                self.dirty = false;
                self.last_error = None;
                self.redo_stack.clear();
                self.refresh_status();
                true
            }
            Err(err) => {
                self.last_error = Some(format!("Could not open: {err}"));
                self.refresh_status();
                false
            }
        }
    }

    fn open_from_path_field(&mut self) -> bool {
        let Some(path) = self.path_from_field() else {
            return false;
        };
        self.open_path(path)
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

    fn save_as_from_path_field(&mut self) -> bool {
        let Some(path) = self.path_from_field() else {
            return false;
        };
        match fs::write(&path, self.editor.text()) {
            Ok(()) => {
                self.document_path = Some(path);
                self.sync_path_field();
                self.saved_text = self.editor.text().to_string();
                self.dirty = false;
                self.last_error = None;
                self.refresh_status();
                true
            }
            Err(err) => {
                self.last_error = Some(format!("Could not save as: {err}"));
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
            1 => self.open_from_path_field(),
            2 => self.save_document(),
            3 => self.save_as_from_path_field(),
            4 => self.undo(),
            5 => self.redo(),
            6 => self.copy_document(),
            7 => self.paste_document(),
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
        let path_h = 30.0;
        let status_h = 24.0;
        let editor_h = (rect.height - toolbar_h - path_h - status_h).max(0.0);

        self.toolbar
            .set_rect(Rect::new(rect.x, rect.y, rect.width, toolbar_h));
        let _ = self
            .toolbar
            .layout(LayoutConstraint::tight(Size::new(rect.width, toolbar_h)));

        self.path_label.set_rect(Rect::new(
            rect.x + 8.0,
            rect.y + toolbar_h + 4.0,
            46.0,
            22.0,
        ));
        let _ = self
            .path_label
            .layout(LayoutConstraint::tight(Size::new(46.0, 22.0)));

        let path_field_x = rect.x + 58.0;
        let path_field_w = (rect.width - 66.0).max(0.0);
        self.path_field.set_rect(Rect::new(
            path_field_x,
            rect.y + toolbar_h + 2.0,
            path_field_w,
            26.0,
        ));
        let _ = self
            .path_field
            .layout(LayoutConstraint::tight(Size::new(path_field_w, 26.0)));

        self.editor.set_rect(Rect::new(
            rect.x,
            rect.y + toolbar_h + path_h,
            rect.width,
            editor_h,
        ));
        let _ = self
            .editor
            .layout(LayoutConstraint::tight(Size::new(rect.width, editor_h)));

        self.status.set_rect(Rect::new(
            rect.x,
            rect.y + toolbar_h + path_h + editor_h,
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
        self.path_label.draw(theme);
        self.path_field.draw(theme);
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
                        if modifiers.shift {
                            self.save_as_from_path_field();
                        } else {
                            self.save_document();
                        }
                        return EventResult::Handled;
                    }
                    KeyCode::O => {
                        self.open_from_path_field();
                        return EventResult::Handled;
                    }
                    KeyCode::Z if modifiers.shift => {
                        self.redo();
                        return EventResult::Handled;
                    }
                    KeyCode::Z => {
                        self.undo();
                        return EventResult::Handled;
                    }
                    KeyCode::X => {
                        self.cut_document();
                        return EventResult::Handled;
                    }
                    KeyCode::C => {
                        self.copy_document();
                        return EventResult::Handled;
                    }
                    KeyCode::V => {
                        self.paste_document();
                        return EventResult::Handled;
                    }
                    KeyCode::A => {
                        self.select_all_document();
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
            if self.path_field.rect().contains(*point) {
                self.path_focused = true;
                return EventResult::Handled;
            }
            if self.editor.rect().contains(*point) {
                self.path_focused = false;
            }
        }

        if self.path_focused {
            match event {
                Event::Char { .. }
                | Event::KeyDown {
                    key: KeyCode::Backspace,
                    ..
                } => {
                    let result = self.path_field.handle_event(event);
                    if matches!(result, EventResult::Handled) {
                        self.last_error = None;
                        self.refresh_status();
                    }
                    return result;
                }
                _ => {}
            }
        }

        let before_edit = self.editor.text().to_string();
        let result = self.editor.handle_event(event);
        if matches!(result, EventResult::Handled) {
            if self.editor.text() != before_edit {
                if self.undo_stack.last() != Some(&before_edit) {
                    self.undo_stack.push(before_edit);
                    if self.undo_stack.len() > 100 {
                        self.undo_stack.remove(0);
                    }
                }
                self.redo_stack.clear();
            }
            self.mark_dirty_from_editor();
        }
        result
    }

    fn update(&mut self) {
        self.toolbar.update();
        self.path_label.update();
        self.path_field.update();
        self.editor.update();
        self.status.update();
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        None
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![
            &self.toolbar,
            &self.path_label,
            &self.path_field,
            &self.editor,
            &self.status,
        ]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![
            &mut self.toolbar,
            &mut self.path_label,
            &mut self.path_field,
            &mut self.editor,
            &mut self.status,
        ]
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

        let result = click_toolbar_button(&mut view, 2);

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello!");
        assert!(!view.dirty);
        assert!(view.status.text.contains("Saved"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_open_toolbar_loads_path_field_document() {
        let path = temp_textedit_path("open.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "opened from path").unwrap();

        let mut view = TextEditView::open(None);
        view.layout(LayoutConstraint::tight(Size::new(700.0, 460.0)));
        view.path_field.set_text(path.display().to_string());

        let result = click_toolbar_button(&mut view, 1);

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(view.editor.text(), "opened from path");
        assert_eq!(view.document_path.as_deref(), Some(path.as_path()));
        assert!(!view.dirty);
        assert!(view.status.text.contains("Saved"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_save_as_toolbar_writes_path_field_document() {
        let path = temp_textedit_path("saved-as.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut view = TextEditView::open(None);
        view.layout(LayoutConstraint::tight(Size::new(700.0, 460.0)));
        view.editor.set_text("save as body");
        view.mark_dirty_from_editor();
        view.path_field.set_text(path.display().to_string());

        let result = click_toolbar_button(&mut view, 3);

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(fs::read_to_string(&path).unwrap(), "save as body");
        assert_eq!(view.document_path.as_deref(), Some(path.as_path()));
        assert!(!view.dirty);
        assert!(view.status.text.contains("Saved"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_shift_cmd_s_runs_save_as() {
        let path = temp_textedit_path("shortcut-save-as.txt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut view = TextEditView::open(None);
        view.editor.set_text("shortcut body");
        view.mark_dirty_from_editor();
        view.path_field.set_text(path.display().to_string());

        let result = view.handle_event(&Event::KeyDown {
            key: KeyCode::S,
            modifiers: Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        });

        assert!(matches!(result, EventResult::Handled));
        assert_eq!(fs::read_to_string(&path).unwrap(), "shortcut body");
        assert_eq!(view.document_path.as_deref(), Some(path.as_path()));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_path_field_accepts_typed_path_when_focused() {
        let mut view = TextEditView::open(None);
        view.layout(LayoutConstraint::tight(Size::new(700.0, 460.0)));
        let rect = view.path_field.rect();

        let focus = view.handle_event(&Event::MouseDown {
            button: MouseButton::Left,
            point: Point::new(rect.x + 4.0, rect.y + 4.0),
            modifiers: Modifiers::NONE,
        });
        let typed = view.handle_event(&Event::Char { character: 'x' });

        assert!(matches!(focus, EventResult::Handled));
        assert!(matches!(typed, EventResult::Handled));
        assert_eq!(view.path_field.text(), "x");
        assert!(!view.editor.text().ends_with('x'));
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
        assert_eq!(view.path_field.text(), "");
        assert!(!view.dirty);

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn textedit_undo_and_redo_restore_editor_text() {
        let mut view = TextEditView::open(None);
        view.editor.set_text("hello");
        view.saved_text = "hello".to_string();
        view.dirty = false;

        let _ = view.handle_event(&Event::Char { character: '!' });
        assert_eq!(view.editor.text(), "hello!");

        let undo = view.handle_event(&Event::KeyDown {
            key: KeyCode::Z,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });
        assert!(matches!(undo, EventResult::Handled));
        assert_eq!(view.editor.text(), "hello");
        assert!(!view.dirty);

        let redo = view.handle_event(&Event::KeyDown {
            key: KeyCode::Z,
            modifiers: Modifiers {
                shift: true,
                control: false,
                alt: false,
                meta: true,
            },
        });
        assert!(matches!(redo, EventResult::Handled));
        assert_eq!(view.editor.text(), "hello!");
        assert!(view.dirty);
    }

    #[test]
    fn textedit_copy_cut_and_paste_use_clipboard() {
        Clipboard::clear();
        let mut view = TextEditView::open(None);
        view.editor.set_text("clip");
        view.saved_text.clear();
        view.dirty = true;

        let copy = view.handle_event(&Event::KeyDown {
            key: KeyCode::C,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });
        assert!(matches!(copy, EventResult::Handled));
        assert_eq!(Clipboard::paste(), "clip");

        let cut = view.handle_event(&Event::KeyDown {
            key: KeyCode::X,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });
        assert!(matches!(cut, EventResult::Handled));
        assert_eq!(view.editor.text(), "");
        assert_eq!(Clipboard::paste(), "clip");

        let paste = view.handle_event(&Event::KeyDown {
            key: KeyCode::V,
            modifiers: Modifiers {
                shift: false,
                control: false,
                alt: false,
                meta: true,
            },
        });
        assert!(matches!(paste, EventResult::Handled));
        assert_eq!(view.editor.text(), "clip");
    }
}
