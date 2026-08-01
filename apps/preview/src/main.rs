//! Preview — the native SLOPOS image viewer.

mod viewer;

use slopos_kit::event::{KeyCode, Modifiers};
use slopos_kit::window::Window;
use slopos_kit::Size;
use slopos_sdk::{build_menu, Application};
use viewer::{parse_cli_args, PreviewView};

const BUNDLE_ID: &str = "com.slopos.preview";
const ACTION_OPEN: &str = "com.slopos.preview.file.open";
const ACTION_ZOOM_IN: &str = "com.slopos.preview.zoom.in";
const ACTION_ZOOM_OUT: &str = "com.slopos.preview.zoom.out";
const ACTION_ZOOM_FIT: &str = "com.slopos.preview.zoom.fit";
const ACTION_ZOOM_ACTUAL: &str = "com.slopos.preview.zoom.actual_size";
const ACTION_EXTRACT_TEXT: &str = "com.slopos.preview.vision.extract_text";
const ACTION_LIFT_SUBJECT: &str = "com.slopos.preview.vision.lift_subject";

fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let image_path = match parse_cli_args(std::env::args_os()) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("preview: {error}");
            eprintln!("usage: preview [IMAGE_PATH]");
            std::process::exit(2);
        }
    };

    let mut app = Application::new("Preview", BUNDLE_ID);
    app.set_initial_size(Size::new(960.0, 700.0));
    app.set_menus(preview_menus());
    app.on_menu_action(|action, window| {
        let Some(content) = window.content.as_mut() else {
            return;
        };
        let Some(view) = content.as_any_mut().downcast_mut::<PreviewView>() else {
            return;
        };
        view.handle_action(action);
    });

    let mut view = PreviewView::new(image_path);
    view.set_event_loop_waker(app.event_waker());
    let window_title = view.window_title();
    let mut window = Window::new(window_title);
    window.set_content(Box::new(view));
    app.set_main_window(window);
    app.run();
}

fn preview_menus() -> Vec<slopos_kit::Menu> {
    let mut file = build_menu("File");
    file.add_action("Open...")
        .with_shortcut(KeyCode::O, meta_shortcut())
        .with_action(ACTION_OPEN);

    let mut zoom = build_menu("Zoom");
    zoom.add_action("Zoom In")
        .with_shortcut(KeyCode::Equals, meta_shortcut())
        .with_action(ACTION_ZOOM_IN);
    zoom.add_action("Zoom Out")
        .with_shortcut(KeyCode::Minus, meta_shortcut())
        .with_action(ACTION_ZOOM_OUT);
    zoom.add_action("Fit to Window")
        .with_action(ACTION_ZOOM_FIT);
    zoom.add_action("Actual Size")
        .with_shortcut(KeyCode::Key0, meta_shortcut())
        .with_action(ACTION_ZOOM_ACTUAL);

    let mut vision = build_menu("Vision");
    vision
        .add_action("Extract Text")
        .with_action(ACTION_EXTRACT_TEXT);
    vision
        .add_action("Lift Subject")
        .with_action(ACTION_LIFT_SUBJECT);

    vec![file, zoom, vision]
}

const fn meta_shortcut() -> Modifiers {
    Modifiers {
        shift: false,
        control: false,
        alt: false,
        meta: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn cli_accepts_no_path_or_one_path() {
        assert_eq!(parse_cli_args([OsString::from("preview")]).unwrap(), None);
        assert_eq!(
            parse_cli_args([OsString::from("preview"), OsString::from("photo.png")])
                .unwrap()
                .as_deref(),
            Some(std::path::Path::new("photo.png"))
        );
    }

    #[test]
    fn cli_rejects_multiple_paths() {
        let error = parse_cli_args([
            OsString::from("preview"),
            OsString::from("one.png"),
            OsString::from("two.png"),
        ])
        .unwrap_err();
        assert!(error.contains("one image path"));
    }
}
