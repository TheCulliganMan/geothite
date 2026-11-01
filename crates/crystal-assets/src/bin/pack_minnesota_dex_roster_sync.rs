//! Publish the expanded regional research roster to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_dex_roster_sync PARENT POKEMON_ROOT OUTPUT")?,
    )
    .canonicalize()?;
    let root = PathBuf::from(args.next().context("Pokemon source root")?);
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let output = output
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(input != output && !output.exists(), "output must be fresh");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "pack must remain untracked"
    );
    let parent = crystal_assets::read_verified_compiled_game_pack(input)?;
    let pack = crystal_assets::build_minnesota_dex_roster_sync(
        &parent,
        &std::fs::read_to_string(root.join("roster.json"))?,
        &std::fs::read_to_string(root.join("roster-curation.json"))?,
    )?;
    pack.write_preserving_storage(&output)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        reopened.identity()? == pack.identity()?,
        "research pack readback mismatch"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        reopened.data().maps.len()
    );
    Ok(())
}
