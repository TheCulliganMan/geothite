//! Publish the native pump maze, witnessed drilling and crew quest.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context(
        "usage: pack_northstar_drill_lair PACK EVENTS_JSON APPROACH_JSON GALLERY_JSON CORE_JSON OUTPUT",
    )?).canonicalize()?;
    let events = std::fs::read_to_string(args.next().context("drilling events")?)?;
    let approach = std::fs::read_to_string(args.next().context("crosscut source")?)?;
    let gallery = std::fs::read_to_string(args.next().context("gallery source")?)?;
    let core = std::fs::read_to_string(args.next().context("drill-room source")?)?;
    let output = PathBuf::from(args.next().context("ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let output = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("output name")?);
    ensure!(
        output != input && (!output.exists() || output.canonicalize()? != input),
        "output aliases its parent"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "generated pack must remain untracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let result =
        crystal_assets::build_northstar_drill_lair(&base, &events, &approach, &gallery, &core)?;
    result.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == result.identity()?,
        "published identity differs"
    );
    println!(
        "Verified {}: {} maps, {} species",
        output.display(),
        written.data().maps.len(),
        written.data().pokemon.len()
    );
    Ok(())
}
