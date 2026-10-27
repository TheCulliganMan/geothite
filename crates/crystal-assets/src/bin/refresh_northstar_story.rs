//! Update installed reveal/finale text without rebuilding terrain or progression.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    read_verified_compiled_game_pack, refresh_boundary_basecamp_story_dialogue,
    refresh_duluth_story_dialogue,
};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let base = PathBuf::from(
        args.next()
            .context("usage: refresh_northstar_story PACK STORY OUTPUT")?,
    );
    let story = std::fs::read_to_string(args.next().context("canonical story source")?)?;
    let output = PathBuf::from(args.next().context("output pack")?);
    ensure!(args.next().is_none(), "unexpected publisher argument");
    ensure!(
        base.canonicalize()? != output.canonicalize().unwrap_or_else(|_| output.clone()),
        "output must differ from source pack"
    );
    let pack = refresh_duluth_story_dialogue(&read_verified_compiled_game_pack(base)?, &story)?;
    let pack = refresh_boundary_basecamp_story_dialogue(&pack)?;
    pack.write_preserving_storage(&output)?;
    let verified = read_verified_compiled_game_pack(&output)
        .context("read back refreshed native Northstar story")?;
    println!(
        "exported {} with {} maps",
        output.display(),
        verified.data().maps.len()
    );
    Ok(())
}
