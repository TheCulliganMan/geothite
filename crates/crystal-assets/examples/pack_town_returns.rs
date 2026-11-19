//! Build the revised town interactions and verify the archive after writing.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_town_returns};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_town_returns PARENT FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let pack = revise_town_returns(&read_verified_compiled_game_pack(parent)?)?;
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(written == pack, "town interactions changed during readback");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
