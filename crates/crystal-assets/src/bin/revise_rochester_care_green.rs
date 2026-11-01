//! Publish the source-bound Rochester streets into fresh ignored storage.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(args.len() == 4, "PACK SHA256 SOURCE OUTPUT");
    let input = PathBuf::from(&args[0]);
    let expected = args[1].to_str().context("parent SHA256")?;
    let source = std::fs::read_to_string(&args[2])?;
    let binding: serde_json::Value = serde_json::from_str(&source)?;
    ensure!(
        binding["parent_pack_sha256"] == expected,
        "bound parent differs"
    );
    let output = PathBuf::from(&args[3]);
    ensure!(!output.exists(), "preserve prior candidates");
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
        "parent changed"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let child = crystal_assets::revise_rochester_care_green(&base, &source)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed during publication"
    );
    child.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == child,
        "strict readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
