//! Publish one explicitly authored expansion of an installed Minnesota city.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{expand_authored_city, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: expand_authored_city PACK LAYOUT CONTRACT OUTPUT")?,
    );
    let layout = std::fs::read_to_string(args.next().context("city layout")?)?;
    let contract = std::fs::read_to_string(args.next().context("city expansion contract")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected argument");
    ensure!(
        input.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from input"
    );
    let base = read_verified_compiled_game_pack(&input)?;
    let expanded = expand_authored_city(&base, &layout, &contract)?;
    expanded.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("verify published city expansion")?;
    println!("published expanded city in {}", output.display());
    Ok(())
}
