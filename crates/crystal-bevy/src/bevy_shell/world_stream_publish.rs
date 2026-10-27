//! Compose future world coverage without advancing the production controller.
use std::hash::{Hash, Hasher};

use bevy::prelude::{Assets, Image};

use super::{
    BevyRuntimeShell, PublishedDestination, RenderedTilesetArt, RuntimeShellSnapshot, TilePosition,
    connected_render_map_placements_to_depth, source_frame,
};

pub(super) fn publish(
    snapshot: &RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
    target: Option<&crystal_voxel_view::WorldStreamTarget>,
    previous: &mut PublishedDestination,
    request: &mut crystal_voxel_view::DestinationPreload,
    cache: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
) {
    let Some(target) = target else {
        request.stream_frame = None;
        previous.stream_key = None;
        return;
    };
    let Some(map) = snapshot
        .maps
        .iter()
        .find(|map| map.map_name == target.map_id.as_ref())
    else {
        return;
    };
    let (Ok(x), Ok(y)) = (i16::try_from(target.tile.x), i16::try_from(target.tile.y)) else {
        return;
    };
    let period = match snapshot.progression.time.time_of_day {
        crystal_core::world::encounters::TimeOfDay::Morning => "morn",
        crystal_core::world::encounters::TimeOfDay::Day => "day",
        crystal_core::world::encounters::TimeOfDay::Night => "nite",
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    target.map_id.hash(&mut hash);
    target.tile.hash(&mut hash);
    period.hash(&mut hash);
    snapshot
        .progression
        .active_engine_flags
        .contains("ENGINE_FOREST_IS_RESTLESS")
        .hash(&mut hash);
    for (source, x, y) in
        connected_render_map_placements_to_depth(&map.map_name, &snapshot.maps, None)
            .unwrap_or_default()
    {
        source.blocks.hash(&mut hash);
        x.hash(&mut hash);
        y.hash(&mut hash);
    }
    let key = hash.finish();
    if previous.stream_key == Some(key) {
        return;
    }
    match source_frame(
        snapshot,
        runtime,
        map,
        TilePosition::new(x, y),
        period,
        cache,
        images,
    ) {
        Ok(frame) => request.stream_frame = Some(frame),
        Err(error) => bevy::log::warn!("world coverage source unavailable: {error:#}"),
    }
    previous.stream_key = Some(key);
}
