//! Publish R10's native grass-side cliff and exact authored terrain revision.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    add_r10_cache_art, read_verified_compiled_game_pack, revise_authored_route_terrain,
};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input =
        PathBuf::from(args.next().context(
            "usage: revise_r10_cache PACK SHA256 BEFORE AFTER DOSSIER ART_OUTPUT OUTPUT",
        )?);
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("parent SHA256 must be UTF-8"))?;
    let previous = std::fs::read_to_string(args.next().context("exact before layout")?)?;
    let revised = std::fs::read_to_string(args.next().context("revised layout")?)?;
    let dossier = std::fs::read_to_string(args.next().context("native route dossier")?)?;
    let art_output = PathBuf::from(args.next().context("fresh art-stage pack")?);
    let output = PathBuf::from(args.next().context("fresh terrain pack")?);
    ensure!(
        args.next().is_none() && art_output != output && !art_output.exists() && !output.exists(),
        "two fresh distinct outputs required"
    );
    let hash = || -> Result<String> { Ok(format!("{:x}", Sha256::digest(std::fs::read(&input)?))) };
    ensure!(hash()? == expected, "R10 parent SHA256 differs");
    let base = read_verified_compiled_game_pack(&input)?;
    let art = add_r10_cache_art(&base)?;
    art.write_preserving_storage(&art_output)?;
    let art =
        read_verified_compiled_game_pack(&art_output).context("written R10 grass edge readback")?;
    let terrain = revise_authored_route_terrain(&art, &previous, &revised, &dossier)?;
    ensure!(hash()? == expected, "R10 parent changed during publication");
    terrain.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("written R10 terrain readback")?;
    println!("{}", output.display());
    Ok(())
}
