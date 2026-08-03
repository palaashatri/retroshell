// Copyright (c) 2026 Palaash Atri
// SPDX-License-Identifier: MIT

//! Minimal real Wayland client used by the compositor runtime smoke gate.
//!
//! The client connects through `WAYLAND_DISPLAY`, binds wl_compositor and
//! xdg_wm_base, creates an xdg_toplevel, completes the initial configure/ack
//! handshake, commits once more, and exits. It deliberately attaches no buffer:
//! this verifies protocol dispatch and role/configure correctness without
//! pretending that headless mode proves rendering.

use std::error::Error;

use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_compositor, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

#[derive(Default)]
struct State {
    configured: bool,
    close_requested: bool,
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
        // GlobalList owns registry bookkeeping. No application event is needed.
    }
}

wayland_client::delegate_noop!(State: ignore wl_compositor::WlCompositor);
wayland_client::delegate_noop!(State: ignore wl_surface::WlSurface);

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
            state.configured = true;
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

fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<State>(&connection)?;
    let queue_handle = event_queue.handle();

    let compositor = globals.bind::<wl_compositor::WlCompositor, _, _>(
        &queue_handle,
        1..=6,
        (),
    )?;
    let wm_base =
        globals.bind::<xdg_wm_base::XdgWmBase, _, _>(&queue_handle, 1..=6, ())?;

    let wl_surface = compositor.create_surface(&queue_handle, ());
    let xdg_surface = wm_base.get_xdg_surface(&wl_surface, &queue_handle, ());
    let toplevel = xdg_surface.get_toplevel(&queue_handle, ());
    toplevel.set_title("SLOPOS compositor protocol smoke".to_owned());
    toplevel.set_app_id("io.github.palaashatri.slopos.compositor-smoke".to_owned());

    // A newly created xdg_surface must commit without a buffer before the
    // compositor sends the initial configure.
    wl_surface.commit();

    let mut state = State::default();
    while !state.configured && !state.close_requested {
        event_queue.blocking_dispatch(&mut state)?;
    }
    if state.close_requested {
        return Err("compositor closed the smoke-test toplevel before configure".into());
    }

    // Ack has been sent from the configure handler. A second commit completes
    // the protocol handshake; no buffer is attached because headless runtime
    // verification is not rendering verification.
    wl_surface.commit();
    connection.flush()?;

    println!(
        "SLOPOS_XDG_TOPLEVEL_CONFIGURED id={} version={}",
        toplevel.id().protocol_id(),
        toplevel.version()
    );

    toplevel.destroy();
    xdg_surface.destroy();
    wl_surface.destroy();
    connection.flush()?;
    Ok(())
}
