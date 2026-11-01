//! Source-space motion receipts; no gameplay mutation or renderer settling.
use bevy::prelude::Transform;
use crystal_render_api::{VisualActorId, VisualWorldFrame};
use crystal_voxel_view::VoxelViewStatus;
use serde::Serialize;

#[derive(Default, Serialize)]
pub(super) struct MotionSample {
    pub map: String,
    pub active: bool,
    pub view_center: [f32; 2],
    pub camera_eye: Option<[f32; 3]>,
    pub camera_rotation: Option<[f32; 4]>,
    pub player_foot: Option<[f32; 2]>,
    /// Source-cell translation from the preceding map into this map.
    /// Absent on unproven handoffs, including ordinary door warps.
    pub connected_shift: Option<[i32; 2]>,
    pub tile_size: [f32; 2],
    pub terrain_map: Option<String>,
    pub connected_terrain: bool,
}

#[derive(Default)]
pub(super) struct MotionAudit {
    basis: Option<VisualWorldFrame>,
}

impl MotionAudit {
    pub fn observe(
        &mut self,
        frame: &VisualWorldFrame,
        status: &VoxelViewStatus,
        camera: Option<&Transform>,
    ) -> MotionSample {
        let changed = self
            .basis
            .as_ref()
            .is_none_or(|old| old.map_id != frame.map_id);
        let connected_shift = if changed {
            self.basis
                .as_ref()
                .and_then(|old| old.connected_source_shift(frame))
                .map(|shift| shift.to_array())
        } else {
            None
        };
        if changed {
            // The reciprocal placement proof only reads map IDs and regions.
            // Do not clone thousands of terrain cells on an audit update.
            self.basis = Some(VisualWorldFrame {
                map_id: frame.map_id.clone(),
                source_regions: frame.source_regions.clone(),
                ..Default::default()
            });
        }
        let player_foot = frame
            .actors
            .iter()
            .find(|actor| actor.id == VisualActorId::Player)
            .map(|actor| [actor.center.x, actor.center.y - actor.size.y * 0.5]);
        MotionSample {
            map: frame.map_id.to_string(),
            active: frame.active && status.active,
            view_center: frame.view_center.to_array(),
            camera_eye: camera.map(|camera| camera.translation.to_array()),
            camera_rotation: camera.map(|camera| camera.rotation.to_array()),
            player_foot,
            connected_shift,
            tile_size: frame.tile_size.to_array(),
            terrain_map: status.terrain_map.clone(),
            connected_terrain: status.connected_terrain,
        }
    }
}
