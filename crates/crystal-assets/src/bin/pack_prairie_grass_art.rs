use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_prairie_grass_art_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let source = PathBuf::from(
        args.next()
            .context("usage: pack_prairie_grass_art BASE OUTPUT")?,
    );
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        source.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from base"
    );
    let base = read_verified_compiled_game_pack(source)?;
    build_prairie_grass_art_pack(&base)?.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output)?;
    println!("published golden grass art only: {}", output.display());
    Ok(())
}
