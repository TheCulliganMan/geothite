//! Build an isolated source-bound inland winter art candidate; never select it.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(args.len() == 4, "PACK SHA256 PROPOSAL OUTPUT");
    let input = PathBuf::from(&args[0]);
    let expected = args[1].to_str().context("source parent SHA256")?;
    let proposal = std::fs::read_to_string(&args[2])?;
    let binding: serde_json::Value = serde_json::from_str(&proposal)?;
    ensure!(
        binding["source_parent_sha256"] == expected,
        "bound source parent differs"
    );
    let output = PathBuf::from(&args[3]);
    ensure!(!output.exists(), "preserve every prior candidate");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "generated game content must remain ignored"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "source parent changed"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let child = crystal_assets::build_inland_winter_belt(&base, &proposal)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "source changed during native composition"
    );
    child.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == child,
        "strict readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
