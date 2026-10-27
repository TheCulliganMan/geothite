use anyhow::{Context, Result, ensure};
use crystal_assets::{
    PrairieGrassRevision, build_prairie_grass_modpack, read_verified_compiled_game_pack,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 8,
        "usage: pack_prairie_grass BASE R02_BEFORE R02_AFTER R02_DOSSIER R16_BEFORE R16_AFTER R16_DOSSIER OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[7].canonicalize().unwrap_or_else(|_| args[7].clone()),
        "output must differ from base"
    );
    let source = args[1..7]
        .iter()
        .map(|p| std::fs::read_to_string(p).with_context(|| format!("source {}", p.display())))
        .collect::<Result<Vec<_>>>()?;
    let base = read_verified_compiled_game_pack(&args[0])?;
    build_prairie_grass_modpack(
        &base,
        [
            PrairieGrassRevision {
                previous_layout: &source[0],
                layout: &source[1],
                dossier: &source[2],
            },
            PrairieGrassRevision {
                previous_layout: &source[3],
                layout: &source[4],
                dossier: &source[5],
            },
        ],
    )?
    .write_preserving_storage(&args[7])?;
    read_verified_compiled_game_pack(&args[7])?;
    println!(
        "published native golden prairie fields: {}",
        args[7].display()
    );
    Ok(())
}
