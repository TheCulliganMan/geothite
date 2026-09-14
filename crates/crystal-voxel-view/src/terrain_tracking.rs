//! Keep one authored geometry/atlas pair anchored while the camera scrolls.
use bevy::prelude::*;
use crystal_render_api::VisualWorldFrame;

pub(super) fn offset(built: &VisualWorldFrame, live: &VisualWorldFrame) -> Vec2 {
    (built.grid_origin - live.grid_origin).as_vec2() * live.tile_size
}

pub(super) fn can_reuse(built: &VisualWorldFrame, live: &VisualWorldFrame) -> bool {
    if built.map_id != live.map_id
        || built.grid_size != live.grid_size
        || built.tile_size != live.tile_size
        || built.viewport_size != live.viewport_size
        || built.tiles.len() != live.tiles.len()
        || built.tiles.is_empty()
    {
        return false;
    }
    let delta = live.grid_origin - built.grid_origin;
    // Retain half the authored halo for camera pitch and complete objects.
    let visible = (live.viewport_size / live.tile_size).ceil().as_ivec2();
    let allowance = ((live.grid_size.as_ivec2() - visible) / 4).max(IVec2::ZERO);
    if delta.abs().cmpgt(allowance).any() {
        return false;
    }
    let width = live.grid_size.x as i32;
    let height = live.grid_size.y as i32;
    for y in 0..height {
        for x in 0..width {
            let old = IVec2::new(x, y) + delta;
            if old.x < 0 || old.y < 0 || old.x >= width || old.y >= height {
                continue;
            }
            let a = &built.tiles[(old.y * width + old.x) as usize];
            let b = &live.tiles[(y * width + x) as usize];
            if a.source != b.source || a.priority != b.priority {
                return false;
            }
            if a.texture != b.texture
                && !matches!(
                    crate::profile::shape_for_source_on_map(live.map_id.as_ref(), &b.source),
                    crate::profile::CellShape::Flat
                        | crate::profile::CellShape::Water
                        | crate::profile::CellShape::Waterfall
                        | crate::profile::CellShape::Cutout {
                            solid: crate::profile::SolidKind::Flower,
                            ..
                        }
                )
            {
                return false;
            }
        }
    }
    true
}

/// Refresh animated flat tiles at their BUILT atlas slots, never at the
/// scrolling viewport slots. Geometry-derived silhouettes still rebuild.
pub(super) fn refresh_animation(
    built: &mut VisualWorldFrame,
    live: &VisualWorldFrame,
    images: &mut Assets<Image>,
) -> Result<bool, ()> {
    let delta = live.grid_origin - built.grid_origin;
    let width = built.grid_size.x as i32;
    let height = built.grid_size.y as i32;
    let mut updates = Vec::new();
    let mut flowers_changed = false;
    for tile in &live.tiles {
        let old = IVec2::new(tile.column as i32, tile.row as i32) + delta;
        if old.x < 0 || old.y < 0 || old.x >= width || old.y >= height {
            continue;
        }
        let index = (old.y * width + old.x) as usize;
        if built.tiles[index].texture != tile.texture {
            let image = images.get(&tile.texture).ok_or(())?;
            if image.width() != 8 || image.height() != 8 || image.data.len() != 256 {
                return Err(());
            }
            flowers_changed |= crate::flower::flower_shape(&tile.source).is_some();
            updates.push((index, tile.texture.clone(), image.data.clone()));
        }
    }
    if !updates.is_empty() {
        let atlas = images.get_mut(&built.map_texture).ok_or(())?;
        if atlas.width() != width as u32 * 8 || atlas.height() != height as u32 * 8 {
            return Err(());
        }
        for (index, handle, pixels) in updates {
            let x = index % width as usize * 8;
            let y = index / width as usize * 8;
            for row in 0..8 {
                let start = ((y + row) * width as usize * 8 + x) * 4;
                atlas.data[start..start + 32].copy_from_slice(&pixels[row * 32..row * 32 + 32]);
            }
            built.tiles[index].texture = handle;
        }
    }
    built.terrain_revision = live.terrain_revision;
    Ok(flowers_changed)
}

