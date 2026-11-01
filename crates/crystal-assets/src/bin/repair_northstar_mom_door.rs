//! Produce an unselected review pack with Mom's door approach repaired.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "usage: repair_northstar_mom_door PARENT FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "choose a fresh output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let revised = crystal_assets::repair_northstar_mom_door(&base)?;
    revised.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == revised,
        "pack readback differs"
    );
    println!("{} {}", output.display(), revised.identity()?.content_hash);
    Ok(())
}
