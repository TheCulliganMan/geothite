//! Build and read back Minnesota's Trail Discoveries revision.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_trail_discoveries};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [parent, output] = args.as_slice() else {
        anyhow::bail!("usage: pack_trail_discoveries PARENT FRESH_OUTPUT");
    };
    ensure!(!output.try_exists()?, "choose a fresh output pack");
    let base = read_verified_compiled_game_pack(parent)?;
    let pack = revise_trail_discoveries(&base)?;
    drop(base);
    pack.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(
        written == pack,
        "Trail Discoveries pack changed during readback"
    );
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
