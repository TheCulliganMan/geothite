//! Publish Minnesota outdoor ramp stair art onto an exact parent.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_minnesota_outdoor_ramp_stairs};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: revise_outdoor_ramp_stairs PACK SHA256 PLAN OUTPUT")?,
    );
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let plan = std::fs::read_to_string(args.next().context("ramp stair plan")?)?;
    let output = PathBuf::from(args.next().context("fresh output pack")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and exact arguments required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent SHA256 differs"
    );
    let base = read_verified_compiled_game_pack(&input)?;
    let revised = revise_minnesota_outdoor_ramp_stairs(&base, &plan)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed during publication"
    );
    revised.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("outdoor ramp stair publication readback")?;
    println!("{}", output.display());
    Ok(())
}
