//! Publish the exact compact Superior structural revision to a fresh pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 8,
        "usage: build_compact_superior BASE CONTRACT EXPECTED_CONTRACT_SHA COAST_CONTRACT R12_LAYOUT R12_DOSSIER CEDAR_CONTRACT FRESH_OUTPUT"
    );
    let input = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[7]);
    ensure!(!output.exists(), "compact output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?))
            == "65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17",
        "exact completed parent file SHA required"
    );
    let read = |i: usize| std::fs::read_to_string(PathBuf::from(&args[i]));
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let revised = crystal_assets::build_compact_superior_pack(
        &base,
        &read(1)?,
        args[2]
            .to_str()
            .context("reviewed compact contract SHA UTF-8")?,
        &read(3)?,
        &read(4)?,
        &read(5)?,
        &read(6)?,
    )?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == revised,
        "written compact pack differs from validated revision"
    );
    println!(
        "Published compact Superior: {} ({})",
        output.display(),
        revised.identity()?.content_hash
    );
    Ok(())
}
