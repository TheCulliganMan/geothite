//! Pack-contained, headless verification of actual authored-model consumption.
//! No disassembly or extracted game assets are required or written to disk.
use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    prelude::{Assets, Handle, Image, UVec2, Vec2, Vec3},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::systems::script_blocks::{CHANGE_BLOCK_COORD_STRIDE, ScriptBlockChange};
use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldFrame};
use crystal_runtime::CrystalRuntime;
use crystal_voxel_view::{audit_cell_coverage_on_map, audit_terrain_mesh_with_images};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[path = "audit_authored_assets/script_variants.rs"]
mod script_variants;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: audit_authored_assets PACK OUTPUT [MAP ...]")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("output report path is required")?);
    let mut requested: Vec<_> = args.collect();
    let inspect_roofs = requested.iter().position(|arg| arg == "--roof-coverage")
        .map(|index| requested.remove(index)).is_some();
    let seam_window = if let Some(i) = requested.iter().position(|arg| arg == "--mineral-seams") {
        ensure!(i + 1 < requested.len(), "--mineral-seams needs x0,y0,x1,y1 in source cells");
        let values = requested.remove(i + 1).split(',').map(str::parse::<f32>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        requested.remove(i);
        ensure!(values.len() == 4 && values.iter().all(|v| v.is_finite())
            && values[0] < values[2] && values[1] < values[3], "invalid mineral seam window");
        Some([values[0], values[1], values[2], values[3]])
    } else { None };
    let script_variants = requested.iter().any(|arg| arg == "--script-block-variants");
    let kanto_interiors = requested.iter().any(|arg| arg == "--kanto-interiors");
    let outdoors = requested.iter().any(|arg| arg == "--outdoors");
    let jump_maps = requested.iter().any(|arg| arg == "--jump-maps");
    requested.retain(|arg| arg != "--script-block-variants" && arg != "--kanto-interiors" && arg != "--outdoors" && arg != "--jump-maps");
    let root = AssetRoot::new(pack.parent().context("pack directory")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let actor_sources: Vec<_> = loaded
        .pack()
        .runtime_files()
        .keys()
        .filter_map(|path| {
            path.strip_prefix("gfx/sprites/")
                .and_then(|name| name.strip_suffix(".png"))
                .map(str::to_owned)
                .or_else(|| {
                    path.strip_prefix("gfx/icons/")
                        .and_then(|name| name.strip_suffix(".png"))
                        .map(|name| format!("icon_{name}"))
                })
        })
        .map(|source| {
            let model = crystal_voxel_view::authored_actor_source(&source);
            json!({"source":source,"authored_model":model})
        })
        .collect();
    let battle_species: Vec<_> = loaded.pack().data().pokemon.keys().map(|species|
        json!({"species":species,"authored_model":crystal_voxel_view::has_authored_battle_species(species)})).collect();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let mut maps: Vec<_> = runtime.map_ids().into_iter().collect();
    maps.sort();
    if !requested.is_empty() {
        for map in &requested {
            ensure!(maps.contains(map), "unknown map {map}");
        }
        maps.retain(|map| requested.contains(map));
    }
    if kanto_interiors {
        let mut selected = Vec::new();
        for map in maps {
            let region = &runtime.data().pokegear_landmark_for_map(&map)?.region;
            let environment = runtime.data().map_environment(&map)?;
            if region == "KANTO" && !matches!(environment, "TOWN" | "ROUTE") {
                selected.push(map);
            }
        }
        maps = selected;
    }
    if outdoors {
        let mut selected = Vec::new();
        for map in maps {
            if matches!(runtime.data().map_environment(&map)?, "TOWN" | "ROUTE") {
                selected.push(map);
            }
        }
        maps = selected;
    }
    if jump_maps {
        let mut selected = Vec::new();
        for map in maps {
            let module = &runtime.data().maps[&map];
            let collision = runtime.data().tileset_collision(&module.attributes.tileset_name)?;
            if !crystal_bevy::bevy_shell::authored_visual_jump_ledges(
                module.attributes.width, &module.blocks, &collision, bevy::prelude::IVec2::ZERO,
            )?.is_empty() {
                selected.push(map);
            }
        }
        maps = selected;
    }
    let mut reports = Vec::new();
    let mut totals = BTreeMap::<String, usize>::new();
    let mut all_families = BTreeMap::<String, usize>::new();
    let mut errors = Vec::new();
    let mut variant_reports = Vec::new();
    for map_id in &maps {
        let started = std::time::Instant::now();
        let module = &runtime.data().maps[map_id];
        let tileset = &module.attributes.tileset_name;
        let width = usize::from(module.attributes.width) * 4;
        let height = usize::from(module.attributes.height) * 4;
        let layout = runtime
            .runtime_file(&format!("data/tilesets/{tileset}_metatiles.bin"))
            .context("packed metatile layout")?;
        let png = runtime
            .runtime_file(&format!("gfx/tilesets/{tileset}.png"))
            .context("packed tileset PNG")?;
        let sheet = image::load_from_memory(png)?.to_rgba8();
        ensure!(
            sheet.width() % 8 == 0 && sheet.height() % 8 == 0,
            "{tileset} has non-tile dimensions"
        );
        let mut images = Assets::<Image>::default();
        let mut handles = Vec::<Handle<Image>>::new();
        for y in (0..sheet.height()).step_by(8) {
            for x in (0..sheet.width()).step_by(8) {
                let crop = image::imageops::crop_imm(&sheet, x, y, 8, 8).to_image();
                handles.push(images.add(Image::new(
                    Extent3d {
                        width: 8,
                        height: 8,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    crop.into_raw(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::MAIN_WORLD,
                )));
            }
        }
        let mut tiles = Vec::with_capacity(width * height);
        for row in 0..height {
            for column in 0..width {
                let metatile = module.blocks[(row / 4) * (width / 4) + column / 4];
                let tile_index =
                    u16::from(layout[usize::from(metatile) * 16 + (row % 4) * 4 + column % 4]);
                let palette = runtime.data().tilesets[tileset]
                    .palette_map
                    .get(usize::from(tile_index))
                    .copied()
                    .unwrap_or(0);
                let sample_index = crystal_bevy::bevy_shell::resolve_tileset_tile_index(
                    handles.len(),
                    usize::from(tile_index),
                    (palette >> 3) & 1,
                );
                let texture = handles
                    .get(sample_index)
                    .with_context(|| format!("{tileset} missing art {tile_index}"))?
                    .clone();
                tiles.push(VisualTile {
                    column: column as u32,
                    row: row as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from(tileset.as_str()),
                        metatile_id: metatile,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index,
                    },
                    texture,
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let mut frame = VisualWorldFrame {
            active: true,
            map_id: Arc::from(map_id.as_str()),
            map_texture: handles[0].clone(),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(width as u32, height as u32),
            source_map_size_core_tiles: Some(UVec2::new(width as u32 / 2, height as u32 / 2)),
            boundary_warps: module.events.warps.iter().map(|warp|
                bevy::prelude::IVec2::new(i32::from(warp.x), i32::from(warp.y))).collect(),
            viewport_size: Vec2::new(width as f32 * 8.0, height as f32 * 8.0),
            tiles,
            ..Default::default()
        };
        frame.jump_ledges = crystal_bevy::bevy_shell::authored_visual_jump_ledges(
            module.attributes.width,&module.blocks,&runtime.data().tileset_collision(tileset)?,
            bevy::prelude::IVec2::ZERO)?;
        let domain_collision=runtime.data().tileset_collision(tileset)?;
        let domain_sources=frame.tiles.iter().map(|tile| {
            let s=&tile.source;
            let permission=domain_collision.metatiles[s.metatile_id as usize].collision[
                usize::from(s.subtile_row/2)*2+usize::from(s.subtile_column/2)];
            let land=crystal_core::world::collision::describe_collision(permission).terrain
                ==crystal_core::world::collision::Terrain::Land
                || [crystal_core::world::map::Direction::Up,crystal_core::world::map::Direction::Down,
                    crystal_core::world::map::Direction::Left,crystal_core::world::map::Direction::Right]
                    .into_iter().any(|d|crystal_core::world::collision::allows_ledge_direction(permission,d));
            (bevy::prelude::IVec2::new(tile.column as i32,tile.row as i32),s.clone(),land)
        }).collect::<Vec<_>>();
        frame.terrace_datum=crystal_voxel_view::source_terrace_datum(map_id,&frame.jump_ledges,&domain_sources);
        let legacy = audit_cell_coverage_on_map(map_id, &frame.tiles, width, height)
            .map_err(|error| anyhow::anyhow!("{map_id}: {error:?}"))?;
        let mesh = match audit_terrain_mesh_with_images(&frame, &images) {
            Ok(mesh) => mesh,
            Err(error) => {
                errors.push(json!({"map": map_id, "error": format!("{error:?}")}));
                eprintln!("{map_id}: mesher error {error:?}");
                continue;
            }
        };
        ensure!(
            mesh.authored_cells.len() == frame.tiles.len(),
            "missing audit data for {map_id}"
        );
        if inspect_roofs {
            eprintln!("{map_id}: roof coverage {}", roof_coverage(&mesh.solid));
            eprintln!("{map_id}: terrace edges {:?}", frame.jump_ledges);
        }
        ensure!(
            mesh.footing_heights.len() == frame.tiles.len()
                && mesh.footing_heights.iter().all(|value| value.is_finite()),
            "invalid footing for {map_id}"
        );
        let mut jump_edge_samples = 0;
        let mut rejected_jump_origins = 0;
        // Verify actual production takeoff/landing positions. An inked lip
        // cell can fold onto the face and is not itself the player's takeoff.
        // TryJump reads HOP permissions at the current runtime position.
        use crystal_core::world::{
            map::{Direction, TilePosition},
            movement::{attempt_ledge_jump, LedgeJumpOutcome, PlayerMovementState, StepOptions},
        };
        let authored_map = runtime.data().overworld_map(map_id)?;
        let collision = runtime.data().tileset_collision(tileset)?;
        for y in 0..height / 2 {
            for x in 0..width / 2 {
                let origin = TilePosition::new(x as i16, y as i16);
                // The low-level jump helper accepts a wall quadrant when its
                // complement carries HOP bits. A real player still needs an
                // occupiable origin, as required by production session startup.
                if ![Direction::Down, Direction::Up, Direction::Left, Direction::Right]
                    .into_iter().any(|direction| crystal_core::world::collision::can_enter_tile(
                        &authored_map, &collision, origin, direction,
                        crystal_core::world::collision::PlayerTraversalState::Walk)) {
                    rejected_jump_origins += 1;
                    continue;
                }
                for direction in [Direction::Down, Direction::Up, Direction::Left, Direction::Right] {
                    let mut player = PlayerMovementState::new(TilePosition::new(x as i16, y as i16));
                    let LedgeJumpOutcome::Jumped { from, to, .. } = attempt_ledge_jump(
                        &mut player, direction, &authored_map, &collision, StepOptions::default(),
                    ) else { continue; };
                    let index = |p: TilePosition| {
                        let x = i32::from(p.x) * 2 + 1;
                        let y = i32::from(p.y) * 2 + 1;
                        (x >= 0 && y >= 0 && x < width as i32 && y < height as i32)
                            .then(|| y as usize * width + x as usize)
                    };
                    let (Some(a), Some(b)) = (index(from), index(to)) else { continue; };
                    jump_edge_samples += 1;
                    let support = |p: TilePosition| {
                        let foot = Vec2::new(
                            -(width as f32 * 4.) + (f32::from(p.x) * 2. + 1.) * 8.,
                            // Production actor_ground_point samples the tile
                            // center, one source cell north of the sprite foot.
                            height as f32 * 4. - (f32::from(p.y) * 2. + 1.) * 8.,
                        );
                        crystal_voxel_view::resolved_footing_height(&frame, foot, &mesh.footing_heights)
                            .expect("in-bounds production footing")
                    };
                    let approach = support(from);
                    let landing = support(to);
                    let changes_level = crystal_voxel_view::jump_changes_floor_level(map_id);
                    if if changes_level { approach <= landing + 0.0001 }
                        else { (approach - landing).abs() > 0.0001 } {
                        errors.push(json!({"map":map_id,"error":if changes_level {
                            "production jump does not descend" } else { "production barrier hop changes floor level" },
                            "direction":format!("{direction:?}"),
                            "approach_tile":[from.x,from.y],"landing_tile":[to.x,to.y],
                            "approach_height":approach,"landing_height":landing,
                            "approach_source":format!("{:?}",frame.tiles[a].source),
                            "landing_source":format!("{:?}",frame.tiles[b].source),
                            "jump_edges":frame.jump_ledges.iter().map(|edge|json!({
                                "start":[edge.start.x,edge.start.y],"end":[edge.end.x,edge.end.y],
                                "descent":[edge.descent.x,edge.descent.y]})).collect::<Vec<_>>()}));
                    }
                }
            }
        }
        let mut families = BTreeMap::<String, usize>::new();
        let mut family_bounds = BTreeMap::<String, [u32; 4]>::new();
        let mut remaining = BTreeMap::<String, usize>::new();
        let mut facade_cells = Vec::new();
        let mut blank_windows = Vec::new();
        let mut sources = BTreeMap::<String, usize>::new();
        for ((tile, modeled), old) in frame.tiles.iter().zip(&mesh.authored_cells).zip(legacy) {
            if let Some(family) = modeled {
                let bounds=family_bounds.entry((*family).to_owned())
                    .or_insert([tile.column,tile.row,tile.column,tile.row]);
                bounds[0]=bounds[0].min(tile.column);
                bounds[1]=bounds[1].min(tile.row);
                bounds[2]=bounds[2].max(tile.column);
                bounds[3]=bounds[3].max(tile.row);
                if matches!(*family,"world-exterior/kanto_blank"|"outdoor/kanto_flat_house")
                    && matches!(tile.source.tile_index,0x0a|0x32) {
                    blank_windows.push(json!({"column":tile.column,"row":tile.row,
                        "tile":tile.source.tile_index}));
                }
                *families.entry((*family).to_owned()).or_default() += 1;
                *all_families.entry((*family).to_owned()).or_default() += 1;
                *totals.entry("modeled".into()).or_default() += 1;
            } else {
                *remaining.entry(old.label().into()).or_default() += 1;
                if old.label()=="facade" {
                    facade_cells.push(json!({"column":tile.column,"row":tile.row,
                        "metatile":tile.source.metatile_id,"tile":tile.source.tile_index}));
                }
                *totals
                    .entry(format!("unmodeled_{}", old.label()))
                    .or_default() += 1;
                let key = format!(
                    "{}:{:02x}:{},{}:{:02x}",
                    tile.source.tileset_id,
                    tile.source.metatile_id,
                    tile.source.subtile_column,
                    tile.source.subtile_row,
                    tile.source.tile_index
                );
                *sources.entry(key).or_default() += 1;
            }
        }
        let modeled: usize = families.values().sum();
        eprintln!(
            "{map_id}: {modeled}/{} modeled cells, {:.1}ms",
            frame.tiles.len(),
            started.elapsed().as_secs_f64() * 1000.0
        );
        let environment = runtime.data().map_environment(map_id)?;
        let city_window_sources = crystal_voxel_view::audit_city_window_sources(map_id, &mesh.solid);
        let landmark = runtime.data().pokegear_landmark_for_map(map_id).ok();
        let mut map_report = json!({ "map": map_id, "environment": environment,
            "region": landmark.map(|landmark| landmark.region.as_str()),
            "city_window_sources": city_window_sources,
            "landmark": landmark.map(|landmark| landmark.constant.as_str()), "tileset": tileset, "source_cells": frame.tiles.len(), "modeled_cells": modeled, "families": families, "unmodeled_source_classification": remaining, "unmodeled_sources": sources, "solid_vertices": mesh.solid.positions.len(), "solid_triangles": mesh.solid.indices.len()/3, "textured_vertices": mesh.textured.positions.len(), "textured_triangles": mesh.textured.indices.len()/3, "footing_cells": mesh.footing_heights.len(),
            "footing_height_min":mesh.footing_heights.iter().copied().reduce(f32::min),
            "footing_height_max":mesh.footing_heights.iter().copied().reduce(f32::max),
            "jump_edge_samples":jump_edge_samples });
        map_report["rejected_unoccupiable_jump_origins"] = json!(rejected_jump_origins);
        map_report["open_water_level"]=json!(frame.terrace_datum.as_ref()
            .and_then(|d|d.open_water.as_ref()).map(|r|r.level));
        map_report["coastal_terrace_field"] = json!(frame.terrace_datum.is_some());
        map_report["unmodeled_facade_cells"] = json!(facade_cells);
        map_report["authored_family_source_bounds"] = json!(family_bounds);
        map_report["flat_house_native_window_cells"] = json!(blank_windows);
        if let Some(window) = seam_window {
            map_report["mineral_seams"] = mineral_seams(&mesh.solid, window);
        }
        reports.push(map_report);
        if script_variants && !module.script_block_changes.is_empty() {
            match audit_script_variants(
                map_id,
                &frame,
                &module.script_block_changes,
                layout,
                &runtime.data().tilesets[tileset].palette_map,
                &handles,
                &images,
            ) {
                Ok(variants) => variant_reports.extend(variants),
                Err(error) => errors.push(
                    json!({"map":map_id,"state":"script-block-variants","error":error.to_string()}),
                ),
            }
        }
    }
    let report = json!({ "format": "geothite-authored-model-coverage-v1", "map_count": maps.len(), "verified_maps": reports.len(), "totals": totals, "families": all_families, "mesher_errors": errors, "actor_sources": actor_sources, "battle_species": battle_species, "limitations": ["Base maps and optional declared block substitutions; coverage is exhaustive only where each variant reports all-declared-alternative-combinations, otherwise declared-script-prefixes. Script reachability, cross-script combinations, actors and decorations need separate runtime review", "Unmodeled source classifications are legacy classifier evidence, not proof of an authored model", "Greyscale source samples validate topology and ownership; actual game palette and lighting need native visual review"], "maps": reports, "script_block_variants":variant_reports });
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_string_pretty(&report)? + "\n")?;
    println!("wrote {}", output.display());
    ensure!(
        errors.is_empty(),
        "{} terrain/jump audit errors; see report",
        errors.len()
    );
    Ok(())
}

/// Inspect the final production geometry, after placement and terrain finishes.
/// A model-local coverage check cannot catch a deck lifted through its tiles by
/// a later scene transform. Only live-painted pitched surfaces participate.
fn roof_coverage(mesh: &crystal_voxel_view::SurfaceMeshData) -> serde_json::Value {
    let mut inverted = 0;
    let faces = mesh.indices.chunks_exact(3).filter_map(|triangle| {
        let ids = [triangle[0], triangle[1], triangle[2]].map(|index| index as usize);
        if !ids.iter().all(|&i| (mesh.uvs[i][0] - 85000.).abs() < 0.5) {
            return None;
        }
        let points = ids.map(|i| Vec3::from_array(mesh.positions[i]));
        let face = (points[1] - points[0]).cross(points[2] - points[0]);
        let normal = face.normalize_or_zero();
        let authored_normal = Vec3::from_array(mesh.normals[ids[0]]);
        if (0.35..0.98).contains(&authored_normal.y) && normal.dot(authored_normal) < 0. {
            inverted += 1;
        }
        if !(0.35..0.98).contains(&normal.y) { return None; }
        // Source samples are linear colors; even a light tile can have a
        // low numeric brightness. Broad structural planes distinguish the
        // roof deck from finely tessellated tiles without a pigment guess.
        Some((points, face.length() * 0.5))
    }).collect::<Vec<_>>();
    let mut blocked = Vec::new();
    let mut sampled = 0;
    for (points, area) in &faces {
        if *area >= 100. { continue; }
        let center = points.iter().copied().sum::<Vec3>() / 3.;
        let q = Vec2::new(center.x, center.z);
        sampled += 1;
        let mut covering_dark = None;
        for (deck, area) in &faces {
            if *area < 200. { continue; }
            // Entry shelters and lower tiers legitimately sit beneath a
            // main roof. Check only samples above this deck's eave datum.
            if center.y < deck.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - 0.01 {
                continue;
            }
            let a = Vec2::new(deck[0].x, deck[0].z);
            let ab = Vec2::new(deck[1].x, deck[1].z) - a;
            let ac = Vec2::new(deck[2].x, deck[2].z) - a;
            let determinant = ab.perp_dot(ac);
            if determinant.abs() < 0.000001 { continue; }
            let u = (q - a).perp_dot(ac) / determinant;
            let v = ab.perp_dot(q - a) / determinant;
            if u < 0. || v < 0. || u + v > 1. { continue; }
            let height = deck[0].y + u * (deck[1].y - deck[0].y) + v * (deck[2].y - deck[0].y);
            if height > center.y + 0.01 {
                covering_dark = Some(height);
            }
        }
        if let Some(height) = covering_dark {
            blocked.push(json!({"tile_center": center.to_array(), "deck_height": height}));
        }
    }
    json!({"pitched_faces":faces.len(),"structural_faces":faces.iter().filter(|(_,area)|*area>=200.).count(),
        "inverted_pitched_faces":inverted,"tile_samples":sampled,
        "buried_tile_samples":blocked.len(),"first_buried_tiles":blocked.into_iter().take(12).collect::<Vec<_>>()})
}

/// Unpaired triangle edges locate inspection candidates, not proven holes:
/// ground contacts and differently subdivided joined faces can also appear.
fn mineral_seams(mesh: &crystal_voxel_view::SurfaceMeshData, window: [f32; 4]) -> serde_json::Value {
    type Point = [i64; 3];
    let key = |i: u32| mesh.positions[i as usize].map(|v| (v * 10000.).round() as i64);
    let mut edges = BTreeMap::<(Point, Point), (usize, [u32; 2])>::new();
    for tri in mesh.indices.chunks_exact(3) {
        if !tri.iter().all(|&i| mesh.uvs[i as usize][0] < -9000.) { continue; }
        for [a, b] in [[tri[0], tri[1]], [tri[1], tri[2]], [tri[2], tri[0]]] {
            let (p, q) = (key(a), key(b));
            if p == q { continue; }
            let edge = if p < q { (p, q) } else { (q, p) };
            let entry = edges.entry(edge).or_insert((0, [a, b]));
            entry.0 += 1;
        }
    }
    let source = |i: u32| {
        let uv = mesh.uvs[i as usize];
        [(-uv[0] - 10000.) / 8.,
            (-uv[1] - if uv[1] < -30000. { 40000. } else { 20000. }) / 8.]
    };
    let mut samples = Vec::new();
    let mut count = 0;
    for (_, (uses, ids)) in edges {
        if uses != 1 { continue; }
        let uv = ids.map(source);
        let midpoint = [(uv[0][0] + uv[1][0]) * 0.5, (uv[0][1] + uv[1][1]) * 0.5];
        if midpoint[0] < window[0] || midpoint[0] > window[2]
            || midpoint[1] < window[1] || midpoint[1] > window[3] { continue; }
        count += 1;
        if samples.len() < 2048 {
            samples.push(json!({"positions": ids.map(|i| mesh.positions[i as usize]),
                "source_points": uv}));
        }
    }
    json!({"source_cell_window": window, "unpaired_edges": count,
        "samples": samples, "position_quantization": 0.0001,
        "limitation": "Unpaired edges include ground contacts and T-junctions; inspect geometry and native pixels before calling them holes"})
}

/// Exercise every combination of declared substitutions without pretending to
/// execute a script or advance the game. Runtime input still owns progression.
fn audit_script_variants(
    map: &str,
    base: &VisualWorldFrame,
    changes: &[ScriptBlockChange],
    layout: &[u8],
    palettes: &[u8],
    handles: &[Handle<Image>],
    images: &Assets<Image>,
) -> Result<Vec<serde_json::Value>> {
    let width = base.grid_size.x as usize;
    let height = base.grid_size.y as usize;
    for change in changes {
        ensure!(
            change.x % CHANGE_BLOCK_COORD_STRIDE == 0 && change.y % CHANGE_BLOCK_COORD_STRIDE == 0,
            "unaligned block change on {map}"
        );
        let xy = (
            usize::from(change.x / CHANGE_BLOCK_COORD_STRIDE) * 4,
            usize::from(change.y / CHANGE_BLOCK_COORD_STRIDE) * 4,
        );
        ensure!(
            xy.0 + 4 <= width && xy.1 + 4 <= height,
            "block change outside {map}"
        );
    }
    let plan = script_variants::plan(changes);
    let mut reports = Vec::new();
    for (ordinal, edits) in plan.states.iter().enumerate() {
        let mask = ordinal + 1;
        let mut frame = base.clone();
        let mut changed = Vec::new();
        for (&(x, y), &block) in edits {
            let drawing = layout
                .get(usize::from(block) * 16..usize::from(block) * 16 + 16)
                .with_context(|| format!("missing block {block:02x} on {map}"))?;
            for dy in 0..4 {
                for dx in 0..4 {
                    let index = (y + dy) * width + x + dx;
                    let tile = &mut frame.tiles[index];
                    let art = u16::from(drawing[dy * 4 + dx]);
                    let palette = palettes
                        .get(usize::from(art))
                        .with_context(|| format!("missing palette for {map} tile {art}"))?;
                    let sample = crystal_bevy::bevy_shell::resolve_tileset_tile_index(
                        handles.len(),
                        usize::from(art),
                        (palette >> 3) & 1,
                    );
                    tile.source.metatile_id = block;
                    tile.source.subtile_column = dx as u8;
                    tile.source.subtile_row = dy as u8;
                    tile.source.tile_index = art;
                    tile.texture = handles
                        .get(sample)
                        .context("missing variant tile art")?
                        .clone();
                    changed.push(index);
                }
            }
        }
        let mesh = audit_terrain_mesh_with_images(&frame, images)
            .map_err(|error| anyhow::anyhow!("{map} block mask {mask}: {error:?}"))?;
        ensure!(
            mesh.authored_cells.len() == frame.tiles.len()
                && mesh.footing_heights.len() == frame.tiles.len()
                && mesh.footing_heights.iter().all(|v| v.is_finite()),
            "invalid variant audit on {map}"
        );
        let legacy = audit_cell_coverage_on_map(map, &frame.tiles, width, height)
            .map_err(|error| anyhow::anyhow!("{map} block mask {mask}: {error:?}"))?;
        let mut families = BTreeMap::<String, usize>::new();
        let mut remaining = BTreeMap::<String, usize>::new();
        for &index in &changed {
            if let Some(family) = mesh.authored_cells[index] {
                *families.entry(family.to_owned()).or_default() += 1;
            } else {
                *remaining
                    .entry(legacy[index].label().to_owned())
                    .or_default() += 1;
            }
        }
        reports.push(
            json!({"map":map,"variant_index":mask,
            "block_mask":plan.binary_choices.then_some(mask),
            "coverage":plan.coverage,"possible_combinations":plan.possible_combinations,
            "audited_states":plan.states.len(),"block_edits":edits.iter().map(|(&(x,y), &block)|
                json!({"source_cell":[x,y],"block":block,
                    "original_block":base.tiles[y*width+x].source.metatile_id})).collect::<Vec<_>>(),
            "edited_cells":changed.len(),
            "edited_families":families,"edited_unmodeled_classification":remaining,
            "footing_cells":mesh.footing_heights.len()}),
        );
    }
    eprintln!(
        "{map}: {} block-substitution states verified ({}, {} possible combinations)",
        reports.len(), plan.coverage, plan.possible_combinations
    );
    Ok(reports)
}
