use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_outdoor_terrain_modpack, read_verified_compiled_game_pack};

#[cfg(test)]
#[path = "../outdoor_terrain_blocks.rs"]
mod blocks;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_outdoor_terrain <verified-base-pack> <output-pack>")?,
    )
    .canonicalize()
    .context("resolve base pack")?;
    let output = PathBuf::from(args.next().context("missing output pack")?);
    ensure!(args.next().is_none(), "unexpected extra argument");
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    std::fs::create_dir_all(parent)?;
    let output = parent
        .canonicalize()?
        .join(output.file_name().context("output requires filename")?);
    ensure!(base != output, "output must differ from verified base pack");
    let pack = read_verified_compiled_game_pack(&base)?;
    build_outdoor_terrain_modpack(&pack)?.write_preserving_storage(&output)?;
    println!("exported {}", output.display());
    Ok(())
}
