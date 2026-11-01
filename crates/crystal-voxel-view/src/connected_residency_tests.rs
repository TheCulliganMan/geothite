// Resident source coverage survives authored map coordinate changes.
use crate::world_stream::test_frames::connected_frames;
use bevy::prelude::IVec2;

#[test]
fn world_coverage_survives_all_four_logical_map_edges() {
    for offset in [
        IVec2::new(40, 0),
        IVec2::new(-40, 0),
        IVec2::new(0, 40),
        IVec2::new(0, -40),
    ] {
        let (built, live) = connected_frames(offset);
        assert!(
            super::can_reuse(&built, &live),
            "resident source coverage must not expire at a logical map edge: {offset}"
        );
    }
}

#[test]
fn changed_source_and_exhausted_coverage_still_require_new_work() {
    let (built, mut live) = connected_frames(IVec2::new(40, 0));
    live.tiles[14 * 28 + 14].source.tile_index += 1;
    assert!(!super::can_reuse(&built, &live));
    let (_, mut live) = connected_frames(IVec2::new(40, 0));
    live.grid_origin.x += 16;
    assert!(!super::can_reuse(&built, &live));
    let (_, mut live) = connected_frames(IVec2::new(40, 0));
    live.source_regions = Default::default();
    assert!(!super::can_reuse(&built, &live));
}

#[test]
fn connected_retention_requires_the_same_mineral_walking_ownership() {
    let shift = IVec2::new(40, 0);
    let (mut built, mut live) = connected_frames(shift);
    let point = built.grid_origin + IVec2::splat(14);
    let datum = crystal_render_api::VisualTerraceDatum {
        origin: built.grid_origin,
        size: built.grid_size,
        levels: vec![[0.; 4]; built.tiles.len()].into(),
        water_barriers: Default::default(),
        walkable_mineral: vec![point].into(),
        open_water: None,
    };
    built.terrace_datum = Some(std::sync::Arc::new(datum.clone()));
    live.terrace_datum = Some(std::sync::Arc::new(
        crystal_render_api::VisualTerraceDatum {
            origin: datum.origin - shift,
            walkable_mineral: vec![point - shift].into(),
            ..datum
        },
    ));
    assert!(super::can_reuse(&built, &live));
    std::sync::Arc::make_mut(live.terrace_datum.as_mut().expect("live datum")).walkable_mineral =
        Default::default();
    assert_eq!(
        super::connected_frame_shift(&built, &live),
        Err("connected elevation context differs")
    );
}
