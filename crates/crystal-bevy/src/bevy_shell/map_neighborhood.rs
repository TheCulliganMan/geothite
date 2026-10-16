//! Spatial presentation topology. Walking still uses the runtime's own edges.
//! Resolve the connected component once in active-map coordinates, rather than
//! stopping at the first map edge. Inconsistent cycles never relocate a map
//! that has already been reached by a shorter path.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Placement {
    pub index: usize,
    pub origin: IVec2,
    pub size: IVec2,
    pub depth: usize,
}

impl Placement {
    pub fn intersects(self, min: IVec2, max: IVec2) -> bool {
        self.origin.x < max.x
            && self.origin.y < max.y
            && self.origin.x + self.size.x > min.x
            && self.origin.y + self.size.y > min.y
    }
}

pub(super) fn connected<M: VisibleMapCatalogEntry>(active: &str, maps: &[M]) -> Vec<Placement> {
    let indices: HashMap<_, _> = maps
        .iter()
        .enumerate()
        .map(|(i, m)| (m.visible_map().map_name.as_str(), i))
        .collect();
    let Some(&index) = indices.get(active) else {
        return Vec::new();
    };
    let size = |i: usize| {
        let a = &maps[i].visible_map().attributes;
        IVec2::new(i32::from(a.width), i32::from(a.height)) * i32::from(RENDER_METATILE_WIDTH)
    };
    let mut result = vec![Placement {
        index,
        origin: IVec2::ZERO,
        size: size(index),
        depth: 0,
    }];
    let mut visited = HashSet::from([index]);
    let mut cursor = 0;
    while cursor < result.len() {
        let parent = result[cursor];
        cursor += 1;
        for connection in &maps[parent.index].visible_map().attributes.connections {
            let Some(&index) = indices.get(connection.target_map.as_str()) else {
                continue;
            };
            if visited.contains(&index) {
                continue;
            }
            let target_size = size(index);
            let Some(offset) = connection
                .offset
                .checked_mul(i32::from(RENDER_METATILE_WIDTH))
            else {
                continue;
            };
            let local = match connection.direction.to_ascii_lowercase().as_str() {
                "north" => IVec2::new(offset, -target_size.y),
                "south" => IVec2::new(offset, parent.size.y),
                "west" => IVec2::new(-target_size.x, offset),
                "east" => IVec2::new(parent.size.x, offset),
                _ => continue,
            };
            let (Some(x), Some(y)) = (
                parent.origin.x.checked_add(local.x),
                parent.origin.y.checked_add(local.y),
            ) else {
                continue;
            };
            visited.insert(index);
            result.push(Placement {
                index,
                origin: IVec2::new(x, y),
                size: target_size,
                depth: parent.depth + 1,
            });
        }
        // Ports are outdoor scenery reached through an underground passage,
        // so they have no cardinal connection in the ROM. Give those two
        // verified harbor pairs a presentation seam at the city's south edge.
        // The passage's reciprocal warps authenticate the pair and align X;
        // its folded underground corridor is never laid onto the landscape.
        for (city_name, port_name, passage_name) in [
            ("VermilionCity", "VermilionPort", "VermilionPortPassage"),
            ("OlivineCity", "OlivinePort", "OlivinePortPassage"),
        ] {
            let (Some(&city), Some(&port), Some(&passage)) = (
                indices.get(city_name),
                indices.get(port_name),
                indices.get(passage_name),
            ) else {
                continue;
            };
            if parent.index != city && parent.index != port {
                continue;
            }
            let city_map = maps[city].visible_map();
            let port_map = maps[port].visible_map();
            let tunnel = maps[passage].visible_map();
            if !city_map
                .attributes
                .environment
                .as_deref()
                .or_else(|| city_map.metadata.as_ref().map(|m| m.environment.as_str()))
                .is_some_and(|e| e.eq_ignore_ascii_case("town"))
                || port_map.attributes.tileset_name != "port"
            {
                continue;
            }
            let targets = |warp: &crystal_core::map::WarpEvent,
                           map: &crate::RuntimeMapCatalogSnapshot| {
                warp.target_map == map.map_name
                    || map.attributes.map_constant.as_deref()
                        == Some(warp.target_map_constant.as_str())
            };
            let Some(city_exit) = city_map.events.warps.iter().find(|w| targets(w, tunnel)) else {
                continue;
            };
            let Some(port_exit) = port_map.events.warps.iter().find(|w| targets(w, tunnel)) else {
                continue;
            };
            let reciprocal = |exit: &crystal_core::map::WarpEvent,
                              map: &crate::RuntimeMapCatalogSnapshot| {
                tunnel.events.warps.iter().any(|w| {
                    i16::try_from(w.index).ok() == Some(exit.target_warp_id)
                        && targets(w, map)
                        && i16::try_from(exit.index).ok() == Some(w.target_warp_id)
                })
            };
            if !reciprocal(city_exit, city_map) || !reciprocal(port_exit, port_map) {
                continue;
            }
            let delta = IVec2::new(
                (i32::from(city_exit.x) - i32::from(port_exit.x))
                    * i32::from(RENDER_TILES_PER_RUNTIME_TILE),
                size(city).y,
            );
            let (index, origin) = if parent.index == city {
                (port, parent.origin + delta)
            } else {
                (city, parent.origin - delta)
            };
            if visited.insert(index) {
                result.push(Placement {
                    index,
                    origin,
                    size: size(index),
                    depth: parent.depth + 1,
                });
            }
        }
    }
    result
}

