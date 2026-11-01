//! Reproduce the unselected, mainland-integrated V20 campaign candidate.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{integrate_comprehensive_leech_mainland, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 2,
        "usage: integrate_comprehensive_leech_mainland CURRENT FRESH_OUTPUT"
    );
    let output = paths[1]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[1].file_name().context("output filename")?);
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
    let revised = integrate_comprehensive_leech_mainland(&parent)?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "mainland readback differs"
    );
    println!(
        "Verified {}: {}",
        output.display(),
        revised.identity()?.content_hash
    );
    Ok(())
}
