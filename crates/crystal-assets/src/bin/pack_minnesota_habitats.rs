//! Publish authored Minnesota wildlife habitats on existing native terrain.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_habitats PACK SOURCE_DIRECTORY OUTPUT")?,
    )
    .canonicalize()?;
    let directory = PathBuf::from(args.next().context("authored Minnesota directory")?);
    let output = PathBuf::from(args.next().context("ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let output = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("output name")?);
    ensure!(
        output != input && (!output.exists() || output.canonicalize()? != input),
        "output aliases parent"
    );
    ensure!(
        !output.exists(),
        "output already exists; preserve it and choose a fresh ignored path"
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
    let source = std::fs::read_to_string(directory.join("pokemon/acquisition-v01/habitats.json"))?;
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    crystal_assets::verify_northstar_publication_geometry(&base)?;
    let result = crystal_assets::build_minnesota_habitats(&base, &source)?;
    result.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == result.identity()?,
        "published identity differs"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        written.data().maps.len()
    );
    Ok(())
}
