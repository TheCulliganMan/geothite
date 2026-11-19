//! Build and read back Minnesota's Headwaters Journey revision.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_headwaters_journey};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_headwaters_journey PARENT FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let base = read_verified_compiled_game_pack(parent)?;
    let pack = revise_headwaters_journey(&base)?;
    drop(base);
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(
        written == pack,
        "Headwaters Journey pack changed during readback"
    );
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
