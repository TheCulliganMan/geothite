//! Publish the original campfire's authored ambient flame loop.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{MinnesotaTileAnimationRecipe, MinnesotaTileCatalog};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 4,
        "usage: pack_minnesota_campfire_animation BASE RECIPE OUTPUT PREVIEW_DIR"
    );
    ensure!(
        !args[2].exists() && !args[3].exists(),
        "choose fresh pack and preview output paths"
    );
    let recipe: MinnesotaTileAnimationRecipe = serde_json::from_slice(&fs::read(&args[1])?)?;
    let directory = args[1].parent().context("animation recipe directory")?;
    let catalog_json = fs::read(directory.join(&recipe.base_catalog))?;
    let catalog: MinnesotaTileCatalog = serde_json::from_slice(&catalog_json)?;
    let source = fs::read(directory.join(&recipe.base_source))?;
    let variants = recipe
        .sources
        .iter()
        .map(|frame| fs::read(directory.join(&frame.path)).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    let frames = recipe.compile(&catalog_json, &source, &variants)?;
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let base = if base.data().tilesets.contains_key(&catalog.tileset_id) {
        base
    } else {
        crystal_assets::build_minnesota_tile_art_pack(&base, &catalog, &source)?
    };
    let pack = crystal_assets::with_minnesota_tile_animation(
        &base,
        &catalog.tileset_id,
        128,
        &frames,
        &recipe.frame_order,
        recipe.frame_ticks,
    )?;
    fs::create_dir_all(&args[3])?;
    for (index, frame) in frames.iter().enumerate() {
        for scale in [1, 8] {
            fs::write(
                args[3].join(format!("frame-{index}.{scale}x.png")),
                crystal_assets::minnesota_tile_item_preview(frame, &catalog.items[0], scale)?,
            )?;
        }
        fs::write(
            args[3].join(format!("frame-{index}.indices.json")),
            serde_json::to_vec(&frame.indices)?,
        )?;
    }
    pack.write_preserving_storage(&args[2])?;
    let restored = crystal_assets::read_verified_compiled_game_pack(&args[2])?;
    ensure!(
        restored.identity()? == pack.identity()?,
        "animated pack readback identity drift"
    );
    println!(
        "Published {}: {} original keyframes; ambient flower scheduler; {} ticks per frame.",
        args[2].display(),
        frames.len(),
        recipe.frame_ticks
    );
    Ok(())
}
