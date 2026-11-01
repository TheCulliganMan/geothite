//! Publish the optional Minnesota fossil research quest.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_fossils PACK OUTPUT")?,
    )
    .canonicalize()?;
    let output = PathBuf::from(args.next().context("new ignored output")?);
    ensure!(args.next().is_none(), "unexpected argument");
    let output = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(
        output != input && !output.exists(),
        "output must be fresh and differ from parent"
    );
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
        "pack must be untracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let pack = crystal_assets::build_minnesota_fossils(&base)?;
    pack.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == pack.identity()?,
        "fossil publication identity mismatch"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        written.data().maps.len()
    );
    Ok(())
}
