//! Publish the reviewed waterfront facades to one fresh ignored content pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 4,
        "usage: pack_minneapolis_waterfront BASE CONTRACT REVIEWED_SHA256 NEW_OUTPUT"
    );
    let base_path = PathBuf::from(&args[0]);
    let output_path = PathBuf::from(&args[3]);
    ensure!(
        !output_path.exists(),
        "preserve previous waterfront publications"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output_path)
            .status()?
            .success(),
        "waterfront output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output_path)
            .output()?
            .status
            .success(),
        "waterfront output must not be tracked"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&base_path)?))
            == "1f5604a6c743904e5b0c8fbfe36b32ef056f50dab309a42795aa639457ac0bf3",
        "exact current pine-knoll parent required"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(base_path)?;
    let source = std::fs::read_to_string(&args[1])?;
    let output = crystal_assets::build_minneapolis_waterfront_pack(
        &base,
        &source,
        args[2].to_str().context("ASCII reviewed SHA256")?,
    )?;
    output.write_preserving_storage(&output_path)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output_path)? == output,
        "strict written waterfront readback differs"
    );
    println!(
        "Published native Minneapolis waterfront fronts: {} ({})",
        output_path.display(),
        output.identity()?.content_hash
    );
    Ok(())
}
