//! Publish the once-only Fire-starter research partner to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_starter_return PARENT OUTPUT")?,
    )
    .canonicalize()?;
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
    let pack = crystal_assets::build_minnesota_starter_return(&parent)?;
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
