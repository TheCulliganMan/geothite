//! Publish the finite legendary completion overlay into a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        paths.len() == 4,
        "usage: pack_northstar_legendary_completion PARENT ROSTER CURATION FRESH_OUTPUT"
    );
    let output = paths[3]
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(paths[3].file_name().context("output name")?);
    ensure!(
        !output.exists(),
        "preserve existing packs; output must be fresh"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack output must stay ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let result = crystal_assets::build_northstar_legendary_completion(
        &base,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
    )?;
    result.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == result,
        "written legendary overlay differs"
    );
    println!("Verified {}", output.display());
    Ok(())
}
