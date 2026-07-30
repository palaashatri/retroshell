//! wlr-layer-shell exclusive chrome driver for RetroShell (Phase 3).
//!
//! Three surfaces:
//! - **Background** — wallpaper + icons + in-shell windows
//! - **Top** menu bar — `exclusive_zone = menu_h`
//! - **Bottom** dock — `exclusive_zone = dock_h`
//!
//! Gated behind `RETROSHELL_LAYER_SHELL_CHROME`. Linux only.

#![cfg(target_os = "linux")]

use anyhow::anyhow;
use retro_kit::event::{KeyCode, MouseButton};
use retro_kit::{Event, Widget};
use retro_sdk::{RawSurfaceRenderer, UiRuntime};
use std::ffi::c_void;
use std::time::{SystemTime, UNIX_EPOCH};
use wayland_client::{
    protocol::{wl_keyboard, wl_pointer, wl_registry, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};

use crate::{ShellDesktop, ShellPaintFilter};

const MENU_H: u32 = 24;
const DOCK_H: u32 = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChromeSurfaceKind {
    Background,
    Menu,
    Dock,
}

/// Linux BTN_* codes from `<linux/input-event-codes.h>`.
fn mouse_button_from_linux(code: u32) -> Option<MouseButton> {
    match code {
        0x110 => Some(MouseButton::Left),
        0x111 => Some(MouseButton::Right),
        0x112 => Some(MouseButton::Middle),
        _ => None,
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Map Linux KEY_* (evdev) codes to kit KeyCode.
/// Covers letters/digits/modifiers needed for Super+O/L shell shortcuts and lock typing.
fn keycode_from_linux(code: u32) -> Option<KeyCode> {
    Some(match code {
        1 => KeyCode::Escape,
        2 => KeyCode::Key1,
        3 => KeyCode::Key2,
        4 => KeyCode::Key3,
        5 => KeyCode::Key4,
        6 => KeyCode::Key5,
        7 => KeyCode::Key6,
        8 => KeyCode::Key7,
        9 => KeyCode::Key8,
        10 => KeyCode::Key9,
        11 => KeyCode::Key0,
        12 => KeyCode::Minus,
        13 => KeyCode::Equals,
        14 => KeyCode::Backspace,
        15 => KeyCode::Tab,
        16 => KeyCode::Q,
        17 => KeyCode::W,
        18 => KeyCode::E,
        19 => KeyCode::R,
        20 => KeyCode::T,
        21 => KeyCode::Y,
        22 => KeyCode::U,
        23 => KeyCode::I,
        24 => KeyCode::O,
        25 => KeyCode::P,
        26 => KeyCode::LeftBracket,
        27 => KeyCode::RightBracket,
        28 => KeyCode::Enter,
        29 => KeyCode::ControlLeft,
        30 => KeyCode::A,
        31 => KeyCode::S,
        32 => KeyCode::D,
        33 => KeyCode::F,
        34 => KeyCode::G,
        35 => KeyCode::H,
        36 => KeyCode::J,
        37 => KeyCode::K,
        38 => KeyCode::L,
        39 => KeyCode::Semicolon,
        40 => KeyCode::Quote,
        42 => KeyCode::ShiftLeft,
        43 => KeyCode::Backslash,
        44 => KeyCode::Z,
        45 => KeyCode::X,
        46 => KeyCode::C,
        47 => KeyCode::V,
        48 => KeyCode::B,
        49 => KeyCode::N,
        50 => KeyCode::M,
        51 => KeyCode::Comma,
        52 => KeyCode::Period,
        53 => KeyCode::Slash,
        54 => KeyCode::ShiftRight,
        56 => KeyCode::AltLeft,
        57 => KeyCode::Space,
        58 => KeyCode::CapsLock,
        97 => KeyCode::ControlRight,
        100 => KeyCode::AltRight,
        102 => KeyCode::Home,
        103 => KeyCode::ArrowUp,
        104 => KeyCode::PageUp,
        105 => KeyCode::ArrowLeft,
        106 => KeyCode::ArrowRight,
        107 => KeyCode::End,
        108 => KeyCode::ArrowDown,
        109 => KeyCode::PageDown,
        111 => KeyCode::Delete,
        125 => KeyCode::MetaLeft,
        126 => KeyCode::MetaRight,
        _ => return None,
    })
}

/// Printable char for lock-password / text fields (US QWERTY, no full xkb).
fn char_from_keycode(code: KeyCode, shift: bool) -> Option<char> {
    let ch = match code {
        KeyCode::A => 'a',
        KeyCode::B => 'b',
        KeyCode::C => 'c',
        KeyCode::D => 'd',
        KeyCode::E => 'e',
        KeyCode::F => 'f',
        KeyCode::G => 'g',
        KeyCode::H => 'h',
        KeyCode::I => 'i',
        KeyCode::J => 'j',
        KeyCode::K => 'k',
        KeyCode::L => 'l',
        KeyCode::M => 'm',
        KeyCode::N => 'n',
        KeyCode::O => 'o',
        KeyCode::P => 'p',
        KeyCode::Q => 'q',
        KeyCode::R => 'r',
        KeyCode::S => 's',
        KeyCode::T => 't',
        KeyCode::U => 'u',
        KeyCode::V => 'v',
        KeyCode::W => 'w',
        KeyCode::X => 'x',
        KeyCode::Y => 'y',
        KeyCode::Z => 'z',
        KeyCode::Key0 => '0',
        KeyCode::Key1 => '1',
        KeyCode::Key2 => '2',
        KeyCode::Key3 => '3',
        KeyCode::Key4 => '4',
        KeyCode::Key5 => '5',
        KeyCode::Key6 => '6',
        KeyCode::Key7 => '7',
        KeyCode::Key8 => '8',
        KeyCode::Key9 => '9',
        KeyCode::Space => ' ',
        KeyCode::Minus => '-',
        KeyCode::Equals => '=',
        _ => return None,
    };
    Some(if shift {
        ch.to_ascii_uppercase()
    } else {
        ch
    })
}

/// Decode wl_keyboard mods_depressed (standard xkb mask) into kit Modifiers.
fn modifiers_from_xkb_mask(depressed: u32) -> retro_kit::event::Modifiers {
    retro_kit::event::Modifiers {
        shift: depressed & (1 << 0) != 0,
        control: depressed & (1 << 2) != 0,
        alt: depressed & (1 << 3) != 0,
        // Mod4 — Super/Logo/Meta on typical Linux layouts
        meta: depressed & (1 << 6) != 0,
    }
}

struct LayerSurf {
    kind: ChromeSurfaceKind,
    wl: wl_surface::WlSurface,
    layer: ZwlrLayerSurfaceV1,
    configured: Option<(u32, u32)>,
    renderer: Option<RawSurfaceRenderer>,
}

/// Main entry: exclusive Top/Bottom chrome + Background desktop.
pub fn run_layer_desktop(content: Box<dyn Widget>, width: u32, height: u32) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env().map_err(|e| anyhow!("wayland connect: {}", e))?;

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let display_obj = conn.display();
    let _registry = display_obj.get_registry(&qh, ());

    let mut state = LayerDesktopState {
        compositor: None,
        shm: None,
        layer_shell: None,
        seat: None,
        wl_pointer: None,
        wl_keyboard: None,
        surfaces: Vec::new(),
        runtime: None,
        output_w: width,
        output_h: height,
        running: true,
        last_pointer: (0.0, 0.0),
        pointer_kind: ChromeSurfaceKind::Background,
        modifiers: retro_kit::event::Modifiers::NONE,
    };

    event_queue
        .roundtrip(&mut state)
        .map_err(|e| anyhow!("registry roundtrip: {}", e))?;

    let compositor = state
        .compositor
        .clone()
        .ok_or_else(|| anyhow!("wl_compositor not found"))?;
    let layer_shell = state
        .layer_shell
        .clone()
        .ok_or_else(|| anyhow!("zwlr_layer_shell_v1 not found"))?;

    // Background — wallpaper + icons + in-shell windows
    let bg_wl = compositor.create_surface(&qh, ChromeSurfaceKind::Background);
    let bg_layer = layer_shell.get_layer_surface(
        &bg_wl,
        None,
        Layer::Background,
        "retroshell-desktop".into(),
        &qh,
        ChromeSurfaceKind::Background,
    );
    bg_layer.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
    bg_layer.set_exclusive_zone(0);
    bg_layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    bg_layer.set_size(width, height);
    bg_wl.commit();

    // Top menu — exclusive band
    let menu_wl = compositor.create_surface(&qh, ChromeSurfaceKind::Menu);
    let menu_layer = layer_shell.get_layer_surface(
        &menu_wl,
        None,
        Layer::Top,
        "retroshell-menu".into(),
        &qh,
        ChromeSurfaceKind::Menu,
    );
    menu_layer.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    menu_layer.set_exclusive_zone(MENU_H as i32);
    menu_layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    menu_layer.set_size(width, MENU_H);
    menu_wl.commit();

    // Bottom dock — exclusive band
    let dock_wl = compositor.create_surface(&qh, ChromeSurfaceKind::Dock);
    let dock_layer = layer_shell.get_layer_surface(
        &dock_wl,
        None,
        Layer::Bottom,
        "retroshell-dock".into(),
        &qh,
        ChromeSurfaceKind::Dock,
    );
    dock_layer.set_anchor(Anchor::Bottom | Anchor::Left | Anchor::Right);
    dock_layer.set_exclusive_zone(DOCK_H as i32);
    dock_layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    dock_layer.set_size(width, DOCK_H);
    dock_wl.commit();

    state.surfaces = vec![
        LayerSurf {
            kind: ChromeSurfaceKind::Background,
            wl: bg_wl,
            layer: bg_layer,
            configured: None,
            renderer: None,
        },
        LayerSurf {
            kind: ChromeSurfaceKind::Menu,
            wl: menu_wl,
            layer: menu_layer,
            configured: None,
            renderer: None,
        },
        LayerSurf {
            kind: ChromeSurfaceKind::Dock,
            wl: dock_wl,
            layer: dock_layer,
            configured: None,
            renderer: None,
        },
    ];

    // Wait until all three surfaces have a configure.
    for _ in 0..32 {
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| anyhow!("configure roundtrip: {}", e))?;
        if state.surfaces.iter().all(|s| s.configured.is_some()) {
            break;
        }
    }
    if !state.surfaces.iter().all(|s| s.configured.is_some()) {
        return Err(anyhow!("layer surfaces did not configure"));
    }

    let display_ptr = conn.backend().display_ptr() as *mut c_void;

    for surf in &mut state.surfaces {
        let (cw, ch) = surf.configured.unwrap_or(match surf.kind {
            ChromeSurfaceKind::Background => (width, height),
            ChromeSurfaceKind::Menu => (width, MENU_H),
            ChromeSurfaceKind::Dock => (width, DOCK_H),
        });
        let surface_ptr = surf.wl.id().as_ptr() as *mut c_void;
        let renderer = futures::executor::block_on(unsafe {
            RawSurfaceRenderer::new(display_ptr, surface_ptr, cw, ch)
        })
        .map_err(|e| anyhow!("RawSurfaceRenderer {:?}: {}", surf.kind, e))?;
        surf.renderer = Some(renderer);
    }

    let (desk_w, desk_h) = state
        .surfaces
        .iter()
        .find(|s| s.kind == ChromeSurfaceKind::Background)
        .and_then(|s| s.configured)
        .unwrap_or((width, height));
    state.output_w = desk_w;
    state.output_h = desk_h;

    let mut runtime = UiRuntime::new(content, desk_w, desk_h, 1.0);
    paint_all(&mut state, &mut runtime)?;
    state.runtime = Some(runtime);

    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(|e| anyhow!("dispatch: {}", e))?;

        if let Some(mut runtime) = state.runtime.take() {
            runtime.tick();
            if runtime.is_dirty() {
                paint_all(&mut state, &mut runtime)?;
            }
            state.runtime = Some(runtime);
        }
    }

    Ok(())
}

