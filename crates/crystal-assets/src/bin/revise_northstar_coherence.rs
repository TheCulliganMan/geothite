//! Build an unselected, verified Northstar dialogue candidate.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: revise_northstar_coherence PARENT DIALOGUE FRESH_OUTPUT"
    );
    let output = Path::new(&args[2]);
    ensure!(!output.exists(), "output already exists");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "output must remain ignored"
    );
    let parent = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let source = std::fs::read_to_string(&args[1])?;
    let candidate = crystal_assets::revise_northstar_coherence(&parent, &source)?;
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
