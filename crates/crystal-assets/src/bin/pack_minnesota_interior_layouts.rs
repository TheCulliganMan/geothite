//! Publish explicit native interior furniture and floor revisions.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_minnesota_interior_layout_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: pack_minnesota_interior_layouts BASE LAYOUTS_JSON OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[2].canonicalize().unwrap_or_else(|_| paths[2].clone()),
        "layout output must differ from its base"
    );
    let source = std::fs::read_to_string(&paths[1]).context("read authored room layouts")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let pack = build_minnesota_interior_layout_modpack(&base, &source)?;
    pack.write_preserving_storage(&paths[2])?;
    read_verified_compiled_game_pack(&paths[2]).context("verify written room layouts")?;
    println!("published native interior layouts: {}", paths[2].display());
    Ok(())
}
