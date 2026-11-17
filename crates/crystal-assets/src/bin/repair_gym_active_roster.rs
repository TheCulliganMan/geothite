//! Build an ignored active-roster child of the eight-gym review pack.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 5,
        "usage: repair_gym_active_roster PARENT REPAIRS ROSTER CURATION FRESH_OUTPUT"
    );
    let output = Path::new(&args[4]);
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
    let result = crystal_assets::repair_gym_active_roster(
        &base,
        &std::fs::read_to_string(&args[1])?,
        &std::fs::read_to_string(&args[2])?,
        &std::fs::read_to_string(&args[3])?,
    )?;
    result.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == result,
        "pack readback differs"
    );
    println!("{} {}", output.display(), result.identity()?.content_hash);
    Ok(())
}
