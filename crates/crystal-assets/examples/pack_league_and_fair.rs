//! Build and read back Minnesota's League and fair revision.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_league_and_fair};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_league_and_fair PARENT FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let base = read_verified_compiled_game_pack(parent)?;
    let pack = revise_league_and_fair(&base)?;
    drop(base);
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(
        written == pack,
        "League and fair pack changed during readback"
    );
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
