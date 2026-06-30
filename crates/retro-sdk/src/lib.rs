use retro_bus::RetroBus;
use retro_kit::event::{Modifiers, MouseButton};
use retro_kit::menu::{Menu, MenuItem};
use retro_kit::window::Window;
use retro_kit::{LayoutConstraint, Point, Size, Widget};

pub struct Application {
    pub name: String,
    pub bundle_id: String,
    pub main_window: Option<Window>,
    pub menus: Vec<Menu>,
    pub bus: Option<RetroBus>,
    pub running: bool,
}

impl Application {
    pub fn new(name: &str, bundle_id: &str) -> Self {
        Self {
            name: name.to_string(),
            bundle_id: bundle_id.to_string(),
            main_window: None,
            menus: vec![],
            bus: None,
            running: false,
        }
    }

    pub fn with_bus(mut self, bus: RetroBus) -> Self {
        self.bus = Some(bus);
        self
    }

    pub fn set_main_window(&mut self, window: Window) {
        self.main_window = Some(window);
    }

    pub fn set_menus(&mut self, menus: Vec<Menu>) {
        self.menus = menus;
    }

    pub fn run(&mut self) {
        self.running = true;
        tracing::info!("Application '{}' started", self.name);

        let event_loop = retro_render::event_loop::RetroEventLoop::new();
        let main_window = self.main_window.take();

        struct AppHandler {
            name: String,
            window: Option<Window>,
            platform_window: Option<winit::window::Window>,
            modifiers: winit::keyboard::ModifiersState,
            cursor_position: Point,
            last_click: Option<(MouseButton, Point, std::time::Instant)>,
        }
        impl AppHandler {
            fn modifiers(&self) -> Modifiers {
                modifiers_from_winit(self.modifiers)
            }

            fn dispatch(&mut self, event: retro_kit::Event) -> retro_kit::EventResult {
                if let Some(ref mut win) = self.window {
                    win.handle_event(&event)
                } else {
                    retro_kit::EventResult::Ignored
                }
            }

            fn layout_window(&mut self, width: u32, height: u32) {
                if let Some(ref mut win) = self.window {
                    let size = Size::new(width as f32, height as f32);
                    win.layout(LayoutConstraint::tight(size));
                }
            }
        }

        impl retro_render::event_loop::RetroAppHandler for AppHandler {
            fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
                let attrs = winit::window::Window::default_attributes().with_title(&self.name);
                match event_loop.create_window(attrs) {
                    Ok(window) => {
                        let size = window.inner_size();
                        self.layout_window(size.width, size.height);
                        self.platform_window = Some(window);
                    }
                    Err(err) => {
                        tracing::error!("failed to create application window: {err}");
                        event_loop.exit();
                    }
                }
            }

            fn handle_window_event(
                &mut self,
                event_loop: &winit::event_loop::ActiveEventLoop,
                event: winit::event::WindowEvent,
            ) {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    winit::event::WindowEvent::Resized(size) => {
                        self.layout_window(size.width, size.height);
                    }
                    winit::event::WindowEvent::ModifiersChanged(new_mods) => {
                        self.modifiers = new_mods.state();
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        self.cursor_position = Point::new(position.x as f32, position.y as f32);
                        let _ = self.dispatch(retro_kit::Event::MouseMove {
                            point: self.cursor_position,
                            modifiers: self.modifiers(),
                        });
                    }
                    winit::event::WindowEvent::CursorEntered { .. } => {
                        let _ = self.dispatch(retro_kit::Event::MouseEnter);
                    }
                    winit::event::WindowEvent::CursorLeft { .. } => {
                        let _ = self.dispatch(retro_kit::Event::MouseLeave);
                    }
                    winit::event::WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(button) = winit_to_retro_mouse_button(button) {
                            let event = match state {
                                winit::event::ElementState::Pressed => {
                                    let now = std::time::Instant::now();
                                    let is_double_click = self
                                        .last_click
                                        .as_ref()
                                        .map(|(last_button, last_point, last_time)| {
                                            *last_button == button
                                                && now.duration_since(*last_time)
                                                    <= std::time::Duration::from_millis(500)
                                                && distance_squared(
                                                    *last_point,
                                                    self.cursor_position,
                                                ) <= 16.0
                                        })
                                        .unwrap_or(false);
                                    self.last_click = Some((button, self.cursor_position, now));
                                    if is_double_click {
                                        retro_kit::Event::DoubleClick {
                                            button,
                                            point: self.cursor_position,
                                            modifiers: self.modifiers(),
                                        }
                                    } else {
                                        retro_kit::Event::MouseDown {
                                            button,
                                            point: self.cursor_position,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                }
                                winit::event::ElementState::Released => retro_kit::Event::MouseUp {
                                    button,
                                    point: self.cursor_position,
                                    modifiers: self.modifiers(),
                                },
                            };
                            let _ = self.dispatch(event);
                        }
                    }
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        let delta = winit_to_retro_scroll_delta(delta);
                        let _ = self.dispatch(retro_kit::Event::Scroll {
                            delta,
                            modifiers: self.modifiers(),
                        });
                    }
                    winit::event::WindowEvent::Focused(true) => {
                        let _ = self.dispatch(retro_kit::Event::FocusIn);
                    }
                    winit::event::WindowEvent::Focused(false) => {
                        let _ = self.dispatch(retro_kit::Event::FocusOut);
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        event: key_event, ..
                    } => {
                        let mut handled = false;
                        if let winit::keyboard::PhysicalKey::Code(phys_key) = key_event.physical_key
                        {
                            if let Some(rkey) = winit_to_retro_key(phys_key) {
                                let retro_event = match key_event.state {
                                    winit::event::ElementState::Pressed => {
                                        retro_kit::Event::KeyDown {
                                            key: rkey,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                    winit::event::ElementState::Released => {
                                        retro_kit::Event::KeyUp {
                                            key: rkey,
                                            modifiers: self.modifiers(),
                                        }
                                    }
                                };
                                handled = matches!(
                                    self.dispatch(retro_event),
                                    retro_kit::EventResult::Handled
                                        | retro_kit::EventResult::StopPropagation
                                );
                            }
                        }
                        if key_event.state == winit::event::ElementState::Pressed && !handled {
                            if let Some(ref text) = key_event.text {
                                for character in text.chars() {
                                    let _ = self.dispatch(retro_kit::Event::Char { character });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
                if let Some(ref mut win) = self.window {
                    win.update();
                }
            }
        }

        let mut handler = AppHandler {
            name: self.name.clone(),
            window: main_window,
            platform_window: None,
            modifiers: winit::keyboard::ModifiersState::default(),
            cursor_position: Point::ZERO,
            last_click: None,
        };
        let _ = event_loop.run(&mut handler);
    }

    pub fn quit(&mut self) {
        self.running = false;
        tracing::info!("Application '{}' quit", self.name);
    }
}

pub trait AppDelegate {
    fn app_did_finish_launching(&mut self);
    fn app_will_terminate(&mut self);
    fn app_did_resign_active(&mut self);
    fn app_did_become_active(&mut self);
}

pub fn build_menu(title: &str) -> Menu {
    Menu::new(title)
}

pub fn menu_item(label: &str, action: &str) -> MenuItem {
    let mut item = MenuItem::action(label);
    item.with_action(action);
    item
}

pub fn separator() -> MenuItem {
    MenuItem::separator()
}

pub fn modifiers_from_winit(modifiers: winit::keyboard::ModifiersState) -> Modifiers {
    Modifiers {
        shift: modifiers.shift_key(),
        control: modifiers.control_key(),
        alt: modifiers.alt_key(),
        meta: modifiers.super_key(),
    }
}

pub fn winit_to_retro_mouse_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Back => Some(MouseButton::Back),
        winit::event::MouseButton::Forward => Some(MouseButton::Forward),
        winit::event::MouseButton::Other(_) => None,
    }
}

pub fn winit_to_retro_scroll_delta(delta: winit::event::MouseScrollDelta) -> Point {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(x, y) => Point::new(x * 16.0, y * 16.0),
        winit::event::MouseScrollDelta::PixelDelta(position) => {
            Point::new(position.x as f32, position.y as f32)
        }
    }
}

fn distance_squared(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

pub fn winit_to_retro_key(key: winit::keyboard::KeyCode) -> Option<retro_kit::event::KeyCode> {
    use retro_kit::event::KeyCode as RKey;
    use winit::keyboard::KeyCode as WKey;
    match key {
        WKey::KeyA => Some(RKey::A),
        WKey::KeyB => Some(RKey::B),
        WKey::KeyC => Some(RKey::C),
        WKey::KeyD => Some(RKey::D),
        WKey::KeyE => Some(RKey::E),
        WKey::KeyF => Some(RKey::F),
        WKey::KeyG => Some(RKey::G),
        WKey::KeyH => Some(RKey::H),
        WKey::KeyI => Some(RKey::I),
        WKey::KeyJ => Some(RKey::J),
        WKey::KeyK => Some(RKey::K),
        WKey::KeyL => Some(RKey::L),
        WKey::KeyM => Some(RKey::M),
        WKey::KeyN => Some(RKey::N),
        WKey::KeyO => Some(RKey::O),
        WKey::KeyP => Some(RKey::P),
        WKey::KeyQ => Some(RKey::Q),
        WKey::KeyR => Some(RKey::R),
        WKey::KeyS => Some(RKey::S),
        WKey::KeyT => Some(RKey::T),
        WKey::KeyU => Some(RKey::U),
        WKey::KeyV => Some(RKey::V),
        WKey::KeyW => Some(RKey::W),
        WKey::KeyX => Some(RKey::X),
        WKey::KeyY => Some(RKey::Y),
        WKey::KeyZ => Some(RKey::Z),
        WKey::Digit0 => Some(RKey::Key0),
        WKey::Digit1 => Some(RKey::Key1),
        WKey::Digit2 => Some(RKey::Key2),
        WKey::Digit3 => Some(RKey::Key3),
        WKey::Digit4 => Some(RKey::Key4),
        WKey::Digit5 => Some(RKey::Key5),
        WKey::Digit6 => Some(RKey::Key6),
        WKey::Digit7 => Some(RKey::Key7),
        WKey::Digit8 => Some(RKey::Key8),
        WKey::Digit9 => Some(RKey::Key9),
        WKey::F1 => Some(RKey::F1),
        WKey::F2 => Some(RKey::F2),
        WKey::F3 => Some(RKey::F3),
        WKey::F4 => Some(RKey::F4),
        WKey::F5 => Some(RKey::F5),
        WKey::F6 => Some(RKey::F6),
        WKey::F7 => Some(RKey::F7),
        WKey::F8 => Some(RKey::F8),
        WKey::F9 => Some(RKey::F9),
        WKey::F10 => Some(RKey::F10),
        WKey::F11 => Some(RKey::F11),
        WKey::F12 => Some(RKey::F12),
        WKey::Escape => Some(RKey::Escape),
        WKey::Tab => Some(RKey::Tab),
        WKey::CapsLock => Some(RKey::CapsLock),
        WKey::ShiftLeft => Some(RKey::ShiftLeft),
        WKey::ShiftRight => Some(RKey::ShiftRight),
        WKey::ControlLeft => Some(RKey::ControlLeft),
        WKey::ControlRight => Some(RKey::ControlRight),
        WKey::AltLeft => Some(RKey::AltLeft),
        WKey::AltRight => Some(RKey::AltRight),
        WKey::Space => Some(RKey::Space),
        WKey::Enter => Some(RKey::Enter),
        WKey::Backspace => Some(RKey::Backspace),
        WKey::Delete => Some(RKey::Delete),
        WKey::Insert => Some(RKey::Insert),
        WKey::Home => Some(RKey::Home),
        WKey::End => Some(RKey::End),
        WKey::PageUp => Some(RKey::PageUp),
        WKey::PageDown => Some(RKey::PageDown),
        WKey::ArrowUp => Some(RKey::ArrowUp),
        WKey::ArrowDown => Some(RKey::ArrowDown),
        WKey::ArrowLeft => Some(RKey::ArrowLeft),
        WKey::ArrowRight => Some(RKey::ArrowRight),
        WKey::SuperLeft => Some(RKey::MetaLeft),
        WKey::SuperRight => Some(RKey::MetaRight),
        WKey::Minus => Some(RKey::Minus),
        WKey::Equal => Some(RKey::Equals),
        WKey::BracketLeft => Some(RKey::LeftBracket),
        WKey::BracketRight => Some(RKey::RightBracket),
        WKey::Backslash => Some(RKey::Backslash),
        WKey::Semicolon => Some(RKey::Semicolon),
        WKey::Quote => Some(RKey::Quote),
        WKey::Comma => Some(RKey::Comma),
        WKey::Period => Some(RKey::Period),
        WKey::Slash => Some(RKey::Slash),
        _ => None,
    }
}
