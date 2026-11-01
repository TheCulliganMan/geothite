//! Publish the native Bemidji roof and plateau repair to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(args.len() == 6, "PACK SHA256 BEFORE AFTER CONTRACT OUTPUT");
    let parent = PathBuf::from(&args[0]);
    let expected = args[1].to_str().context("UTF-8 parent hash")?;
    let contract = std::fs::read_to_string(&args[4])?;
    let binding: serde_json::Value = serde_json::from_str(&contract)?;
    ensure!(
        binding["parent_pack_sha256"] == expected,
        "publication parent differs"
    );
    let output = PathBuf::from(&args[5]);
    ensure!(!output.exists(), "retain every prior pack");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "game content must remain ignored"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent SHA drift"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&parent)?;
    let child = crystal_assets::rebuild_bemidji_roof_plateau(
        &base,
        &std::fs::read_to_string(&args[2])?,
        &std::fs::read_to_string(&args[3])?,
        &contract,
    )?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent changed"
    );
    child.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == child,
        "strict readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
