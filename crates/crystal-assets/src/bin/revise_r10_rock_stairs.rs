//! Publish only the reviewed western R10 stair revision from its frozen source.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_r10_rock_stairs};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base =
        PathBuf::from(args.next().context(
            "usage: revise_r10_rock_stairs BASE BEFORE_LAYOUT AFTER_LAYOUT DOSSIER OUTPUT",
        )?);
    let before = std::fs::read_to_string(args.next().context("frozen previous layout")?)?;
    let after = std::fs::read_to_string(args.next().context("frozen candidate layout")?)?;
    let dossier = std::fs::read_to_string(args.next().context("frozen existing dossier")?)?;
    let output = PathBuf::from(args.next().context("output native pack")?);
    ensure!(args.next().is_none(), "unexpected additional argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from completed base"
    );
    let base = read_verified_compiled_game_pack(base)?;
    let revised = revise_r10_rock_stairs(&base, &before, &after, &dossier)?;
    revised.write_preserving_storage(&output)?;
    let written =
        read_verified_compiled_game_pack(&output).context("strict written stair readback")?;
    ensure!(
        written == revised,
        "written R10 staircase differs from intended result"
    );
    println!(
        "Revised native R10 western stone stair in {}",
        output.display()
    );
    Ok(())
}
