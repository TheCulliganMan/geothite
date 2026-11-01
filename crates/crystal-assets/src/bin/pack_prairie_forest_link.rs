use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_prairie_forest_link_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 4,
        "usage: pack_prairie_forest_link BASE LAYOUT DOSSIER OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[3].canonicalize().unwrap_or_else(|_| args[3].clone()),
        "output must differ from the base pack"
    );
    let base = read_verified_compiled_game_pack(&args[0])?;
    let layout = std::fs::read_to_string(&args[1]).context("authored layout")?;
    let dossier = std::fs::read_to_string(&args[2]).context("route dossier")?;
    build_prairie_forest_link_modpack(&base, &layout, &dossier)?
        .write_preserving_storage(&args[3])?;
    println!(
        "Published {}: two clipped prairie maps with native observations and Headbutt. Original HOME retained; regional gates and earned journey remain pending.",
        args[3].display()
    );
    Ok(())
}
