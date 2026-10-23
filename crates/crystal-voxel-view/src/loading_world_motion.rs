//! Browser motion receipts sampled after production camera synchronization.
use std::sync::Arc;

use bevy::prelude::{App, IntoScheduleConfigs, Last, Local, Query, Res, Transform};
use crystal_render_api::{VisualActorId, VisualWorldFrame};
use serde_json::json;

use crate::{VoxelScene, VoxelViewStatus};

pub(super) fn register(app: &mut App) {
    app.add_systems(
        Last,
        (|frame: Option<Res<VisualWorldFrame>>,
          status: Option<Res<VoxelViewStatus>>,
          scene: Option<Res<VoxelScene>>,
          transforms: Query<&Transform>,
          mut basis: Local<Option<VisualWorldFrame>>| {
            let (Some(frame), Some(status)) = (frame, status) else {
                return;
            };
            let changed = basis.as_ref().is_none_or(|old| old.map_id != frame.map_id);
            let shift = changed
                .then(|| {
                    basis
                        .as_ref()
                        .and_then(|old| old.connected_source_shift(&frame))
                })
                .flatten();
            let camera = scene
                .and_then(|scene| scene.camera)
                .and_then(|entity| transforms.get(entity).ok());
            let player = frame
                .actors
                .iter()
                .find(|actor| actor.id == VisualActorId::Player);
            drop(super::span("presentation.world_motion", || {
                json!({
                    "map": frame.map_id.as_ref(),
                    "active": frame.active && status.active,
                    "view_center": frame.view_center.to_array(),
                    "camera_eye": camera.map(|camera| camera.translation.to_array()),
                    "camera_rotation": camera.map(|camera| camera.rotation.to_array()),
                    "player_foot": player.map(|actor|
                        [actor.center.x, actor.center.y - actor.size.y * 0.5]),
                    "connected_shift": shift.map(|shift| shift.to_array()),
                    "tile_size": frame.tile_size.to_array(),
                    "terrain_map": status.terrain_map,
                    "connected_terrain": status.connected_terrain,
                })
            }));
            if changed {
                *basis = Some(VisualWorldFrame {
                    map_id: Arc::clone(&frame.map_id),
                    source_regions: Arc::clone(&frame.source_regions),
                    ..Default::default()
                });
            }
        })
        .run_if(super::trace_enabled),
    );
}