pub(super) fn block_identity(snapshot: &RuntimeShellSnapshot) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for placement in connected(&snapshot.overworld.map_name, &snapshot.maps) {
        let map = &snapshot.maps[placement.index];
        map.map_name.hash(&mut hasher);
        placement.origin.to_array().hash(&mut hasher);
        map.attributes.tileset_name.hash(&mut hasher);
        map.attributes.border_block.hash(&mut hasher);
        map.blocks.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(feature = "voxel-view")]
#[derive(Clone)]
pub(super) struct RetainedMap {
    content_key: u64,
    pub origin: IVec2,
    pub frame: Arc<crystal_render_api::VisualWorldFrame>,
    pub phases: Arc<crystal_render_api::VisualTileAnimationFrame>,
}

/// Select actual map rectangles in coarse patches, rather than allocating
/// the empty bounding rectangle of a horizon view. Include a full metatile
/// margin so a model touching the crop still has its complete source drawing.
#[cfg(feature = "voxel-view")]
pub(super) fn visible_crop(
    origin: IVec2,
    size: IVec2,
    camera: crystal_voxel_view::VoxelCameraControls,
    aspect: f32,
    viewport_origin: IVec2,
) -> Option<(IVec2, IVec2)> {
    const PATCH: i32 = 32;
    const MARGIN: i32 = 16;
    let viewport = Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT);
    let center = viewport_origin.as_vec2()
        + Vec2::new(VIEWPORT_TILES_X as f32, VIEWPORT_TILES_Y as f32) * 0.5;
    let mut low = size;
    let mut high = IVec2::ZERO;
    for y in (0..size.y).step_by(PATCH as usize) {
        for x in (0..size.x).step_by(PATCH as usize) {
            let min = IVec2::new(x, y);
            let max = (min + IVec2::splat(PATCH)).min(size);
            let world_min =
                (origin.as_vec2() + min.as_vec2() - center - Vec2::splat(8.0)) * TILE_SIZE;
            let world_max =
                (origin.as_vec2() + max.as_vec2() - center + Vec2::splat(8.0)) * TILE_SIZE;
            if camera.terrain_bounds_visible(viewport, aspect, world_min, world_max) {
                low = low.min(min);
                high = high.max(max);
            }
        }
    }
    (high.cmpgt(low).all()).then(|| {
        (
            (low - IVec2::splat(MARGIN)).max(IVec2::ZERO),
            (high + IVec2::splat(MARGIN)).min(size),
        )
    })
}

