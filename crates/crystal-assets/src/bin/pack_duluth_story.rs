use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_duluth_story_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: pack_duluth_story PACK STORY OUTPUT")?,
    );
    let story = std::fs::read_to_string(args.next().context("canonical story source")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected publisher argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from source pack"
    );
    let pack = build_duluth_story_modpack(&read_verified_compiled_game_pack(base)?, &story)?;
    pack.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("read back native Duluth story")?;
    println!("exported {}", output.display());
    Ok(())
}