fn paint_all(state: &mut LayerDesktopState, runtime: &mut UiRuntime) -> anyhow::Result<()> {
    let out_w = state.output_w as f32;
    let menu_h = MENU_H as f32;
    let dock_h = DOCK_H as f32;

    // Background
    runtime.with_root_content_mut(|w| {
        if let Some(desktop) = w.as_any_mut().downcast_mut::<ShellDesktop>() {
            desktop.set_paint_filter(ShellPaintFilter::Background);
        }
    });
    if let Some(surf) = state
        .surfaces
        .iter_mut()
        .find(|s| s.kind == ChromeSurfaceKind::Background)
    {
        if let Some(renderer) = surf.renderer.as_mut() {
            runtime
                .paint(renderer)
                .map_err(|e| anyhow!("bg paint: {}", e))?;
            surf.wl.commit();
        }
    }

    // Menu strip
    runtime.with_root_content_mut(|w| {
        if let Some(desktop) = w.as_any_mut().downcast_mut::<ShellDesktop>() {
            desktop.prepare_menu_strip_layout(out_w, menu_h);
            desktop.set_paint_filter(ShellPaintFilter::MenuBar);
        }
    });
    if let Some(surf) = state
        .surfaces
        .iter_mut()
        .find(|s| s.kind == ChromeSurfaceKind::Menu)
    {
        if let Some(renderer) = surf.renderer.as_mut() {
            runtime
                .paint_ex(renderer, false, false)
                .map_err(|e| anyhow!("menu paint: {}", e))?;
            surf.wl.commit();
        }
    }

    // Dock strip
    runtime.with_root_content_mut(|w| {
        if let Some(desktop) = w.as_any_mut().downcast_mut::<ShellDesktop>() {
            desktop.prepare_dock_strip_layout(out_w, dock_h);
            desktop.set_paint_filter(ShellPaintFilter::Dock);
        }
    });
    if let Some(surf) = state
        .surfaces
        .iter_mut()
        .find(|s| s.kind == ChromeSurfaceKind::Dock)
    {
        if let Some(renderer) = surf.renderer.as_mut() {
            runtime
                .paint_ex(renderer, false, false)
                .map_err(|e| anyhow!("dock paint: {}", e))?;
            surf.wl.commit();
        }
    }

    // Restore full desktop layout so menu/dock hit-testing uses output coords.
    runtime.with_root_content_mut(|w| {
        if let Some(desktop) = w.as_any_mut().downcast_mut::<ShellDesktop>() {
            desktop.set_paint_filter(ShellPaintFilter::Background);
        }
    });
    runtime.resize(state.output_w, state.output_h, 1.0);

    Ok(())
}

