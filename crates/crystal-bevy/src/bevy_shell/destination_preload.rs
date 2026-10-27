//! Prepare nearby doors and outdoor connections without advancing game state.
use super::*;

#[path = "world_stream_publish.rs"]
mod streaming;

#[derive(Resource, Default)]
pub(super) struct PublishedDestination {
    key: Option<u64>,
    actor_map: String,
    anchor: Option<(String, TilePosition)>,
    snapshot: Option<Arc<RuntimeShellSnapshot>>,
    last_error: Option<String>,
    stream_key: Option<u64>,
    stream_target: Option<crystal_voxel_view::WorldStreamTarget>,
}

fn is_outdoors(map: &crate::RuntimeMapCatalogSnapshot) -> bool {
    map.attributes.environment.as_deref()
        .or_else(|| map.metadata.as_ref().map(|metadata| metadata.environment.as_str()))
        .is_some_and(|environment| environment.eq_ignore_ascii_case("town")
            || environment.eq_ignore_ascii_case("route"))
}

fn exterior_destination(snapshot: &RuntimeShellSnapshot) -> Option<(&crate::RuntimeMapCatalogSnapshot, TilePosition)> {
    let current = snapshot.maps.iter().find(|m| m.map_name == snapshot.overworld.map_name)?;
    if is_outdoors(current) { return None; }
    let mut queue = VecDeque::from([(current.as_ref(), 0)]);
    let mut visited = HashSet::from([current.map_name.as_str()]);
    while let Some((map, depth)) = queue.pop_front() {
        if depth >= 3 { continue; }
        for warp in &map.events.warps {
            let Some(target) = snapshot.maps.iter().find(|m| m.map_name == warp.target_map
                || m.metadata.as_ref().is_some_and(|metadata| metadata.constant == warp.target_map_constant)) else { continue; };
            if warp.target_warp_id <= 0 { continue; }
            let Some(arrival) = target.events.warps.get(warp.target_warp_id as usize - 1)
                .and_then(warp_tile_position_checked) else { continue; };
            if is_outdoors(target) { return Some((target, arrival)); }
            if visited.insert(target.map_name.as_str()) { queue.push_back((target, depth + 1)); }
        }
    }
    None
}

// Keep one nearby destination warm. Indoors, retain the exterior lookahead
// across stairs; outdoors, prepare the nearest doorway or connected boundary.
fn destination(snapshot: &RuntimeShellSnapshot) -> Option<(&crate::RuntimeMapCatalogSnapshot, TilePosition)> {
    let current = snapshot.maps.iter().find(|m| m.map_name == snapshot.overworld.map_name)?;
    if !is_outdoors(current) { return exterior_destination(snapshot); }
    let doorway = current.events.warps.iter().filter_map(|warp| {
        if warp.target_warp_id <= 0 { return None; }
        let door = warp_tile_position_checked(warp)?;
        let distance = (i32::from(door.x) - i32::from(snapshot.overworld.tile.x)).abs()
            + (i32::from(door.y) - i32::from(snapshot.overworld.tile.y)).abs();
        if distance > 8 { return None; }
        let target = snapshot.maps.iter().find(|m| m.map_name == warp.target_map
            || m.metadata.as_ref().is_some_and(|metadata| metadata.constant == warp.target_map_constant))?;
        if target.map_name == current.map_name { return None; }
        let arrival = target.events.warps.get(warp.target_warp_id as usize - 1)
            .and_then(warp_tile_position_checked)?;
        Some((distance, target, arrival))
    }).min_by_key(|(distance, _, _)| *distance);
    let connection = current.attributes.connections.iter().filter_map(|connection| {
        let target = snapshot.maps.iter().find(|map| map.map_name == connection.target_map)?;
        if !is_outdoors(target) { return None; }
        let width = i32::from(current.attributes.width) * 2;
        let height = i32::from(current.attributes.height) * 2;
        let tw = i32::from(target.attributes.width) * 2;
        let th = i32::from(target.attributes.height) * 2;
        let offset = connection.offset * 2;
        let x = i32::from(snapshot.overworld.tile.x);
        let y = i32::from(snapshot.overworld.tile.y);
        let (distance, ax, ay) = match connection.direction.to_ascii_lowercase().as_str() {
            "north" if (0..tw).contains(&(x - offset)) => (y + 1, x - offset, th - 1),
            "south" if (0..tw).contains(&(x - offset)) => (height - y, x - offset, 0),
            "west" if (0..th).contains(&(y - offset)) => (x + 1, tw - 1, y - offset),
            "east" if (0..th).contains(&(y - offset)) => (width - x, 0, y - offset),
            _ => return None,
        };
        // Start before the boundary enters view; only one hidden scene is resident.
        if distance > 40 { return None; }
        Some((distance, target, TilePosition::new(ax as i16, ay as i16)))
    }).min_by_key(|(distance, _, _)| *distance);
    doorway.into_iter().chain(connection).min_by_key(|(distance, _, _)| *distance)
        .map(|(_, map, arrival)| (map.as_ref(), arrival))
}

