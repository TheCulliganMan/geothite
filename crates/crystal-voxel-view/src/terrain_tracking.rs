//! Keep one authored geometry/atlas pair anchored while the camera scrolls.
#[cfg(test)]
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use crystal_render_api::VisualWorldFrame;

/// Leave at least 64 source cells of the expanded outdoor halo resident even
/// when worker publication falls behind movement. Smaller fixtures and rooms
/// retain their existing proportional margin.
pub(super) fn retention_allowance(frame: &VisualWorldFrame) -> IVec2 {
    let visible = (frame.viewport_size / frame.tile_size).ceil().as_ivec2();
    let allowance = ((frame.grid_size.as_ivec2() - visible) / 4).max(IVec2::ZERO);
    if frame.grid_size.min_element() >= 192 {
        // The host retains its source atlas through another 16 cells of LCD
        // scrolling. Together these reserves leave the distant view resident.
        allowance.min(IVec2::splat(16))
    } else {
        allowance
    }
}

pub(super) fn offset(built: &VisualWorldFrame, live: &VisualWorldFrame) -> Vec2 {
    let shift = built.connected_source_shift(live).unwrap_or(IVec2::ZERO);
    (built.grid_origin + shift - live.grid_origin).as_vec2() * live.tile_size
}

/// Prove that the same retained source cells cover a connected viewport.
/// The atlas and build receipt remain owned by their original source frame.
pub fn connected_frame_shift(built: &VisualWorldFrame, live: &VisualWorldFrame) -> Result<IVec2, &'static str> {
    if built.map_id == live.map_id { return Err("same map uses ordinary retention"); }
    let _trace=crate::loading_trace::span("region.prove_connected", || serde_json::json!({"map":live.map_id.as_ref()}));
    if !built.active || !live.active || built.validate().is_err() || live.validate().is_err() {
        return Err("invalid connected frame");
    }
    let shift = built.connected_source_shift(live).ok_or("unproven connected placement")?;
    if built.grid_size != live.grid_size || built.tile_size != live.tile_size
        || built.viewport_size != live.viewport_size || built.tiles.len() != live.tiles.len()
        || built.tiles.is_empty() { return Err("different retained grid geometry"); }
    if crate::interior_cutaway::is_cave(built) || crate::interior_cutaway::is_cave(live)
        || built.tiles.iter().chain(&live.tiles).any(|t| crate::interior::has_back_wall(&t.source.tileset_id)) {
        return Err("connected retention requires outdoor terrain");
    }
    let delta = live.grid_origin - built.grid_origin - shift;
    let allowance = retention_allowance(live);
    if delta.abs().cmpgt(allowance).any() { return Err("connected view exhausts retained halo"); }
    let owned = |frame: &VisualWorldFrame, p: IVec2| frame.source_regions.iter().any(|r| {
        let q = p - r.origin;
        q.cmpge(IVec2::ZERO).all() && q.cmplt(r.size.as_ivec2()).all()
    });
    let levels = |frame: &VisualWorldFrame, p: IVec2| -> ([f32; 4], bool, Option<f32>, bool) {
        let Some(d) = frame.terrace_datum.as_deref() else { return ([0.; 4], false, None, false); };
        let q = p - d.origin;
        let height = if q.cmpge(IVec2::ZERO).all() && q.cmplt(d.size.as_ivec2()).all() {
            d.levels[(q.y as u32 * d.size.x + q.x as u32) as usize]
        } else { [0.; 4] };
        (height, d.water_barrier_at(p), d.open_water_at(p),
            d.walkable_mineral.binary_search_by_key(&(p.y, p.x), |point| (point.y, point.x)).is_ok())
    };
    let width = built.grid_size.x as i32;
    let height = built.grid_size.y as i32;
    for tile in &live.tiles {
        let new = IVec2::new(tile.column as i32, tile.row as i32);
        let old = new + delta;
        // The outer quarter halo is reserved for replacement work, as in
        // ordinary same-map retention. Compare the complete retained inner
        // band, including its context beyond the LCD viewport.
        if new.cmplt(allowance).any()
            || new.cmpge(live.grid_size.as_ivec2() - allowance).any() { continue; }
        if old.x < 0 || old.y < 0 || old.x >= width || old.y >= height { continue; }
        let previous = &built.tiles[(old.y * width + old.x) as usize];
        let old_point = built.grid_origin + old;
        let new_point = live.grid_origin + new;
        if previous.source != tile.source || previous.priority != tile.priority {
            return Err("connected source cells differ");
        }
        if previous.texture != tile.texture && !(previous.animation_frames.is_some()
            && previous.animation_frames == tile.animation_frames) {
            return Err("connected source art differs");
        }
        if !owned(built, old_point) && !owned(live, new_point) {
            // Synthetic fill is not native acreage, but its source border,
            // local seeds and paint must remain stable through the handoff.
            if built.scenery_map_at(old.x as u32,old.y as u32) != live.scenery_map_at(tile.column,tile.row)
                || built.scenery_grid_origin_at(old.x as u32,old.y as u32)+old
                    != live.scenery_grid_origin_at(tile.column,tile.row)+new {
                return Err("connected border style differs");
            }
            continue;
        }
        if owned(built, old_point) || owned(live, new_point) {
            if !owned(built, old_point) || !owned(live, new_point)
                || built.source_map_at(old.x as u32, old.y as u32) != live.source_map_at(tile.column, tile.row)
                || built.source_grid_origin_at(old.x as u32, old.y as u32) + old
                    != live.source_grid_origin_at(tile.column, tile.row) + new {
                return Err("connected source ownership differs");
            }
        }
        if crate::profile::shape_for_source_on_map(
            built.scenery_map_at(old.x as u32, old.y as u32), &previous.source)
            != crate::profile::shape_for_source_on_map(
                live.scenery_map_at(tile.column, tile.row), &tile.source) {
            return Err("connected shape context differs");
        }
        if levels(built, old_point) != levels(live, new_point) {
            return Err("connected elevation context differs");
        }
    }
    Ok(shift)
}