#[cfg(feature = "voxel-view")]
pub(super) fn retain_maps(
    previous: &[RetainedMap],
    sources: &[ResolvedRenderSource<'_>],
    art: &RenderedTilesetArt,
    images: &mut Assets<Image>,
    data: &crystal_assets::GameDataSet,
    camera: crystal_voxel_view::VoxelCameraControls,
    aspect: f32,
    viewport_origin: IVec2,
    frame_number: u64,
    restless: bool,
) -> anyhow::Result<Vec<RetainedMap>> {
    use crystal_render_api::{
        VisualTile, VisualTileAnimationFrame, VisualTileSource, VisualWorldFrame,
    };
    let mut result = Vec::new();
    for (source_index, source) in sources.iter().enumerate().skip(1) {
        let origin = IVec2::new(source.origin_x, source.origin_y);
        let size = IVec2::new(source.width, source.height);
        let Some((mut low, mut high)) = visible_crop(origin, size, camera, aspect, viewport_origin)
        else {
            continue;
        };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.map.map_name.hash(&mut hasher);
        source.art_key.hash(&mut hasher);
        source.map.blocks.hash(&mut hasher);
        size.to_array().hash(&mut hasher);
        // Authored connection cycles need not embed perfectly in one plane.
        // Preserve the nearest-source ownership used by the classic halo;
        // farther routes cannot put a second floor over a nearby harbor.
        let overlaps: Vec<_> = sources[..source_index]
            .iter()
            .filter(|prior| {
                prior.origin_x < origin.x + size.x
                    && prior.origin_x + prior.width > origin.x
                    && prior.origin_y < origin.y + size.y
                    && prior.origin_y + prior.height > origin.y
            })
            .collect();
        for prior in &overlaps {
            (
                prior.origin_x - origin.x,
                prior.origin_y - origin.y,
                prior.width,
                prior.height,
            )
                .hash(&mut hasher);
        }
        let content_key = hasher.finish();
        let old = previous.iter().find(|old| {
            old.frame.map_id.as_ref() == source.map.map_name && old.content_key == content_key
        });
        if let Some(old) = old {
            let old_low = old.frame.grid_origin;
            let old_high = old_low + old.frame.grid_size.as_ivec2();
            if old_low.cmple(low).all() && old_high.cmpge(high).all() {
                low = old_low;
                high = old_high;
            }
        }
        let source_art = art
            .cache
            .get(&source.art_key)
            .ok_or_else(|| anyhow::anyhow!("missing neighbor art for {}", source.map.map_name))?;
        let grid = (high - low).as_uvec2();
        let retained =
            old.filter(|old| old.frame.grid_origin == low && old.frame.grid_size == grid);
        let frame = if let Some(old) = retained {
            old.frame.clone()
        } else {
            let mut tiles = Vec::with_capacity((grid.x * grid.y) as usize);
            let mut handles = Vec::with_capacity(tiles.capacity());
            for row in 0..grid.y {
                for column in 0..grid.x {
                    let p = low + IVec2::new(column as i32, row as i32);
                    let block = source.map.blocks[(p.y / 4) as usize
                        * source.map.attributes.width as usize
                        + (p.x / 4) as usize];
                    let [sx, sy] = [(p.x % 4) as usize, (p.y % 4) as usize];
                    let tile_index = *source_art
                        .metatile_layout
                        .get(block as usize * METATILE_TILE_COUNT + sy * 4 + sx)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "missing neighbor metatile {block} for {}",
                                source.map.map_name
                            )
                        })?;
                    let texture = source_art
                        .tile_handle_at_frame(block, sx, sy, frame_number, restless)
                        .ok_or_else(|| {
                            anyhow::anyhow!("missing neighbor tile for {}", source.map.map_name)
                        })?;
                    handles.push(texture.clone());
                    let world = origin + p;
                    let foreign = overlaps.iter().any(|prior| {
                        (prior.origin_x..prior.origin_x + prior.width).contains(&world.x)
                            && (prior.origin_y..prior.origin_y + prior.height).contains(&world.y)
                    });
                    tiles.push(VisualTile {
                        column,
                        row,
                        source: VisualTileSource {
                            tileset_id: source.tileset.tileset_id.as_str().into(),
                            metatile_id: if foreign { u16::MAX } else { block },
                            subtile_column: sx as u8,
                            subtile_row: sy as u8,
                            tile_index: if foreign {
                                u16::MAX - 1
                            } else {
                                tile_index as u16
                            },
                        },
                        texture,
                        animation_frames: (!foreign)
                            .then(|| {
                                source_art
                                    .animated_tiles
                                    .get(&(tile_index as usize))
                                    .filter(|a| !a.frames.is_empty())
                                    .map(|a| a.frames.clone())
                            })
                            .flatten(),
                        priority: false,
                    });
                }
            }
            let texture = compose_visual_world_tiles(
                &handles,
                grid.x as usize,
                grid.y as usize,
                None,
                images,
            );
            let collision = data.tileset_collision(&source.tileset.tileset_id)?;
            Arc::new(VisualWorldFrame {
                active: true,
                map_id: source.map.map_name.as_str().into(),
                source_map_size_core_tiles: Some((size / 2).as_uvec2()),
                boundary_warps: source
                    .map
                    .events
                    .warps
                    .iter()
                    .filter_map(warp_tile_position_checked)
                    .map(|p| IVec2::new(i32::from(p.x), i32::from(p.y)))
                    .collect(),
                jump_ledges: terrain_ledges::from_map(
                    source.map.attributes.width,
                    &source.map.blocks,
                    &collision,
                    IVec2::ZERO,
                )?,
                terrain_revision: visual_terrain_revision(
                    content_key,
                    (low.x as i16, low.y as i16),
                    &tiles,
                ),
                grid_origin: low,
                map_texture: texture,
                center: Vec2::ZERO,
                viewport_size: grid.as_vec2() * TILE_SIZE,
                tile_size: Vec2::splat(TILE_SIZE),
                grid_size: grid,
                tiles,
                actors: Vec::new(),
            })
        };
        let tiles: Vec<_> = frame
            .tiles
            .iter()
            .filter(|t| t.animation_frames.is_some())
            .map(|tile| {
                let mut tile = tile.clone();
                tile.texture = source_art
                    .tile_handle_at_frame(
                        tile.source.metatile_id,
                        tile.source.subtile_column as usize,
                        tile.source.subtile_row as usize,
                        frame_number,
                        restless,
                    )
                    .expect("retained authenticated animation family");
                tile
            })
            .collect();
        let phases = retained
            .filter(|old| old.phases.tiles == tiles)
            .map(|old| old.phases.clone())
            .unwrap_or_else(|| {
                Arc::new(VisualTileAnimationFrame {
                    map_id: frame.map_id.clone(),
                    grid_origin: low,
                    tiles,
                })
            });
        result.push(RetainedMap {
            content_key,
            origin,
            frame,
            phases,
        });
    }
    Ok(result)
}