pub(super) fn align_footings(
    built: &VisualWorldFrame,
    live: &VisualWorldFrame,
    heights: &[f32],
    output: &mut Vec<f32>,
) {
    output.clear();
    let delta = live.grid_origin - built.grid_origin;
    let width = built.grid_size.x as i32;
    let height = built.grid_size.y as i32;
    for y in 0..live.grid_size.y as i32 {
        for x in 0..live.grid_size.x as i32 {
            let old = IVec2::new(x, y) + delta;
            // These outer halo cells are outside the retained mesh. NaN makes
            // accidental sampling detectable instead of inventing ground.
            output.push(
                if old.x >= 0 && old.y >= 0 && old.x < width && old.y < height {
                    heights[(old.y * width + old.x) as usize]
                } else {
                    f32::NAN
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileSource};
    fn frame(origin: IVec2) -> VisualWorldFrame {
        let width = 28;
        let mut frame = VisualWorldFrame {
            map_id: "test-map".into(),
            grid_origin: origin,
            grid_size: UVec2::splat(width),
            viewport_size: Vec2::splat(32.0),
            tile_size: Vec2::splat(8.0),
            ..default()
        };
        for y in 0..width {
            for x in 0..width {
                let id = ((origin.y + y as i32) * 100 + origin.x + x as i32 + 10000) as u16;
                frame.tiles.push(VisualTile {
                    column: x,
                    row: y,
                    source: VisualTileSource {
                        tileset_id: "test".into(),
                        metatile_id: id,
                        subtile_column: 0,
                        subtile_row: 0,
                        tile_index: id,
                    },
                    texture: Handle::weak_from_u128(id as u128 + 1),
                    priority: false,
                });
            }
        }
        frame
    }
    #[test]
    fn walking_reuses_geometry_but_changes_and_halo_exhaustion_rebuild() {
        let built = frame(IVec2::ZERO);
        for direction in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            for step in 1..=6 {
                assert!(can_reuse(&built, &frame(direction * step)));
            }
            assert!(!can_reuse(&built, &frame(direction * 7)));
        }
        let mut changed = frame(IVec2::new(2, 0));
        changed.tiles[400].source.tile_index += 1;
        assert!(!can_reuse(&built, &changed));
        changed = frame(IVec2::new(2, 0));
        changed.map_id = "other".into();
        assert!(!can_reuse(&built, &changed));
    }
    #[test]
    fn flower_animation_does_not_rebuild_static_terrain() {
        let mut built = frame(IVec2::ZERO);
        built.tiles[400].source.tileset_id = "johto".into();
        built.tiles[400].source.tile_index = 3;
        let mut live = built.clone();
        live.tiles[400].texture = Handle::weak_from_u128(900000);
        assert!(can_reuse(&built, &live));
    }

    #[test]
    fn animated_pixels_update_the_built_slot_after_viewport_shift() {
        use bevy::render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        };
        let mut images = Assets::<Image>::default();
        let make = |size: u32, color: [u8; 4]| {
            Image::new_fill(
                Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &color,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )
        };
        let original = images.add(make(8, [10, 20, 30, 255]));
        let animated = images.add(make(8, [80, 90, 100, 255]));
        let live_atlas = images.add(make(224, [10, 20, 30, 255]));
        let owned = images.add(images.get(&live_atlas).unwrap().clone());
        let mut built = frame(IVec2::ZERO);
        built.map_texture = owned.clone();
        for tile in &mut built.tiles {
            tile.texture = original.clone();
        }
        let mut live = frame(IVec2::new(2, 0));
        live.map_texture = live_atlas.clone();
        for tile in &mut live.tiles {
            tile.texture = original.clone();
        }
        live.tiles[14 * 28 + 12].texture = animated;
        // Host scroll overwrites its own image while the old mesh is visible.
        images.get_mut(&live_atlas).unwrap().data.fill(222);
        assert_eq!(&images.get(&owned).unwrap().data[..4], &[10, 20, 30, 255]);
        assert!(can_reuse(&built, &live));
        refresh_animation(&mut built, &live, &mut images).unwrap();
        let image = images.get(&owned).unwrap();
        let old_slot = ((14 * 8) * 224 + 14 * 8) * 4;
        let scrolling_slot = ((14 * 8) * 224 + 12 * 8) * 4;
        assert_eq!(&image.data[old_slot..old_slot + 4], &[80, 90, 100, 255]);
        assert_eq!(
            &image.data[scrolling_slot..scrolling_slot + 4],
            &[10, 20, 30, 255]
        );
    }

    #[test]
    fn raised_footing_stays_with_its_world_tile_after_step() {
        let built = frame(IVec2::ZERO);
        let live = frame(IVec2::new(2, 0));
        let mut heights = vec![0.0; 28 * 28];
        heights[14 * 28 + 14] = 16.0;
        let mut shifted = Vec::new();
        align_footings(&built, &live, &heights, &mut shifted);
        assert_eq!(shifted[14 * 28 + 12], 16.0);
        assert_eq!(shifted[14 * 28 + 14], 0.0);
        assert!(shifted[27].is_nan());
    }
}
