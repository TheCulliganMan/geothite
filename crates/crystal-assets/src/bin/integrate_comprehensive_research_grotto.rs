//! Reproduce the unselected, research and grotto V21 campaign candidate.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{integrate_comprehensive_research_grotto, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: integrate_comprehensive_research_grotto CURRENT FRESH_RESEARCH_OUTPUT FRESH_OUTPUT"
    );
    let output = paths[2]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[2].file_name().context("output filename")?);
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
    ensure!(!paths[1].exists(), "research output must be fresh");
    ensure!(paths[1] != paths[2], "stage and final outputs must differ");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&paths[1])
            .status()?
            .success(),
        "research output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&paths[1])
            .output()?
            .status
            .success(),
        "research output must remain untracked"
    );
    let (research, revised) = integrate_comprehensive_research_grotto(&parent)?;
    research.write_preserving_storage(&paths[1])?;
    ensure!(
        read_verified_compiled_game_pack(&paths[1])? == research,
        "research readback differs"
    );
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "grotto readback differs"
    );
    println!(
        "Verified {}: {}",
        output.display(),
        revised.identity()?.content_hash
    );
    Ok(())
}
