//! Build an isolated city-note candidate from a completed local Minnesota pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_city_note_mimics PACK NOTES_JSON OUTPUT")?,
    )
    .canonicalize()?;
    let source = std::fs::read_to_string(args.next().context("notes JSON")?)?;
    let output = PathBuf::from(args.next().context("ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let output = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("filename")?);
    ensure!(
        output != input && (!output.exists() || output.canonicalize()? != input),
        "must not overwrite parent"
    );
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
    ensure!(
        !output.exists(),
        "retain existing candidate; choose a new filename"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let result = crystal_assets::build_city_note_mimics(&base, &source)?;
    result.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == result.identity()?,
        "written candidate differs"
    );
    println!(
        "Verified {} ({})",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
