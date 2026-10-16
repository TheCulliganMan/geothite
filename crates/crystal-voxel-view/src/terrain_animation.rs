//! Authenticated sparse texture phases and selected prop poses; terrain stays cached.
use super::*;
use crystal_render_api::VisualTileAnimationFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    None,
    Atlas,
    Geometry,
}

fn refresh(
    built: &mut VisualWorldFrame,
    live: &VisualTileAnimationFrame,
    selected: &[[usize; 2]],
    images: &mut Assets<Image>,
) -> Result<Change, ()> {
    if built.map_id != live.map_id {
        return Ok(Change::None);
    }
    let width = built.grid_size.x as usize;
    let height = built.grid_size.y as usize;
    if width.checked_mul(height) != Some(built.tiles.len()) {
        return Err(());
    }
    let delta = live.grid_origin - built.grid_origin;
    let mut updates = Vec::new();
    let mut geometry_changed = false;
    for tile in &live.tiles {
        let slot = IVec2::new(tile.column as i32, tile.row as i32) + delta;
        if slot.x < 0 || slot.y < 0 || slot.x as usize >= width || slot.y as usize >= height {
            continue;
        }
        let [x, y] = [slot.x as usize, slot.y as usize];
        let index = y * width + x;
        let old = &built.tiles[index];
        if old.source != tile.source
            || old.animation_frames != tile.animation_frames
            || old.texture == tile.texture
        {
            continue;
        }
        let Some(family) = old.animation_frames.as_ref() else {
            continue;
        };
        if !family.contains(&tile.texture) {
            continue;
        }
        let source = images.get(&tile.texture).ok_or(())?;
        let pixels = source.data.as_ref().ok_or(())?;
        if source.width() != 8 || source.height() != 8 || pixels.len() != 256 {
            return Err(());
        }
        // Native blooms persist across source animation phases; only authored
        // pillar poses need a geometry upload.
        geometry_changed |= mesh::animated_pillar_source(&built.map_id, &tile.source)
            && selected
                .iter()
                .any(|&[a, b]| x >= a && x - a < 4 && y >= b && y - b < 6);
        updates.push((index, tile.texture.clone(), pixels.clone()));
    }
    if updates.is_empty() {
        return Ok(Change::None);
    }
    let mut atlas = images.get_mut(&built.map_texture).ok_or(())?;
    if atlas.width() != built.grid_size.x * 8 || atlas.height() != built.grid_size.y * 8 {
        return Err(());
    }
    let pixels = atlas.data.as_mut().ok_or(())?;
    if pixels.len() != width * height * 256 {
        return Err(());
    }
    for (index, handle, source) in updates {
        let x = index % width * 8;
        let y = index / width * 8;
        for row in 0..8 {
            let start = ((y + row) * width * 8 + x) * 4;
            pixels[start..start + 32].copy_from_slice(&source[row * 32..row * 32 + 32]);
        }
        built.tiles[index].texture = handle;
    }
    // Sparse phases never advance simulation or invalidate the terrain key.
    Ok(if geometry_changed {
        Change::Geometry
    } else {
        Change::Atlas
    })
}

fn update_domain(meshes: &mut Assets<Mesh>, cached: &mut Option<Handle<Mesh>>, next: Mesh) {
    // Keep an empty domain empty without repeatedly uploading it. A domain
    // that was previously populated still receives its clearing revision.
    if next.count_vertices() == 0
        && !cached
            .as_ref()
            .and_then(|h| meshes.get(h))
            .is_some_and(|m| m.count_vertices() > 0)
    {
        return;
    }
    update_mesh_asset(meshes, cached, retain_animated_mesh_cpu(next));
}

pub(super) fn sync(
    frame: Res<VisualWorldFrame>,
    phases: Option<Res<VisualTileAnimationFrame>>,
    mut status: ResMut<VoxelViewStatus>,
    mut cache: ResMut<TerrainRevisionCache>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    if !status.active || !frame.active {
        return;
    }
    let Some(phases) = phases else {
        return;
    };
    if phases.map_id != frame.map_id || phases.tiles.is_empty() {
        return;
    }
    if let Err(error) = sync_cache(&phases, &mut cache, &mut images, &mut meshes, &mut materials) {
        status.active = false;
        status.inactive_reason = Some(error);
    }
}