/// Replace only the optional visual grid. The classic atlas, priority layer,
/// transition tiles and collision lookups retain their original direct edges.
pub(super) fn apply_visual_tiles(
    tiles: &mut [crystal_render_api::VisualTile],
    sources: &[ResolvedRenderSource<'_>],
    origin: IVec2,
    fallback_min: IVec2,
    fallback_max: IVec2,
) {
    for tile in tiles {
        let p = origin + IVec2::new(tile.column as i32, tile.row as i32);
        let contains = |source: &&ResolvedRenderSource<'_>| {
            (source.origin_x..source.origin_x + source.width).contains(&p.x)
                && (source.origin_y..source.origin_y + source.height).contains(&p.y)
        };
        if sources.first().is_some_and(|source| contains(&source)) {
            continue;
        }
        let neighbor = sources.iter().skip(1).any(|source| contains(&source));
        if neighbor || p.cmplt(fallback_min).any() || p.cmpge(fallback_max).any() {
            // An independently rendered map owns this rectangle. MAX-1 leaves
            // a hole in the backdrop as well, avoiding a second ground plane
            // over its water. MAX/MAX is unknown space with a flat backdrop.
            tile.source.metatile_id = u16::MAX;
            tile.source.tile_index = if neighbor { u16::MAX - 1 } else { u16::MAX };
            tile.animation_frames = None;
            tile.priority = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires an external pack via CRYSTAL_RENDER_TEST_PACK"]
    fn external_pack_neighborhood_preserves_transitive_offsets_and_cycles() {
        let pack = PathBuf::from(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
                .expect("set CRYSTAL_RENDER_TEST_PACK to a completed external pack"),
        );
        let root = AssetRoot::new(pack.parent().unwrap());
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let controller = VisibleShellController::new_game(root, runtime, "CHRIS", None).unwrap();
        let mut snapshot = controller.shell.shell.snapshot().unwrap();
        snapshot.overworld.map_name = "NewBarkTown".into();
        let maps = &snapshot.maps;
        let placements = connected("NewBarkTown", maps);
        let find = |name: &str| {
            *placements
                .iter()
                .find(|p| maps[p.index].map_name == name)
                .unwrap()
        };
        let new_bark = find("NewBarkTown");
        let route29 = find("Route29");
        let cherry = find("CherrygroveCity");
        let route27 = find("Route27");
        let route26 = find("Route26");
        assert_eq!(route29.origin, IVec2::new(-route29.size.x, 0));
        assert_eq!(cherry.origin, route29.origin - IVec2::new(cherry.size.x, 0));
        assert_eq!(route27.origin, IVec2::new(new_bark.size.x, 0));
        assert_eq!(
            route26.origin,
            route27.origin + IVec2::new(route27.size.x, -45 * 4)
        );
        assert_eq!(cherry.depth, 2);
        assert_eq!(route26.depth, 2);
        assert_eq!(
            placements.len(),
            placements
                .iter()
                .map(|p| p.index)
                .collect::<HashSet<_>>()
                .len()
        );
        assert!(
            !placements
                .iter()
                .any(|p| maps[p.index].map_name == "PlayersHouse1F")
        );
        assert!(!cherry.intersects(IVec2::ZERO, new_bark.size));
        assert!(cherry.intersects(cherry.origin, cherry.origin + cherry.size));
        let mut changed = snapshot.clone();
        let far_map = Arc::make_mut(&mut changed.maps[cherry.index]);
        far_map.blocks[0] ^= 1;
        assert_ne!(
            block_identity(&snapshot),
            block_identity(&changed),
            "a remembered changeblock in a distant visible map must invalidate 3D terrain"
        );
        assert_eq!(
            visible_map_block_identity(&snapshot),
            visible_map_block_identity(&changed),
            "the classic compositor still depends only on immediate connections"
        );
        for (city, port) in [
            ("VermilionCity", "VermilionPort"),
            ("OlivineCity", "OlivinePort"),
        ] {
            let neighborhood = connected(city, maps);
            let city_placement = neighborhood[0];
            let harbor = neighborhood
                .iter()
                .find(|p| maps[p.index].map_name == port)
                .expect("verified outdoor harbor seam");
            assert_eq!(harbor.origin.y, city_placement.size.y);
            assert!(!harbor.intersects(IVec2::ZERO, city_placement.size));
            assert!(
                !neighborhood
                    .iter()
                    .any(|p| maps[p.index].map_name.ends_with("Passage"))
            );
            let reverse = connected(port, maps);
            let city_from_harbor = reverse
                .iter()
                .find(|p| maps[p.index].map_name == city)
                .unwrap();
            assert_eq!(city_from_harbor.origin, -harbor.origin);
        }
    }

    #[cfg(feature = "voxel-view")]
    #[test]
    #[ignore = "requires an external pack via CRYSTAL_RENDER_TEST_PACK"]
    fn external_pack_neighborhood_renders_ship_without_changing_classic_or_gameplay() {
        let pack = PathBuf::from(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
                .expect("set CRYSTAL_RENDER_TEST_PACK to a completed external pack"),
        );
        let root = AssetRoot::new(pack.parent().unwrap());
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
        let spawn_identifier = runtime.title_new_game_spawn_identifier().unwrap();
        let make_app = |enabled| {
            let shell = initialize_bevy_runtime_shell(
                root.clone(),
                runtime.clone(),
                BevyShellStart::NewGameAtRuntimeTile {
                    spawn_identifier: spawn_identifier.clone(),
                    map_name: "VermilionCity".into(),
                    tile_x: 19,
                    tile_y: 31,
                },
                BevyShellConfig {
                    smoke_player_name: Some("CHRIS".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            let mut app = App::new();
            app.insert_resource(shell)
                .init_resource::<RenderedViewport>()
                .init_resource::<RenderedTilesetArt>()
                .init_resource::<Assets<Image>>()
                .init_resource::<crystal_render_api::VisualWorldFrame>()
                .init_resource::<crystal_render_api::VisualWorldNeighborhood>()
                .insert_resource(crystal_voxel_view::VoxelViewSettings {
                    enabled,
                    camera: crystal_voxel_view::VoxelCameraControls::with_pose(
                        0.0, 4.0, 35.0, false,
                    ),
                    ..Default::default()
                })
                .add_systems(
                    Update,
                    (render_playfield, publish_visual_world_frame).chain(),
                );
            app.update();
            app
        };
        let classic = make_app(false);
        let mut native = make_app(true);
        let checksum = |app: &App| {
            app.world()
                .resource::<BevyRuntimeShell>()
                .shell
                .snapshot()
                .unwrap()
                .state_checksum
        };
        assert_eq!(checksum(&classic), checksum(&native));
        let pixels = |app: &App, priority| {
            let rendered = app.world().resource::<RenderedViewport>();
            let handle = if priority {
                rendered.map_priority_texture.as_ref()
            } else {
                rendered.map_texture.as_ref()
            }
            .unwrap();
            app.world()
                .resource::<Assets<Image>>()
                .get(handle)
                .unwrap()
                .data
                .clone()
        };
        assert_eq!(pixels(&classic, false), pixels(&native, false));
        assert_eq!(pixels(&classic, true), pixels(&native, true));
        let rendered = native.world().resource::<RenderedViewport>();
        assert_eq!(rendered.map_name.as_deref(), Some("VermilionCity"));
        assert!(
            rendered
                .visual_neighbors
                .iter()
                .any(|map| map.frame.map_id.as_ref() == "VermilionPort"
                    && map
                        .frame
                        .tiles
                        .iter()
                        .any(|t| t.source.tileset_id.as_ref() == "port"
                            && (0x18..=0x1f).contains(&t.source.metatile_id))),
            "the ship's actual source drawing must be present from the city"
        );
        assert!(rendered.visual_tiles_revision.is_some());
        assert!(
            native
                .world()
                .resource::<crystal_render_api::VisualWorldFrame>()
                .active
        );
        let before = rendered.visual_world_grid_size;
        let port = rendered
            .visual_neighbors
            .iter()
            .find(|map| map.frame.map_id.as_ref() == "VermilionPort")
            .unwrap()
            .frame
            .clone();
        assert!(port.validate().is_ok());
        assert!(!rendered.visual_tiles.iter().any(|t| t.source.tileset_id.as_ref() == "port" && t.source.metatile_id != u16::MAX),
            "the active-map mesh must not render foreign scenery with its own profiles");
        let published = native
            .world()
            .resource::<crystal_render_api::VisualWorldNeighborhood>();
        assert_eq!(published.active_map_id.as_ref(), "VermilionCity");
        assert!(published.maps.iter().any(|m| Arc::ptr_eq(&m.frame, &port)));
        let mut acreage = HashSet::new();
        for map in &rendered.visual_neighbors {
            for tile in &map.frame.tiles {
                if tile.source.metatile_id == u16::MAX {
                    continue;
                }
                let p = map.origin
                    + map.frame.grid_origin
                    + IVec2::new(tile.column as i32, tile.row as i32);
                assert!(
                    acreage.insert(p),
                    "neighbor maps must not render overlapping source floors at {p:?}"
                );
                let active = rendered.source_map_size_core_tiles.unwrap().as_ivec2() * 2;
                assert!(
                    p.cmplt(IVec2::ZERO).any() || p.cmpge(active).any(),
                    "neighbor floor overlaps the active map"
                );
            }
        }
        native
            .world_mut()
            .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
            .camera
            .rotation_step = 4.01;
        native.update();
        let retained = native
            .world()
            .resource::<RenderedViewport>()
            .visual_neighbors
            .iter()
            .find(|map| map.frame.map_id.as_ref() == "VermilionPort")
            .unwrap();
        assert!(
            Arc::ptr_eq(&retained.frame, &port),
            "a small camera orbit must retain the immutable map atlas and source grid"
        );
        native
            .world_mut()
            .resource_mut::<crystal_voxel_view::VoxelViewSettings>()
            .camera = crystal_voxel_view::VoxelCameraControls::with_pose(1.0, 4.0, 0.0, true);
        native.update();
        let after = native
            .world()
            .resource::<RenderedViewport>()
            .visual_world_grid_size;
        assert!(
            native
                .world()
                .resource::<RenderedViewport>()
                .visual_tiles_revision
                .is_some()
        );
        assert!(
            native
                .world()
                .resource::<crystal_render_api::VisualWorldFrame>()
                .active
        );
        assert_eq!(
            after, before,
            "a horizon view selects map patches without growing the active atlas"
        );
        assert!(
            native
                .world()
                .resource::<crystal_render_api::VisualWorldNeighborhood>()
                .maps
                .iter()
                .any(|map| map.frame.map_id.as_ref() == "VermilionPort")
        );
        assert_eq!(checksum(&classic), checksum(&native));
        assert_eq!(pixels(&classic, false), pixels(&native, false));
    }
}
