//! Create a fresh, verified child containing only the three civic sign anchors.
use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_grand_rapids_sign_anchors};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context("PACK SHA256 BEFORE AFTER OUTPUT")?);
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let before = std::fs::read_to_string(args.next().context("before source")?)?;
    let after = std::fs::read_to_string(args.next().context("after source")?)?;
    let output = PathBuf::from(args.next().context("fresh output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and exact arguments required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent SHA drift"
    );
    let base = read_verified_compiled_game_pack(&input)?;
    let child = revise_grand_rapids_sign_anchors(&base, &before, &after)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed"
    );
    child.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("sign child readback")?;
    println!("{}", output.display());
    Ok(())
}
