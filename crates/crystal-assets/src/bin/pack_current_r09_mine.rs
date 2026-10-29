//! Publish one reviewed current-story mine overlay to a fresh ignored file.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 6,
        "usage: pack_current_r09_mine BASE PREVIOUS REVISED CONTRACT REVIEWED_SHA256 NEW_OUTPUT"
    );
    let output = PathBuf::from(&args[5]);
    ensure!(!output.exists(), "retain previous mine publications");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "mine output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "mine output must not be tracked"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&args[0])?))
            == "a69aa145e6ea870b459d79859958806170b56036af4f85e0c1e007629a2f67d6",
        "exact current R10 parent file required"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let old = std::fs::read_to_string(&args[1])?;
    let new = std::fs::read_to_string(&args[2])?;
    let contract = std::fs::read_to_string(&args[3])?;
    let pack = crystal_assets::build_current_r09_mine_pack(
        &base,
        &old,
        &new,
        &contract,
        args[4].to_str().context("ASCII reviewed hash")?,
    )?;
    pack.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == pack,
        "strict written mine readback differs"
    );
    println!(
        "Published current regional mine: {} ({})",
        output.display(),
        pack.identity()?.content_hash
    );
    Ok(())
}
