//! Pure conversion from classic 2D visual coordinates to voxel footing.

use bevy::prelude::{UVec2, Vec2, Vec3};
use crystal_render_api::{VisualActor, VisualWorldFrame};

use crate::profile::support_height;

const FOOTING_SAMPLE_EPSILON: f32 = 0.01;

pub fn actor_foot(actor: &VisualActor) -> Vec2 {
    actor.center - Vec2::new(0.0, actor.size.y * 0.5)
}

/// Ground-rooted 3D characters stand in the center of their gameplay tile.
/// The source sprite's bottom is on its south edge, one 8px render cell away.
/// Use the map scale rather than sprite height: followers and large icons can
/// have different image sizes while occupying the same gameplay tile.
pub(crate) fn actor_ground_point(frame: &VisualWorldFrame, actor: &VisualActor) -> Vec2 {
    actor_foot(actor) + Vec2::Y * frame.tile_size.y
}

pub fn tile_at_visual_point(frame: &VisualWorldFrame, point: Vec2) -> Option<UVec2> {
    if !point.is_finite() || !frame.center.is_finite() || !frame.viewport_size.is_finite() {
        return None;
    }
    if frame.tile_size.x <= 0.0 || frame.tile_size.y <= 0.0 {
        return None;
    }

    let left = frame.center.x - frame.viewport_size.x * 0.5;
    let top = frame.center.y + frame.viewport_size.y * 0.5;
    let relative_x = point.x - left;
    let relative_y = top - point.y;
    let terrain_size = frame.tile_size * frame.grid_size.as_vec2();
    let terrain_margin = (terrain_size - frame.viewport_size) * 0.5;
    let grid_x = relative_x + terrain_margin.x;
    let grid_y = relative_y + terrain_margin.y;
    // The pitched camera sees actors beyond the classic LCD. Their feet
    // must sample the published terrain halo too, including raised stages.
    if grid_x < 0.0 || grid_y < 0.0
        || grid_x >= terrain_size.x || grid_y >= terrain_size.y
    {
        return None;
    }
    let column = (grid_x / frame.tile_size.x).floor() as u32;
    let row = (grid_y / frame.tile_size.y).floor() as u32;
    (column < frame.grid_size.x && row < frame.grid_size.y).then_some(UVec2::new(column, row))
}

pub fn footing_height(frame: &VisualWorldFrame, foot: Vec2) -> Option<f32> {
    if !foot.is_finite() {
        return None;
    }
    // Sprite bottoms normally sit exactly on a tile boundary. Sample just
    // inside the sprite footprint while rendering at the exact bottom point.
    let Some(coordinate) = tile_at_visual_point(frame, foot + Vec2::Y * FOOTING_SAMPLE_EPSILON)
    else {
        // An actor beyond even the published terrain halo has no sampled
        // support. Keep the ordinary-ground fallback for that outer seam.
        return Some(0.0);
    };
    Some(
        frame
            .tiles
            .iter()
            .find(|tile| tile.column == coordinate.x && tile.row == coordinate.y)
            .map(|tile| support_height(&tile.source, frame.tile_size.y))
            .unwrap_or(0.0),
    )
}

pub fn resolved_footing_height(
    frame: &VisualWorldFrame,
    foot: Vec2,
    resolved_heights: &[f32],
) -> Option<f32> {
    if !foot.is_finite() {
        return None;
    }
    let Some(coordinate) = tile_at_visual_point(frame, foot + Vec2::Y * FOOTING_SAMPLE_EPSILON)
    else {
        return Some(0.0);
    };
    let width = usize::try_from(frame.grid_size.x).ok()?;
    let index = usize::try_from(coordinate.y)
        .ok()?
        .checked_mul(width)?
        .checked_add(usize::try_from(coordinate.x).ok()?)?;
    let resolved = resolved_heights.get(index).copied()?;
    Some(resolved + crate::mesh::lighthouse_stair_support(frame,coordinate,foot,resolved_heights)
        .map_or(0.,|height|height-resolved)
        + crate::waterfall::support_correction(frame,coordinate,foot,resolved_heights)
        + crate::mesh::terrace_support_correction(frame,coordinate,foot)
        + crate::mesh::pond_support_correction(frame,coordinate,foot,resolved_heights)
        + crate::mesh::league_stair_support_correction(frame,coordinate,foot,resolved_heights)
        + crate::mesh::hall_stair_support_correction(frame,coordinate,foot,resolved_heights)
        + crate::mesh::radio_stair_support_correction(frame,coordinate,foot,resolved_heights))
}

