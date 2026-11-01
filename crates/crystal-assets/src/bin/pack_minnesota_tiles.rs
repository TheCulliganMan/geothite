//! Import original ImageGen props and publish an additive native tileset.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    MinnesotaTileCatalog, build_minnesota_tile_art_pack, minnesota_tile_item_preview,
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 5,
        "usage: pack_minnesota_tiles BASE CATALOG_JSON SOURCE_PNG OUTPUT_PACK PREVIEW_DIR"
    );
    ensure!(
        !args[3].exists(),
        "output pack already exists; choose a new version"
    );
    ensure!(
        !args[4].exists(),
        "preview directory already exists; choose a new version"
    );
    let catalog: MinnesotaTileCatalog = serde_json::from_slice(&fs::read(&args[1])?)?;
    let source = fs::read(&args[2])?;
    let pixels = catalog.convert(&source)?;
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let pack = build_minnesota_tile_art_pack(&base, &catalog, &source)?;
    fs::create_dir_all(&args[4])?;
    for (item, pixels) in catalog.items.iter().zip(&pixels) {
        for scale in [1, 8] {
            fs::write(
                args[4].join(format!("{}.{}x.png", item.id, scale)),
                minnesota_tile_item_preview(pixels, item, scale)?,
            )?;
        }
        fs::write(
            args[4].join(format!("{}.indices.json", item.id)),
            serde_json::to_vec_pretty(&pixels.indices)?,
        )?;
    }
    pack.write_preserving_storage(&args[3])?;
    let restored = crystal_assets::read_verified_compiled_game_pack(&args[3])
        .context("read back original prop extension")?;
    ensure!(
        restored.identity()? == pack.identity()?,
        "published pack identity drift"
    );
    println!(
        "Published {} with {} prop items in {}; native previews: {}",
        args[3].display(),
        catalog.items.len(),
        catalog.tileset_id,
        args[4].display()
    );
    Ok(())
}
