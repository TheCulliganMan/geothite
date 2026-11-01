//! Publish the missing native Route 13 exit apron.
use anyhow::{Context, Result, ensure};
use crystal_assets::{align_r13_boundary, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context("PACK SHA256 FRESH_OUTPUT")?);
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("UTF-8 SHA required"))?;
    let output = PathBuf::from(args.next().context("fresh output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "exact arguments and fresh output required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent SHA drift"
    );
    let child = align_r13_boundary(&read_verified_compiled_game_pack(&input)?)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed"
    );
    child.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == child,
        "child readback drift"
    );
    println!("{}", output.display());
    Ok(())
}
