//! Publish a reviewed park over the exact selected parent to a fresh ignored file.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_mille_lacs_park, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: pack_mille_lacs_park BASE TOWN LAYOUT CONTRACT OUTPUT"
    );
    let output = &paths[4];
    ensure!(
        !output.exists(),
        "preserve previous park publications; choose fresh output"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "park pack output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(output)
            .output()?
            .status
            .success(),
        "park pack cannot be tracked"
    );
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let town = std::fs::read_to_string(&paths[1]).context("authored park town")?;
    let layout = std::fs::read_to_string(&paths[2]).context("authored park layout")?;
    let contract = std::fs::read_to_string(&paths[3]).context("exact parent/source contract")?;
    let result = build_mille_lacs_park(&base, &town, &layout, &contract)?;
    result.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(written == result, "strict written park readback differs");
    println!("Published Mille Lacs park: {}", output.display());
    Ok(())
}
