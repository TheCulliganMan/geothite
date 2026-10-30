use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    read_verified_compiled_game_pack, revise_iron_cliff_stairs, validate_compact_superior_geography,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(args.next().context("PACK BEFORE AFTER DOSSIER OUTPUT")?);
    let old = std::fs::read_to_string(args.next().context("BEFORE")?)?;
    let new = std::fs::read_to_string(args.next().context("AFTER")?)?;
    let dossier = std::fs::read_to_string(args.next().context("DOSSIER")?)?;
    let output = PathBuf::from(args.next().context("OUTPUT")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and exact arguments required"
    );
    let pack = read_verified_compiled_game_pack(&base)?;
    validate_compact_superior_geography(&pack)?;
    let child = revise_iron_cliff_stairs(&pack, &old, &new, &dossier)?;
    validate_compact_superior_geography(&child)?;
    child.write_preserving_storage(&output)?;
    let written = read_verified_compiled_game_pack(&output)?;
    validate_compact_superior_geography(&written)?;
    println!("{}", output.display());
    Ok(())
}