pub(super) fn sync_cache(
    phases: &VisualTileAnimationFrame,
    cache: &mut TerrainRevisionCache,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
) -> Result<(), String> {
    let selected = cache.built_animated_pillars.clone();
    let Some(built) = cache.built_frame.as_mut() else { return Ok(()); };
    let change = refresh(built, phases, &selected, images)
        .map_err(|()| "terrain animation: SourceTextureUnavailable".to_owned())?;
    if change == Change::None { return Ok(()); }
    if change == Change::Geometry {
        let (textured, solid) = mesh::build_animated_flowers(built, images, &selected)
            .map_err(|error| format!("terrain animation: {error:?}"))?;
        cache.built_animated_bounds = [textured.get_aabb(), solid.get_aabb()];
        update_domain(meshes, &mut cache.animated_textured_mesh, textured);
        update_domain(meshes, &mut cache.animated_solid_mesh, solid);
    }
    // Atlas uploads replace their GPU texture. Refresh every bound material,
    // including cutaway/reveal clones, without rebuilding static geometry.
    let atlas = cache.built_frame.as_ref().unwrap().map_texture.clone();
    let users: Vec<_> = materials.iter().filter_map(|(id, m)|
        (m.base.base_color_texture.as_ref() == Some(&atlas)).then_some(id)).collect();
    for id in users { let _ = materials.get_mut(id); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::RenderAssetUsages,
        mesh::VertexAttributeValues,
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    use crystal_render_api::{VisualTile, VisualTileSource};
    fn image(width: u32, height: u32, color: [u8; 4]) -> Image {
        Image::new_fill(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &color,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
    fn fixture() -> (VisualWorldFrame, Assets<Image>) {
        let mut images = Assets::default();
        let handles: Vec<_> = (0..5)
            .map(|n| images.add(image(8, 8, [n * 30, 80, 100, 255])))
            .collect();
        let family: std::sync::Arc<[Handle<Image>]> =
            [0, 1, 2, 3, 4, 3, 2, 1].map(|i| handles[i].clone()).into();
        let drawing = [
            [0x2d, 0x2e, 0x2e, 0x2f],
            [0x3d, 0x3e, 0x3e, 0x3f],
            [0x3c, 0x3e, 1, 0x2c],
            [0x4d, 0x4e, 0x4e, 0x4f],
            [0x4d, 0x4e, 0x4e, 0x4f],
            [0x5d, 0x5e, 0x5e, 0x5f],
        ];
        let tiles = (0..24)
            .map(|i| {
                let x = i % 4;
                let y = i / 4;
                let tile = drawing[y][x];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    priority: false,
                    texture: handles[2].clone(),
                    animation_frames: matches!(
                        tile,
                        0x2d | 0x2f | 0x3d | 0x3f | 0x3c | 0x2c | 0x4d | 0x4f | 0x5d | 0x5f
                    )
                    .then(|| family.clone()),
                    source: VisualTileSource {
                        tileset_id: "tower".into(),
                        metatile_id: match (x < 2, y < 4) {
                            (true, true) => 0x10,
                            (false, true) => 0x11,
                            (true, false) => 0x14,
                            (false, false) => 0x15,
                        },
                        subtile_column: ((x + 2) % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: tile,
                    },
                }
            })
            .collect();
        let frame = VisualWorldFrame {
            active: true,
            map_id: "SproutTower3F".into(),
            terrain_revision: 71,
            tile_size: Vec2::splat(8.),
            grid_size: UVec2::new(4, 6),
            viewport_size: Vec2::new(32., 48.),
            map_texture: images.add(image(32, 48, [50, 80, 100, 255])),
            tiles,
            ..Default::default()
        };
        (frame, images)
    }
    fn phases(frame: &VisualWorldFrame) -> VisualTileAnimationFrame {
        VisualTileAnimationFrame {
            map_id: frame.map_id.clone(),
            grid_origin: frame.grid_origin,
            tiles: frame
                .tiles
                .iter()
                .filter(|t| t.animation_frames.is_some())
                .cloned()
                .map(|mut t| {
                    t.texture = t.animation_frames.as_ref().unwrap()[4].clone();
                    t
                })
                .collect(),
        }
    }
    fn positions(mesh: &Mesh) -> Vec<[f32; 3]> {
        let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("position attribute")
        };
        p.clone()
    }
    #[test]
    fn flower_texture_phases_do_not_remove_blooms_or_rebuild_geometry() {
        let (mut frame, mut images) = fixture();
        frame.map_id = "Route35".into();
        for tile in &mut frame.tiles {
            tile.source.tileset_id = "johto".into();
            tile.source.tile_index = 3;
        }
        let live = phases(&frame);
        assert_eq!(refresh(&mut frame, &live, &[], &mut images), Ok(Change::Atlas));
        assert!(frame.tiles.iter().all(|tile| crate::flower::flower_shape(&tile.source).is_some()));
    }

    #[test]
    fn sparse_pillar_phase_moves_selected_mesh_with_stable_revision_handles_and_footing() {
        let (frame, mut images) = fixture();
        let live = phases(&frame);
        let original_atlas = images.get(&frame.map_texture).unwrap().data.clone();
        let original_handle = frame.map_texture.clone();
        let mut built = frame.clone();
        built.map_texture = images.add(images.get(&frame.map_texture).unwrap().clone());
        let (textured, solid) = mesh::build_animated_flowers(&built, &images, &[[0, 0]]).unwrap();
        let before = positions(&textured);
        let indices = textured.indices().unwrap().clone();
        let mut meshes = Assets::<Mesh>::default();
        let texture_handle = meshes.add(textured);
        let solid_handle = meshes.add(solid);
        let key = TerrainCacheKey::from_frame(&frame);
        let mut app = App::new();
        app.insert_resource(images)
            .insert_resource(meshes)
            .insert_resource(Assets::<VoxelMaterial>::default())
            .insert_resource(frame.clone())
            .insert_resource(live)
            .insert_resource(VoxelViewStatus {
                active: true,
                terrain_builds: 7,
                ..Default::default()
            })
            .insert_resource(TerrainRevisionCache {
                built_frame: Some(built),
                built_animated_pillars: vec![[0, 0]],
                animated_textured_mesh: Some(texture_handle.clone()),
                animated_solid_mesh: Some(solid_handle.clone()),
                built_footing_heights: vec![5.; 24],
                key: Some(key.clone()),
                ..Default::default()
            })
            .add_systems(Update, sync);
        app.update();
        let cache = app.world().resource::<TerrainRevisionCache>();
        assert_eq!(cache.key, Some(key));
        assert_eq!(cache.built_frame.as_ref().unwrap().terrain_revision, 71);
        assert_eq!(cache.built_footing_heights, vec![5.; 24]);
        assert_eq!(cache.animated_textured_mesh, Some(texture_handle.clone()));
        assert_eq!(cache.animated_solid_mesh, Some(solid_handle));
        assert_eq!(app.world().resource::<VoxelViewStatus>().terrain_builds, 7);
        assert!(app.world().resource::<VoxelViewStatus>().active);
        assert_eq!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&original_handle)
                .unwrap()
                .data,
            original_atlas,
            "publisher atlas is immutable to retained animation"
        );
        let meshes = app.world().resource::<Assets<Mesh>>();
        let changed = meshes.get(&texture_handle).unwrap();
        let after = positions(changed);
        assert_eq!(changed.indices().unwrap(), &indices);
        assert_eq!(before.len(), after.len());
        assert!(before.iter().zip(&after).any(|(a, b)| a != b));
        for (a, b) in before.iter().zip(&after).filter(|(a, _)| a[1] == 0.) {
            assert_eq!(a, b, "foot stays planted");
        }
        let once = after;
        app.update();
        assert_eq!(
            positions(
                app.world()
                    .resource::<Assets<Mesh>>()
                    .get(&texture_handle)
                    .unwrap()
            ),
            once,
            "idle phase is retained"
        );
    }
    #[test]
    fn sparse_pillar_phases_reject_unselected_stale_forged_and_cross_map_art_and_align_retained_slots()
     {
        let (frame, mut images) = fixture();
        let good = phases(&frame);
        for mode in 0..5 {
            let mut built = frame.clone();
            let mut live = good.clone();
            let selected = if mode == 0 { vec![] } else { vec![[0, 0]] };
            match mode {
                1 => live.map_id = "SproutTower2F".into(),
                2 => {
                    for t in &mut live.tiles {
                        t.source.tile_index ^= 0x80
                    }
                }
                3 => {
                    for t in &mut live.tiles {
                        t.animation_frames = None
                    }
                }
                4 => {
                    let forged = images.add(image(8, 8, [1, 2, 3, 255]));
                    for t in &mut live.tiles {
                        t.texture = forged.clone();
                    }
                }
                _ => {}
            }
            assert_eq!(
                refresh(&mut built, &live, &selected, &mut images),
                Ok(if mode == 0 {
                    Change::Atlas
                } else {
                    Change::None
                })
            );
            if mode != 0 {
                assert_eq!(built.tiles, frame.tiles);
            }
        }
        let mut built = frame.clone();
        let mut tile = good
            .tiles
            .iter()
            .find(|t| t.column == 3 && t.row == 3)
            .unwrap()
            .clone();
        tile.column = 2;
        let handle = tile.texture.clone();
        let shifted = VisualTileAnimationFrame {
            grid_origin: IVec2::new(1, 0),
            tiles: vec![tile],
            ..good
        };
        assert_eq!(
            refresh(&mut built, &shifted, &[[0, 0]], &mut images),
            Ok(Change::Geometry)
        );
        assert_eq!(built.tiles[15].texture, handle);
        assert_eq!(built.tiles[14].texture, frame.tiles[14].texture);
        assert_eq!(built.terrain_revision, frame.terrain_revision);
    }
    #[test]
    fn sparse_pillar_invalid_authenticated_image_reports_failure_without_an_atlas_or_mesh_fallback()
    {
        let (mut built, mut images) = fixture();
        let bad = images.add(image(16, 8, [1, 2, 3, 255]));
        built.tiles[0].animation_frames =
            Some(vec![built.tiles[0].texture.clone(), bad.clone()].into());
        let original_texture = built.tiles[0].texture.clone();
        let before = images.get(&built.map_texture).unwrap().data.clone();
        let mut tile = built.tiles[0].clone();
        tile.texture = bad;
        let phases = VisualTileAnimationFrame {
            map_id: built.map_id.clone(),
            grid_origin: built.grid_origin,
            tiles: vec![tile],
        };
        let original_atlas = built.map_texture.clone();
        let mut app = App::new();
        app.insert_resource(images)
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<VoxelMaterial>::default())
            .insert_resource(built.clone())
            .insert_resource(phases)
            .insert_resource(VoxelViewStatus {
                active: true,
                ..Default::default()
            })
            .insert_resource(TerrainRevisionCache {
                built_frame: Some(built),
                built_animated_pillars: vec![[0, 0]],
                built_footing_heights: vec![7.; 24],
                ..Default::default()
            })
            .add_systems(Update, sync);
        app.update();
        let status = app.world().resource::<VoxelViewStatus>();
        assert!(!status.active);
        assert!(
            status
                .inactive_reason
                .as_ref()
                .unwrap()
                .contains("SourceTextureUnavailable")
        );
        let cache = app.world().resource::<TerrainRevisionCache>();
        assert_eq!(
            cache.built_frame.as_ref().unwrap().tiles[0].texture,
            original_texture
        );
        assert_eq!(cache.built_footing_heights, vec![7.; 24]);
        assert!(cache.animated_textured_mesh.is_none());
        assert_eq!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&original_atlas)
                .unwrap()
                .data,
            before
        );
    }
    #[test]
    fn sparse_pillar_and_water_lava_phases_keep_static_mesh_and_source_atlas_while_repainting_retained_slots()
     {
        for (map, tileset, tile_index) in [
            ("CeruleanGym", "port", 0x14),
            ("BlackthornGym1F", "cave", 0x5b),
        ] {
            let (mut frame, mut images) = fixture();
            frame.map_id = map.into();
            for tile in &mut frame.tiles {
                tile.source.tileset_id = tileset.into();
                tile.source.tile_index = tile_index;
            }
            let phase = phases(&frame);
            let source_handle = frame.map_texture.clone();
            let source_pixels = images.get(&source_handle).unwrap().data.clone();
            let mut built = frame.clone();
            built.map_texture = images.add(images.get(&source_handle).unwrap().clone());
            let retained = built.map_texture.clone();
            let key = TerrainCacheKey::from_frame(&frame);
            let mut meshes = Assets::<Mesh>::default();
            let geometry = Mesh::from(Cuboid::new(2., 3., 4.));
            let before = positions(&geometry);
            let static_handle = meshes.add(geometry);
            let mut materials = Assets::<VoxelMaterial>::default();
            let material = materials.add(textured_terrain_material(retained.clone()));
            let mut app = App::new();
            app.insert_resource(frame)
                .insert_resource(phase)
                .insert_resource(images)
                .insert_resource(meshes)
                .insert_resource(materials)
                .insert_resource(VoxelViewStatus {
                    active: true,
                    terrain_builds: 5,
                    ..Default::default()
                })
                .insert_resource(TerrainRevisionCache {
                    built_frame: Some(built),
                    key: Some(key.clone()),
                    built_footing_heights: vec![9.; 24],
                    textured_material: Some(material.clone()),
                    ..Default::default()
                })
                .add_systems(Update, sync);
            app.update();
            let cache = app.world().resource::<TerrainRevisionCache>();
            assert_eq!(cache.key, Some(key));
            assert_eq!(cache.built_footing_heights, vec![9.; 24]);
            assert!(cache.animated_textured_mesh.is_none() && cache.animated_solid_mesh.is_none());
            let images = app.world().resource::<Assets<Image>>();
            assert_eq!(images.get(&source_handle).unwrap().data, source_pixels);
            assert_eq!(
                &images.get(&retained).unwrap().data.as_ref().unwrap()[..4],
                &[120, 80, 100, 255]
            );
            assert_eq!(
                positions(
                    app.world()
                        .resource::<Assets<Mesh>>()
                        .get(&static_handle)
                        .unwrap()
                ),
                before
            );
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
            assert_eq!(
                app.world()
                    .resource::<Assets<VoxelMaterial>>()
                    .get(&material)
                    .unwrap()
                    .base
                    .base_color_texture,
                Some(retained)
            );
            assert_eq!(app.world().resource::<VoxelViewStatus>().terrain_builds, 5);
            assert!(app.world().resource::<VoxelViewStatus>().active);
        }
    }
}