pub(super) fn publish(
    current: Res<crystal_render_api::VisualWorldFrame>,
    stream: Res<crystal_voxel_view::WorldStreamStatus>,
    runtime: Res<BevyRuntimeShell>,
    mut previous: ResMut<PublishedDestination>,
    mut cache: ResMut<RenderedTilesetArt>,
    mut images: ResMut<Assets<Image>>,
    mut request: ResMut<crystal_voxel_view::DestinationPreload>,
) {
    // Native workers can prepare the known destination while the initial room
    // settles. This reads the source catalog without advancing controller input.
    if !current.active || !crystal_voxel_view::destination_preload_supported() {
        return;
    }
    // Source publication must continue before the first scene and while a
    // late resident window lacks player support. Workers need the latest
    // coverage request to recover; visible 3D is not a prerequisite for it.
    let Some((_, snapshot)) = runtime.cached_snapshot.as_ref() else {
        return;
    };
    // Animation/camera updates retain this immutable snapshot. Do not traverse
    // the map graph and hash every connected block again on display refreshes.
    let snapshot_changed = previous
        .snapshot
        .as_ref()
        .is_none_or(|old| !Arc::ptr_eq(old, snapshot));
    let stream_changed = previous.stream_target != stream.target;
    if !snapshot_changed && !stream_changed {
        return;
    }
    previous.snapshot = Some(Arc::clone(snapshot));
    previous.stream_target.clone_from(&stream.target);
    streaming::publish(
        snapshot,
        &runtime,
        stream.target.as_ref(),
        &mut previous,
        &mut request,
        &mut cache,
        &mut images,
    );
    let Some((map, mut arrival)) = destination(snapshot) else {
        return;
    };
    // Preserve in-flight work across small movements parallel to a boundary.
    if let Some((name, old)) = &previous.anchor {
        if name == &map.map_name
            && (i32::from(old.x) - i32::from(arrival.x)).abs() <= 4
            && (i32::from(old.y) - i32::from(arrival.y)).abs() <= 4
        {
            arrival = *old;
        }
    }
    let period = match snapshot.progression.time.time_of_day {
        crystal_core::world::encounters::TimeOfDay::Morning => "morn",
        crystal_core::world::encounters::TimeOfDay::Day => "day",
        crystal_core::world::encounters::TimeOfDay::Night => "nite",
    };
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    map.map_name.hash(&mut hash);
    arrival.x.hash(&mut hash);
    arrival.y.hash(&mut hash);
    period.hash(&mut hash);
    snapshot.trainer.player_gender.hash(&mut hash);
    snapshot.script_events.variable_sprites.hash(&mut hash);
    // Include all contributing map writes, including a connected neighbor.
    for (source, x, y) in
        connected_render_map_placements_to_depth(&map.map_name, &snapshot.maps, None)
            .unwrap_or_default()
    {
        source.blocks.hash(&mut hash);
        x.hash(&mut hash);
        y.hash(&mut hash);
    }
    let key = hash.finish();
    if previous.key == Some(key) {
        if previous.actor_map != snapshot.overworld.map_name {
            request.actor_sources = lookahead_actor_sources(snapshot, &runtime, map);
            previous.actor_map = snapshot.overworld.map_name.clone();
        }
        return;
    }
    match source_frame(
        snapshot,
        &runtime,
        map,
        arrival,
        period,
        &mut cache,
        &mut images,
    ) {
        Ok(frame) => {
            previous.last_error = None;
            request.actor_sources = lookahead_actor_sources(snapshot, &runtime, map);
            previous.actor_map = snapshot.overworld.map_name.clone();
            request.frame = Some(frame);
            previous.anchor = Some((map.map_name.clone(), arrival));
            previous.key = Some(key);
        }
        Err(error) => {
            // Speculation must not block a real warp. The ordinary renderer
            // still validates and builds its actual destination on arrival.
            previous.last_error = Some(format!("{error:#}"));
            bevy::log::warn!("outdoor preload source unavailable: {error:#}");
            previous.key = Some(key);
        }
    }
}

