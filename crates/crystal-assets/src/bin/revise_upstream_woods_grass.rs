use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    read_verified_compiled_game_pack, revise_upstream_woods_grass,
    validate_compact_superior_geography,
};
use sha2::{Digest, Sha256};

fn file_hash(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(std::fs::read(path)?)))
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(args.next().context(
        "usage: revise_upstream_woods_grass PACK EXPECTED_SHA256 BEFORE_LAYOUT AFTER_LAYOUT OUTPUT",
    )?);
    let expected = args
        .next()
        .context("expected parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let before_path = PathBuf::from(args.next().context("before source")?);
    let after_path = PathBuf::from(args.next().context("after source")?);
    let output = PathBuf::from(args.next().context("fresh output pack")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and exact arguments required"
    );
    ensure!(
        file_hash(&parent)? == expected,
        "parent file SHA256 differs"
    );
    let before = std::fs::read_to_string(&before_path)?;
    let after = std::fs::read_to_string(&after_path)?;
    let base = read_verified_compiled_game_pack(&parent)?;
    let revised = revise_upstream_woods_grass(&base, &before, &after)?;
    ensure!(
        file_hash(&parent)? == expected
            && std::fs::read_to_string(before_path)? == before
            && std::fs::read_to_string(after_path)? == after,
        "publication inputs changed"
    );
    revised.write_preserving_storage(&output)?;
    let written = read_verified_compiled_game_pack(&output)?;
    validate_compact_superior_geography(&written)?;
    println!(
        "revised Upstream Woods grassy loop and westward return in {}",
        output.display()
    );
    Ok(())
}
