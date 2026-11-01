//! Publish reliable field HM handoffs on a completed Minnesota pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_hm_progression PACK OUTPUT")?,
    )
    .canonicalize()?;
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
    let result = crystal_assets::build_minnesota_hm_progression(&base)?;
    result.write_preserving_storage(&resolved)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&resolved)?;
    ensure!(
        written.identity()? == result.identity()?,
        "written field HM pack identity differs"
    );
    println!(
        "Verified {}: {} maps, {} species",
        resolved.display(),
        written.data().maps.len(),
        written.data().pokemon.len()
    );
    Ok(())
}
