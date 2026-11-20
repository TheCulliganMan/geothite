//! Build Ely’s League registration or Winona’s Itemfinder follow-up and verify the written external archive.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{
    read_verified_compiled_game_pack, revise_ely_challenge, revise_finder_lesson,
};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let (parent, output, lesson) = match args.as_slice() {
        [parent, output] => (parent, output, false),
        [mode, parent, output] if mode.as_os_str() == "--finder-lesson" => (parent, output, true),
        _ => anyhow::bail!("usage: pack_ely_challenge [--finder-lesson] PARENT FRESH_OUTPUT"),
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let pack = read_verified_compiled_game_pack(parent)?;
    let pack = if lesson {
        revise_finder_lesson(pack)?
    } else {
        revise_ely_challenge(pack)?
    };
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(written == pack, "content changed during readback");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