/// Lift only an accepted uphill step or authored ledge jump. Sampling both
/// endpoints avoids snapping to the upper terrace midway through the step.
pub(crate) fn traversal_height(
    frame: &VisualWorldFrame,
    actor: &VisualActor,
    heights: &[f32],
    traversal: Option<&crystal_render_api::VisualPlayerTraversal>,
) -> Option<(f32, f32)> {
    let ground = resolved_footing_height(frame, actor_ground_point(frame, actor), heights)?;
    let Some(traversal) = traversal.filter(|t| t.map_id == frame.map_id) else {
        return Some((ground, 0.0));
    };
    let Some(step) = traversal.step.filter(|_| actor.id == crystal_render_api::VisualActorId::Player
        && !actor.source_id.ends_with("_bike") && !actor.source_id.starts_with("surf")) else {
        return Some((ground, 0.0));
    };
    let offset = Vec2::Y * frame.tile_size.y;
    let from_point = step.from + offset;
    let to_point = step.to + offset;
    if tile_at_visual_point(frame, from_point + Vec2::Y * FOOTING_SAMPLE_EPSILON).is_none()
        || tile_at_visual_point(frame, to_point + Vec2::Y * FOOTING_SAMPLE_EPSILON).is_none()
    {
        return Some((ground, 0.0));
    }
    let from = resolved_footing_height(frame, from_point, heights)?;
    let to = resolved_footing_height(frame, to_point, heights)?;
    let rise = to - from;
    // Complete lighthouse flights supply actual tread contact without turning
    // their steep visual rise into an authored ledge jump.
    let lighthouse_flight=[from_point,to_point].into_iter().any(|point|
        tile_at_visual_point(frame,point+Vec2::Y*FOOTING_SAMPLE_EPSILON)
            .and_then(|cell|crate::mesh::lighthouse_stair_support(frame,cell,point,heights)).is_some());
    // Stairs and shallow ramps keep their ordinary gait.
    if !step.ledge_jump && (lighthouse_flight || rise < frame.tile_size.y * 0.4) {
        return Some((ground, 0.0));
    }
    let p = step.progress.clamp(0.0, 1.0);
    let arc = 4.0 * p * (1.0 - p);
    let clearance = rise.abs().max(frame.tile_size.y * 0.75);
    Some((from + rise * p + clearance * arc, arc))
}

