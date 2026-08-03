// Copyright (c) 2026 Palaash Atri
// SPDX-License-Identifier: MIT

//! Real Wayland protocol client used by the compositor headless runtime gate.
//!
//! It verifies, without attaching a render buffer:
//! - xdg-toplevel initial configure/ack;
//! - xdg-popup initial configure/ack;
//! - xdg-popup reposition acknowledgement and reconfigure;
//! - orderly role destruction.
//!
//! This is protocol/lifecycle evidence only. It does not claim rendering,
//! pointer grabs, input delivery, DRM/KMS, HDR, VRR, or hardware support.

use std::error::Error;

use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_compositor, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::xdg::shell::client::{
    xdg_popup, xdg_positioner, xdg_surface, xdg_toplevel, xdg_wm_base,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SurfaceRole {
    Toplevel,
    Popup,
}

#[derive(Default)]
struct State {
    toplevel_configured: bool,
    popup_surface_configured: bool,
    popup_geometry: Option<(i32, i32, i32, i32)>,
    popup_done: bool,
    close_requested: bool,
    repositioned_token: Option<u32>,
    popup_configure_count: u32,
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
wayland_client::delegate_noop!(State: ignore xdg_positioner::XdgPositioner);

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

impl Dispatch<xdg_surface::XdgSurface, SurfaceRole> for State {
    fn event(
        state: &mut Self,
        surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        role: &SurfaceRole,
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            match role {
                SurfaceRole::Toplevel => state.toplevel_configured = true,
                SurfaceRole::Popup => state.popup_surface_configured = true,
            }
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

impl Dispatch<xdg_popup::XdgPopup, ()> for State {
    fn event(
        state: &mut Self,
        _popup: &xdg_popup::XdgPopup,
        event: xdg_popup::Event,
        _data: &(),
        _connection: &Connection,
        _queue_handle: &QueueHandle<Self>,
    ) {
        match event {
            xdg_popup::Event::Configure {
                x,
                y,
                width,
                height,
            } => {
                state.popup_geometry = Some((x, y, width, height));
                state.popup_configure_count = state.popup_configure_count.saturating_add(1);
            }
            xdg_popup::Event::PopupDone => state.popup_done = true,
            xdg_popup::Event::Repositioned { token } => {
                state.repositioned_token = Some(token);
            }
            _ => {}
        }
    }
}

fn dispatch_until(
    event_queue: &mut wayland_client::EventQueue<State>,
    state: &mut State,
    mut predicate: impl FnMut(&State) -> bool,
    failure: &'static str,
) -> Result<(), Box<dyn Error>> {
    while !predicate(state) {
        if state.close_requested {
            return Err(format!("{failure}: compositor closed the parent toplevel").into());
        }
        if state.popup_done {
            return Err(format!("{failure}: compositor dismissed the popup").into());
        }
        event_queue.blocking_dispatch(state)?;
    }
    Ok(())
}

fn configure_positioner(positioner: &xdg_positioner::XdgPositioner, offset: i32) {
    // size and anchor_rect are the mandatory positioner fields. Avoid relying on
    // optional anchor/gravity policy in this lifecycle smoke test.
    positioner.set_size(180, 96);
    positioner.set_anchor_rect(12, 12, 240, 32);
    positioner.set_offset(offset, offset);
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

    let parent_wl_surface = compositor.create_surface(&queue_handle, ());
    let parent_xdg_surface = wm_base.get_xdg_surface(
        &parent_wl_surface,
        &queue_handle,
        SurfaceRole::Toplevel,
    );
    let toplevel = parent_xdg_surface.get_toplevel(&queue_handle, ());
    toplevel.set_title("SLOPOS compositor protocol smoke".to_owned());
    toplevel.set_app_id("io.github.palaashatri.slopos.compositor-smoke".to_owned());

    // A newly created xdg_surface must commit without a buffer before the
    // compositor sends the initial configure.
    parent_wl_surface.commit();

    let mut state = State::default();
    dispatch_until(
        &mut event_queue,
        &mut state,
        |state| state.toplevel_configured,
        "toplevel configure failed",
    )?;
    parent_wl_surface.commit();
    connection.flush()?;

    println!(
        "SLOPOS_XDG_TOPLEVEL_CONFIGURED id={} version={}",
        toplevel.id().protocol_id(),
        toplevel.version()
    );

    let positioner = wm_base.create_positioner(&queue_handle, ());
    configure_positioner(&positioner, 0);
    let popup_wl_surface = compositor.create_surface(&queue_handle, ());
    let popup_xdg_surface = wm_base.get_xdg_surface(
        &popup_wl_surface,
        &queue_handle,
        SurfaceRole::Popup,
    );
    let popup = popup_xdg_surface.get_popup(
        Some(&parent_xdg_surface),
        &positioner,
        &queue_handle,
        (),
    );
    popup_wl_surface.commit();

    dispatch_until(
        &mut event_queue,
        &mut state,
        |state| state.popup_surface_configured && state.popup_geometry.is_some(),
        "popup configure failed",
    )?;
    let initial_geometry = state
        .popup_geometry
        .ok_or("popup xdg_surface configured without xdg_popup geometry")?;
    if initial_geometry.2 <= 0 || initial_geometry.3 <= 0 {
        return Err(format!("popup configure had invalid geometry: {initial_geometry:?}").into());
    }
    popup_wl_surface.commit();
    connection.flush()?;

    println!(
        "SLOPOS_XDG_POPUP_CONFIGURED id={} geometry={},{},{},{}",
        popup.id().protocol_id(),
        initial_geometry.0,
        initial_geometry.1,
        initial_geometry.2,
        initial_geometry.3
    );

    if popup.version() >= 3 {
        const REPOSITION_TOKEN: u32 = 0x534c_4f50;
        let repositioner = wm_base.create_positioner(&queue_handle, ());
        configure_positioner(&repositioner, 24);
        let configure_count_before = state.popup_configure_count;
        popup.reposition(&repositioner, REPOSITION_TOKEN);
        connection.flush()?;

        dispatch_until(
            &mut event_queue,
            &mut state,
            |state| {
                state.repositioned_token == Some(REPOSITION_TOKEN)
                    && state.popup_configure_count > configure_count_before
            },
            "popup reposition failed",
        )?;
        println!(
            "SLOPOS_XDG_POPUP_REPOSITIONED id={} token={} configure_count={}",
            popup.id().protocol_id(),
            REPOSITION_TOKEN,
            state.popup_configure_count
        );
        repositioner.destroy();
    } else {
        println!(
            "SLOPOS_XDG_POPUP_REPOSITION_SKIPPED version={}",
            popup.version()
        );
    }

    popup.destroy();
    popup_xdg_surface.destroy();
    popup_wl_surface.destroy();
    positioner.destroy();
    toplevel.destroy();
    parent_xdg_surface.destroy();
    parent_wl_surface.destroy();
    connection.flush()?;
    Ok(())
}
