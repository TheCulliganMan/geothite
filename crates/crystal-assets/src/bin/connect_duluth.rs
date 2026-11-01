use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{connect_duluth, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: connect_duluth PACK TOWN OUTPUT")?,
    );
    let town = std::fs::read_to_string(args.next().context("Duluth source")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(
        args.next().is_none(),
        "unexpected connection publisher argument"
    );
    ensure!(
        input.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from input"
    );
    let pack = connect_duluth(&read_verified_compiled_game_pack(input)?, &town)?;
    pack.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("strict readback of Duluth connections")?;
    println!("exported {}", output.display());
    Ok(())
}
