//! Publish the frozen pine art append to a fresh external pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 4,
        "usage: build_inland_snow_pines BASE CONTRACT EXPECTED_PARENT_FILE_SHA NEW_OUTPUT"
    );
    let base = PathBuf::from(&args[0]);
    let contract = PathBuf::from(&args[1]);
    let expected = args[2].to_str().context("UTF-8 actual parent file hash")?;
    let output = PathBuf::from(&args[3]);
    ensure!(!output.exists(), "pine art output must be fresh");
    ensure!(
        expected.len() == 64 && format!("{:x}", Sha256::digest(std::fs::read(&base)?)) == expected,
        "actual pine parent file hash differs"
    );
    let original = crystal_assets::read_verified_compiled_game_pack(base)?;
    let result = crystal_assets::build_inland_snow_pines_pack(
        &original,
        &std::fs::read_to_string(contract)?,
    )?;
    result.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == result,
        "pine written readback differs"
    );
    println!(
        "Published inland pine art: {} ({})",
        output.display(),
        result.identity()?.content_hash
    );
    Ok(())
}