// Keep characters on the immediate indoor waypoint warm as well as the
// exterior. This does not allocate another terrain scene or advance a script.
fn lookahead_actor_sources(snapshot:&RuntimeShellSnapshot,runtime:&BevyRuntimeShell,
    target:&crate::RuntimeMapCatalogSnapshot)->Vec<Arc<str>> {
    let mut sources=actor_sources(snapshot,runtime,target);
    if let Some(current)=snapshot.maps.iter().find(|m|m.map_name==snapshot.overworld.map_name)
        .filter(|current|!is_outdoors(current) && is_outdoors(target)) {
        sources.extend(actor_sources(snapshot,runtime,current));
        let mut visited=HashSet::from([current.map_name.as_str()]);
        for warp in current.events.warps.iter().filter(|warp|warp.target_warp_id>0).take(4) {
            let Some(room)=snapshot.maps.iter().find(|m|m.map_name==warp.target_map
                || m.metadata.as_ref().is_some_and(|metadata|metadata.constant==warp.target_map_constant))
                .filter(|room|!is_outdoors(room)) else {continue;};
            if visited.insert(room.map_name.as_str()) {
                sources.extend(actor_sources(snapshot,runtime,room));
            }
        }
    }
    sources.sort_unstable();sources.dedup();sources
}

fn actor_sources(snapshot: &RuntimeShellSnapshot, runtime: &BevyRuntimeShell,
    map: &crate::RuntimeMapCatalogSnapshot) -> Vec<Arc<str>> {
    let mut sources = vec![Arc::from(if snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE {
        "kris"
    } else { "chris" })];
    for object in &map.objects {
        let bitmap = resolve_visible_object_sprite_asset_id(&runtime.asset_root, &object.sprite,
            &snapshot.script_events.variable_sprites, &snapshot.presentation.menu_icons);
        sources.push(Arc::from(visible_map_object_model_source_from_data(runtime.shell.runtime().data(), &map.map_name, object,
            &object.sprite, &bitmap, &snapshot.script_events.variable_sprites,
            &snapshot.presentation.menu_icons)));
    }
    sources.sort_unstable(); sources.dedup();
    sources
}

