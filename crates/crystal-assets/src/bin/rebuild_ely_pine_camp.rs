//! Publish the source-bound snowy Ely refinement to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 7,
        "PACK SHA256 BEFORE AFTER ACTIVITIES CONTRACT OUTPUT"
    );
    let parent = PathBuf::from(&args[0]);
    let expected = args[1].to_str().context("UTF-8 parent hash")?;
    let contract = std::fs::read_to_string(&args[5])?;
    let binding: serde_json::Value = serde_json::from_str(&contract)?;
    ensure!(
        binding["parent_pack_sha256"] == expected,
        "publication parent differs"
    );
    let output = PathBuf::from(&args[6]);
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
    let child = crystal_assets::rebuild_ely_pine_camp(
        &base,
        &std::fs::read_to_string(&args[2])?,
        &std::fs::read_to_string(&args[3])?,
        &std::fs::read_to_string(&args[4])?,
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
