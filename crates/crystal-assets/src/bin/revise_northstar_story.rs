//! Publish a staged dialogue revision of the accepted current regional game.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: revise_northstar_story PACK STORY_JSON OUTPUT")?,
    )
    .canonicalize()?;
    let source = PathBuf::from(args.next().context("canonical story JSON")?);
    let output = PathBuf::from(args.next().context("ignored output pack")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "choose a fresh output path"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack output must be ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let revised = crystal_assets::revise_northstar_story(&base, &std::fs::read_to_string(source)?)?;
    revised.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == revised.identity()?,
        "written revision identity differs"
    );
    println!(
        "Verified {}: {} maps; {}",
        output.display(),
        written.data().maps.len(),
        written.identity()?.content_hash
    );
    Ok(())
}
