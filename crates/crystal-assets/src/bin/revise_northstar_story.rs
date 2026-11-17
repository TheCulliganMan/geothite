//! Publish a staged dialogue revision of the accepted current regional game.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context(
        "usage: revise_northstar_story PACK STORY_JSON OUTPUT [--installed-text|--campaign-polish]",
    )?)
    .canonicalize()?;
    let source = PathBuf::from(args.next().context("canonical story JSON")?);
    let output = PathBuf::from(args.next().context("ignored output pack")?);
    let mode = args.next();
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
    let source = std::fs::read_to_string(source)?;
    let (revised, deferred) = match mode.as_deref().and_then(std::ffi::OsStr::to_str) {
        None if mode.is_none() => (
            crystal_assets::revise_northstar_story(&base, &source)?,
            Vec::new(),
        ),
        Some("--installed-text") => {
            crystal_assets::refresh_installed_northstar_story(&base, &source)?
        }
        Some("--campaign-polish") => crystal_assets::polish_northstar_campaign(&base, &source)?,
        _ => anyhow::bail!("unexpected option {mode:?}"),
    };
    for label in deferred {
        eprintln!("Deferred dialogue: {label}");
    }
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