pub(super) fn source_frame(
    snapshot: &RuntimeShellSnapshot,
    runtime: &BevyRuntimeShell,
    map: &crate::RuntimeMapCatalogSnapshot,
    arrival: TilePosition,
    period: &str,
    cache: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
) -> Result<crystal_render_api::VisualWorldFrame> {
    let (width, height) = render_tile_bounds_i16(&map.map_name, map.attributes.width, map.attributes.height)?;
    let (x, y) = connection_composite_viewport_origin(snapshot, map, arrival.x * 2, arrival.y * 2, width, height)
        .context("destination viewport")?;
    let window = visual_world_map_window(true, IVec2::new(i32::from(x), i32::from(y)), map);
    let mut sources = Vec::new();
    for (map, x, y) in
        connected_world_sources::placements(&map.map_name, &snapshot.maps, window)?
    {
        let tileset = snapshot.tilesets.iter().find(|t| t.tileset_id == map.attributes.tileset_name)
            .context("destination tileset")?;
        let key = TilesetArtKey { tileset_id: tileset.tileset_id.clone(),
            time_of_day: visible_effective_map_time_of_day(map, period, false).into(),
            palette_map: tileset.palette_map.clone() };
        if !cache.cache.contains_key(&key) {
            let art = load_tileset_art(&runtime.asset_root, &key.tileset_id, &key.time_of_day,
                &key.palette_map, images)?;
            cache.cache.insert(key.clone(), art);
        }
        sources.push(ResolvedRenderSource { map, tileset, art_key: key, origin_x: x, origin_y: y,
            width: i32::from(map.attributes.width) * 4, height: i32::from(map.attributes.height) * 4 });
    }
    let (w,h) = (window.size.x as i16,window.size.y as i16);
    let mut tiles = Vec::with_capacity(w as usize * h as usize);
    let mut handles = Vec::with_capacity(tiles.capacity());
    let restless = snapshot.progression.active_engine_flags.contains("ENGINE_FOREST_IS_RESTLESS");
    for row in 0..h { for column in 0..w {
        let (source, sx, sy) = resolved_visual_render_source_at(&sources,
            window.origin.x + i32::from(column), window.origin.y + i32::from(row));
        let art = &cache.cache[&source.art_key];
        let (block, sub_x, sub_y, index, texture, priority, _) = source_tile_art(
            source, art, sx, sy, runtime.lcd_animation_frame, restless)?;
        handles.push(texture.clone());
        tiles.push(crystal_render_api::VisualTile { column: column as u32, row: row as u32,
            source: crystal_render_api::VisualTileSource {
                tileset_id: source.tileset.tileset_id.as_str().into(), metatile_id: block,
                subtile_column: sub_x as u8, subtile_row: sub_y as u8, tile_index: index.into() },
            texture, priority, animation_frames: art.animated_tiles.get(&(index as usize))
                .filter(|a| !a.frames.is_empty()).map(|a| a.frames.clone()) });
    }}
    let ledges = terrain_ledges::collect(&sources, runtime.shell.runtime().data())?;
    let datum = terrain_ledges::datum(&sources, runtime.shell.runtime(), &ledges)?;
    let mut identity = std::collections::hash_map::DefaultHasher::new();
    period.hash(&mut identity);
    for tile in &tiles { tile.texture.id().hash(&mut identity); }
    let revision = visual_terrain_revision(identity.finish(), (x, y), &tiles);
    let mut frame = crystal_render_api::VisualWorldFrame { active: true, map_id: map.map_name.as_str().into(),
        source_regions: visual_source_regions(&sources),
        source_map_size_core_tiles: Some(UVec2::new(width as u32 / 2, height as u32 / 2)),
        boundary_warps: map.events.warps.iter().filter_map(warp_tile_position_checked)
            .map(|tile| IVec2::new(i32::from(tile.x), i32::from(tile.y))).collect(),
        jump_ledges: ledges, terrace_datum: datum, terrain_revision: revision,
        grid_origin: window.origin,
        map_texture: compose_visual_world_tiles(&handles, w as usize, h as usize, None, images),
        center: Vec2::ZERO, view_center: Vec2::ZERO, viewport_size: Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT),
        tile_size: Vec2::splat(TILE_SIZE), grid_size: UVec2::new(w as u32, h as u32), tiles,
        actors: Vec::new() };
    frame.anchor_to_map();
    Ok(frame)
}


