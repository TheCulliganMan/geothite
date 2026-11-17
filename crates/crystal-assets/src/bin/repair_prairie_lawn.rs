use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, repair_prairie_lawn_art};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let source = PathBuf::from(
        args.next()
            .context("usage: repair_prairie_lawn INPUT OUTPUT")?,
    );
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        source.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from input"
    );
    let pack = read_verified_compiled_game_pack(source)?;
    repair_prairie_lawn_art(&pack)?.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output)?;
    println!("repaired prairie lawn art: {}", output.display());
    Ok(())
}
