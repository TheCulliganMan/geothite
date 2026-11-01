//! Bind the Minnesota story to active original species.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input =
        PathBuf::from(args.next().context(
            "usage: pack_northstar_regional_cast PACK POLICY_JSON ROSTER_JSON CURATION_JSON PROFILES_JSON OUTPUT",
        )?)
        .canonicalize()?;
    let source = std::fs::read_to_string(args.next().context("canonical story source")?)?;
    let roster = std::fs::read_to_string(args.next().context("regional roster")?)?;
    let curation = std::fs::read_to_string(args.next().context("active curation")?)?;
    let profiles = std::fs::read_to_string(args.next().context("species profiles")?)?;
    let output = PathBuf::from(args.next().context("ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let resolved = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(
        resolved != input
            && std::process::Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index"])
                .arg(&resolved)
                .status()?
                .success(),
        "output must be ignored and differ from source"
    );
    ensure!(
        (!resolved.exists() || resolved.canonicalize()? != input)
            && !std::process::Command::new("git")
                .args(["ls-files", "--error-unmatch", "--"])
                .arg(&resolved)
                .output()?
                .status
                .success(),
        "output must be untracked and must not alias the source"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    crystal_assets::verify_northstar_publication_geometry(&base)?;
    let result = crystal_assets::build_northstar_regional_cast(
        &base, &source, &roster, &curation, &profiles,
    )?;
    result.write_preserving_storage(&resolved)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&resolved)?;
    ensure!(
        written.identity()? == result.identity()?,
        "written lair pack identity differs"
    );
    println!(
        "Verified {}: {} maps, {} species",
        resolved.display(),
        written.data().maps.len(),
        written.data().pokemon.len()
    );
    Ok(())
}
