//! Compose the authored story onto the certified current regional parent.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context(
        "usage: pack_northstar_current_story PACK SOURCE_DIRECTORY OUTPUT [COMPOSITION_JSON]",
    )?)
    .canonicalize()?;
    let directory = PathBuf::from(args.next().context("authored Minnesota directory")?);
    let output = PathBuf::from(args.next().context("ignored output")?);
    let composition = args.next().map(PathBuf::from);
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
        "preserve completed publications; choose a fresh output path"
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
    let result = if let Some(path) = composition {
        crystal_assets::build_northstar_current_story_from_composition(
            &base,
            &directory,
            &std::fs::read_to_string(path).context("explicit composition contract")?,
        )?
    } else {
        crystal_assets::build_northstar_current_story(&base, &directory)?
    };
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
