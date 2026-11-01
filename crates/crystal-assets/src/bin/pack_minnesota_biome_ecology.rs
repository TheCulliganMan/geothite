//! Publish current Minnesota biome corrections into a fresh ignored pack.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_minnesota_biome_ecology PACK PLAN FRESH_OUTPUT"
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
    let pack =
        crystal_assets::build_minnesota_biome_ecology(&base, &std::fs::read_to_string(&args[1])?)?;
    pack.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(pack == written, "written biome pack differs");
    println!("{} {}", output.display(), pack.identity()?.content_hash);
    Ok(())
}
