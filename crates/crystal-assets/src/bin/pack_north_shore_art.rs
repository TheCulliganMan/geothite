use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_north_shore_art_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_north_shore_art BASE OUTPUT")?,
    );
    let output = PathBuf::from(
        args.next()
            .context("usage: pack_north_shore_art BASE OUTPUT")?,
    );
    ensure!(
        args.next().is_none(),
        "usage: pack_north_shore_art BASE OUTPUT"
    );
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from the base pack"
    );
    let source = read_verified_compiled_game_pack(base)?;
    build_north_shore_art_pack(&source)?.write_preserving_storage(&output)?;
    println!(
        "Prepared {}: derived art only; original maps and spawn retained.",
        output.display()
    );
    Ok(())
}
