//! Pack-contained, headless verification of actual authored-model consumption.
//! No disassembly or extracted game assets are required or written to disk.
use anyhow::{Context, Result, ensure};
use bevy::{
    prelude::{Assets, Handle, Image, UVec2, Vec2},
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_render_api::{VisualTile, VisualTileSource, VisualWorldFrame};
use crystal_runtime::CrystalRuntime;
use crystal_voxel_view::{audit_cell_coverage_on_map, audit_terrain_mesh_with_images};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: audit_authored_assets PACK OUTPUT [MAP ...]")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("output report path is required")?);
    let requested: Vec<_> = args.collect();
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
    let mut reports = Vec::new();
    let mut totals = BTreeMap::<String, usize>::new();
    let mut all_families = BTreeMap::<String, usize>::new();
    let mut errors = Vec::new();
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
        let frame = VisualWorldFrame {
            active: true,
            map_id: Arc::from(map_id.as_str()),
            map_texture: handles[0].clone(),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(width as u32, height as u32),
            viewport_size: Vec2::new(width as f32 * 8.0, height as f32 * 8.0),
            tiles,
            ..Default::default()
        };
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
        ensure!(
            mesh.footing_heights.len() == frame.tiles.len()
                && mesh.footing_heights.iter().all(|value| value.is_finite()),
            "invalid footing for {map_id}"
        );
        let mut families = BTreeMap::<String, usize>::new();
        let mut remaining = BTreeMap::<String, usize>::new();
        let mut sources = BTreeMap::<String, usize>::new();
        for ((tile, modeled), old) in frame.tiles.iter().zip(&mesh.authored_cells).zip(legacy) {
            if let Some(family) = modeled {
                *families.entry((*family).to_owned()).or_default() += 1;
                *all_families.entry((*family).to_owned()).or_default() += 1;
                *totals.entry("modeled".into()).or_default() += 1;
            } else {
                *remaining.entry(old.label().into()).or_default() += 1;
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
        reports.push(json!({ "map": map_id, "tileset": tileset, "source_cells": frame.tiles.len(), "modeled_cells": modeled, "families": families, "unmodeled_source_classification": remaining, "unmodeled_sources": sources, "solid_vertices": mesh.solid.positions.len(), "solid_triangles": mesh.solid.indices.len()/3, "footing_cells": mesh.footing_heights.len() }));
    }
    let report = json!({ "format": "geothite-authored-model-coverage-v1", "map_count": maps.len(), "verified_maps": reports.len(), "totals": totals, "families": all_families, "mesher_errors": errors, "actor_sources": actor_sources, "battle_species": battle_species, "limitations": ["Base map state; dynamic blocks and decorations require separate state fixtures", "Unmodeled source classifications are legacy classifier evidence, not proof of an authored model", "Greyscale source samples validate topology and ownership; actual game palette and lighting need native visual review"], "maps": reports });
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_string_pretty(&report)? + "\n")?;
    println!("wrote {}", output.display());
    ensure!(
        errors.is_empty(),
        "{} maps failed meshing; see report",
        errors.len()
    );
    Ok(())
}
