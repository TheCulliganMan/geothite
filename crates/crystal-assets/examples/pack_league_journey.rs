//! Write the League journey revision and verify the resulting external archive.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_league_journey};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_league_journey PARENT FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let pack = revise_league_journey(&read_verified_compiled_game_pack(parent)?)?;
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(written == pack, "League journey changed during readback");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
