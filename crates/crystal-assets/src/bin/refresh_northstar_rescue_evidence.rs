//! Publish the bounded rescue-evidence correction as an unselected review child.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, refresh_northstar_rescue_evidence};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 2,
        "usage: refresh_northstar_rescue_evidence PARENT FRESH_OUTPUT"
    );
    let output = paths[1]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[1].file_name().context("output filename")?);
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
    let revised = refresh_northstar_rescue_evidence(&current)?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "rescue evidence readback differs"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        revised.data().maps.len()
    );
    Ok(())
}
