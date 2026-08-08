// Copyright (c) 2026 Palaash Atri
// SPDX-License-Identifier: MIT

//! Native Wayland clipboard source/sink used by the compositor runtime gate.
//!
//! The source and sink are separate client processes. The source creates a real
//! `wl_data_source`, offers two text MIME types, and keeps its focused toplevel
//! alive. The sink creates another toplevel, receives the selection offer, reads
//! the exact payload, and verifies that an unsupported MIME request terminates
//! with EOF. This is protocol/runtime evidence only; it is not GTK, Qt,
//! XWayland, physical-input, DnD or hardware compatibility evidence.

use std::{
    env,
    error::Error,
    io::{Read, Write},
    os::fd::AsFd,
    os::unix::net::UnixStream,
    thread,
    time::Duration,
};

use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{
    wl_compositor, wl_data_device, wl_data_device_manager, wl_data_offer, wl_data_source,
    wl_registry, wl_seat, wl_surface,
};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

const MIME_TEXT_UTF8: &str = "text/plain;charset=utf-8";
const MIME_TEXT: &str = "text/plain";
const MIME_LARGE: &str = "application/x-slopos-large";
const MIME_MISSING: &str = "application/x-slopos-missing";
const PAYLOAD: &[u8] = b"SLOPOS native clipboard transfer\nUTF-8: cafe\xCC\x81\n";
const LARGE_PAYLOAD_SIZE: usize = 1024 * 1024;
static LARGE_PAYLOAD: [u8; LARGE_PAYLOAD_SIZE] = [b'L'; LARGE_PAYLOAD_SIZE];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Source,
    SourceOnce,
    Sink,
    SinkAfterSourceDeath,
}

#[derive(Default)]
struct State {
    toplevel_configured: bool,
    close_requested: bool,
    data_offer: Option<wl_data_offer::WlDataOffer>,
    offered_mimes: Vec<String>,
    selection_received: bool,
    source_send_count: u32,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
    }
}

wayland_client::delegate_noop!(State: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(State: ignore wl_seat::WlSeat);
wayland_client::delegate_noop!(State: ignore wl_surface::WlSurface);

impl Dispatch<wl_data_device_manager::WlDataDeviceManager, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_data_device_manager::WlDataDeviceManager,
        _event: wl_data_device_manager::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_data_device::WlDataDevice, ()> for State {
    wayland_client::event_created_child!(State, wl_data_device::WlDataDevice, [
        0 => (wl_data_offer::WlDataOffer, ())
    ]);

    fn event(
        state: &mut Self,
        _proxy: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => state.data_offer = Some(id),
            wl_data_device::Event::Selection { id } => {
                state.data_offer = id;
                state.selection_received = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_data_offer::WlDataOffer, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_data_offer::WlDataOffer,
        event: wl_data_offer::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        if let wl_data_offer::Event::Offer { mime_type } = event {
            if state.data_offer.as_ref().is_none_or(|offer| offer == proxy) {
                state.data_offer = Some(proxy.clone());
                state.offered_mimes.push(mime_type);
            }
        }
    }
}

impl Dispatch<wl_data_source::WlDataSource, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &wl_data_source::WlDataSource,
        event: wl_data_source::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_source::Event::Send { mime_type, fd } => {
                let mut file = std::fs::File::from(fd);
                let data = match mime_type.as_str() {
                    MIME_TEXT_UTF8 | MIME_TEXT => Some(PAYLOAD),
                    MIME_LARGE => Some(&LARGE_PAYLOAD[..]),
                    _ => None,
                };
                if let Some(data) = data {
                    let _ = file.write_all(data);
                    let _ = file.flush();
                    state.source_send_count = state.source_send_count.saturating_add(1);
                    println!(
                        "SLOPOS_CLIPBOARD_SOURCE_SENT mime={mime_type} bytes={}",
                        data.len()
                    );
                    let _ = std::io::stdout().flush();
                }
                // Unsupported MIME requests intentionally receive EOF by closing
                // the compositor-provided fd without writing any bytes.
            }
            wl_data_source::Event::Cancelled => {
                println!("SLOPOS_CLIPBOARD_SOURCE_CANCELLED");
            }
            _ => {}
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _state: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            state.toplevel_configured = true;
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _toplevel: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        if matches!(event, xdg_toplevel::Event::Close) {
            state.close_requested = true;
        }
    }
}

