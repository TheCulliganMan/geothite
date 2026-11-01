//! Read-only source-map publication for the panorama below Tin Tower.
use super::*;
use crystal_voxel_view::{SummitBackdrop, SummitBackdropMap};

pub(super) fn publish(
    frame: Res<crystal_render_api::VisualWorldFrame>,
    runtime: Res<BevyRuntimeShell>,
    mut art_cache: ResMut<RenderedTilesetArt>,
    mut images: ResMut<Assets<Image>>,
    mut backdrop: ResMut<SummitBackdrop>,
    settings: Option<Res<crystal_voxel_view::VoxelViewSettings>>,
    status: Option<Res<crystal_voxel_view::VoxelViewStatus>>,
) {
    if !frame.active { return; }
    if frame.map_id.as_ref() != "TinTowerRoof"
        && !crystal_voxel_view::summit_preload_interior(&frame.map_id) {
        if backdrop.revision.is_some() {
            *backdrop = SummitBackdrop::default();
        }
        return;
    }
    if !settings.as_ref().is_some_and(|settings| settings.enabled) { return; }
    if frame.map_id.as_ref() != "TinTowerRoof"
        && !status.as_ref().is_some_and(|s|crystal_voxel_view::summit_preload_foreground_ready(s)) { return; }
    let Some((_, snapshot)) = runtime.cached_snapshot.as_ref() else {
        return;
    };
    match source_maps_budgeted(snapshot, &runtime, &mut art_cache, &mut images, &backdrop, 1) {
        Ok(Some(next)) => *backdrop = next,
        Ok(None) => {}
        Err(error) => bevy::log::warn!("summit source panorama unavailable: {error:#}"),
    }
}
pub(super) fn source_maps(
    snapshot: &RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
    cache: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    previous: &SummitBackdrop,
) -> Result<Option<SummitBackdrop>> {
    source_maps_budgeted(snapshot, runtime, cache, images, previous, usize::MAX)
}