fn map_pointer_to_desktop(
    kind: ChromeSurfaceKind,
    surface_x: f64,
    surface_y: f64,
    output_h: u32,
) -> (f32, f32) {
    match kind {
        ChromeSurfaceKind::Background | ChromeSurfaceKind::Menu => {
            (surface_x as f32, surface_y as f32)
        }
        ChromeSurfaceKind::Dock => {
            let y = (output_h.saturating_sub(DOCK_H) as f64) + surface_y;
            (surface_x as f32, y as f32)
        }
    }
}

struct LayerDesktopState {
    compositor: Option<wayland_client::protocol::wl_compositor::WlCompositor>,
    shm: Option<wayland_client::protocol::wl_shm::WlShm>,
    layer_shell: Option<ZwlrLayerShellV1>,
    seat: Option<wayland_client::protocol::wl_seat::WlSeat>,
    wl_pointer: Option<wayland_client::protocol::wl_pointer::WlPointer>,
    wl_keyboard: Option<wl_keyboard::WlKeyboard>,
    surfaces: Vec<LayerSurf>,
    runtime: Option<UiRuntime>,
    output_w: u32,
    output_h: u32,
    running: bool,
    last_pointer: (f64, f64),
    pointer_kind: ChromeSurfaceKind,
    /// Current keyboard modifiers from wl_keyboard::Modifiers (xkb mask).
    modifiers: retro_kit::event::Modifiers,
}

