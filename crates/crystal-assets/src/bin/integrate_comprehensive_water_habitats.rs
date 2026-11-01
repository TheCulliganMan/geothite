//! Reproduce the unselected, habitat-integrated campaign campaign candidate.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{integrate_comprehensive_water_habitats, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: integrate_comprehensive_water_habitats CURRENT COMMON DONOR FRESH_OUTPUT"
    );
    let output = paths[3]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[3].file_name().context("output filename")?);
    ensure!(
        !output.exists(),
        "preserve existing candidates; output must be fresh"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "pack output must remain untracked"
    );
    let parent = read_verified_compiled_game_pack(&paths[0])?;
    let common = read_verified_compiled_game_pack(&paths[1])?;
    let donor = read_verified_compiled_game_pack(&paths[2])?;
    let revised = integrate_comprehensive_water_habitats(&parent, &common, &donor)?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "habitat integration readback differs"
    );
    println!(
        "Verified {}: {}",
        output.display(),
        revised.identity()?.content_hash
    );
    Ok(())
}
