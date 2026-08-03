// Copyright (c) 2026 Palaash Atri
// SPDX-License-Identifier: MIT

//! Cross-module compositor completion contracts.
//!
//! These tests intentionally exercise public policy APIs as a consumer would.
//! They protect the state invariants that the nested and DRM backends must
//! share: reversible presentation transitions, gapless tiling, dynamic Spaces,
//! and deterministic output migration.

use slopos_compositor::{
    calculate_presentation_geometry, transition_presentation_state, MultiMonitorPolicy,
    SpaceTarget, SpacesModel, TilePlacement, WindowGeometry, WindowPresentationState,
};

#[test]
fn presentation_round_trip_preserves_the_original_normal_frame() {
    let normal = WindowGeometry::new(137, 91, 731, 509);
    let work_area = WindowGeometry::new(0, 24, 1600, 876);
    let output_area = WindowGeometry::new(0, 0, 1600, 900);

    let filled = transition_presentation_state(
        WindowPresentationState::Normal,
        normal,
        None,
        WindowPresentationState::Filled,
        work_area,
        output_area,
        None,
        "output-1",
        7,
    );
    assert_eq!(filled.state, WindowPresentationState::Filled);
    assert_eq!(filled.geometry, work_area);
    assert_eq!(
        filled
            .restore_state
            .as_ref()
            .expect("Fill must capture a restore frame")
            .normal_geometry,
        normal
    );

    let fullscreen = transition_presentation_state(
        filled.state,
        filled.geometry,
        filled.restore_state.as_ref(),
        WindowPresentationState::Fullscreen,
        work_area,
        output_area,
        None,
        "output-1",
        7,
    );
    assert_eq!(fullscreen.state, WindowPresentationState::Fullscreen);
    assert_eq!(fullscreen.geometry, output_area);
    assert_eq!(
        fullscreen
            .restore_state
            .as_ref()
            .expect("Fullscreen must retain the original restore frame")
            .normal_geometry,
        normal
    );

    let restored = transition_presentation_state(
        fullscreen.state,
        fullscreen.geometry,
        fullscreen.restore_state.as_ref(),
        WindowPresentationState::Normal,
        work_area,
        output_area,
        None,
        "output-1",
        7,
    );
    assert_eq!(restored.state, WindowPresentationState::Normal);
    assert_eq!(restored.geometry, normal);
    assert!(restored.restore_state.is_none());
    assert_eq!(
        restored
            .restored_from
            .as_ref()
            .expect("restore metadata must be exposed to the backend")
            .normal_geometry,
        normal
    );
}

#[test]
fn minimize_does_not_destroy_a_preexisting_restore_frame() {
    let normal = WindowGeometry::new(80, 70, 640, 480);
    let work_area = WindowGeometry::new(0, 22, 1280, 778);
    let output_area = WindowGeometry::new(0, 0, 1280, 800);

    let zoomed = transition_presentation_state(
        WindowPresentationState::Normal,
        normal,
        None,
        WindowPresentationState::SmartZoomed,
        work_area,
        output_area,
        Some((900, 650)),
        "output-1",
        1,
    );
    assert_eq!(zoomed.state, WindowPresentationState::SmartZoomed);

    let minimized = transition_presentation_state(
        zoomed.state,
        zoomed.geometry,
        zoomed.restore_state.as_ref(),
        WindowPresentationState::Minimized,
        work_area,
        output_area,
        None,
        "output-1",
        1,
    );
    assert_eq!(minimized.state, WindowPresentationState::Minimized);
    assert_eq!(minimized.geometry, zoomed.geometry);
    assert_eq!(
        minimized
            .restore_state
            .as_ref()
            .expect("minimize must retain the restore record")
            .normal_geometry,
        normal
    );

    let restored = transition_presentation_state(
        minimized.state,
        minimized.geometry,
        minimized.restore_state.as_ref(),
        WindowPresentationState::Normal,
        work_area,
        output_area,
        None,
        "output-1",
        1,
    );
    assert_eq!(restored.geometry, normal);
}

