//! Write an unselected pack with corrected Minnesota Mart arrivals.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: correct_minnesota_mart_arrivals PARENT FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "choose a fresh output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "output must be ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let candidate = crystal_assets::correct_minnesota_mart_arrivals(&base)?;
    candidate.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == candidate,
        "pack readback differs"
    );
    println!(
        "{} {}",
        output.display(),
        candidate.identity()?.content_hash
    );
    Ok(())
}
