//! Publish a complete Minnesota encounter curve into a fresh ignored pack.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_minnesota_encounter_progression PACK PLAN FRESH_OUTPUT"
    );
    let output = Path::new(&args[2]);
    ensure!(
        !output.exists(),
        "preserve completed packs; choose a fresh output"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let pack = crystal_assets::build_minnesota_encounter_progression(
        &base,
        &std::fs::read_to_string(&args[1])?,
    )?;
    pack.write_preserving_storage(output)?;
    ensure!(
        pack == crystal_assets::read_verified_compiled_game_pack(output)?,
        "written progression pack differs"
    );
    println!("{} {}", output.display(), pack.identity()?.content_hash);
    Ok(())
}
