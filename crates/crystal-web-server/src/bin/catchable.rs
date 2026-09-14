//! Build the same encounter expansion served by the multiplayer application.
use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .context("usage: catchable <input.crystalpack> <output.crystalpack>")?;
    let output = args.next().context("output .crystalpack path required")?;
    ensure!(args.next().is_none(), "unexpected argument");
    let input = std::fs::canonicalize(input)?;
    ensure!(
        !std::path::Path::new(&output).exists() || std::fs::canonicalize(&output)? != input,
        "output must differ from source"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let pack = crystal_web_server::catchable::build(&base)?;
    pack.write_preserving_storage(&output)?;
    println!("{}", serde_json::to_string_pretty(&pack.identity()?)?);
    Ok(())
}