#[cfg(test)]
mod tests {
    use super::*;
    include!("world_stream_support_tests.rs");
    #[test]
    fn routes_preload_connected_arrivals_in_source_coordinates() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root.repository_root.join("content-packs/text-tui.crystalpack"));
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let initial = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                map_name: "Route29".into(), tile_x: 50, tile_y: 9,
            }, BevyShellConfig::default()).unwrap();
        let mut catalog = initial.shell.snapshot().unwrap();
        for name in ["Route26", "Route29"] {
            let (w, h) = runtime.data().saved_map_tile_bounds(name).unwrap();
            catalog.overworld.map_name = name.into();
            let connections = catalog.maps.iter().find(|map| map.map_name == name).unwrap()
                .attributes.connections.clone();
            let mut fixture = None;
            for connection in &connections {
                let (button, direction, mut candidates): (_, _, Vec<_>) = match connection.direction.to_ascii_lowercase().as_str() {
                    "north" => (GameButton::Up, Direction::Up, (0..w).map(|x| TilePosition::new(x as i16, 0)).collect()),
                    "south" => (GameButton::Down, Direction::Down, (0..w).map(|x| TilePosition::new(x as i16, h as i16 - 1)).collect()),
                    "west" => (GameButton::Left, Direction::Left, (0..h).map(|y| TilePosition::new(0, y as i16)).collect()),
                    "east" => (GameButton::Right, Direction::Right, (0..h).map(|y| TilePosition::new(w as i16 - 1, y as i16)).collect()),
                    _ => continue,
                };
                candidates.sort_by_key(|tile| (i32::from(tile.x) - i32::from(w / 2)).abs()
                    + (i32::from(tile.y) - i32::from(h / 2)).abs());
                // Saved connection sources can occupy blocked border cells;
                // select a real walking path using the core collision rules.
                let tile = candidates.into_iter().find(|&tile| {
                    let Ok(source) = runtime.data().overworld_session(name, tile, 0) else { return false; };
                    if !crate::core::world::collision::can_enter_tile(&source.map, &source.tileset,
                        tile, direction, crate::core::world::collision::PlayerTraversalState::Walk) { return false; }
                    catalog.overworld.tile = tile;
                    let Some((target, arrival)) = destination(&catalog) else { return false; };
                    if target.map_name != connection.target_map || !is_outdoors(target) { return false; }
                    let Ok(target) = runtime.data().overworld_session(&target.map_name, arrival, 0) else { return false; };
                    crate::core::world::collision::can_enter_tile(&target.map, &target.tileset,
                        arrival, direction, crate::core::world::collision::PlayerTraversalState::Walk)
                });
                if let Some(tile) = tile { fixture = Some((tile, button)); break; }
            }
            let (tile, button) = fixture.unwrap_or_else(|| panic!("{name} needs a walkable connection fixture: {connections:?}"));
            let shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                    map_name: name.into(), tile_x: tile.x, tile_y: tile.y,
                }, BevyShellConfig { smoke_player_name: Some("CHRIS".into()), ..default() }).unwrap();
            let mut controller = VisibleShellController { shell };
            let before = controller.snapshot().unwrap();
            let (selected, arrival) = destination(&before).expect("nearby route connection");
            assert!(is_outdoors(selected));
            let predicted = (selected.map_name.clone(), arrival);
            println!("connection fixture: {name} {tile:?} {button:?} -> {predicted:?}");
            assert_eq!(controller.snapshot().unwrap().state_checksum, before.state_checksum,
                "lookahead must not advance gameplay");
            // Compare against production controller movement, rather than
            // reimplementing the connection-offset formula in the test.
            let mut crossed = None;
            for _ in 0..16 {
                controller.press(button).unwrap();
                let after = controller.snapshot().unwrap();
                if after.overworld.map_name != name { crossed = Some(after); break; }
            }
            let after = crossed.unwrap_or_else(|| {
                let after = controller.snapshot().unwrap();
                panic!("{name} connection did not execute: {:?} -> {} {:?}; phase {:?}; error {:?}",
                    tile, after.overworld.map_name, after.overworld.tile, after.phase, controller.shell.last_error)
            });
            assert_eq!((after.overworld.map_name, after.overworld.tile), predicted,
                "{name} must preload the authoritative arrival");
        }
    }

    #[test]
    fn connected_route_views_retain_the_same_pack_cells_across_controller_crossings() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root.repository_root.join("content-packs/text-tui.crystalpack"));
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        for (name, x, y, button) in [("Route29", 58, 9, GameButton::Right), ("Route26", 1, 94, GameButton::Left)] {
            let shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                    map_name: name.into(), tile_x: x, tile_y: y,
                }, BevyShellConfig { render_test_hour: Some(12), smoke_player_name: Some("CHRIS".into()), ..default() }).unwrap();
            let mut controller = VisibleShellController { shell };
            let before = controller.snapshot().unwrap();
            let mut art = RenderedTilesetArt::default();
            let mut images = Assets::<Image>::default();
            let map = before.maps.iter().find(|m| m.map_name == name).unwrap();
            let built = source_frame(&before, &controller.shell, map, before.overworld.tile, "day", &mut art, &mut images).unwrap();
            let after = (0..8).find_map(|_| {
                controller.press(button).unwrap();
                let next = controller.snapshot().unwrap();
                (next.overworld.map_name != name).then_some(next)
            }).expect("production connection must execute");
            let map = after.maps.iter().find(|m| m.map_name == after.overworld.map_name).unwrap();
            let live = source_frame(&after, &controller.shell, map, after.overworld.tile, "day", &mut art, &mut images).unwrap();
            let shift = crystal_voxel_view::connected_frame_shift(&built, &live);
            assert!(shift.is_ok(), "{name} -> {}: {shift:?}; grids {:?}/{:?}; origins {:?}/{:?}; placements {:?}/{:?}",
                live.map_id, built.grid_size, live.grid_size, built.grid_origin, live.grid_origin,
                built.source_regions, live.source_regions);
            let mut changed = live.clone();
            let player_cell = IVec2::new(
                i32::from(after.overworld.tile.x) * 2,
                i32::from(after.overworld.tile.y) * 2,
            );
            let index = changed.tiles.iter().position(|tile| {
                let p = changed.grid_origin + IVec2::new(tile.column as i32, tile.row as i32);
                p == player_cell
                    && changed.source_regions.iter().any(|r| {
                        let q = p-r.origin;
                        q.cmpge(IVec2::ZERO).all() && q.cmplt(r.size.as_ivec2()).all()
                    })
            }).expect("crossing retains actual authored acreage near the player");
            changed.tiles[index].priority = !changed.tiles[index].priority;
            assert!(crystal_voxel_view::connected_frame_shift(&built, &changed).is_err(),
                "a changed authored cell must reject retention");
            let mut changed = live.clone();
            let tile = &changed.tiles[index];
            let point = changed.grid_origin + IVec2::new(tile.column as i32, tile.row as i32);
            let datum = Arc::make_mut(changed.terrace_datum.as_mut().expect("outdoor elevation"));
            let local = point-datum.origin;
            Arc::make_mut(&mut datum.levels)[(local.y as u32*datum.size.x+local.x as u32) as usize][0] += 1.;
            assert!(crystal_voxel_view::connected_frame_shift(&built, &changed).is_err(),
                "a changed authored elevation must reject retention");
        }
    }

    #[test]
    fn both_house_floors_preload_the_real_new_bark_exit_without_advancing_gameplay() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root.repository_root.join("content-packs/core-modular.crystalpack"));
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        for room in ["PlayersHouse2F", "PlayersHouse1F"] {
            let mut shell = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                    map_name: room.into(), tile_x: if room == "PlayersHouse1F" { 6 } else { 3 },
                    tile_y: if room == "PlayersHouse1F" { 6 } else { 3 },
                }, BevyShellConfig { render_test_hour: Some(12), smoke_player_name: Some("CHRIS".into()), ..default() }).unwrap();
            let snapshot = shell.shell.snapshot().unwrap();
            let before = snapshot.state_checksum.clone();
            let (map, arrival) = exterior_destination(&snapshot).unwrap_or_else(|| panic!("house exterior: {:?}", snapshot.maps.iter()
                .filter(|m| matches!(m.map_name.as_str(), "PlayersHouse2F" | "PlayersHouse1F" | "NewBarkTown"))
                .map(|m| (&m.map_name, &m.attributes.environment, &m.events.warps)).collect::<Vec<_>>()));
            assert_eq!(map.map_name, "NewBarkTown");
            let sources=lookahead_actor_sources(&snapshot,&shell,map);
            assert!(sources.iter().any(|source|source.as_ref()=="mom"),
                "the route through the ground floor must prepare Mom while preserving exterior terrain");
            let mut cache = RenderedTilesetArt::default();
            let mut images = Assets::<Image>::default();
            let frame = source_frame(&snapshot, &shell, map, arrival, "day", &mut cache, &mut images).unwrap();
            frame.validate().unwrap();
            assert_eq!(frame.map_id.as_ref(), "NewBarkTown");
            assert!(frame.tiles.iter().any(|tile| tile.source.tileset_id.as_ref() == map.attributes.tileset_name));
            assert!(!frame.jump_ledges.is_empty());
            assert!(frame.terrace_datum.is_some());
            assert_eq!(shell.shell.snapshot().unwrap().state_checksum, before);
            #[cfg(not(target_arch = "wasm32"))]
            let shell = {
                cached_runtime_snapshot(&mut shell).unwrap();
                let mut app=App::new();
                app.insert_resource(shell)
                    .insert_resource(crystal_render_api::VisualWorldFrame {
                        active:true,map_id:room.into(),..default()})
                    .insert_resource(crystal_voxel_view::VoxelViewStatus {
                        active:false,terrain_pending:true,..default()})
                    .init_resource::<PublishedDestination>()
                    .init_resource::<crystal_voxel_view::WorldStreamStatus>()
                    .init_resource::<RenderedTilesetArt>()
                    .init_resource::<Assets<Image>>()
                    .init_resource::<crystal_voxel_view::DestinationPreload>()
                    .add_systems(Update,publish);
                app.update();
                let request=app.world().resource::<crystal_voxel_view::DestinationPreload>();
                let early=request.frame.as_ref().unwrap_or_else(|| panic!(
                    "publish exterior before the room is playable: {:?}",
                    app.world().resource::<PublishedDestination>().last_error));
                early.validate().unwrap();
                assert_eq!(early.map_id.as_ref(),"NewBarkTown");
                assert!(request.actor_sources.iter().any(|id|id.as_ref()=="mom"));
                assert!(!app.world().resource::<crystal_voxel_view::VoxelViewStatus>().active);
                let mut shell=app.world_mut().remove_resource::<BevyRuntimeShell>().unwrap();
                assert_eq!(shell.shell.snapshot().unwrap().state_checksum,before,
                    "early publication cannot advance gameplay");
                shell
            };
            if room == "PlayersHouse1F" {
                let mut exterior = initialize_bevy_runtime_shell(root.clone(), runtime.clone(),
                    BevyShellStart::NewGameAtRuntimeTile {
                        spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                        map_name: map.map_name.clone(), tile_x: arrival.x, tile_y: arrival.y,
                    }, BevyShellConfig { render_test_hour: Some(12), smoke_player_name: Some("CHRIS".into()), ..default() }).unwrap();
                exterior.lcd_animation_frame = shell.lcd_animation_frame;
                let mut app = App::new();
                app.insert_resource(exterior).insert_resource(cache).insert_resource(images)
                    .init_resource::<RenderedViewport>()
                    .insert_resource(crystal_voxel_view::VoxelViewSettings { enabled: true, ..default() })
                    .add_systems(Update, render_playfield);
                app.update();
                let rendered = app.world().resource::<RenderedViewport>();
                assert_eq!(app.world().resource::<BevyRuntimeShell>().last_error, None);
                assert_eq!(rendered.visual_tiles, frame.tiles,
                    "speculation must use exactly the production source grid, art and priorities");
                assert_eq!(rendered.visual_jump_ledges, frame.jump_ledges);
                assert_eq!(rendered.visual_terrace_datum, frame.terrace_datum);
                let images = app.world().resource::<Assets<Image>>();
                assert_eq!(images.get(rendered.visual_world_texture.as_ref().unwrap()).unwrap().data,
                    images.get(&frame.map_texture).unwrap().data,
                    "preloaded and normal destination pixels must be identical");
            }

        }
    }
    #[test]
    fn outdoor_approach_preloads_the_nearest_real_building() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap());
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root.repository_root.join("content-packs/core-modular.crystalpack"));
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root,
            crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap()).unwrap();
        let shell = initialize_bevy_runtime_shell(root, runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                map_name: "NewBarkTown".into(), tile_x: 6, tile_y: 5,
            }, BevyShellConfig { render_test_hour: Some(12), smoke_player_name: Some("CHRIS".into()), ..default() }).unwrap();
        let mut snapshot = shell.shell.snapshot().unwrap();
        for (tile, expected) in [(TilePosition::new(6, 5), "ElmsLab"),
            (TilePosition::new(13, 6), "PlayersHouse1F"),
            (TilePosition::new(3, 12), "PlayersNeighborsHouse")] {
            snapshot.overworld.tile = tile;
            let (map, arrival) = destination(&snapshot).expect("nearby authored door");
            assert_eq!(map.map_name, expected);
            if expected == "ElmsLab" {
                let sources = actor_sources(&snapshot, &shell, map);
                for expected_source in ["chris", "elm", "scientist"] {
                    assert!(sources.iter().any(|source| source.as_ref() == expected_source),
                        "destination must prepare {expected_source}: {sources:?}");
                }
            }
            assert!(map.events.warps.iter().filter_map(warp_tile_position_checked).any(|p| p == arrival));
        }
        snapshot.overworld.tile = TilePosition::new(0, 0);
        let current = snapshot.maps.iter().find(|map| map.map_name == "NewBarkTown").unwrap();
        let (selected, _) = destination(&snapshot).expect("nearby authored route connection");
        assert!(current.attributes.connections.iter().any(|connection|
            connection.target_map == selected.map_name),
            "a nearby route may be prepared even when every door is distant");
        // Isolate the distant-door contract from the independent connection
        // lookahead. Clearing this disposable catalog cannot change gameplay.
        let current = snapshot.maps.iter_mut().find(|map| map.map_name == "NewBarkTown").unwrap();
        Arc::make_mut(current).attributes.connections.clear();
        assert!(destination(&snapshot).is_none(), "distant doors must not displace the spare scene");
    }

}
