//! Publish the authored cross-floor ICE headquarters without rebinding a save.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_northstar_ice_headquarters PACK SOURCE_DIRECTORY OUTPUT")?,
    )
    .canonicalize()?;
    let directory = PathBuf::from(args.next().context("authored headquarters directory")?);
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
    let network = std::fs::read_to_string(directory.join("network.json"))?;
    let layouts = ["operations", "records", "maintenance", "transfer"]
        .map(|name| std::fs::read_to_string(directory.join(format!("{name}.json"))))
        .into_iter()
        .collect::<std::io::Result<Vec<_>>>()?;
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let result = crystal_assets::build_northstar_ice_headquarters(&base, &network, &layouts)?;
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
