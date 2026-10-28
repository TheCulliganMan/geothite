//! Publish the frozen Rainy River region onto an explicitly pinned winter-art pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_winter_region_modpack, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 5,
        "usage: build_winter_region ART_PARENT LAYOUT DOSSIER EXPECTED_PARENT_SHA256 NEW_OUTPUT"
    );
    let parent = PathBuf::from(&args[0])
        .canonicalize()
        .context("completed winter-art parent")?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == args[3],
        "actual chosen art-parent file SHA differs"
    );
    let layout = std::fs::read_to_string(&args[1]).context("frozen winter layout")?;
    let dossier = std::fs::read_to_string(&args[2]).context("frozen winter dossier")?;
    let requested = PathBuf::from(&args[4]);
    ensure!(
        !requested.exists(),
        "winter publication requires a fresh output file"
    );
    let output = requested
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(requested.file_name().context("output filename")?);
    let base = read_verified_compiled_game_pack(&parent)?;
    let pack = build_winter_region_modpack(&base, &layout, &dossier)?;
    pack.write_preserving_storage(&output)?;
    let written =
        read_verified_compiled_game_pack(&output).context("strict written winter pack readback")?;
    ensure!(written == pack, "written winter region differs");
    println!(
        "Published Rainy River Icefields: {} ({})",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
