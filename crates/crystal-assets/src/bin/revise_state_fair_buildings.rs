//! Publish the bounded fair facade repair over a current regional pack.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{self, revise_state_fair_buildings};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("usage: revise_state_fair_buildings PACK LAYOUT OUTPUT")?,
    );
    let layout = fs::read_to_string(args.next().context("fair layout")?)?;
    let output = PathBuf::from(args.next().context("fresh output pack")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "exact arguments and fresh output required"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(parent)?;
    let revised = revise_state_fair_buildings(&base, &layout)?;
    revised.write_preserving_storage(&output)?;
    let restored = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        restored.identity()? == revised.identity()?,
        "fair repair readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
