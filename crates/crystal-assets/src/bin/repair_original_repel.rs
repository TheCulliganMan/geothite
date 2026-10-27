//! Repair only the original native Repel duration from the supplied pinned source.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, repair_original_repel_duration};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: repair_original_repel BASE ITEM_EFFECTS_ASM OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[2].canonicalize().unwrap_or_else(|_| paths[2].clone()),
        "repair output must differ from its completed base"
    );
    let source = std::fs::read_to_string(&paths[1]).context("read exact source item effects")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let repaired = repair_original_repel_duration(&base, &source)?;
    repaired.write_preserving_storage(&paths[2])?;
    let written =
        read_verified_compiled_game_pack(&paths[2]).context("strict written Repel readback")?;
    ensure!(
        written == repaired,
        "strict written Repel repair differs from intended result"
    );
    println!(
        "Repaired original REPEL200 to source100 in {}",
        paths[2].display()
    );
    Ok(())
}
