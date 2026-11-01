//! Publish free, courier-gated passenger service in five existing native halls.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_glacier_line_service PACK SOURCE_JSON FRESH_OUTPUT")?,
    )
    .canonicalize()?;
    let source = PathBuf::from(args.next().context("native station contract")?);
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let output = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("output file name")?);
    ensure!(
        !output.exists() && output != input,
        "output must be fresh and distinct from parent"
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
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    crystal_assets::verify_northstar_publication_geometry(&base)?;
    let result =
        crystal_assets::build_glacier_line_service(&base, &std::fs::read_to_string(source)?)?;
    crystal_assets::verify_northstar_publication_geometry(&result)?;
    result.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == result.identity()?,
        "written native identity differs"
    );
    println!(
        "Verified five-stop Glacier Line: {} ({} maps)",
        output.display(),
        written.data().maps.len()
    );
    Ok(())
}
