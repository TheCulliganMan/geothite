//! Publish an exact terrain-only revision of the installed R03 discovery.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_castorock_gallery};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_castorock_gallery BASE BEFORE_SITE AFTER_SITE OUTPUT"
    );
    ensure!(
        !paths[3].exists(),
        "preserve prior publications; choose a fresh output"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&paths[3])
            .status()?
            .success(),
        "published content must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&paths[3])
            .output()?
            .status
            .success(),
        "published content must not be tracked"
    );
    let before = std::fs::read_to_string(&paths[1]).context("previous discovery source")?;
    let after = std::fs::read_to_string(&paths[2]).context("revised discovery source")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    crystal_assets::validate_compact_superior_geography(&base)?;
    let revised = revise_castorock_gallery(&base, &before, &after)?;
    crystal_assets::validate_compact_superior_geography(&revised)?;
    revised.write_preserving_storage(&paths[3])?;
    let written = read_verified_compiled_game_pack(&paths[3])?;
    ensure!(
        written == revised,
        "strict published discovery readback differs"
    );
    println!("Revised native discovery terrain in {}", paths[3].display());
    Ok(())
}
