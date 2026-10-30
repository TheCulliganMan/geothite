//! Publish the four native north-facing gray stair crossings on an exact parent.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_minnesota_outdoor_north_stairs};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: revise_outdoor_north_stairs PACK SHA256 PLAN OUTPUT")?,
    );
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let plan = std::fs::read_to_string(args.next().context("north stair plan")?)?;
    let output = PathBuf::from(args.next().context("fresh output pack")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "exact arguments and fresh output required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent hash differs"
    );
    let base = read_verified_compiled_game_pack(&input)?;
    let result = revise_minnesota_outdoor_north_stairs(&base, &plan)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed during publication"
    );
    result.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("native north stair readback")?;
    println!("{}", output.display());
    Ok(())
}
