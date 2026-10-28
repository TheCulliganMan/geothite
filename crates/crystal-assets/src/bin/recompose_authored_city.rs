//! Publish a hand-recomposed exterior without recreating installed city rooms.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, recompose_authored_city};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: recompose_authored_city PACK LAYOUT CONTRACT OUTPUT")?,
    );
    let layout = std::fs::read_to_string(args.next().context("compact city layout")?)?;
    let contract = std::fs::read_to_string(args.next().context("city relocation contract")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        input.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from input"
    );
    let base = read_verified_compiled_game_pack(&input)?;
    let revised = recompose_authored_city(&base, &layout, &contract)?;
    revised.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("verify published city recomposition")?;
    println!("published recomposed city in {}", output.display());
    Ok(())
}