pub(super) fn source_maps_budgeted(
    snapshot: &RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
    cache: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
    previous: &SummitBackdrop,
    budget: usize,
) -> Result<Option<SummitBackdrop>> {
    let placements = connected_render_map_placements_to_depth("EcruteakCity", &snapshot.maps, None)?;
    let city = placements[0].0;
    let tower = city
        .events
        .warps
        .iter()
        .find(|w| w.target_map_constant == "TIN_TOWER_1F")
        .context("Ecruteak's authored Tin Tower entrance")?;
    let period = match snapshot.progression.time.time_of_day {
        crystal_core::world::encounters::TimeOfDay::Morning => "morn",
        crystal_core::world::encounters::TimeOfDay::Day => "day",
        crystal_core::world::encounters::TimeOfDay::Night => "nite",
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    period.hash(&mut hash);
    tower.x.hash(&mut hash);
    tower.y.hash(&mut hash);
    for (map, x, y) in &placements {
        map.map_name.hash(&mut hash);
        map.blocks.hash(&mut hash);
        map.attributes.width.hash(&mut hash);
        map.attributes.height.hash(&mut hash);
        map.attributes.border_block.hash(&mut hash);
        visible_effective_map_time_of_day(map, period, false).hash(&mut hash);
        x.hash(&mut hash);
        y.hash(&mut hash);
        map.attributes.tileset_name.hash(&mut hash);
        if let Some(tileset) = snapshot
            .tilesets
            .iter()
            .find(|t| t.tileset_id == map.attributes.tileset_name)
        {
            tileset.palette_map.hash(&mut hash);
        }
    }
    let revision = hash.finish();
    if previous.revision == Some(revision) && !previous.source_pending {
        return Ok(None);
    }
    let continuing = previous.revision == Some(revision);
    let animation_frame = if continuing { previous.animation_frame } else { runtime.lcd_animation_frame };
    if !continuing && std::env::var_os("CRYSTAL_SUMMIT_TRACE").is_some() {
        eprintln!("summit source maps ({}): {:?}", placements.len(),
            placements.iter().map(|(map, _, _)| map.map_name.as_str()).collect::<Vec<_>>());
    }
    let mut sources = Vec::new();
    for (map, x, y) in placements {
        let tileset = snapshot
            .tilesets
            .iter()
            .find(|t| t.tileset_id == map.attributes.tileset_name)
            .with_context(|| format!("missing {} tileset", map.map_name))?;
        let key = TilesetArtKey {
            tileset_id: tileset.tileset_id.clone(),
            time_of_day: visible_effective_map_time_of_day(map, period, false).into(),
            palette_map: tileset.palette_map.clone(),
        };
        sources.push(ResolvedRenderSource {
            map,
            tileset,
            art_key: key,
            origin_x: x,
            origin_y: y,
            width: i32::from(map.attributes.width) * 4,
            height: i32::from(map.attributes.height) * 4,
        });
    }
    let mut maps = if continuing { previous.maps.clone() } else { Vec::new() };
    let first = maps.len();
    for source in sources.iter().skip(first).take(budget.max(1)) {
        let key = source.art_key.clone();
        if !cache.cache.contains_key(&key) {
            let art = load_tileset_art(
                &runtime.asset_root,
                &key.tileset_id,
                &key.time_of_day,
                &key.palette_map,
                images,
            )?;
            cache.cache.insert(key.clone(), art);
        }
        let art = &cache.cache[&source.art_key];
        let w = source.width as usize;
        let h = source.height as usize;
        let mut tiles = Vec::with_capacity(w * h);
        let mut handles = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let block = source.map.blocks[y / 4 * (w / 4) + x / 4];
                let index = *art
                    .metatile_layout
                    .get(block as usize * 16 + y % 4 * 4 + x % 4)
                    .context("panorama metatile")?;
                let texture = art
                    .tile_handle_at_frame(block, x % 4, y % 4, animation_frame, false)
                    .context("panorama tile art")?;
                handles.push(texture.clone());
                tiles.push(crystal_render_api::VisualTile {
                    column: x as u32,
                    row: y as u32,
                    source: crystal_render_api::VisualTileSource {
                        tileset_id: source.tileset.tileset_id.as_str().into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: index.into(),
                    },
                    texture,
                    animation_frames: art
                        .animated_tiles
                        .get(&(index as usize))
                        .map(|a| a.frames.clone()),
                    priority: false,
                });
            }
        }
        // Floor evidence has the same two-hop neighborhood as native play.
        // The panorama's larger reach must not rerun a whole-region terrace
        // solve for every individual route or change its local floor datum.
        let neighborhood = connected_render_map_placements(&source.map.map_name, &snapshot.maps)?;
        let relative = neighborhood.iter().map(|(map, x, y)| {
            let resolved = sources.iter().find(|s| s.map.map_name == map.map_name)
                .expect("complete connected panorama contains every local neighbor");
            ResolvedRenderSource {
                map: resolved.map,
                tileset: resolved.tileset,
                art_key: resolved.art_key.clone(),
                origin_x: *x,
                origin_y: *y,
                width: resolved.width,
                height: resolved.height,
            }
        }).collect::<Vec<_>>();
        let ledges = terrain_ledges::collect(&relative, runtime.shell.runtime().data())?;
        let datum = terrain_ledges::datum(&relative, runtime.shell.runtime(), &ledges)?;
        let texture = compose_visual_world_tiles(&handles, w, h, None, images);
        let visual = crystal_render_api::VisualWorldFrame {
            active: true,
            map_id: source.map.map_name.as_str().into(),
            grid_size: UVec2::new(w as u32, h as u32),
            tile_size: Vec2::splat(TILE_SIZE),
            viewport_size: Vec2::new(w as f32 * TILE_SIZE, h as f32 * TILE_SIZE),
            map_texture: texture,
            source_map_size_core_tiles: Some(UVec2::new(w as u32 / 2, h as u32 / 2)),
            terrain_revision: revision,
            jump_ledges: ledges,
            terrace_datum: datum,
            tiles,
            ..Default::default()
        };
        maps.push(SummitBackdropMap {
            frame: Arc::new(visual),
            origin_source: IVec2::new(source.origin_x, source.origin_y),
        });
    }
    if maps.len() < sources.len() {
        return Ok(Some(SummitBackdrop {
            revision: Some(revision), source_pending: true, animation_frame,
            tower_core: IVec2::new(tower.x.into(), tower.y.into()), maps, border: None,
        }));
    }
    // Match the ordinary exterior renderer's source-authored border, without
    // repeating finite map roads, water, or buildings over their neighbors.
    let source = &sources[0];
    let art = &cache.cache[&source.art_key];
    let block = u16::from(source.map.attributes.border_block);
    let mut tiles = Vec::new();
    let mut handles = Vec::new();
    let ground = maps[0]
        .frame
        .tiles
        .iter()
        .find(|t| {
            t.source.tile_index == 5
                && matches!(
                    crystal_voxel_view::shape_for_source(&t.source),
                    crystal_voxel_view::CellShape::Flat
                )
        })
        .context("real city lawn paint for the border trees")?;
    for y in 0..4usize {
        for x in 0..5usize {
            if x == 4 {
                let mut sample = ground.clone();
                sample.column = 4;
                sample.row = y as u32;
                handles.push(sample.texture.clone());
                tiles.push(sample);
                continue;
            }
            let index = *art
                .metatile_layout
                .get(block as usize * 16 + y * 4 + x)
                .context("summit authored border metatile")?;
            let texture = art
                .tile_handle_at_frame(block, x, y, animation_frame, false)
                .context("summit authored border art")?;
            handles.push(texture.clone());
            tiles.push(crystal_render_api::VisualTile {
                column: x as u32,
                row: y as u32,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: source.tileset.tileset_id.as_str().into(),
                    metatile_id: block,
                    subtile_column: x as u8,
                    subtile_row: y as u8,
                    tile_index: index.into(),
                },
                texture,
                animation_frames: art
                    .animated_tiles
                    .get(&(index as usize))
                    .map(|a| a.frames.clone()),
                priority: false,
            });
        }
    }
    let border = Arc::new(crystal_render_api::VisualWorldFrame {
        active: true,
        map_id: source.map.map_name.as_str().into(),
        grid_size: UVec2::new(5, 4),
        tile_size: Vec2::splat(TILE_SIZE),
        viewport_size: Vec2::new(5. * TILE_SIZE, 4. * TILE_SIZE),
        map_texture: compose_visual_world_tiles(&handles, 5, 4, None, images),
        terrain_revision: revision,
        tiles,
        ..Default::default()
    });
    Ok(Some(SummitBackdrop {
        revision: Some(revision),
        source_pending: false,
        animation_frame,
        tower_core: IVec2::new(tower.x.into(), tower.y.into()),
        maps,
        border: Some(border),
    }))
}
