use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    read_verified_compiled_game_pack, restore_coastal_cedar_warmth,
    validate_compact_superior_geography,
};
use sha2::{Digest, Sha256};

fn file_hash(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("usage: restore_coastal_cedar_warmth PACK EXPECTED_SHA256 OUTPUT")?,
    );
    let expected = args
        .next()
        .context("expected parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let output = PathBuf::from(args.next().context("fresh output pack")?);
    ensure!(args.next().is_none(), "unexpected extra argument");
    ensure!(!output.exists(), "output pack must be a fresh path");
    ensure!(
        file_hash(&parent)? == expected,
        "parent file SHA256 changed"
    );
    let base = read_verified_compiled_game_pack(&parent)?;
    let revised = restore_coastal_cedar_warmth(&base)?;
    ensure!(
        file_hash(&parent)? == expected,
        "parent changed during warm correction"
    );
    revised.write_preserving_storage(&output)?;
    let written = read_verified_compiled_game_pack(&output)?;
    validate_compact_superior_geography(&written)?;
    println!(
        "restored warm coastal Cedar approach in {}",
        output.display()
    );
    Ok(())
}
