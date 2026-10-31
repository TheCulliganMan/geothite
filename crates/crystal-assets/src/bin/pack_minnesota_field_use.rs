//! Publish the optional native Flash gallery and Whirlpool cove as an unselected review child.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_minnesota_field_use_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: pack_minnesota_field_use PARENT SITE NATIVE_FORCED_MOVEMENT_SOURCE FRESH_OUTPUT"
    );
    let output = paths[3]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[3].file_name().context("output filename")?);
    ensure!(
        !output.exists(),
        "preserve completed candidates; output must be fresh"
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
    let current = read_verified_compiled_game_pack(&paths[0])?;
    let revised = build_minnesota_field_use_pack(
        &current,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
    )?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "Native field-use readback differs"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        revised.data().maps.len()
    );
    Ok(())
}