pub(super) fn can_reuse(built: &VisualWorldFrame, live: &VisualWorldFrame) -> bool {
    if built.map_id != live.map_id {
        // Authored map ownership does not retire resident world geometry.
        // Reuse only after proving source art, shapes, elevation and coverage
        // in the destination's coordinate basis.
        return connected_frame_shift(built, live).is_ok();
    }
    if built.source_map_size_core_tiles != live.source_map_size_core_tiles
        || built.boundary_warps != live.boundary_warps
        || built.jump_ledges != live.jump_ledges
        || built.terrace_datum != live.terrace_datum
        || built.grid_size != live.grid_size
        || built.tile_size != live.tile_size
        || built.viewport_size != live.viewport_size
        || built.tiles.len() != live.tiles.len()
        || built.tiles.is_empty()
    {
        return false;
    }
    let delta = live.grid_origin - built.grid_origin
        - built.connected_source_shift(live).unwrap_or(IVec2::ZERO);
    // Retain half the authored halo for camera pitch and complete objects.
    let allowance = retention_allowance(live);
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
            let well_paint = b.source.tileset_id.as_ref() == "johto_modern"
                && (0x5b..=0x5e).contains(&b.source.metatile_id)
                && matches!(b.source.tile_index, 0x29 | 0x5b);
            // The gate and threshold now bake live pigment even though their
            // source cells remain flat. Atlas refresh alone cannot recolor them.
            let park_paint = b.source.tileset_id.as_ref() == "park"
                && ((matches!(b.source.metatile_id,
                    0x28..=0x2a | 0x2c..=0x2e | 0x30..=0x32)
                    && matches!(b.source.tile_index, 0x21 | 0x41))
                    || (matches!(b.source.metatile_id, 0x0a | 0x0b)
                        && b.source.tile_index == 0x4f));
            if a.texture != b.texture
                && (well_paint || park_paint || !matches!(
                    crate::profile::shape_for_source_on_map(live.map_id.as_ref(), &b.source),
                    crate::profile::CellShape::Flat
                        | crate::profile::CellShape::Water
                        | crate::profile::CellShape::Waterfall
                        | crate::profile::CellShape::Cutout {
                            solid: crate::profile::SolidKind::Flower,
                            ..
                        }
                ))
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
    let delta = live.grid_origin - built.grid_origin
        - built.connected_source_shift(live).unwrap_or(IVec2::ZERO);
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
            if image.width() != 8 || image.height() != 8 || image.data.as_ref().expect("CPU image pixels").len() != 256 {
                return Err(());
            }
            flowers_changed |= crate::flower::flower_shape(&tile.source).is_some()
                || crate::mesh::animated_pillar_source(live.map_id.as_ref(), &tile.source);
            updates.push((index, tile.texture.clone(), image.data.as_ref().expect("CPU image pixels").clone()));
        }
    }
    if !updates.is_empty() {
        let mut atlas = images.get_mut(&built.map_texture).ok_or(())?;
        if atlas.width() != width as u32 * 8 || atlas.height() != height as u32 * 8 {
            return Err(());
        }
        for (index, handle, pixels) in updates {
            let x = index % width as usize * 8;
            let y = index / width as usize * 8;
            for row in 0..8 {
                let start = ((y + row) * width as usize * 8 + x) * 4;
                atlas.data.as_mut().expect("CPU image pixels")[start..start + 32].copy_from_slice(&pixels[row * 32..row * 32 + 32]);
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
    let delta = live.grid_origin - built.grid_origin
        - built.connected_source_shift(live).unwrap_or(IVec2::ZERO);
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
    include!("connected_residency_tests.rs");
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
                    animation_frames: None,
                    column: x,
                    row: y,
                    source: VisualTileSource {
                        tileset_id: "test".into(),
                        metatile_id: id,
                        subtile_column: 0,
                        subtile_row: 0,
                        tile_index: id,
                    },
                    texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(id as u128 + 1), std::marker::PhantomData),
                    priority: false,
                });
            }
        }
        frame
    }
    #[test]
    fn source_painted_johto_fences_and_roofs_refresh_when_palette_handles_change() {
        for (metatile,row,tile) in [(0x40,0,0x5a),(0x40,1,0x59),(0x18,0,0x20)] {
            let mut built=frame(IVec2::ZERO);
            built.map_id="CherrygroveCity".into();
            built.tiles[400].source=VisualTileSource {
                tileset_id:"johto".into(),metatile_id:metatile,
                subtile_column:0,subtile_row:row,tile_index:tile,
            };
            let mut live=built.clone();
            assert!(can_reuse(&built,&live));
            // The source renderer publishes a different paletted image handle
            // at a time-of-day boundary. Baked source paint must be resampled,
            // even though the map layout and authoritative footing are identical.
            live.tiles[400].texture=Handle::Uuid(
                bevy::asset::uuid::Uuid::from_u128(999999),std::marker::PhantomData);
            assert!(!can_reuse(&built,&live),"stale source paint for {metatile:x}/{tile:x}");
            assert_eq!(built.grid_origin,live.grid_origin);
            assert_eq!(built.tiles[400].source,live.tiles[400].source);
        }
    }

    #[test]
    fn park_gate_and_threshold_palette_changes_rebuild_live_paint() {
        for map in ["NationalPark", "NationalParkBugContest"] {
            for (block, column, row, tile) in [
                (0x29, 1, 2, 0x21), // Gate roof pigment donor.
                (0x2d, 1, 3, 0x41), // Gate masonry pigment donor.
                (0x0a, 0, 2, 0x4f), // North threshold.
                (0x0b, 2, 0, 0x4f), // East threshold.
            ] {
                let mut built = frame(IVec2::ZERO);
                built.map_id = map.into();
                built.tiles[400].source = VisualTileSource {
                    tileset_id: "park".into(),
                    metatile_id: block,
                    subtile_column: column,
                    subtile_row: row,
                    tile_index: tile,
                };
                assert!(matches!(
                    crate::profile::shape_for_source_on_map(map, &built.tiles[400].source),
                    crate::profile::CellShape::Flat
                ));
                let mut live = built.clone();
                assert!(can_reuse(&built, &live));
                live.tiles[400].texture = Handle::Uuid(
                    bevy::asset::uuid::Uuid::from_u128(999999),
                    std::marker::PhantomData,
                );
                assert!(!can_reuse(&built, &live), "stale park paint: {map}/{block:x}");
                assert_eq!(built.tiles[400].source, live.tiles[400].source);
            }
            let mut built = frame(IVec2::ZERO);
            built.map_id = map.into();
            built.tiles[400].source = VisualTileSource {
                tileset_id: "park".into(), metatile_id: 1,
                subtile_column: 0, subtile_row: 0, tile_index: 0,
            };
            let mut live = built.clone();
            live.tiles[400].texture = Handle::Uuid(
                bevy::asset::uuid::Uuid::from_u128(999999), std::marker::PhantomData,
            );
            assert!(can_reuse(&built, &live), "ordinary flat paving still uses atlas refresh");
        }
    }

    #[test]
    fn well_stone_and_stair_palette_changes_rebuild_baked_paint() {
        for (block, column, row, tile) in [(0x5b, 0, 0, 0x29), (0x5d, 2, 0, 0x5b)] {
            let mut built = frame(IVec2::ZERO);
            built.map_id = "AzaleaTown".into();
            built.tiles[400].source = VisualTileSource {
                tileset_id: "johto_modern".into(), metatile_id: block,
                subtile_column: column, subtile_row: row, tile_index: tile,
            };
            assert!(matches!(crate::profile::shape_for_source(&built.tiles[400].source),
                crate::profile::CellShape::Flat));
            let mut live = built.clone();
            assert!(can_reuse(&built, &live));
            live.tiles[400].texture = Handle::Uuid(
                bevy::asset::uuid::Uuid::from_u128(999999), std::marker::PhantomData);
            assert!(!can_reuse(&built, &live));
            assert_eq!(built.tiles[400].source, live.tiles[400].source);
            // Flat lawn animation still follows the atlas-only refresh path.
            built.tiles[400].source.tile_index = 0x05;
            live.tiles[400].source = built.tiles[400].source.clone();
            assert!(can_reuse(&built, &live));
        }
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
        changed = frame(IVec2::new(2, 0));
        changed.source_map_size_core_tiles = Some(UVec2::splat(20));
        assert!(!can_reuse(&built, &changed), "changed source acreage requires a new partition");
        changed = frame(IVec2::new(2, 0));
        changed.boundary_warps = vec![IVec2::new(3, 7)].into();
        assert!(!can_reuse(&built, &changed), "changed exits require new wall openings");
        changed = frame(IVec2::new(2, 0));
        changed.jump_ledges = vec![crystal_render_api::VisualJumpLedge {
            start: IVec2::new(0,4), end: IVec2::new(8,4), descent: IVec2::Y,
        }].into();
        assert!(!can_reuse(&built, &changed), "changed authored jump levels require new terrain support");
        changed = frame(IVec2::new(2, 0));
        changed.terrace_datum=Some(std::sync::Arc::new(crystal_render_api::VisualTerraceDatum {walkable_mineral:Default::default(),open_water:None,water_barriers:std::sync::Arc::from([]),
            origin:IVec2::ZERO,size:UVec2::ONE,levels:vec![[0.;4]].into(),
        }));
        assert!(!can_reuse(&built,&changed),"changed source land domains require new terrain and support");
        let previous=changed.clone();
        std::sync::Arc::make_mut(changed.terrace_datum.as_mut().unwrap()).water_barriers=vec![IVec2::ZERO].into();
        assert!(!can_reuse(&previous,&changed),"a water-backing-only change must rebuild terrain");
        let previous=changed.clone();
        std::sync::Arc::make_mut(changed.terrace_datum.as_mut().unwrap()).open_water=
            Some(crystal_render_api::VisualOpenWaterDatum {level:2.,cells:vec![IVec2::ZERO].into()});
        assert!(!can_reuse(&previous,&changed),"a river-level-only change must rebuild terrain and support");
    }
    #[test]
    fn flower_animation_does_not_rebuild_static_terrain() {
        let mut built = frame(IVec2::ZERO);
        built.tiles[400].source.tileset_id = "johto".into();
        built.tiles[400].source.tile_index = 3;
        let mut live = built.clone();
        live.tiles[400].texture = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(900000), std::marker::PhantomData);
        assert!(can_reuse(&built, &live));
    }

    #[test]
    fn animated_pixels_update_the_built_slot_after_viewport_shift() {
        use bevy::{

            render::render_resource::{
            Extent3d, TextureDimension, TextureFormat},
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
        images.get_mut(&live_atlas).unwrap().data.as_mut().expect("CPU image pixels").fill(222);
        assert_eq!(&images.get(&owned).unwrap().data.as_ref().expect("CPU image pixels")[..4], &[10, 20, 30, 255]);
        assert!(can_reuse(&built, &live));
        refresh_animation(&mut built, &live, &mut images).unwrap();
        let image = images.get(&owned).unwrap();
        let old_slot = ((14 * 8) * 224 + 14 * 8) * 4;
        let scrolling_slot = ((14 * 8) * 224 + 12 * 8) * 4;
        assert_eq!(&image.data.as_ref().expect("CPU image pixels")[old_slot..old_slot + 4], &[80, 90, 100, 255]);
        assert_eq!(
            &image.data.as_ref().expect("CPU image pixels")[scrolling_slot..scrolling_slot + 4],
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
