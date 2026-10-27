use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_authored_route_terrain};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base =
        PathBuf::from(args.next().context(
            "usage: revise_authored_route PACK BEFORE_LAYOUT AFTER_LAYOUT DOSSIER OUTPUT",
        )?);
    let previous = std::fs::read_to_string(args.next().context("previous layout file")?)?;
    let revised = std::fs::read_to_string(args.next().context("revised layout file")?)?;
    let dossier = std::fs::read_to_string(args.next().context("existing route dossier")?)?;
    let output = PathBuf::from(args.next().context("output pack file")?);
    ensure!(args.next().is_none(), "unexpected additional argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from base pack"
    );
    let pack = read_verified_compiled_game_pack(&base)?;
    let revised = revise_authored_route_terrain(&pack, &previous, &revised, &dossier)?;
    revised.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output)
        .context("strictly verify written terrain revision")?;
    println!("revised terrain in {}", output.display());
    Ok(())
}
