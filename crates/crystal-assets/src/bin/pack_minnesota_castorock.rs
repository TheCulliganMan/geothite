//! Publish the reviewed, native Strength-gated incisor gallery.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_castorock PACK SITE_JSON OUTPUT")?,
    )
    .canonicalize()?;
    let source = PathBuf::from(args.next().context("hand-authored site JSON")?);
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
        "pack must remain untracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let pack = crystal_assets::build_minnesota_castorock(&base, &std::fs::read_to_string(source)?)?;
    pack.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == pack.identity()?,
        "incisor gallery readback identity mismatch"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        written.data().maps.len()
    );
    Ok(())
}