impl Dispatch<wl_registry::WlRegistry, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let v = version.min(4);
                    state.compositor = Some(registry.bind(name, v, qh, ()));
                }
                "wl_shm" => {
                    state.shm = Some(registry.bind(name, 1, qh, ()));
                }
                "zwlr_layer_shell_v1" => {
                    let v = version.min(4);
                    state.layer_shell = Some(registry.bind(name, v, qh, ()));
                }
                "wl_seat" => {
                    let v = version.min(9);
                    state.seat = Some(registry.bind(name, v, qh, ()));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wayland_client::protocol::wl_compositor::WlCompositor, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_compositor::WlCompositor,
        _: wayland_client::protocol::wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wayland_client::protocol::wl_shm::WlShm, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_shm::WlShm,
        _: wayland_client::protocol::wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wayland_client::protocol::wl_surface::WlSurface, ChromeSurfaceKind>
    for LayerDesktopState
{
    fn event(
        _: &mut Self,
        _: &wayland_client::protocol::wl_surface::WlSurface,
        _: wayland_client::protocol::wl_surface::Event,
        _: &ChromeSurfaceKind,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wayland_client::protocol::wl_seat::WlSeat, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        seat: &wayland_client::protocol::wl_seat::WlSeat,
        event: wayland_client::protocol::wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_seat::Event::Capabilities { capabilities } = event {
            if let WEnum::Value(caps) = capabilities {
                if caps.contains(wayland_client::protocol::wl_seat::Capability::Pointer)
                    && state.wl_pointer.is_none()
                {
                    state.wl_pointer = Some(seat.get_pointer(qh, ()));
                }
                if caps.contains(wayland_client::protocol::wl_seat::Capability::Keyboard)
                    && state.wl_keyboard.is_none()
                {
                    state.wl_keyboard = Some(seat.get_keyboard(qh, ()));
                }
            }
        }
    }
}

impl Dispatch<wayland_client::protocol::wl_pointer::WlPointer, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        _: &wayland_client::protocol::wl_pointer::WlPointer,
        event: wayland_client::protocol::wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.last_pointer = (surface_x, surface_y);
                let (dx, dy) = map_pointer_to_desktop(
                    state.pointer_kind,
                    surface_x,
                    surface_y,
                    state.output_h,
                );
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.pointer_moved(dx, dy);
                }
            }
            wl_pointer::Event::Button {
                button,
                state: btn_state,
                ..
            } => {
                let Some(mouse) = mouse_button_from_linux(button) else {
                    return;
                };
                let pressed = matches!(
                    btn_state,
                    WEnum::Value(wl_pointer::ButtonState::Pressed)
                );
                let (px, py) = state.last_pointer;
                let (dx, dy) =
                    map_pointer_to_desktop(state.pointer_kind, px, py, state.output_h);
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.pointer_moved(dx, dy);
                    let _ = runtime.pointer_button(mouse, pressed, now_ms());
                }
            }
            wl_pointer::Event::Enter {
                surface,
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_kind = state
                    .surfaces
                    .iter()
                    .find(|s| s.wl.id() == surface.id())
                    .map(|s| s.kind)
                    .unwrap_or(ChromeSurfaceKind::Background);
                state.last_pointer = (surface_x, surface_y);
                let (dx, dy) = map_pointer_to_desktop(
                    state.pointer_kind,
                    surface_x,
                    surface_y,
                    state.output_h,
                );
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.pointer_moved(dx, dy);
                    runtime.set_focus(true);
                }
            }
            wl_pointer::Event::Leave { .. } => {
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.set_focus(false);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for LayerDesktopState {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Modifiers {
                mods_depressed, ..
            } => {
                state.modifiers = modifiers_from_xkb_mask(mods_depressed);
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.set_modifiers(state.modifiers);
                }
            }
            wl_keyboard::Event::Key {
                key,
                state: key_state,
                ..
            } => {
                let Some(runtime) = state.runtime.as_mut() else {
                    return;
                };
                let Some(code) = keycode_from_linux(key) else {
                    return;
                };
                let pressed = matches!(
                    key_state,
                    WEnum::Value(wl_keyboard::KeyState::Pressed)
                );
                let mods = state.modifiers;
                let ev = if pressed {
                    Event::KeyDown {
                        key: code,
                        modifiers: mods,
                    }
                } else {
                    Event::KeyUp {
                        key: code,
                        modifiers: mods,
                    }
                };
                runtime.key(ev);
                // Feed printable chars for lock password / text fields.
                if pressed && !mods.control && !mods.alt && !mods.meta {
                    if let Some(ch) = char_from_keycode(code, mods.shift) {
                        runtime.key(Event::Char { character: ch });
                    }
                }
            }
            wl_keyboard::Event::Enter { .. } => {
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.set_focus(true);
                }
            }
            wl_keyboard::Event::Leave { .. } => {
                if let Some(runtime) = state.runtime.as_mut() {
                    runtime.set_focus(false);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for LayerDesktopState {
    fn event(
        _: &mut Self,
        _: &ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ChromeSurfaceKind> for LayerDesktopState {
    fn event(
        state: &mut Self,
        surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        kind: &ChromeSurfaceKind,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            let w = if width == 0 {
                state.output_w
            } else {
                width
            };
            let h = if height == 0 {
                match kind {
                    ChromeSurfaceKind::Menu => MENU_H,
                    ChromeSurfaceKind::Dock => DOCK_H,
                    ChromeSurfaceKind::Background => state.output_h,
                }
            } else {
                height
            };
            if let Some(surf) = state.surfaces.iter_mut().find(|s| s.kind == *kind) {
                surf.configured = Some((w, h));
            }
            surface.ack_configure(serial);
        }
    }
}
