//! Repair a verified external pack's native Rochester Center rival movement.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: repair_minnesota_center_rival PARENT FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
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
    let repaired = crystal_assets::repair_minnesota_center_rival_movements(&parent)?;
    repaired.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == repaired,
        "pack readback differs"
    );
    println!("{} {}", output.display(), repaired.identity()?.content_hash);
    Ok(())
}
