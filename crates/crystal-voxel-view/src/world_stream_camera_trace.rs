//! Opt-in support and traversal receipts at actual terrain-root publication.
use bevy::prelude::{Vec2, Vec3};
use crystal_render_api::{VisualActorId, VisualPlayerTraversal, VisualWorldFrame};
use serde_json::json;

use crate::{TerrainRevisionCache, VoxelCameraPose, VoxelViewSettings, footing, loading_trace};

pub(super) fn publish(
    frame: &VisualWorldFrame,
    settings: &VoxelViewSettings,
    cache: &TerrainRevisionCache,
    traversal: Option<&VisualPlayerTraversal>,
    old_eye: Option<Vec3>,
    pose: VoxelCameraPose,
) {
    drop(loading_trace::span("world.camera_handoff", || {
        let player = frame
            .actors
            .iter()
            .find(|actor| actor.id == VisualActorId::Player);
        let ground = player.map(|player| footing::actor_ground_point(frame, player));
        let step = traversal.and_then(|traversal| traversal.step);
        let offset = Vec2::Y * frame.tile_size.y;
        let support =
            |point| footing::resolved_footing_height(frame, point, &cache.footing_heights);
        json!({
            "map": frame.map_id.as_ref(), "origin": frame.grid_origin.to_array(),
            "built_map": cache.built_frame.as_ref().map(|built| built.map_id.as_ref()),
            "built_origin": cache.built_frame.as_ref().map(|built| built.grid_origin.to_array()),
            "player_ground": ground.map(|ground| ground.to_array()),
            "ground_height": ground.and_then(support),
            "old_eye": old_eye.map(|eye| eye.to_array()), "eye": pose.eye.to_array(),
            "base_eye": settings.camera.world_pose(frame, &cache.footing_heights).eye.to_array(),
            "traversal_map": traversal.map(|traversal| traversal.map_id.as_ref()),
            "from": step.map(|step| step.from.to_array()), "to": step.map(|step| step.to.to_array()),
            "from_height": step.and_then(|step| support(step.from + offset)),
            "to_height": step.and_then(|step| support(step.to + offset)),
            "progress": step.map(|step| step.progress), "ledge_jump": step.map(|step| step.ledge_jump),
            "traversal_height_arc": player.and_then(|player|
                footing::traversal_height(frame, player, &cache.footing_heights, traversal)),
        })
    }));
}
