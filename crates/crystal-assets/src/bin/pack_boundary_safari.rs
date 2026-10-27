use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_boundary_safari_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 5,
        "usage: pack_boundary_safari BASE LAYOUT ZONE CABIN OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[4].canonicalize().unwrap_or_else(|_| args[4].clone()),
        "output must differ from the base pack"
    );
    let base = read_verified_compiled_game_pack(&args[0])?;
    let layout = std::fs::read_to_string(&args[1]).context("authored preserve layout")?;
    let zone = std::fs::read_to_string(&args[2]).context("preserve design")?;
    let cabin = std::fs::read_to_string(&args[3]).context("authored Root Beer cabin")?;
    build_boundary_safari_modpack(&base, &layout, &zone, &cabin)?
        .write_preserving_storage(&args[4])?;
    println!(
        "Published {}: four preserve maps and the actual cabin. Expedition admission remains closed; Safari sessions and regional access remain pending.",
        args[4].display()
    );
    Ok(())
}