fn create_toplevel(
    compositor: &wl_compositor::WlCompositor,
    wm_base: &xdg_wm_base::XdgWmBase,
    queue_handle: &QueueHandle<State>,
    title: &str,
) -> (
    wl_surface::WlSurface,
    xdg_surface::XdgSurface,
    xdg_toplevel::XdgToplevel,
) {
    let wl_surface = compositor.create_surface(queue_handle, ());
    let xdg_surface = wm_base.get_xdg_surface(&wl_surface, queue_handle, ());
    let toplevel = xdg_surface.get_toplevel(queue_handle, ());
    toplevel.set_title(title.to_owned());
    toplevel.set_app_id("io.github.palaashatri.slopos.clipboard-smoke".to_owned());
    wl_surface.commit();
    (wl_surface, xdg_surface, toplevel)
}

fn wait_for_toplevel(
    event_queue: &mut wayland_client::EventQueue<State>,
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    while !state.toplevel_configured {
        if state.close_requested {
            return Err("compositor closed clipboard toplevel before configure".into());
        }
        event_queue.blocking_dispatch(state)?;
    }
    Ok(())
}

fn run_source(connection: &Connection, keep_alive: bool) -> Result<(), Box<dyn Error>> {
    let (globals, mut event_queue) = registry_queue_init::<State>(connection)?;
    let queue_handle = event_queue.handle();
    let compositor = globals.bind::<wl_compositor::WlCompositor, _, _>(&queue_handle, 1..=6, ())?;
    let wm_base = globals.bind::<xdg_wm_base::XdgWmBase, _, _>(&queue_handle, 1..=6, ())?;
    let manager = globals.bind::<wl_data_device_manager::WlDataDeviceManager, _, _>(
        &queue_handle,
        1..=3,
        (),
    )?;
    let seat = globals.bind::<wl_seat::WlSeat, _, _>(&queue_handle, 1..=9, ())?;
    let data_device = manager.get_data_device(&seat, &queue_handle, ());
    let source = manager.create_data_source(&queue_handle, ());
    source.offer(MIME_TEXT_UTF8.to_owned());
    source.offer(MIME_TEXT.to_owned());
    source.offer(MIME_LARGE.to_owned());

    let (surface, _xdg_surface, _toplevel) = create_toplevel(
        &compositor,
        &wm_base,
        &queue_handle,
        "SLOPOS clipboard source",
    );
    connection.flush()?;
    let mut state = State::default();
    wait_for_toplevel(&mut event_queue, &mut state)?;
    surface.commit();
    data_device.set_selection(Some(&source), 0);
    connection.flush()?;
    println!("SLOPOS_CLIPBOARD_SOURCE_READY offers={MIME_TEXT_UTF8},{MIME_TEXT},{MIME_LARGE}");
    std::io::stdout().flush()?;

    if !keep_alive {
        // Let the compositor consume SetSelection before this source dies. The
        // sink launched afterwards must observe that the selection was cleared.
        thread::sleep(Duration::from_millis(250));
        return Ok(());
    }

    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}

