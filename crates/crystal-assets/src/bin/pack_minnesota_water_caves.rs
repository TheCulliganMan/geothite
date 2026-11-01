//! Publish the authored current-region water and cave habitats.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 4,
        "usage: pack_minnesota_water_caves PACK PLAN NATIVE_SWEET_SCENT_SOURCE FRESH_OUTPUT"
    );
    let output = Path::new(&args[3]);
    ensure!(
        !output.exists(),
        "preserve completed packs; output must be fresh"
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
    let pack = crystal_assets::build_minnesota_water_cave_habitats(
        &base,
        &std::fs::read_to_string(&args[1])?,
        &std::fs::read_to_string(&args[2])?,
    )?;
    pack.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(written == pack, "written habitat pack differs");
    println!("{} {}", output.display(), pack.identity()?.content_hash);
    Ok(())
}