pub fn visual_point_to_voxel(point: Vec2, height: f32) -> Vec3 {
    Vec3::new(point.x, height, -point.y)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::prelude::{Handle, Image, UVec2};
    use crystal_render_api::{VisualActorId, VisualTile, VisualTileSource, VisualWorldFrame};

    use super::*;

    fn source(metatile_id: u16, row: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("johto"),
            metatile_id,
            subtile_column: 0,
            subtile_row: row,
            tile_index: 1,
        }
    }

    fn frame() -> VisualWorldFrame {
        VisualWorldFrame { source_regions: Default::default(), view_center: Vec2::ZERO, terrace_datum: None,
            active: true,
            map_id: Arc::from("NewBarkTown"),
            source_map_size_core_tiles: None,
            jump_ledges: Default::default(),
            boundary_warps: Default::default(),
            terrain_revision: 1,
            grid_origin: bevy::prelude::IVec2::ZERO,
            map_texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData),
            center: Vec2::new(-16.0, 16.0),
            viewport_size: Vec2::new(16.0, 8.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(2, 1),
            tiles: vec![
                VisualTile {
                    animation_frames: None,
                    column: 1,
                    row: 0,
                    source: source(0x54, 0),
                    texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData),
                    priority: true,
                },
                VisualTile {
                    animation_frames: None,
                    column: 0,
                    row: 0,
                    source: source(0x01, 0),
                    texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(3), std::marker::PhantomData),
                    priority: false,
                },
            ],
            actors: Vec::new(),
        }
    }

    #[test]
    fn cave_wall_backed_ladder_footing_stays_level_during_movement() {
        for tileset in ["cave","dark_cave"] {
            for block in [0x14,0x15] {
                for scale in [1.,2.] {
                    let mut f=frame();
                    f.center=Vec2::new(-37.,29.);
                    f.viewport_size=Vec2::new(16.,16.)*scale;
                    f.tile_size=Vec2::splat(8.*scale);
                    f.grid_size=UVec2::new(2,2);
                    f.tiles.clear();
                    let art=if block==0x14 {[[0x2a,0x2b],[0x3a,0x3b]]}
                        else {[[0x22,0x23],[0x32,0x33]]};
                    for y in 0..2 {for x in 0..2 {
                        f.tiles.push(VisualTile {column:x,row:y,priority:false,
                            texture:Handle::default(),animation_frames:None,
                            source:VisualTileSource {tileset_id:tileset.into(),metatile_id:block,
                                subtile_column:(x+2)as u8,subtile_row:(y+2)as u8,
                                tile_index:art[y as usize][x as usize]}});
                    }}
                    f.tiles.reverse();
                    let heights=[0.;4];
                    let top=f.center.y+8.*scale;
                    for step in 1..=32 {
                        let depth=step as f32*0.5*scale;
                        let foot=Vec2::new(f.center.x,top-depth);
                        assert!((resolved_footing_height(&f,foot,&heights).unwrap()
                            -0.).abs()<0.00001);
                    }
                    let midpoint=Vec2::new(f.center.x,f.center.y);
                    assert_eq!(resolved_footing_height(&f,midpoint,&heights),Some(0.));
                    let modified=[9.*scale;4];
                    assert_eq!(resolved_footing_height(&f,midpoint,&modified),Some(9.*scale));
                    for tile in &mut f.tiles {tile.source.tile_index=0xff;}
                    assert_eq!(resolved_footing_height(&f,midpoint,&heights),Some(0.),
                        "custom art must preserve the resolved floor datum");
                }
            }
        }
    }

    #[test]
    fn uphill_hop_clears_the_terrace_and_lands_on_its_exact_height() {
        use crystal_render_api::{VisualPlayerTraversal, VisualTraversalStep};
        let frame = frame();
        let from = Vec2::new(-20.0, 8.0);
        let to = Vec2::new(-12.0, 8.0);
        let mut actor = VisualActor {
            id: VisualActorId::Player, source_id: "player".into(), model_palette: None,
            texture: Handle::default(), center: from + Vec2::Y * 8.0,
            size: Vec2::new(8.0, 16.0), flip_x: false, above_priority: false, facing: Some(Vec2::X),
        };
        let mut traversal = VisualPlayerTraversal {
            map_id: frame.map_id.clone(),
            step: Some(VisualTraversalStep { from, to, progress: 0.0, ledge_jump: false }),
        };
        for (progress, expected) in [(0.0, 0.0), (0.5, 9.0), (1.0, 6.0)] {
            traversal.step.as_mut().unwrap().progress = progress;
            actor.center = from.lerp(to, progress) + Vec2::Y * 8.0;
            let (height, _) = traversal_height(&frame, &actor, &[0.0, 6.0], Some(&traversal)).unwrap();
            assert_eq!(height, expected);
            assert_eq!(actor_foot(&actor), from.lerp(to, progress), "hop never moves the ground foot");
        }
        traversal.step.as_mut().unwrap().progress = 0.5;
        actor.center = from.lerp(to, 0.5) + Vec2::Y * 8.0;
        for heights in [[0.0, 0.0], [0.0, 2.0], [6.0, 0.0]] {
            assert_eq!(traversal_height(&frame, &actor, &heights, Some(&traversal)).unwrap().1, 0.0,
                "flat ground, stairs, and ordinary descent keep their gait");
        }
        traversal.step.as_mut().unwrap().ledge_jump = true;
        assert!(traversal_height(&frame, &actor, &[6.0, 0.0], Some(&traversal)).unwrap().0 > 6.0);
        actor.id = VisualActorId::Object(1);
        assert_eq!(traversal_height(&frame, &actor, &[0.0, 6.0], Some(&traversal)).unwrap().1, 0.0);
        actor.id = VisualActorId::Player;
        traversal.map_id = "OtherMap".into();
        assert_eq!(traversal_height(&frame, &actor, &[0.0, 6.0], Some(&traversal)).unwrap().1, 0.0);
    }

    #[test]
    fn actor_card_is_anchored_at_the_bottom_center() {
        let actor = VisualActor { model_palette: None,
            id: VisualActorId::Player,
            source_id: Arc::from("player"),
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData),
            center: Vec2::new(10.0, 20.0),
            size: Vec2::new(8.0, 16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
        };
        assert_eq!(actor_foot(&actor), Vec2::new(10.0, 12.0));
    }

    #[test]
    fn modeled_ground_point_centers_edge_tiles_at_every_map_scale() {
        for scale in [1., 4.] {
            let mut frame = frame();
            frame.center = Vec2::new(-37., 29.);
            frame.tile_size = Vec2::splat(8. * scale);
            frame.grid_size = UVec2::new(12, 8);
            frame.viewport_size = frame.grid_size.as_vec2() * frame.tile_size;
            frame.source_map_size_core_tiles = Some(UVec2::new(6, 4));
            let west = frame.center.x - frame.viewport_size.x * 0.5;
            let north = frame.center.y + frame.viewport_size.y * 0.5;
            for (x, y) in [(0, 0), (5, 0), (0, 3), (5, 3)] {
                let expected = Vec2::new(west + (x as f32 * 2. + 1.) * frame.tile_size.x,
                    north - (y as f32 * 2. + 1.) * frame.tile_size.y);
                for height in [16., 32.] {
                    let mut actor = VisualActor {
                        id: VisualActorId::Player, source_id: "chris".into(), model_palette: None,
                        texture: Handle::default(), center: expected + Vec2::Y * (height * scale * 0.5 - frame.tile_size.y),
                        size: Vec2::new(16., height) * scale, flip_x: false,
                        above_priority: false, facing: Some(Vec2::NEG_Y),
                    };
                    let source = actor.clone();
                    for facing in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
                        actor.facing = Some(facing);
                        assert_eq!(actor_ground_point(&frame, &actor), expected);
                        assert_eq!(tile_at_visual_point(&frame, actor_ground_point(&frame, &actor)),
                            Some(UVec2::new(x * 2 + 1, y * 2 + 1)));
                    }
                    actor.facing = source.facing;
                    assert_eq!(actor, source, "centering never changes the source sprite");
                }
            }
        }
    }

    #[test]
    fn footing_uses_explicit_tile_coordinates_not_vector_order() {
        let frame = frame();
        let left_cell = Vec2::new(-20.0, 16.0);
        let right_cell = Vec2::new(-12.0, 16.0);

        assert_eq!(footing_height(&frame, left_cell), Some(0.0));
        assert_eq!(footing_height(&frame, right_cell), Some(0.0));
    }

    #[test]
    fn voxel_coordinates_preserve_absolute_presentation_position() {
        assert_eq!(
            visual_point_to_voxel(Vec2::new(-8.0, 12.0), 3.0),
            Vec3::new(-8.0, 3.0, -12.0),
        );
    }

    #[test]
    fn points_on_the_exclusive_right_edge_are_not_clamped() {
        let frame = frame();
        let right_edge = frame.center + Vec2::new(frame.viewport_size.x * 0.5, 0.0);
        assert_eq!(tile_at_visual_point(&frame, right_edge), None);
        assert_eq!(footing_height(&frame, right_edge), Some(0.0));
    }

    #[test]
    fn boundary_footing_samples_last_occupied_row_but_keeps_door_support_ground() {
        let mut frame = frame();
        frame.center = Vec2::ZERO;
        frame.viewport_size = Vec2::new(8.0, 16.0);
        frame.grid_size = UVec2::new(1, 2);
        frame.tiles = vec![
            VisualTile {
                animation_frames: None,
                column: 0,
                row: 0,
                source: source(0x16, 3),
                texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData),
                priority: true,
            },
            VisualTile {
                animation_frames: None,
                column: 0,
                row: 1,
                source: source(0x01, 0),
                texture: Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(3), std::marker::PhantomData),
                priority: false,
            },
        ];

        assert_eq!(
            tile_at_visual_point(&frame, Vec2::ZERO),
            Some(UVec2::new(0, 1))
        );
        assert_eq!(footing_height(&frame, Vec2::ZERO), Some(0.0));
    }
}
