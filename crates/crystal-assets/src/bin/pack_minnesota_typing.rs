//! Publish authored typing changes to a fresh ignored regional pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_typing INPUT ROSTER OUTPUT")?,
    )
    .canonicalize()?;
    let roster = PathBuf::from(args.next().context("roster JSON")?);
    let output = PathBuf::from(args.next().context("output pack")?);
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
        "output must be ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "output must be untracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let pack =
        crystal_assets::build_minnesota_typing_revision(&base, &std::fs::read_to_string(roster)?)?;
    pack.write_preserving_storage(&output)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        reopened.identity()? == pack.identity()?,
        "pack readback mismatch"
    );
    println!("Verified {}", output.display());
    Ok(())
}
