//! Publish the two frozen, hand-authored northern map expansions together.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    expand_boundary_safari_modpack, expand_north_shore_modpack, read_verified_compiled_game_pack,
};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 9,
        "usage: expand_minnesota_northwoods BASE OLD_R11_LAYOUT NEW_R11_LAYOUT OLD_R11_DOSSIER NEW_R11_DOSSIER OLD_SAFARI_LAYOUT NEW_SAFARI_LAYOUT NEW_SAFARI_ZONE OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[8].canonicalize().unwrap_or_else(|_| paths[8].clone()),
        "expanded output must differ from completed base"
    );
    let inputs = paths[1..8]
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .with_context(|| format!("capture northern authoring input {}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let shore = expand_north_shore_modpack(&base, &inputs[0], &inputs[1], &inputs[2], &inputs[3])
        .context("expand native North Shore without replacing existing story content")?;
    let result = expand_boundary_safari_modpack(&shore, &inputs[4], &inputs[5], &inputs[6])
        .context("expand native Boundary Waters and relocate existing hospitality")?;
    result.write_preserving_storage(&paths[8])?;
    let written = read_verified_compiled_game_pack(&paths[8])
        .context("strictly reload expanded northern pack")?;
    println!(
        "Published {} maps to {}. Safari expedition admission remains closed.",
        written.data().maps.len(),
        paths[8].display()
    );
    Ok(())
}