fn read_offer(
    connection: &Connection,
    offer: &wl_data_offer::WlDataOffer,
    mime_type: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let (mut reader, writer) = UnixStream::pair()?;
    reader.set_read_timeout(Some(Duration::from_secs(10)))?;
    offer.receive(mime_type.to_owned(), writer.as_fd());
    connection.flush()?;
    drop(writer);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn run_sink(connection: &Connection, expect_selection: bool) -> Result<(), Box<dyn Error>> {
    let (globals, mut event_queue) = registry_queue_init::<State>(connection)?;
    let queue_handle = event_queue.handle();
    let compositor = globals.bind::<wl_compositor::WlCompositor, _, _>(&queue_handle, 1..=6, ())?;
    let wm_base = globals.bind::<xdg_wm_base::XdgWmBase, _, _>(&queue_handle, 1..=6, ())?;
    let manager = globals.bind::<wl_data_device_manager::WlDataDeviceManager, _, _>(
        &queue_handle,
        1..=3,
        (),
    )?;
    let seat = globals.bind::<wl_seat::WlSeat, _, _>(&queue_handle, 1..=9, ())?;
    let _data_device = manager.get_data_device(&seat, &queue_handle, ());
    let (surface, _xdg_surface, _toplevel) = create_toplevel(
        &compositor,
        &wm_base,
        &queue_handle,
        "SLOPOS clipboard sink",
    );
    connection.flush()?;
    let mut state = State::default();
    wait_for_toplevel(&mut event_queue, &mut state)?;
    surface.commit();
    connection.flush()?;

    if !expect_selection {
        for _ in 0..20 {
            event_queue.dispatch_pending(&mut state)?;
            if state.selection_received {
                return Err("clipboard selection survived source disconnect".into());
            }
            thread::sleep(Duration::from_millis(100));
        }
        println!("SLOPOS_CLIPBOARD_SOURCE_DEATH_CLEARED");
        std::io::stdout().flush()?;
        return Ok(());
    }

    while !state.selection_received {
        if state.close_requested {
            return Err("compositor closed clipboard sink before selection".into());
        }
        event_queue.blocking_dispatch(&mut state)?;
    }
    let offer = state
        .data_offer
        .clone()
        .ok_or("selection event did not include a data offer")?;
    if !state
        .offered_mimes
        .iter()
        .any(|mime| mime == MIME_TEXT_UTF8)
        || !state.offered_mimes.iter().any(|mime| mime == MIME_TEXT)
        || !state.offered_mimes.iter().any(|mime| mime == MIME_LARGE)
    {
        return Err(format!(
            "clipboard offer missing expected MIME types: {:?}",
            state.offered_mimes
        )
        .into());
    }
    println!(
        "SLOPOS_CLIPBOARD_OFFER_VERIFIED mimes={}",
        state.offered_mimes.join(",")
    );

    let bytes = read_offer(connection, &offer, MIME_TEXT_UTF8)?;
    if bytes != PAYLOAD {
        return Err(format!("clipboard payload mismatch: got {} bytes", bytes.len()).into());
    }
    println!("SLOPOS_CLIPBOARD_TRANSFER_VERIFIED bytes={}", bytes.len());

    let large = read_offer(connection, &offer, MIME_LARGE)?;
    if large.len() != LARGE_PAYLOAD_SIZE || large.iter().any(|byte| *byte != b'L') {
        return Err(format!(
            "large clipboard payload mismatch: got {} bytes",
            large.len()
        )
        .into());
    }
    println!(
        "SLOPOS_CLIPBOARD_LARGE_TRANSFER_VERIFIED bytes={}",
        large.len()
    );

    let missing = read_offer(connection, &offer, MIME_MISSING)?;
    if !missing.is_empty() {
        return Err(format!(
            "unsupported MIME returned {} bytes instead of EOF",
            missing.len()
        )
        .into());
    }
    println!("SLOPOS_CLIPBOARD_MISSING_MIME_EOF_VERIFIED mime={MIME_MISSING}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mode = match env::args().nth(1).as_deref() {
        Some("source") => Mode::Source,
        Some("source-once") => Mode::SourceOnce,
        Some("sink") => Mode::Sink,
        Some("sink-after-source-death") => Mode::SinkAfterSourceDeath,
        _ => return Err(
            "usage: headless_clipboard_client <source|source-once|sink|sink-after-source-death>"
                .into(),
        ),
    };
    let connection = Connection::connect_to_env()?;
    match mode {
        Mode::Source => run_source(&connection, true),
        Mode::SourceOnce => run_source(&connection, false),
        Mode::Sink => run_sink(&connection, true),
        Mode::SinkAfterSourceDeath => run_sink(&connection, false),
    }
}