#[test]
fn odd_sized_tiling_partitions_the_work_area_without_gaps() {
    let area = WindowGeometry::new(11, 29, 1001, 701);
    let normal = WindowGeometry::new(100, 100, 500, 400);

    let left = calculate_presentation_geometry(
        area,
        WindowPresentationState::Tiled(TilePlacement::Left),
        None,
        normal,
    );
    let right = calculate_presentation_geometry(
        area,
        WindowPresentationState::Tiled(TilePlacement::Right),
        None,
        normal,
    );

    assert_eq!(left.x, area.x);
    assert_eq!(right.x, left.x + left.width);
    assert_eq!(left.width + right.width, area.width);
    assert_eq!(left.height, area.height);
    assert_eq!(right.height, area.height);

    let top_left = calculate_presentation_geometry(
        area,
        WindowPresentationState::Tiled(TilePlacement::TopLeft),
        None,
        normal,
    );
    let top_right = calculate_presentation_geometry(
        area,
        WindowPresentationState::Tiled(TilePlacement::TopRight),
        None,
        normal,
    );
    let bottom_left = calculate_presentation_geometry(
        area,
        WindowPresentationState::Tiled(TilePlacement::BottomLeft),
        None,
        normal,
    );
    let bottom_right = calculate_presentation_geometry(
        area,
        WindowPresentationState::Tiled(TilePlacement::BottomRight),
        None,
        normal,
    );

    assert_eq!(top_left.width + top_right.width, area.width);
    assert_eq!(bottom_left.width + bottom_right.width, area.width);
    assert_eq!(top_left.height + bottom_left.height, area.height);
    assert_eq!(top_right.height + bottom_right.height, area.height);
    assert_eq!(top_right.x, top_left.x + top_left.width);
    assert_eq!(bottom_right.x, bottom_left.x + bottom_left.width);
    assert_eq!(bottom_left.y, top_left.y + top_left.height);
    assert_eq!(bottom_right.y, top_right.y + top_right.height);
}

#[test]
fn dynamic_spaces_keep_window_membership_valid_during_removal() {
    let mut spaces = SpacesModel::with_initial_name("Personal").unwrap();
    let personal = spaces.active_space();
    let work = spaces.create_space("Work").unwrap();
    let media = spaces.create_space("Media").unwrap();

    spaces
        .assign_window("finder-window", SpaceTarget::Named("Work".into()))
        .unwrap();
    spaces
        .assign_window("music-window", SpaceTarget::All)
        .unwrap();
    spaces.activate_space(work).unwrap();

    assert_eq!(spaces.window_spaces("finder-window"), vec![work]);
    assert_eq!(
        spaces.window_spaces("music-window"),
        vec![personal, work, media]
    );

    let fallback = spaces.remove_space(work).unwrap();
    assert_eq!(spaces.active_space(), fallback);
    assert!(spaces.space(work).is_none());
    assert_eq!(spaces.window_spaces("finder-window"), vec![fallback]);
    assert_eq!(spaces.window_spaces("music-window").len(), 2);
    spaces.validate().unwrap();
}

#[test]
fn independent_display_spaces_migrate_without_changing_identity_or_order() {
    let mut spaces = SpacesModel::with_initial_name("Laptop").unwrap();
    let laptop = spaces.active_space();
    let external = spaces.create_space("External").unwrap();
    let reference_order = spaces.space_ids();

    spaces.set_multi_monitor_policy(MultiMonitorPolicy::IndependentPerDisplay);
    spaces
        .assign_space_to_output(laptop, "eDP-1")
        .unwrap();
    spaces
        .assign_space_to_output(external, "DP-1")
        .unwrap();

    assert_eq!(spaces.spaces_for_output("eDP-1").unwrap(), vec![laptop]);
    assert_eq!(spaces.spaces_for_output("DP-1").unwrap(), vec![external]);

    let migrated = spaces.migrate_output("DP-1", Some("HDMI-A-1")).unwrap();
    assert_eq!(migrated, vec![external]);
    assert_eq!(spaces.output_for_space(external), Some("HDMI-A-1"));
    assert_eq!(spaces.space_ids(), reference_order);
    spaces.validate().unwrap();
}

#[test]
fn overview_projection_tracks_reorder_and_active_state() {
    let mut spaces = SpacesModel::with_initial_name("One").unwrap();
    let one = spaces.active_space();
    let two = spaces.create_space("Two").unwrap();
    let three = spaces.create_space("Three").unwrap();

    spaces.reorder_space(three, 0).unwrap();
    spaces.activate_space(two).unwrap();

    let overview = spaces.overview_projection();
    assert_eq!(overview.len(), 3);
    assert_eq!(overview[0].id(), three);
    assert_eq!(overview[0].order(), 0);
    assert_eq!(overview[1].id(), one);
    assert_eq!(overview[2].id(), two);
    assert!(overview[2].is_active());
    assert_eq!(overview.iter().filter(|row| row.is_active()).count(), 1);
}
