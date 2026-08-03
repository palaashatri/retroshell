#!/usr/bin/env python3
"""Apply the guarded popup lifecycle/rendering finalization to both backends."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates" / "slopos-compositor" / "src"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


def replace_between(text: str, start: str, end: str, replacement: str, label: str) -> str:
    begin = text.find(start)
    if begin < 0:
        raise RuntimeError(f"{label}: start anchor not found")
    finish = text.find(end, begin)
    if finish < 0:
        raise RuntimeError(f"{label}: end anchor not found")
    return text[:begin] + replacement + text[finish:]


def update_nested() -> None:
    path = SRC / "main.rs"
    text = path.read_text()
    text = replace_once(
        text,
        "        find_popup_root_surface, utils::under_from_surface_tree, PopupGrab, PopupKeyboardGrab,\n",
        "        find_popup_root_surface, get_popup_toplevel_coords, utils::under_from_surface_tree,\n"
        "        PopupGrab, PopupKeyboardGrab,\n",
        "nested popup coordinate import",
    )
    text = replace_once(
        text,
        '''        fn popup_origin(
            window: &MappedWindow,
            popup: &PopupKind,
            popup_offset: Point<i32, Logical>,
        ) -> Point<i32, Logical> {
            let geometry = popup.geometry();
            Point::from((
                window.position.x + popup_offset.x - geometry.loc.x,
                window.position.y + popup_offset.y - geometry.loc.y,
            ))
        }
''',
        '''        fn popup_origin(
            root_origin: Point<i32, Logical>,
            popup: &PopupKind,
            popup_offset: Point<i32, Logical>,
        ) -> Point<i32, Logical> {
            let geometry = popup.geometry();
            Point::from((
                root_origin.x + popup_offset.x - geometry.loc.x,
                root_origin.y + popup_offset.y - geometry.loc.y,
            ))
        }
''',
        "nested generic popup origin",
    )
    text = replace_between(
        text,
        "        fn layer_surface_under(\n",
        "        fn surface_under(\n",
        '''        fn layer_surface_under(
            layer: &MappedLayer,
            pt: Point<f64, Logical>,
        ) -> Option<(WlSurface, Point<f64, Logical>)> {
            for (popup, popup_offset) in
                PopupManager::popups_for_surface(layer.surface.wl_surface())
            {
                let origin = Self::popup_origin(layer.geo.loc, &popup, popup_offset);
                if let Some((surface, surface_origin)) = under_from_surface_tree(
                    popup.wl_surface(),
                    pt,
                    origin,
                    WindowSurfaceType::ALL,
                ) {
                    return Some((surface, surface_origin.to_f64()));
                }
            }

            let local = Point::from((pt.x - layer.geo.loc.x as f64, pt.y - layer.geo.loc.y as f64));
            let (surface, origin) = under_from_surface_tree(
                layer.surface.wl_surface(),
                local,
                (0, 0),
                WindowSurfaceType::ALL,
            )?;
            Some((surface, layer_surface_hit_origin(layer.geo.loc, origin)))
        }

''',
        "nested layer popup hit testing",
    )
    text = text.replace("Self::popup_origin(window, &popup, popup_offset)", "Self::popup_origin(window.position, &popup, popup_offset)")
    text = text.replace("Self::popup_origin(w, &popup, popup_offset)", "Self::popup_origin(w.position, &popup, popup_offset)")

    marker = '''        fn activated_window_for_surface(&self, surface: &WlSurface) -> Option<String> {
'''
    insert = '''        fn popup_root_origin(&self, popup: &PopupKind) -> Option<Point<i32, Logical>> {
            let root = find_popup_root_surface(popup).ok()?;
            if let Some(window) = self
                .windows
                .iter()
                .find(|window| window.toplevel.wl_surface() == &root)
            {
                return Some(window.position);
            }
            self.layer_surfaces
                .iter()
                .find(|layer| layer.surface.wl_surface() == &root)
                .map(|layer| layer.geo.loc)
        }

        fn constrained_popup_geometry(
            &self,
            popup: &PopupKind,
            positioner: PositionerState,
        ) -> Rectangle<i32, Logical> {
            let Some(root_origin) = self.popup_root_origin(popup) else {
                return positioner.get_geometry();
            };
            let parent_offset = get_popup_toplevel_coords(popup);
            let output = self.output_area();
            let target = Rectangle::new(
                Point::from((
                    output.x - root_origin.x - parent_offset.x,
                    output.y - root_origin.y - parent_offset.y,
                )),
                Size::from((output.width.max(1), output.height.max(1))),
            );
            positioner.get_unconstrained_geometry(target)
        }

'''
    text = replace_once(text, marker, insert + marker, "nested popup constraint helpers")

    text = replace_between(
        text,
        "        fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {",
        "        fn grab(&mut self, surface: PopupSurface, seat: wl_seat::WlSeat, serial: WlSerial) {",
        '''        fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
            let popup = PopupKind::from(surface.clone());
            if let Err(err) = self.popup_manager.track_popup(popup.clone()) {
                tracing::debug!(?err, "failed to track xdg popup");
                return;
            }
            let root_ready = find_popup_root_surface(&popup).is_ok();
            let geometry = self.constrained_popup_geometry(&popup, positioner);
            surface.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });
            if root_ready {
                if let Err(err) = surface.send_configure() {
                    tracing::debug!(?err, "failed to configure xdg popup");
                }
            } else {
                tracing::debug!("deferring parentless popup configure until layer-shell association");
            }
            self.request_redraw();
        }

''',
        "nested initial popup constraints",
    )
    text = replace_between(
        text,
        "        fn reposition_request(\n",
        "    }\n\n    delegate_xdg_shell!(SloposCompositor);",
        '''        fn reposition_request(
            &mut self,
            surface: PopupSurface,
            positioner: PositionerState,
            token: u32,
        ) {
            let popup = PopupKind::from(surface.clone());
            let geometry = self.constrained_popup_geometry(&popup, positioner);
            surface.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });
            let _serial = surface.send_repositioned(token);
            self.request_redraw();
        }
''',
        "nested popup reposition constraints",
    )

    layer_destroy_marker = '''        fn layer_destroyed(&mut self, surface: LayerSurface) {
'''
    layer_popup = '''        fn new_popup(&mut self, _parent: LayerSurface, surface: PopupSurface) {
            let popup = PopupKind::from(surface.clone());
            let positioner = surface.with_pending_state(|state| state.positioner);
            let geometry = self.constrained_popup_geometry(&popup, positioner);
            surface.with_pending_state(|state| {
                state.positioner = positioner;
                state.geometry = geometry;
            });
            if let Err(err) = surface.send_configure() {
                tracing::debug!(?err, "failed to configure layer-shell popup");
            }
            self.request_redraw();
        }

'''
    text = replace_once(text, layer_destroy_marker, layer_popup + layer_destroy_marker, "nested layer popup association")

    under_old = '''            for &i in &under {
                let layer = &self.layer_surfaces[i];
                let loc = Point::<i32, Physical>::from((layer.geo.loc.x, layer.geo.loc.y));
                surface_elements.extend(render_elements_from_surface_tree(
                    renderer,
                    layer.surface.wl_surface(),
                    loc,
                    1.0_f64,
                    1.0_f32,
                    Kind::Unspecified,
                ));
            }
'''
    under_new = '''            for &i in &under {
                let layer = &self.layer_surfaces[i];
                let popup_elements = PopupManager::popups_for_surface(layer.surface.wl_surface())
                    .flat_map(|(popup, popup_offset)| {
                        let popup_loc = Self::popup_origin(layer.geo.loc, &popup, popup_offset);
                        render_elements_from_surface_tree(
                            renderer,
                            popup.wl_surface(),
                            Point::<i32, Physical>::from((popup_loc.x, popup_loc.y)),
                            1.0_f64,
                            1.0_f32,
                            Kind::Unspecified,
                        )
                    });
                surface_elements.extend(popup_elements);
                let loc = Point::<i32, Physical>::from((layer.geo.loc.x, layer.geo.loc.y));
                surface_elements.extend(render_elements_from_surface_tree(
                    renderer,
                    layer.surface.wl_surface(),
                    loc,
                    1.0_f64,
                    1.0_f32,
                    Kind::Unspecified,
                ));
            }
'''
    text = replace_once(text, under_old, under_new, "nested lower-layer popup rendering")
    over_old = under_old.replace("&under", "&over")
    over_new = under_new.replace("&under", "&over")
    text = replace_once(text, over_old, over_new, "nested upper-layer popup rendering")

    callback_old = '''                for layer in &self.layer_surfaces {
                    send_frames_surface_tree(
                        layer.surface.wl_surface(),
                        &output,
                        now,
                        Some(Duration::ZERO),
                        |_, _| None,
                    );
                }
'''
    callback_new = '''                for layer in &self.layer_surfaces {
                    send_frames_surface_tree(
                        layer.surface.wl_surface(),
                        &output,
                        now,
                        Some(Duration::ZERO),
                        |_, _| None,
                    );
                    for (popup, _) in PopupManager::popups_for_surface(layer.surface.wl_surface()) {
                        send_frames_surface_tree(
                            popup.wl_surface(),
                            &output,
                            now,
                            Some(Duration::ZERO),
                            |_, _| None,
                        );
                    }
                }
'''
    text = replace_once(text, callback_old, callback_new, "nested layer popup frame callbacks")
    path.write_text(text)


def update_drm() -> None:
    path = SRC / "session_drm.rs"
    text = path.read_text()
    text = replace_once(
        text,
        "    find_popup_root_surface, PopupGrab, PopupKeyboardGrab, PopupKind, PopupManager,\n",
        "    find_popup_root_surface, get_popup_toplevel_coords, PopupGrab, PopupKeyboardGrab,\n"
        "    PopupKind, PopupManager,\n",
        "DRM popup coordinate import",
    )

    collect_marker = '''fn collect_render_elements(
'''
    helper = '''fn popup_origin(
    root_origin: Point<i32, Logical>,
    popup: &PopupKind,
    popup_offset: Point<i32, Logical>,
) -> Point<i32, Logical> {
    let geometry = popup.geometry();
    Point::from((
        root_origin.x + popup_offset.x - geometry.loc.x,
        root_origin.y + popup_offset.y - geometry.loc.y,
    ))
}

'''
    text = replace_once(text, collect_marker, helper + collect_marker, "DRM generic popup origin")

    top_old = '''    for layer in state.layer_surfaces.iter().rev() {
        if matches!(layer.layer, Layer::Overlay | Layer::Top) {
            elements.extend(render_elements_from_surface_tree(
                renderer,
                layer.surface.wl_surface(),
                physical_point(layer.geo.loc.x as f64, layer.geo.loc.y as f64),
                output_scale,
                1.0,
                Kind::Unspecified,
            ));
        }
    }
'''
    top_new = '''    for layer in state.layer_surfaces.iter().rev() {
        if matches!(layer.layer, Layer::Overlay | Layer::Top) {
            for (popup, popup_offset) in PopupManager::popups_for_surface(layer.surface.wl_surface()) {
                let popup_loc = popup_origin(layer.geo.loc, &popup, popup_offset);
                elements.extend(render_elements_from_surface_tree(
                    renderer,
                    popup.wl_surface(),
                    physical_point(popup_loc.x as f64, popup_loc.y as f64),
                    output_scale,
                    1.0,
                    Kind::Unspecified,
                ));
            }
            elements.extend(render_elements_from_surface_tree(
                renderer,
                layer.surface.wl_surface(),
                physical_point(layer.geo.loc.x as f64, layer.geo.loc.y as f64),
                output_scale,
                1.0,
                Kind::Unspecified,
            ));
        }
    }
'''
    text = replace_once(text, top_old, top_new, "DRM upper-layer popup rendering")
    bottom_old = top_old.replace("Layer::Overlay | Layer::Top", "Layer::Bottom | Layer::Background")
    bottom_new = top_new.replace("Layer::Overlay | Layer::Top", "Layer::Bottom | Layer::Background")
    text = replace_once(text, bottom_old, bottom_new, "DRM lower-layer popup rendering")

    text = replace_once(
        text,
        '''                let geometry = popup.geometry();
                let popup_loc = (
                    w.position.x + popup_offset.x - geometry.loc.x,
                    w.position.y + popup_offset.y - geometry.loc.y,
                );
''',
        '''                let popup_loc = popup_origin(w.position, &popup, popup_offset);
''',
        "DRM window popup origin",
    )
    text = text.replace("popup_loc.0 as f64, popup_loc.1 as f64", "popup_loc.x as f64, popup_loc.y as f64")

    callback_old = '''    for surface in layers {
        send_frames_surface_tree(&surface, &output, now, Some(Duration::ZERO), |_, _| None);
    }
'''
    callback_new = '''    for surface in layers {
        send_frames_surface_tree(&surface, &output, now, Some(Duration::ZERO), |_, _| None);
        for (popup, _) in PopupManager::popups_for_surface(&surface) {
            send_frames_surface_tree(
                popup.wl_surface(),
                &output,
                now,
                Some(Duration::ZERO),
                |_, _| None,
            );
        }
    }
'''
    text = replace_once(text, callback_old, callback_new, "DRM layer popup frame callbacks")

    text = replace_between(
        text,
        "    fn layer_surface_under(\n",
        "    fn surface_under(\n",
        '''    fn layer_surface_under(
        layer: &MappedLayer,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        for (popup, popup_offset) in PopupManager::popups_for_surface(layer.surface.wl_surface()) {
            let origin = popup_origin(layer.geo.loc, &popup, popup_offset);
            if let Some((surface, surface_origin)) = under_from_surface_tree(
                popup.wl_surface(),
                pos,
                origin,
                WindowSurfaceType::ALL,
            ) {
                return Some((surface, surface_origin.to_f64()));
            }
        }

        let local = Point::from((pos.x - layer.geo.loc.x as f64, pos.y - layer.geo.loc.y as f64));
        let (surface, origin) = under_from_surface_tree(
            layer.surface.wl_surface(),
            local,
            (0, 0),
            WindowSurfaceType::ALL,
        )?;
        Some((surface, layer_surface_hit_origin(layer.geo.loc, origin)))
    }

''',
        "DRM layer popup hit testing",
    )

    marker = '''    fn activated_window_for_surface(&self, surface: &WlSurface) -> Option<String> {
'''
    insert = '''    fn popup_root_origin(&self, popup: &PopupKind) -> Option<Point<i32, Logical>> {
        let root = find_popup_root_surface(popup).ok()?;
        if let Some(window) = self
            .windows
            .iter()
            .find(|window| window.toplevel.wl_surface() == &root)
        {
            return Some(window.position);
        }
        self.layer_surfaces
            .iter()
            .find(|layer| layer.surface.wl_surface() == &root)
            .map(|layer| layer.geo.loc)
    }

    fn constrained_popup_geometry(
        &self,
        popup: &PopupKind,
        positioner: PositionerState,
    ) -> Rectangle<i32, Logical> {
        let Some(root_origin) = self.popup_root_origin(popup) else {
            return positioner.get_geometry();
        };
        let parent_offset = get_popup_toplevel_coords(popup);
        let output = self.output_area();
        let target = Rectangle::new(
            Point::from((
                output.x - root_origin.x - parent_offset.x,
                output.y - root_origin.y - parent_offset.y,
            )),
            Size::from((output.width.max(1), output.height.max(1))),
        );
        positioner.get_unconstrained_geometry(target)
    }

'''
    text = replace_once(text, marker, insert + marker, "DRM popup constraint helpers")

    text = replace_between(
        text,
        "    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {",
        "    fn grab(&mut self, surface: PopupSurface, seat: wl_seat::WlSeat, serial: Serial) {",
        '''    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        let popup = PopupKind::from(surface.clone());
        if let Err(err) = self.popup_manager.track_popup(popup.clone()) {
            tracing::debug!(?err, "failed to track DRM xdg popup");
            return;
        }
        let root_ready = find_popup_root_surface(&popup).is_ok();
        let geometry = self.constrained_popup_geometry(&popup, positioner);
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = geometry;
        });
        if root_ready {
            if let Err(err) = surface.send_configure() {
                tracing::debug!(?err, "failed to configure DRM xdg popup");
            }
        } else {
            tracing::debug!("deferring parentless DRM popup configure until layer-shell association");
        }
        self.request_redraw();
    }

''',
        "DRM initial popup constraints",
    )
    text = replace_between(
        text,
        "    fn reposition_request(\n",
        "}\n\ndelegate_xdg_shell!(DrmSessionState);",
        '''    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        let popup = PopupKind::from(surface.clone());
        let geometry = self.constrained_popup_geometry(&popup, positioner);
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = geometry;
        });
        let _serial = surface.send_repositioned(token);
        self.request_redraw();
    }
''',
        "DRM popup reposition constraints",
    )

    layer_destroy_marker = '''    fn layer_destroyed(&mut self, surface: LayerSurface) {
'''
    layer_popup = '''    fn new_popup(&mut self, _parent: LayerSurface, surface: PopupSurface) {
        let popup = PopupKind::from(surface.clone());
        let positioner = surface.with_pending_state(|state| state.positioner);
        let geometry = self.constrained_popup_geometry(&popup, positioner);
        surface.with_pending_state(|state| {
            state.positioner = positioner;
            state.geometry = geometry;
        });
        if let Err(err) = surface.send_configure() {
            tracing::debug!(?err, "failed to configure DRM layer-shell popup");
        }
        self.request_redraw();
    }

'''
    text = replace_once(text, layer_destroy_marker, layer_popup + layer_destroy_marker, "DRM layer popup association")

    text = text.replace(
        "    /// Client GL scanout of SHM trees is not yet wired on the DRM path (dumb-buffer\n    /// pageflip only); this filter is the live listing contract for focus and any\n    /// future composite path.\n",
        "    /// The GL composition path consumes this ordering; the dumb-buffer pageflip\n    /// remains only the explicit fallback when DrmCompositor initialization fails.\n",
    )
    path.write_text(text)


if __name__ == "__main__":
    update_nested()
    update_drm()
    print("Applied popup lifecycle/rendering finalization")
